// A single debug session: owns a spawned adapter process and a tokio task
// that reads DAP messages from its stdout. Requests are sent over a channel
// and correlated to responses by sequence number.

use parking_lot::Mutex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use super::transport::{read_message, write_message};

/// One DAP event observed from the adapter — surfaced to higher layers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DapEvent {
    pub event: String,
    pub body: Option<Value>,
}

#[derive(Debug)]
pub enum SessionError {
    Io(std::io::Error),
    NoResponse,
    Timeout(Duration),
    Adapter(String),
    Closed,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::NoResponse => write!(f, "adapter response missing"),
            Self::Timeout(d) => write!(f, "request timed out after {d:?}"),
            Self::Adapter(m) => write!(f, "adapter returned error: {m}"),
            Self::Closed => write!(f, "session closed"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<std::io::Error> for SessionError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub struct Session {
    seq: AtomicI64,
    pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Result<Option<Value>, String>>>>>,
    write_tx: mpsc::Sender<Value>,
    /// Channel of DAP events. The owner of the session drains this and
    /// dispatches further (e.g. forwards to the frontend via Tauri events).
    pub events_rx: Mutex<Option<mpsc::Receiver<DapEvent>>>,
    /// Kept alive so dropping the session kills the adapter.
    _child: Arc<Mutex<Option<Child>>>,
}

impl Session {
    /// Spawn an adapter and return a session ready to send DAP requests.
    ///
    /// `program` and `args` are the adapter command, e.g. `python` /
    /// `["-m", "debugpy.adapter"]` or `node` / `["path/to/dapDebugServer.js"]`.
    pub async fn spawn(program: &str, args: &[String]) -> Result<Self, SessionError> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| {
            SessionError::Io(std::io::Error::other("no stdin on adapter"))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            SessionError::Io(std::io::Error::other("no stdout on adapter"))
        })?;

        let pending: Arc<
            Mutex<HashMap<i64, oneshot::Sender<Result<Option<Value>, String>>>>,
        > = Arc::new(Mutex::new(HashMap::new()));
        let (events_tx, events_rx) = mpsc::channel::<DapEvent>(256);
        let (write_tx, mut write_rx) = mpsc::channel::<Value>(256);

        // Reader task: parses messages and dispatches responses / events.
        let pending_r = pending.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let msg = match read_message(&mut reader).await {
                    Ok(v) => v,
                    Err(_) => break,
                };
                let kind = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
                match kind {
                    "response" => {
                        let req_seq = msg
                            .get("request_seq")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(-1);
                        let success = msg
                            .get("success")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let body = msg.get("body").cloned();
                        let err_msg = msg
                            .get("message")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(tx) = pending_r.lock().remove(&req_seq) {
                            let result = if success {
                                Ok(body)
                            } else {
                                Err(err_msg.unwrap_or_else(|| "request failed".into()))
                            };
                            let _ = tx.send(result);
                        }
                    }
                    "event" => {
                        let event = msg
                            .get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let body = msg.get("body").cloned();
                        let _ = events_tx.send(DapEvent { event, body }).await;
                    }
                    "request" => {
                        // Reverse requests (e.g. runInTerminal) — not handled
                        // in phase 1. Adapter will retry / fail gracefully.
                    }
                    _ => {}
                }
            }
        });

        // Writer task: serializes outbound messages onto the adapter's stdin.
        let mut stdin: ChildStdin = stdin;
        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                if write_message(&mut stdin, &msg).await.is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            seq: AtomicI64::new(1),
            pending,
            write_tx,
            events_rx: Mutex::new(Some(events_rx)),
            _child: Arc::new(Mutex::new(Some(child))),
        })
    }

    /// Send a DAP request and wait for the matching response (with a default
    /// 10 s timeout).
    pub async fn request(
        &self,
        command: &str,
        arguments: Option<Value>,
    ) -> Result<Option<Value>, SessionError> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let mut msg = json!({
            "seq": seq,
            "type": "request",
            "command": command,
        });
        if let Some(args) = arguments {
            msg["arguments"] = args;
        }

        let (tx, rx) = oneshot::channel();
        self.pending.lock().insert(seq, tx);

        if self.write_tx.send(msg).await.is_err() {
            self.pending.lock().remove(&seq);
            return Err(SessionError::Closed);
        }

        match timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(body))) => Ok(body),
            Ok(Ok(Err(msg))) => Err(SessionError::Adapter(msg)),
            Ok(Err(_)) => Err(SessionError::NoResponse),
            Err(_) => {
                self.pending.lock().remove(&seq);
                Err(SessionError::Timeout(Duration::from_secs(10)))
            }
        }
    }
}
