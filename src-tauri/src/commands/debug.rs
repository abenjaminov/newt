// Tauri commands for the DAP-based debug engine.
//
// Phase 1: launch / set-breakpoints / control / inspect. Events from the
// adapter (stopped, output, terminated) are forwarded to the frontend via
// a Tauri event named `dap:event:<session_id>`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::debug::{DebugRegistry, Session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLaunchArgs {
    /// Adapter command (e.g. "python", "node", "lldb-dap").
    pub adapter_program: String,
    /// Adapter arguments (e.g. ["-m", "debugpy.adapter"]).
    #[serde(default)]
    pub adapter_args: Vec<String>,
    /// Adapter id passed in the `initialize` request body. e.g. "debugpy".
    pub adapter_id: String,
    /// The full DAP `launch` (or `attach`) request body — passed through to
    /// the adapter as-is. The exact shape is adapter-specific.
    pub launch_args: Value,
    /// Whether to use `attach` instead of `launch` for the post-initialize
    /// request.
    #[serde(default)]
    pub attach: bool,
}

#[tauri::command]
pub async fn debug_launch(
    app: AppHandle,
    state: State<'_, DebugRegistry>,
    args: DebugLaunchArgs,
) -> Result<u32, String> {
    let session = Session::spawn(&args.adapter_program, &args.adapter_args)
        .await
        .map_err(|e| e.to_string())?;

    // initialize.
    session
        .request(
            "initialize",
            Some(json!({
                "clientID": "newt",
                "clientName": "Newt IDE",
                "adapterID": args.adapter_id,
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "supportsRunInTerminalRequest": false,
                "pathFormat": "path",
            })),
        )
        .await
        .map_err(|e| e.to_string())?;

    // launch / attach with adapter-specific args.
    let cmd = if args.attach { "attach" } else { "launch" };
    session
        .request(cmd, Some(args.launch_args))
        .await
        .map_err(|e| e.to_string())?;

    let id = state.insert(session);

    // Pull this session's events receiver out and spawn a forwarder task that
    // emits Tauri events to the frontend.
    let session = state.get(id).ok_or_else(|| "session vanished".to_string())?;
    let rx = session.events_rx.lock().take();
    if let Some(mut rx) = rx {
        let app = app.clone();
        tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                let _ = app.emit(
                    &format!("dap:event:{id}"),
                    json!({ "event": ev.event, "body": ev.body }),
                );
            }
        });
    }

    Ok(id)
}

#[tauri::command]
pub async fn debug_configuration_done(
    state: State<'_, DebugRegistry>,
    id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("configurationDone", None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBreakpointsArgs {
    pub source_path: String,
    /// Lines (1-based) to set breakpoints on.
    pub lines: Vec<u32>,
}

#[tauri::command]
pub async fn debug_set_breakpoints(
    state: State<'_, DebugRegistry>,
    id: u32,
    args: SetBreakpointsArgs,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let bps: Vec<Value> = args.lines.iter().map(|l| json!({ "line": l })).collect();
    let body = session
        .request(
            "setBreakpoints",
            Some(json!({
                "source": { "path": args.source_path },
                "breakpoints": bps,
            })),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_continue(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("continue", Some(json!({ "threadId": thread_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn debug_step_over(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("next", Some(json!({ "threadId": thread_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn debug_step_in(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("stepIn", Some(json!({ "threadId": thread_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn debug_step_out(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("stepOut", Some(json!({ "threadId": thread_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn debug_pause(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    session
        .request("pause", Some(json!({ "threadId": thread_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn debug_threads(
    state: State<'_, DebugRegistry>,
    id: u32,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let body = session
        .request("threads", None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_stack_trace(
    state: State<'_, DebugRegistry>,
    id: u32,
    thread_id: u32,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let body = session
        .request(
            "stackTrace",
            Some(json!({ "threadId": thread_id, "startFrame": 0, "levels": 100 })),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_scopes(
    state: State<'_, DebugRegistry>,
    id: u32,
    frame_id: u32,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let body = session
        .request("scopes", Some(json!({ "frameId": frame_id })))
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_variables(
    state: State<'_, DebugRegistry>,
    id: u32,
    variables_reference: u32,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let body = session
        .request(
            "variables",
            Some(json!({ "variablesReference": variables_reference })),
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_evaluate(
    state: State<'_, DebugRegistry>,
    id: u32,
    expression: String,
    frame_id: Option<u32>,
    context: Option<String>,
) -> Result<Value, String> {
    let session = sess(&state, id)?;
    let mut args = json!({ "expression": expression });
    if let Some(f) = frame_id {
        args["frameId"] = json!(f);
    }
    if let Some(c) = context {
        args["context"] = json!(c);
    }
    let body = session
        .request("evaluate", Some(args))
        .await
        .map_err(|e| e.to_string())?;
    Ok(body.unwrap_or(Value::Null))
}

#[tauri::command]
pub async fn debug_terminate(
    state: State<'_, DebugRegistry>,
    id: u32,
) -> Result<(), String> {
    let session = sess(&state, id)?;
    let _ = session.request("disconnect", Some(json!({ "terminateDebuggee": true }))).await;
    state.remove(id);
    Ok(())
}

#[tauri::command]
pub fn debug_list_sessions(state: State<'_, DebugRegistry>) -> Vec<u32> {
    state.list()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectArgs {
    pub thread_id: u32,
    /// Number of frames to include (default 5). Frames are 0-based, top first.
    #[serde(default = "default_frame_count")]
    pub frame_count: u32,
    /// Maximum depth to expand variable trees (default 2 — locals + one
    /// level of structure). Higher values risk huge payloads on objects with
    /// large fields. The LLM can drill in further with `debug_variables`.
    #[serde(default = "default_var_depth")]
    pub max_var_depth: u32,
    /// Cap on children expanded per container (default 50). Adapters can
    /// return enormous arrays; this keeps payloads manageable.
    #[serde(default = "default_max_children")]
    pub max_children_per_scope: u32,
    /// Optional watch expressions to evaluate against the top frame.
    #[serde(default)]
    pub watches: Vec<String>,
}

fn default_frame_count() -> u32 {
    5
}
fn default_var_depth() -> u32 {
    2
}
fn default_max_children() -> u32 {
    50
}

/// One-shot snapshot of a stopped session: stack frames, each frame's scopes
/// expanded to a bounded depth, and any watch expressions evaluated against
/// the top frame. Designed for LLM consumption — a single call returns
/// everything needed to "see" the stop point.
#[tauri::command]
pub async fn debug_inspect(
    state: State<'_, DebugRegistry>,
    id: u32,
    args: InspectArgs,
) -> Result<Value, String> {
    let session = sess(&state, id)?;

    // 1. Stack trace.
    let stack = session
        .request(
            "stackTrace",
            Some(json!({
                "threadId": args.thread_id,
                "startFrame": 0,
                "levels": args.frame_count,
            })),
        )
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or(Value::Null);

    let frames: Vec<Value> = stack
        .get("stackFrames")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // 2. For each frame, fetch scopes + expand variables.
    let mut frame_snapshots = Vec::with_capacity(frames.len());
    for frame in &frames {
        let frame_id = frame.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let scopes_body = session
            .request("scopes", Some(json!({ "frameId": frame_id })))
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or(Value::Null);
        let raw_scopes: Vec<Value> = scopes_body
            .get("scopes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut scopes = Vec::with_capacity(raw_scopes.len());
        for scope in raw_scopes {
            let name = scope
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let var_ref = scope
                .get("variablesReference")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let expensive = scope
                .get("expensive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            // Skip scopes the adapter marks as expensive — typically Globals
            // on Python; the LLM can still drill in explicitly.
            let vars = if var_ref == 0 || expensive {
                Vec::new()
            } else {
                expand_variables(
                    &session,
                    var_ref,
                    args.max_var_depth,
                    args.max_children_per_scope,
                )
                .await?
            };
            scopes.push(json!({
                "name": name,
                "expensive": expensive,
                "variables_reference": var_ref,
                "variables": vars,
            }));
        }

        frame_snapshots.push(json!({
            "id": frame_id,
            "name": frame.get("name").cloned().unwrap_or(Value::Null),
            "source": frame.get("source").cloned().unwrap_or(Value::Null),
            "line": frame.get("line").cloned().unwrap_or(Value::Null),
            "column": frame.get("column").cloned().unwrap_or(Value::Null),
            "scopes": scopes,
        }));
    }

    // 3. Evaluate watches against the top frame.
    let top_frame_id = frames
        .first()
        .and_then(|f| f.get("id"))
        .and_then(|v| v.as_u64())
        .map(|n| n as u32);
    let mut watches = Vec::with_capacity(args.watches.len());
    for expr in &args.watches {
        let mut eval_args = json!({ "expression": expr, "context": "watch" });
        if let Some(f) = top_frame_id {
            eval_args["frameId"] = json!(f);
        }
        let r = session.request("evaluate", Some(eval_args)).await;
        match r {
            Ok(body) => watches.push(json!({
                "expression": expr,
                "ok": true,
                "result": body.unwrap_or(Value::Null),
            })),
            Err(e) => watches.push(json!({
                "expression": expr,
                "ok": false,
                "error": e.to_string(),
            })),
        }
    }

    Ok(json!({
        "thread_id": args.thread_id,
        "frames": frame_snapshots,
        "watches": watches,
    }))
}

/// Recursively expand a `variables` reference to `depth` levels deep,
/// capped at `max_children` per container. Returns a list of variable
/// snapshots; nested containers carry their own `children` list.
fn expand_variables<'a>(
    session: &'a Session,
    var_ref: u32,
    depth: u32,
    max_children: u32,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Value>, String>> + Send + 'a>> {
    Box::pin(async move {
        let body = session
            .request(
                "variables",
                Some(json!({ "variablesReference": var_ref })),
            )
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or(Value::Null);
        let raw: Vec<Value> = body
            .get("variables")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let total = raw.len();
        let n = (total as u32).min(max_children) as usize;
        let mut out = Vec::with_capacity(n);
        for v in raw.into_iter().take(n) {
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let value = v
                .get("value")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let var_type = v
                .get("type")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            let child_ref = v
                .get("variablesReference")
                .and_then(|x| x.as_u64())
                .unwrap_or(0) as u32;
            let children = if child_ref > 0 && depth > 0 {
                expand_variables(session, child_ref, depth - 1, max_children).await?
            } else {
                Vec::new()
            };
            out.push(json!({
                "name": name,
                "value": value,
                "type": var_type,
                "variables_reference": child_ref,
                "children": children,
                // The LLM can call debug_variables(child_ref) to keep going.
                "truncated_descendants": child_ref > 0 && depth == 0,
            }));
        }
        if n < total {
            // Trailing marker so the LLM knows there are more siblings to
            // page through with debug_variables(parent_ref, start=n, count=…).
            out.push(json!({
                "name": "…",
                "value": format!("(showing {} of {})", n, total),
                "type": null,
                "variables_reference": var_ref,
                "children": [],
                "truncated_siblings": true,
            }));
        }
        Ok(out)
    })
}

fn sess(state: &State<'_, DebugRegistry>, id: u32) -> Result<Arc<Session>, String> {
    state.get(id).ok_or_else(|| format!("debug session {id} not found"))
}
