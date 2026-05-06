use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String, // absolute
    pub rel: String,  // relative to root
    pub name: String,
}

const SKIP_DIR: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".cache",
    ".venv",
    "venv",
    "__pycache__",
    ".pytest_cache",
    ".gradle",
    ".idea",
    ".vscode",
    "vendor",
];

const DEFAULT_MAX: usize = 8000;

/// Recursively list all files under `root`, skipping common build/vendor
/// directories. Capped to keep the command palette responsive.
#[tauri::command]
pub fn list_files(root: String, max: Option<usize>) -> Result<Vec<FileEntry>, String> {
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {}", root));
    }
    let cap = max.unwrap_or(DEFAULT_MAX);
    let mut out = Vec::with_capacity(512);
    walk(root_path, root_path, &mut out, cap);
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<FileEntry>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        if out.len() >= cap {
            return;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if SKIP_DIR.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
                continue;
            }
            walk(root, &entry.path(), out, cap);
        } else if file_type.is_file() {
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            out.push(FileEntry {
                path: path.to_string_lossy().into_owned(),
                rel,
                name,
            });
        }
    }
}
