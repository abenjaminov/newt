use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;

/// Lowercase basenames (extension included) we recognise as project logos.
/// Any subset (e.g. "docked.png" / "docked.svg") matches as long as the stem
/// is in `LOGO_STEMS` and the extension is in `LOGO_EXTS`.
const LOGO_STEMS: &[&str] = &[
    "logo",
    "logo-light",
    "logo-dark",
    "logo-light-mode",
    "logo-dark-mode",
    "icon",
    "app-icon",
    "appicon",
    "favicon",
    "apple-touch-icon",
    "brand",
    "mark",
    "wordmark",
    "docked",
    "banner",
];

const LOGO_EXTS: &[&str] = &["svg", "png", "webp", "jpg", "jpeg", "gif"];

/// Directories to skip while walking. Mirrors the file-index skip list so we
/// don't waste time inside huge build/vendor trees.
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
    ".gradle",
    ".idea",
    ".vscode",
    "vendor",
];

const MAX_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 4;
const MAX_DIRS: usize = 400;

fn mime_for(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webp" => "image/webp",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn is_logo_file(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    let dot = lower.rfind('.')?;
    let stem = &lower[..dot];
    let ext = &lower[dot + 1..];
    if !LOGO_EXTS.contains(&ext) {
        return None;
    }
    // Match exact stem, or stem followed by a separator (like logo@2x, logo-512).
    for s in LOGO_STEMS {
        if stem == *s {
            return Some(mime_for(ext));
        }
        if stem.starts_with(s) {
            let rest = &stem[s.len()..];
            if rest.starts_with('-') || rest.starts_with('_') || rest.starts_with('.') || rest.starts_with('@') {
                return Some(mime_for(ext));
            }
        }
    }
    None
}

/// Scan the project for a likely logo image. Walks subdirectories breadth-first
/// up to MAX_DEPTH, skipping noisy build/vendor directories. Returns a base64
/// data URL for the first match.
#[tauri::command]
pub fn find_logo(path: String) -> Option<String> {
    let root = Path::new(&path);
    if !root.is_dir() {
        return None;
    }

    let mut queue: std::collections::VecDeque<(std::path::PathBuf, usize)> =
        std::collections::VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));
    let mut visited = 0usize;

    while let Some((dir, depth)) = queue.pop_front() {
        visited += 1;
        if visited > MAX_DIRS {
            break;
        }
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Two passes: first look at files in this directory; then enqueue subdirs.
        let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let ft = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ft.is_file() {
                if let Some(mime) = is_logo_file(&name) {
                    let p = entry.path();
                    if let Ok(bytes) = std::fs::read(&p) {
                        if bytes.len() <= MAX_BYTES {
                            let b64 = STANDARD.encode(&bytes);
                            return Some(format!("data:{};base64,{}", mime, b64));
                        }
                    }
                }
            } else if ft.is_dir() {
                if SKIP_DIR.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
                    continue;
                }
                subdirs.push(entry.path());
            }
        }
        if depth + 1 <= MAX_DEPTH {
            for sd in subdirs {
                queue.push_back((sd, depth + 1));
            }
        }
    }
    None
}
