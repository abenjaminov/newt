use ignore::WalkBuilder;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String, // absolute
    pub rel: String,  // relative to root
    pub name: String,
}

const DEFAULT_MAX: usize = 8000;

/// Recursively list all files under `root`, skipping anything matched by
/// .gitignore / .ignore / global excludes (via the `ignore` crate). The
/// `.git` directory is always skipped. Capped to keep the command palette
/// responsive on large repos.
#[tauri::command]
pub fn list_files(root: String, max: Option<usize>) -> Result<Vec<FileEntry>, String> {
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {}", root));
    }
    let cap = max.unwrap_or(DEFAULT_MAX);
    let mut out = Vec::with_capacity(512);

    let mut walker = WalkBuilder::new(root_path);
    walker
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(true)
        .filter_entry(|e| e.file_name() != ".git");

    for entry in walker.build() {
        if out.len() >= cap {
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let ft = match entry.file_type() {
            Some(t) => t,
            None => continue,
        };
        if !ft.is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root_path)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.push(FileEntry {
            path: path.to_string_lossy().into_owned(),
            rel,
            name,
        });
    }
    Ok(out)
}
