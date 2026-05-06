use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ShellInfo {
    pub id: String,           // unique identifier ("cmd", "pwsh", "wsl:Ubuntu", etc.)
    pub label: String,        // human-readable name shown in tab "+" menu
    pub program: String,      // executable path/name to spawn
    pub args: Vec<String>,    // initial args
    pub kind: String,         // "cmd" | "powershell" | "pwsh" | "wsl" | "posix"
    pub default: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_shells() -> Vec<ShellInfo> {
    let mut out: Vec<ShellInfo> = Vec::new();

    // PowerShell 7+ (pwsh) — preferred default if installed.
    if which("pwsh.exe").is_some() {
        out.push(ShellInfo {
            id: "pwsh".into(),
            label: "PowerShell".into(),
            program: "pwsh.exe".into(),
            args: vec!["-NoLogo".into()],
            kind: "pwsh".into(),
            default: true,
        });
    }

    // Built-in Windows PowerShell 5.x.
    out.push(ShellInfo {
        id: "powershell".into(),
        label: "Windows PowerShell".into(),
        program: "powershell.exe".into(),
        args: vec!["-NoLogo".into()],
        kind: "powershell".into(),
        default: !out.iter().any(|s| s.default),
    });

    // cmd.exe always available.
    out.push(ShellInfo {
        id: "cmd".into(),
        label: "Command Prompt".into(),
        program: "cmd.exe".into(),
        args: vec![],
        kind: "cmd".into(),
        default: false,
    });

    // WSL distros, decoded from UTF-16LE output of `wsl.exe -l -q`.
    for distro in list_wsl_distros() {
        out.push(ShellInfo {
            id: format!("wsl:{}", distro),
            label: format!("{} (WSL)", distro),
            program: "wsl.exe".into(),
            args: vec!["-d".into(), distro.clone()],
            kind: "wsl".into(),
            default: false,
        });
    }

    out
}

#[cfg(target_os = "windows")]
fn list_wsl_distros() -> Vec<String> {
    use std::process::Command;
    let output = match Command::new("wsl.exe").args(["-l", "-q"]).output() {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    decode_utf16le(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(target_os = "windows")]
fn decode_utf16le(bytes: &[u8]) -> String {
    // wsl.exe emits UTF-16LE without a BOM marker (sometimes with).
    let start = if bytes.starts_with(&[0xFF, 0xFE]) { 2 } else { 0 };
    let payload = &bytes[start..];
    let mut units = Vec::with_capacity(payload.len() / 2);
    for chunk in payload.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units).replace('\0', "")
}

#[cfg(target_os = "windows")]
fn which(name: &str) -> Option<String> {
    use std::env;
    use std::path::PathBuf;
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate: PathBuf = dir.join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn list_shells() -> Vec<ShellInfo> {
    use std::env;
    let mut out: Vec<ShellInfo> = Vec::new();

    let user_default = env::var("SHELL").ok().filter(|s| !s.is_empty());

    let mut shells: Vec<String> = std::fs::read_to_string("/etc/shells")
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    // Promote $SHELL to the front if not already in the list.
    if let Some(s) = &user_default {
        if !shells.iter().any(|x| x == s) {
            shells.insert(0, s.clone());
        }
    }

    for path in shells {
        let label = path.rsplit('/').next().unwrap_or(&path).to_string();
        let is_default = user_default.as_deref() == Some(path.as_str());
        out.push(ShellInfo {
            id: path.clone(),
            label,
            program: path,
            args: vec![],
            kind: "posix".into(),
            default: is_default,
        });
    }

    if out.is_empty() {
        out.push(ShellInfo {
            id: "/bin/sh".into(),
            label: "sh".into(),
            program: "/bin/sh".into(),
            args: vec![],
            kind: "posix".into(),
            default: true,
        });
    } else if !out.iter().any(|s| s.default) {
        out[0].default = true;
    }

    out
}
