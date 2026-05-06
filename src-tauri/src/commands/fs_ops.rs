use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct DirEntry {
    name: String,
    path: String,
    is_dir: bool,
}

#[tauri::command]
pub fn read_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let p = Path::new(&path);
    let read = std::fs::read_dir(p).map_err(|e| format!("read_dir({}): {}", path, e))?;
    let mut entries: Vec<DirEntry> = Vec::new();
    for entry in read {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let is_dir = file_type.is_dir();
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path().to_string_lossy().into_owned();
        entries.push(DirEntry { name, path, is_dir });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("read_file({}): {}", path, e))
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("write_file({}): {}", path, e))
}

#[derive(Serialize)]
pub struct ImageInfo {
    pub data_url: String,
    pub size: u64,
}

#[tauri::command]
pub fn rename_path(from: String, to: String) -> Result<(), String> {
    if Path::new(&to).exists() {
        return Err(format!("destination already exists: {}", to));
    }
    std::fs::rename(&from, &to).map_err(|e| format!("rename: {}", e))
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let meta = std::fs::metadata(p).map_err(|e| format!("delete: {}", e))?;
    if meta.is_dir() {
        std::fs::remove_dir_all(p).map_err(|e| format!("delete dir: {}", e))
    } else {
        std::fs::remove_file(p).map_err(|e| format!("delete file: {}", e))
    }
}

#[tauri::command]
pub fn create_file(path: String) -> Result<(), String> {
    if Path::new(&path).exists() {
        return Err(format!("already exists: {}", path));
    }
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create_file parent: {}", e))?;
    }
    std::fs::write(&path, "").map_err(|e| format!("create_file: {}", e))
}

#[tauri::command]
pub fn create_dir(path: String) -> Result<(), String> {
    if Path::new(&path).exists() {
        return Err(format!("already exists: {}", path));
    }
    std::fs::create_dir_all(&path).map_err(|e| format!("create_dir: {}", e))
}

#[tauri::command]
pub fn move_into(src: String, dst_dir: String) -> Result<String, String> {
    let src_path = Path::new(&src);
    let name = src_path
        .file_name()
        .ok_or_else(|| format!("invalid src: {}", src))?;
    let dst = Path::new(&dst_dir).join(name);
    if dst == src_path {
        return Ok(dst.to_string_lossy().into_owned());
    }
    if dst.exists() {
        return Err(format!("destination exists: {}", dst.display()));
    }
    std::fs::rename(&src_path, &dst).map_err(|e| format!("move: {}", e))?;
    Ok(dst.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn read_image(path: String) -> Result<ImageInfo, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("read_image({}): {}", path, e))?;
    let size = bytes.len() as u64;
    let ext = Path::new(&path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        "tiff" | "tif" => "image/tiff",
        _ => "application/octet-stream",
    };
    let b64 = STANDARD.encode(&bytes);
    Ok(ImageInfo {
        data_url: format!("data:{};base64,{}", mime, b64),
        size,
    })
}
