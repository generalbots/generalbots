//!  #751 — VIBE RUN keyword and friends (VIBE STATUS / VIBE APPROVE /
//! VIBE CANCEL / VIBE TOOLS / VIBE EVENTS).
//!
//! Bridges the BASIC chat surface to the Vibe agent API running on the
//! local botserver. Every keyword performs a single HTTP call through a
//! short-lived thread + runtime, mirroring the pattern used by the other
//! botserver-local keywords (set_answer_mode, send_mail).
//!
//! The base URL comes from `VIBE_API_URL` (default `http://localhost:8080`).

use botcore::shared::state::AppState;
use botcore::shared::UserSession;
use rhai::{Dynamic, Engine};
use std::sync::Arc;

const DEFAULT_VIBE_URL: &str = "http://localhost:8080";

fn vibe_base_url() -> String {
    std::env::var("VIBE_API_URL").unwrap_or_else(|_| DEFAULT_VIBE_URL.to_string())
}

fn http_json(
    method: &'static str,
    path_and_query: String,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", vibe_base_url(), path_and_query);
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("runtime: {e}")));
                return;
            }
        };
        let result = rt.block_on(async move {
            let client = reqwest::Client::new();
            let builder = match method {
                "GET" => client.get(&url),
                "POST" => client.post(&url),
                "PUT" => client.put(&url),
                "DELETE" => client.delete(&url),
                _ => return Err(format!("unsupported method {method}")),
            };
            let builder = if let Some(b) = body {
                builder.json(&b)
            } else {
                builder
            };
            let resp = match builder.send().await {
                Ok(r) => r,
                Err(e) => return Err(format!("Vibe API request failed: {e}")),
            };
            let status = resp.status();
            let text = match resp.text().await {
                Ok(t) => t,
                Err(e) => return Err(format!("read response: {e}")),
            };
            if !status.is_success() {
                return Err(format!("Vibe API returned {status}: {text}"));
            }
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| format!("parse response: {e}"))
        });
        let _ = tx.send(result);
    });

    match rx.recv() {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(format!("channel error: {e}")),
    }
}

/// `VIBE RUN "{intent}"` — creates an agent run in the Vibe subsystem and
/// returns the run id plus the resolved use case.
pub fn register_vibe_run_keyword(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "RUN", "$expr$"],
        true,
        move |context, inputs| {
            let intent = context.eval_expression_tree(&inputs[0])?.to_string();

            let payload = serde_json::json!({
                "intent": intent,
                "auto_approve": false,
            });
            match http_json("POST", "/api/vibe/run".into(), Some(payload)) {
                Ok(v) => {
                    let run_id = v.get("run_id").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let state = v.get("state").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    log::info!("VIBE RUN created run_id={run_id} state={state}");
                    Ok(Dynamic::from(format!(
                        "Vibe run created: {run_id} (state: {state})"
                    )))
                }
                Err(e) => Err(format!("VIBE RUN failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE RUN syntax: {e}");
    }
}

/// `VIBE STATUS "{run_id}"` — read the current run state and tool results.
pub fn register_vibe_status_command(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "STATUS", "$expr$"],
        true,
        move |context, inputs| {
            let run_id = context.eval_expression_tree(&inputs[0])?.to_string();
            match http_json("GET", format!("/api/vibe/run/{run_id}"), None) {
                Ok(v) => {
                    let state = v.get("state").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let tools = v.get("tool_call_count").and_then(|x| x.as_u64()).unwrap_or(0);
                    let error = v.get("error").and_then(|x| x.as_str()).unwrap_or("");
                    Ok(Dynamic::from(format!(
                        "Vibe run {run_id}: state={state}, tools_executed={tools}{}",
                        if error.is_empty() { String::new() } else { format!(", error={error}") }
                    )))
                }
                Err(e) => Err(format!("VIBE STATUS failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE STATUS syntax: {e}");
    }
}

/// `VIBE APPROVE "{run_id}"` — approve pending tool calls of a run.
pub fn register_vibe_approve_command(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "APPROVE", "$expr$"],
        true,
        move |context, inputs| {
            let run_id = context.eval_expression_tree(&inputs[0])?.to_string();
            match http_json("POST", format!("/api/vibe/run/{run_id}/approve"), None) {
                Ok(v) => {
                    let ok = v.get("success").and_then(|x| x.as_bool()).unwrap_or(false);
                    let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("");
                    Ok(Dynamic::from(format!(
                        "Vibe approval {}: {}",
                        if ok { "accepted" } else { "rejected" },
                        msg
                    )))
                }
                Err(e) => Err(format!("VIBE APPROVE failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE APPROVE syntax: {e}");
    }
}

/// `VIBE CANCEL "{run_id}"` — cancel a running or awaiting-approval run.
pub fn register_vibe_cancel_command(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "CANCEL", "$expr$"],
        true,
        move |context, inputs| {
            let run_id = context.eval_expression_tree(&inputs[0])?.to_string();
            match http_json("POST", format!("/api/vibe/run/{run_id}/cancel"), None) {
                Ok(v) => {
                    let ok = v.get("success").and_then(|x| x.as_bool()).unwrap_or(false);
                    let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("");
                    Ok(Dynamic::from(format!(
                        "Vibe cancel {}: {}",
                        if ok { "accepted" } else { "rejected" },
                        msg
                    )))
                }
                Err(e) => Err(format!("VIBE CANCEL failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE CANCEL syntax: {e}");
    }
}

/// `VIBE TOOLS` — list the tools the agent can use.
pub fn register_vibe_tools_command(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "TOOLS"],
        true,
        move |_context, _inputs| {
            match http_json("GET", "/api/vibe/tools".into(), None) {
                Ok(v) => {
                    if let Some(tools) = v.get("tools").and_then(|x| x.as_array()) {
                        let names: Vec<&str> = tools
                            .iter()
                            .filter_map(|t| {
                                t.get("schema").and_then(|s| s.get("name")).and_then(|n| n.as_str())
                            })
                            .collect();
                        Ok(Dynamic::from(format!(
                            "Available Vibe tools ({}): {}",
                            names.len(),
                            names.join(", ")
                        )))
                    } else {
                        Ok(Dynamic::from("No Vibe tools reported."))
                    }
                }
                Err(e) => Err(format!("VIBE TOOLS failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE TOOLS syntax: {e}");
    }
}

/// `VIBE EVENTS "{run_id}"` — fetch the latest progress events of a run.
pub fn register_vibe_events_command(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let result = engine.register_custom_syntax(
        ["VIBE", "EVENTS", "$expr$"],
        true,
        move |context, inputs| {
            let run_id = context.eval_expression_tree(&inputs[0])?.to_string();
            match http_json("GET", format!("/api/vibe/events/{run_id}"), None) {
                Ok(v) => {
                    if let Some(events) = v.as_array() {
                        let summary: Vec<String> = events
                            .iter()
                            .filter_map(|e| {
                                let step = e.get("step").and_then(|x| x.as_str()).unwrap_or("?");
                                let msg = e.get("message").and_then(|x| x.as_str()).unwrap_or("");
                                Some(format!("[{step}] {msg}"))
                            })
                            .collect();
                        Ok(Dynamic::from(format!(
                            "Vibe events ({}): {}",
                            summary.len(),
                            summary.join(" | ")
                        )))
                    } else {
                        Ok(Dynamic::from("No Vibe events yet."))
                    }
                }
                Err(e) => Err(format!("VIBE EVENTS failed: {e}").into()),
            }
        },
    );
    if let Err(e) = result {
        log::error!("Failed to register VIBE EVENTS syntax: {e}");
    }
}