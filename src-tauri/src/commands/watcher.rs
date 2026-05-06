use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct WatcherState(pub Mutex<Option<Debouncer<notify::RecommendedWatcher>>>);

impl Default for WatcherState {
    fn default() -> Self {
        WatcherState(Mutex::new(None))
    }
}

const SKIP_SEGMENTS: &[&str] = &[
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
    "__pycache__",
    ".gradle",
];

fn should_skip(path: &str) -> bool {
    // Look for any skip segment as a path component (handle both / and \).
    let lower = path.to_lowercase();
    for seg in SKIP_SEGMENTS {
        let with_slash = format!("/{}/", seg);
        let with_back = format!("\\{}\\", seg);
        if lower.contains(&with_slash) || lower.contains(&with_back) {
            return true;
        }
    }
    false
}

#[tauri::command]
pub fn start_watch(path: String, app: AppHandle) -> Result<(), String> {
    let state: State<WatcherState> = app.state();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None; // drop existing watcher (releases handles before creating new one)

    let app_emit = app.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(200),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                let paths: Vec<String> = events
                    .into_iter()
                    .map(|e| e.path.to_string_lossy().into_owned())
                    .filter(|p| !should_skip(p))
                    .collect();
                if !paths.is_empty() {
                    let _ = app_emit.emit("fs:changed", paths);
                }
            }
        },
    )
    .map_err(|e| e.to_string())?;

    debouncer
        .watcher()
        .watch(std::path::Path::new(&path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    *guard = Some(debouncer);
    Ok(())
}

#[tauri::command]
pub fn stop_watch(app: AppHandle) -> Result<(), String> {
    let state: State<WatcherState> = app.state();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}
