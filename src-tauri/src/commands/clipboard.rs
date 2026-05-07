// Native clipboard read/write by shelling out to OS tools. Avoids any C/C++
// compile-time deps (the user's CXX env points at Android NDK clang, which
// breaks crates like `arboard`/`clipboard-win` that need to compile vswhom).
//
// Going through Rust also dodges the WebView2 permission prompt that
// `navigator.clipboard.readText()` raises on every read.

use std::io::Write;
use std::process::{Command, Stdio};

#[cfg(windows)]
fn read_clipboard_native() -> Result<String, String> {
    // PowerShell's Get-Clipboard returns text. -Raw preserves embedded newlines.
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-Clipboard -Raw",
        ])
        .creation_flags_no_window()
        .output()
        .map_err(|e| format!("powershell spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "Get-Clipboard exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    // PowerShell appends a trailing newline; strip exactly one.
    if s.ends_with("\r\n") {
        s.truncate(s.len() - 2);
    } else if s.ends_with('\n') {
        s.truncate(s.len() - 1);
    }
    Ok(s)
}

#[cfg(windows)]
fn write_clipboard_native(text: &str) -> Result<(), String> {
    // clip.exe reads stdin and copies it to the clipboard. It expects ANSI/OEM
    // by default; pipe UTF-8 — Windows users with modern locales handle it.
    let mut child = Command::new("cmd")
        .args(["/C", "clip"])
        .stdin(Stdio::piped())
        .creation_flags_no_window()
        .spawn()
        .map_err(|e| format!("clip spawn failed: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("clip stdin write failed: {e}"))?;
    }
    let status = child
        .wait()
        .map_err(|e| format!("clip wait failed: {e}"))?;
    if !status.success() {
        return Err(format!("clip exited {status}"));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn read_clipboard_native() -> Result<String, String> {
    let out = Command::new("pbpaste")
        .output()
        .map_err(|e| format!("pbpaste spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!("pbpaste exited {}", out.status));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(target_os = "macos")]
fn write_clipboard_native(text: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("pbcopy spawn failed: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("pbcopy stdin write failed: {e}"))?;
    }
    let status = child.wait().map_err(|e| format!("pbcopy wait failed: {e}"))?;
    if !status.success() {
        return Err(format!("pbcopy exited {status}"));
    }
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_clipboard_native() -> Result<String, String> {
    // Try wl-paste (Wayland), xclip, then xsel.
    for (cmd, args) in [
        ("wl-paste", vec!["--no-newline"]),
        ("xclip", vec!["-selection", "clipboard", "-o"]),
        ("xsel", vec!["--clipboard", "--output"]),
    ] {
        if let Ok(out) = Command::new(cmd).args(&args).output() {
            if out.status.success() {
                return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
            }
        }
    }
    Err("no clipboard tool found (install wl-clipboard, xclip, or xsel)".into())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn write_clipboard_native(text: &str) -> Result<(), String> {
    for (cmd, args) in [
        ("wl-copy", vec![]),
        ("xclip", vec!["-selection", "clipboard"]),
        ("xsel", vec!["--clipboard", "--input"]),
    ] {
        let mut child = match Command::new(cmd).args(&args).stdin(Stdio::piped()).spawn() {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).ok();
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }
    Err("no clipboard tool found (install wl-clipboard, xclip, or xsel)".into())
}

// Hide the spawned-process console window on Windows. No-op on other OSes.
#[cfg(windows)]
trait NoWindow {
    fn creation_flags_no_window(&mut self) -> &mut Self;
}
#[cfg(windows)]
impl NoWindow for Command {
    fn creation_flags_no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW
        self.creation_flags(0x0800_0000)
    }
}

#[tauri::command]
pub fn clipboard_read_text() -> Result<String, String> {
    read_clipboard_native()
}

#[tauri::command]
pub fn clipboard_write_text(text: String) -> Result<(), String> {
    write_clipboard_native(&text)
}
