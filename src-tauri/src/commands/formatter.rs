use std::process::Command;
use std::time::Duration;

#[derive(serde::Serialize)]
pub struct FormatResult {
    pub ok: bool,
    pub stderr: String,
    pub stdout: String,
}

const TIMEOUT: Duration = Duration::from_secs(10);

/// Run an external formatter against `file`. The command string is split on
/// whitespace; `{file}` placeholders are replaced with the absolute path.
/// On Windows, runs through `cmd /c` so PATH-resolved tools (`prettier`, etc.)
/// work without the user knowing the absolute path. On Unix, uses `/bin/sh -c`.
#[tauri::command]
pub fn run_formatter(command: String, file: String) -> Result<FormatResult, String> {
    if command.trim().is_empty() {
        return Err("empty command".into());
    }
    let cmd = command.replace("{file}", &shell_quote(&file));

    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &cmd])
            .spawn()
            .map_err(|e| format!("spawn: {}", e))?
    } else {
        Command::new("sh")
            .args(["-c", &cmd])
            .spawn()
            .map_err(|e| format!("spawn: {}", e))?
    };

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(FormatResult {
                    ok: status.success(),
                    stderr: String::new(),
                    stdout: String::new(),
                });
            }
            Ok(None) => {
                if start.elapsed() > TIMEOUT {
                    let _ = child.kill();
                    return Err("formatter timed out (10s)".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("wait: {}", e)),
        }
    }
}

fn shell_quote(s: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
