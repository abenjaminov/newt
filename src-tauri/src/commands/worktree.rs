use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct Worktree {
    pub path: String,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub bare: bool,
    pub detached: bool,
    pub locked: bool,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
}

fn run_git(repo: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
}

#[tauri::command]
pub fn list_worktrees(path: String) -> Result<Vec<Worktree>, String> {
    let inside = run_git(&path, &["rev-parse", "--is-inside-work-tree"])
        .map_err(|e| e.to_string())?;
    if !inside.status.success() {
        return Ok(Vec::new());
    }

    let out = run_git(&path, &["worktree", "list", "--porcelain", "-z"])
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let mut worktrees = parse_worktrees(&out.stdout);

    // Enrich each worktree with dirty/ahead/behind. Skip bare repos and detached HEADs without upstream.
    for wt in worktrees.iter_mut() {
        if wt.bare {
            continue;
        }
        if let Ok(s) = run_git(&wt.path, &["status", "--porcelain"]) {
            if s.status.success() {
                wt.dirty = !s.stdout.is_empty();
            }
        }
        if let Ok(ab) = run_git(
            &wt.path,
            &["rev-list", "--count", "--left-right", "@{u}...HEAD"],
        ) {
            if ab.status.success() {
                let s = String::from_utf8_lossy(&ab.stdout);
                let nums: Vec<&str> = s.split_whitespace().collect();
                if nums.len() == 2 {
                    wt.behind = nums[0].parse().unwrap_or(0);
                    wt.ahead = nums[1].parse().unwrap_or(0);
                }
            }
        }
    }

    Ok(worktrees)
}

fn parse_worktrees(bytes: &[u8]) -> Vec<Worktree> {
    let s = String::from_utf8_lossy(bytes);
    let mut out: Vec<Worktree> = Vec::new();
    for block in s.split("\0\0") {
        if block.is_empty() {
            continue;
        }
        let mut wt = Worktree {
            path: String::new(),
            branch: None,
            head: None,
            bare: false,
            detached: false,
            locked: false,
            dirty: false,
            ahead: 0,
            behind: 0,
        };
        for field in block.split('\0') {
            if field.is_empty() {
                continue;
            }
            if let Some(p) = field.strip_prefix("worktree ") {
                wt.path = p.to_string();
            } else if let Some(h) = field.strip_prefix("HEAD ") {
                wt.head = Some(h.to_string());
            } else if let Some(b) = field.strip_prefix("branch ") {
                wt.branch = Some(b.trim_start_matches("refs/heads/").to_string());
            } else if field == "detached" {
                wt.detached = true;
            } else if field == "bare" {
                wt.bare = true;
            } else if field == "locked" || field.starts_with("locked ") {
                wt.locked = true;
            }
        }
        if !wt.path.is_empty() {
            out.push(wt);
        }
    }
    out
}

#[tauri::command]
pub fn add_worktree(
    repo: String,
    new_path: String,
    branch: String,
    create_new: bool,
) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["worktree", "add"];
    if create_new {
        args.push("-b");
        args.push(&branch);
        args.push(&new_path);
    } else {
        args.push(&new_path);
        args.push(&branch);
    }
    let out = run_git(&repo, &args).map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn remove_worktree(repo: String, path: String, force: bool) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path);
    let out = run_git(&repo, &args).map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}
