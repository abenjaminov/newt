use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct GitStatusEntry {
    pub path: String,
    pub orig_path: Option<String>,
    pub index_status: String,    // "M", "A", "D", "R", "C", "U", "?", " "
    pub worktree_status: String, // same set
}

#[derive(Serialize, Clone)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub entries: Vec<GitStatusEntry>,
}

fn run_git(repo: &str, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| format!("git {}: {}", args.join(" "), e))
}

fn check_status(out: &std::process::Output, ctx: &str) -> Result<(), String> {
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).into_owned();
        return Err(format!("{}: {}", ctx, err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub fn git_status(repo: String) -> Result<GitStatus, String> {
    let inside = run_git(&repo, &["rev-parse", "--is-inside-work-tree"])?;
    if !inside.status.success() {
        return Ok(GitStatus {
            is_repo: false,
            branch: None,
            upstream: None,
            ahead: 0,
            behind: 0,
            entries: Vec::new(),
        });
    }

    let out = run_git(
        &repo,
        &[
            // Don't take any optional locks (e.g. index.lock when refreshing
            // stat cache). Git ships this flag specifically for editors that
            // poll status — without it we can race the user's own git
            // commands and leave a stale .git/index.lock.
            "--no-optional-locks",
            "status",
            "--porcelain=v2",
            "--branch",
            "-z",
            "--untracked-files=all",
            // Match VS Code: render renames as a single entry instead of
            // (delete-old + add-new).
            "--renames",
            // Note: --ignored is intentionally NOT here. Listing ignored
            // entries via git status is expensive on big repos; dimming uses
            // a separate, cached `git_ignored_paths` call.
        ],
    )?;
    check_status(&out, "git status")?;

    let mut branch: Option<String> = None;
    let mut upstream: Option<String> = None;
    let mut ahead: u32 = 0;
    let mut behind: u32 = 0;
    let mut entries: Vec<GitStatusEntry> = Vec::new();

    // The output is null-separated; rename/copy entries consume an extra null for the orig path.
    let bytes = out.stdout;
    let mut pos = 0usize;
    let buf = bytes.as_slice();

    while pos < buf.len() {
        let end = match buf[pos..].iter().position(|&b| b == 0) {
            Some(i) => pos + i,
            None => buf.len(),
        };
        let line_bytes = &buf[pos..end];
        pos = end + 1;
        if line_bytes.is_empty() {
            continue;
        }
        let line = String::from_utf8_lossy(line_bytes);
        let mut chars = line.chars();
        match chars.next() {
            Some('#') => {
                // Header lines: # branch.head <name>, # branch.upstream <name>, # branch.ab +N -M
                if let Some(rest) = line.strip_prefix("# branch.head ") {
                    if rest != "(detached)" {
                        branch = Some(rest.to_string());
                    }
                } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
                    upstream = Some(rest.to_string());
                } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
                    // format: +<n> -<m>
                    let mut parts = rest.split_whitespace();
                    if let Some(a) = parts.next() {
                        ahead = a.trim_start_matches('+').parse().unwrap_or(0);
                    }
                    if let Some(b) = parts.next() {
                        behind = b.trim_start_matches('-').parse().unwrap_or(0);
                    }
                }
            }
            Some('1') => {
                // Format: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
                let parts: Vec<&str> = line.splitn(9, ' ').collect();
                if parts.len() == 9 {
                    let xy = parts[1];
                    let path = parts[8].to_string();
                    let (xs, ys) = split_xy(xy);
                    entries.push(GitStatusEntry {
                        path,
                        orig_path: None,
                        index_status: xs,
                        worktree_status: ys,
                    });
                }
            }
            Some('2') => {
                // Rename/copy: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>
                // Followed by a separately-null-terminated orig path.
                let parts: Vec<&str> = line.splitn(10, ' ').collect();
                if parts.len() == 10 {
                    let xy = parts[1];
                    let path = parts[9].to_string();
                    let (xs, ys) = split_xy(xy);
                    // Consume the orig path (next null-terminated record).
                    let orig_end = match buf[pos..].iter().position(|&b| b == 0) {
                        Some(i) => pos + i,
                        None => buf.len(),
                    };
                    let orig =
                        String::from_utf8_lossy(&buf[pos..orig_end]).into_owned();
                    pos = orig_end + 1;
                    entries.push(GitStatusEntry {
                        path,
                        orig_path: Some(orig),
                        index_status: xs,
                        worktree_status: ys,
                    });
                }
            }
            Some('?') => {
                // ? <path>
                let path = line[2..].to_string();
                entries.push(GitStatusEntry {
                    path,
                    orig_path: None,
                    index_status: "?".into(),
                    worktree_status: "?".into(),
                });
            }
            Some('!') => {
                // ignored entry: "! <path>"
                let path = line[2..].to_string();
                entries.push(GitStatusEntry {
                    path,
                    orig_path: None,
                    index_status: "!".into(),
                    worktree_status: "!".into(),
                });
            }
            Some('u') => {
                // unmerged: u <XY> ...
                let parts: Vec<&str> = line.splitn(11, ' ').collect();
                if parts.len() == 11 {
                    let xy = parts[1];
                    let path = parts[10].to_string();
                    let (xs, ys) = split_xy(xy);
                    entries.push(GitStatusEntry {
                        path,
                        orig_path: None,
                        index_status: xs,
                        worktree_status: ys,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(GitStatus {
        is_repo: true,
        branch,
        upstream,
        ahead,
        behind,
        entries,
    })
}

fn split_xy(xy: &str) -> (String, String) {
    let mut iter = xy.chars();
    let x = iter.next().map(|c| c.to_string()).unwrap_or_default();
    let y = iter.next().map(|c| c.to_string()).unwrap_or_default();
    (x, y)
}

/// Returns the list of paths (relative to repo root) that are ignored by
/// .gitignore. Used for dimming in the file tree. This is a separate command
/// from `git_status` because it can be slow on large repos; the frontend
/// caches the result and only refreshes it on workspace/worktree change.
#[tauri::command]
pub fn git_ignored_paths(repo: String) -> Result<Vec<String>, String> {
    let inside = run_git(&repo, &["rev-parse", "--is-inside-work-tree"])?;
    if !inside.status.success() {
        return Ok(Vec::new());
    }
    let out = run_git(
        &repo,
        &[
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--directory",
            "-z",
        ],
    )?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let s = String::from_utf8_lossy(&out.stdout);
    Ok(s.split('\0')
        .filter(|p| !p.is_empty())
        .map(|p| p.trim_end_matches('/').to_string())
        .collect())
}

#[tauri::command]
pub fn git_file_at_head(repo: String, path: String) -> Result<String, String> {
    let spec = format!("HEAD:{}", path);
    let out = run_git(&repo, &["show", &spec])?;
    if !out.status.success() {
        // file not in HEAD (newly added) — return empty
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[tauri::command]
pub fn git_diff(repo: String, path: String, staged: bool) -> Result<String, String> {
    let mut args: Vec<&str> = vec!["diff", "--no-color"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(&path);
    let out = run_git(&repo, &args)?;
    check_status(&out, "git diff")?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[tauri::command]
pub fn git_stage(repo: String, paths: Vec<String>) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["add", "--"];
    for p in &paths {
        args.push(p);
    }
    let out = run_git(&repo, &args)?;
    check_status(&out, "git add")?;
    Ok(())
}

#[tauri::command]
pub fn git_unstage(repo: String, paths: Vec<String>) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["restore", "--staged", "--"];
    for p in &paths {
        args.push(p);
    }
    let out = run_git(&repo, &args)?;
    check_status(&out, "git restore --staged")?;
    Ok(())
}

#[tauri::command]
pub fn git_discard(repo: String, paths: Vec<String>) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["restore", "--worktree", "--"];
    for p in &paths {
        args.push(p);
    }
    let out = run_git(&repo, &args)?;
    check_status(&out, "git restore --worktree")?;
    Ok(())
}

#[tauri::command]
pub fn git_commit(repo: String, message: String) -> Result<String, String> {
    let out = run_git(&repo, &["commit", "-m", &message])?;
    if !out.status.success() {
        // common cases: nothing staged, hooks failing
        let err = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(format!("{}{}", stdout, err).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[derive(Serialize)]
pub struct BranchList {
    pub current: Option<String>,
    pub local: Vec<String>,
    pub remote: Vec<String>,
}

#[tauri::command]
pub fn git_branches(repo: String) -> Result<BranchList, String> {
    let cur = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let current = if cur.status.success() {
        Some(
            String::from_utf8_lossy(&cur.stdout)
                .trim()
                .to_string(),
        )
    } else {
        None
    };

    let local_out = run_git(&repo, &["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;
    check_status(&local_out, "git for-each-ref")?;
    let local: Vec<String> = String::from_utf8_lossy(&local_out.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let remote_out = run_git(
        &repo,
        &["for-each-ref", "--format=%(refname:short)", "refs/remotes/"],
    )?;
    let remote: Vec<String> = if remote_out.status.success() {
        String::from_utf8_lossy(&remote_out.stdout)
            .lines()
            .filter(|l| !l.ends_with("/HEAD"))
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    Ok(BranchList { current, local, remote })
}

#[tauri::command]
pub fn git_checkout(repo: String, branch: String) -> Result<(), String> {
    let out = run_git(&repo, &["checkout", &branch])?;
    check_status(&out, "git checkout")?;
    Ok(())
}

/// Remove a stale `.git/index.lock` file. Returns true if a lock was removed,
/// false if there was none. Refuses to remove if a recently-modified lock
/// suggests an active git operation.
#[tauri::command]
pub fn git_clear_index_lock(repo: String) -> Result<bool, String> {
    let common = run_git(&repo, &["rev-parse", "--git-common-dir"])
        .map_err(|e| format!("rev-parse: {}", e))?;
    if !common.status.success() {
        return Err(String::from_utf8_lossy(&common.stderr).trim().to_string());
    }
    let dir = String::from_utf8_lossy(&common.stdout).trim().to_string();
    let dir_path = std::path::Path::new(&dir);
    let absolute = if dir_path.is_absolute() {
        dir_path.to_path_buf()
    } else {
        std::path::Path::new(&repo).join(dir_path)
    };
    let lock = absolute.join("index.lock");
    if !lock.exists() {
        return Ok(false);
    }
    // Refuse if the lock is younger than 3 seconds — a real git op might be
    // running. The user can retry after a moment.
    if let Ok(meta) = std::fs::metadata(&lock) {
        if let Ok(modified) = meta.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                if age.as_secs() < 3 {
                    return Err(format!(
                        "Lock was modified {}s ago — looks like an active git operation. Wait a moment and try again.",
                        age.as_secs()
                    ));
                }
            }
        }
    }
    std::fs::remove_file(&lock)
        .map_err(|e| format!("remove {}: {}", lock.display(), e))?;
    Ok(true)
}
