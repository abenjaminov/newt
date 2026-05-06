// In-memory registry of live debug sessions. The Tauri command layer holds a
// `DebugRegistry` in `tauri::State`; each `debug_*` command looks up its
// target session by `id`.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use super::session::Session;

#[derive(Default)]
pub struct DebugRegistry {
    sessions: Mutex<HashMap<u32, Arc<Session>>>,
    next_id: AtomicU32,
}

impl DebugRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: AtomicU32::new(1),
        }
    }

    pub fn insert(&self, session: Session) -> u32 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.sessions.lock().insert(id, Arc::new(session));
        id
    }

    pub fn get(&self, id: u32) -> Option<Arc<Session>> {
        self.sessions.lock().get(&id).cloned()
    }

    pub fn remove(&self, id: u32) -> Option<Arc<Session>> {
        self.sessions.lock().remove(&id)
    }

    pub fn list(&self) -> Vec<u32> {
        self.sessions.lock().keys().copied().collect()
    }
}
