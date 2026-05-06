// DAP-based debugging engine.
//
// Phase 1 scope: spawn a single adapter (e.g. debugpy), drive it through
// initialize/launch/setBreakpoints/configurationDone, and surface events
// (stopped, output, terminated) to the frontend via Tauri events.
//
// Phase 2 will add an MCP server that calls into this same module so LLMs
// see the same session state as the local UI.

pub mod registry;
pub mod session;
pub mod transport;
pub mod types;

pub use registry::DebugRegistry;
pub use session::Session;
#[allow(unused_imports)]
pub use session::{DapEvent, SessionError};
