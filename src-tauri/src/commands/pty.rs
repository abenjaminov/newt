use parking_lot::Mutex;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use tauri::ipc::Channel;

struct Pty {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    os_pid: Option<u32>,
    label: String,
}

static REGISTRY: OnceLock<Mutex<HashMap<u32, Pty>>> = OnceLock::new();
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn registry() -> &'static Mutex<HashMap<u32, Pty>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub fn spawn_pty(
    program: String,
    args: Vec<String>,
    cwd: Option<String>,
    cols: u16,
    rows: u16,
    label: String,
    on_data: Channel<String>,
    on_exit: Channel<i32>,
) -> Result<u32, String> {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new(program);
    for a in args {
        cmd.arg(a);
    }
    if let Some(c) = cwd {
        cmd.cwd(c);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| e.to_string())?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let killer = child.clone_killer();
    let os_pid = child.process_id();

    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

    // Reader thread: pump pty bytes into the channel as UTF-8 strings.
    // Terminal output is overwhelmingly ASCII (text + ANSI escapes); the
    // occasional multi-byte UTF-8 char split across a chunk boundary becomes
    // a replacement character via lossy decoding. A stateful decoder could
    // handle this exactly but is overkill for current usage.
    let on_data_thread = on_data.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    if on_data_thread.send(s.into_owned()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Wait thread: notify exit.
    let on_exit_thread = on_exit.clone();
    std::thread::spawn(move || {
        let code = match child.wait() {
            Ok(s) => s.exit_code() as i32,
            Err(_) => -1,
        };
        let _ = on_exit_thread.send(code);
        registry().lock().remove(&id);
    });

    registry().lock().insert(
        id,
        Pty {
            master: pair.master,
            writer,
            killer,
            os_pid,
            label,
        },
    );

    Ok(id)
}

#[tauri::command]
pub fn write_pty(id: u32, data: String) -> Result<(), String> {
    let mut reg = registry().lock();
    let pty = reg.get_mut(&id).ok_or("pty not found")?;
    pty.writer
        .write_all(data.as_bytes())
        .map_err(|e| e.to_string())?;
    pty.writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn resize_pty(id: u32, cols: u16, rows: u16) -> Result<(), String> {
    let reg = registry().lock();
    let pty = reg.get(&id).ok_or("pty not found")?;
    pty.master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn kill_pty(id: u32) -> Result<(), String> {
    if let Some(mut pty) = registry().lock().remove(&id) {
        let _ = pty.killer.kill();
    }
    Ok(())
}

#[derive(Serialize)]
pub struct PtyEntry {
    pub uid: u32,
    pub os_pid: Option<u32>,
    pub label: String,
}

#[tauri::command]
pub fn list_ptys() -> Vec<PtyEntry> {
    let reg = registry().lock();
    reg.iter()
        .map(|(uid, p)| PtyEntry {
            uid: *uid,
            os_pid: p.os_pid,
            label: p.label.clone(),
        })
        .collect()
}
