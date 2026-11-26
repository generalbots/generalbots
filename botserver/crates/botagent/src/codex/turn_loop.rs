//! The turn itself: `turn/start`, then every message until
//! `turn/completed` (see [`super::bridge`] for the sequence).

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::{json, Value};

use crate::duplex::{Incoming, RpcClient, RpcInbox, ServerRequest};

use super::app_server::{self as proto, ActionOutcome, Answer, COMMAND_APPROVAL, FILE_APPROVAL};
use super::bridge::{rpc_failure, Approvals, BridgeError, Verdict, REQUEST_WAIT};

const STOP_POLL: Duration = Duration::from_millis(500);
/// After `turn/interrupt`, how long `turn/completed` may take.
const INTERRUPT_GRACE: Duration = Duration::from_secs(15);

/// Per-turn bookkeeping of items, tickets and the answer.
struct Turn<T> {
    thread_id: String,
    turn_id: String,
    items: HashMap<String, Value>,
    asked: HashSet<String>,
    tickets: HashMap<String, Vec<T>>,
    text: String,
    streamed: HashSet<String>,
    last_error: Option<String>,
}

impl<T> Turn<T> {
    async fn push_text<H: Approvals<Ticket = T>>(&mut self, handler: &H, item_id: &str, chunk: &str, delta: bool) {
        if chunk.is_empty() {
            return;
        }
        let first = !self.streamed.contains(item_id);
        if delta {
            self.streamed.insert(item_id.to_string());
        }
        let mut piece = String::new();
        if first && !self.text.is_empty() {
            piece.push_str("\n\n");
        }
        piece.push_str(chunk);
        self.text.push_str(&piece);
        handler.text(&piece).await;
    }
}

/// Runs one turn on `thread_id` and returns the agent's answer.
pub async fn run_turn<H: Approvals>(
    client: &RpcClient,
    inbox: &mut RpcInbox,
    thread_id: &str,
    prompt: &str,
    model: &str,
    handler: &H,
) -> Result<String, BridgeError> {
    run_turn_with_images(client, inbox, thread_id, prompt, &[], model, handler).await
}

/// [`run_turn`] with attached pictures given to Codex as image inputs.
pub async fn run_turn_with_images<H: Approvals>(
    client: &RpcClient,
    inbox: &mut RpcInbox,
    thread_id: &str,
    prompt: &str,
    images: &[String],
    model: &str,
    handler: &H,
) -> Result<String, BridgeError> {
    run_turn_tracked(client, inbox, thread_id, prompt, images, model, handler).await.result
}

/// How a turn ended on its process.
pub(crate) struct TurnEnd {
    pub(crate) result: Result<String, BridgeError>,
    /// Codex accepted `turn/start`: the prompt is in the thread.
    pub(crate) started: bool,
    /// `turn/completed` arrived: the process may take another turn.
    pub(crate) completed: bool,
}

/// [`run_turn_with_images`], telling whether the process is ready for
/// another turn.
pub(crate) async fn run_turn_tracked<H: Approvals>(
    client: &RpcClient,
    inbox: &mut RpcInbox,
    thread_id: &str,
    prompt: &str,
    images: &[String],
    model: &str,
    handler: &H,
) -> TurnEnd {
    let started = match client
        .request("turn/start", proto::turn_start_with_images(thread_id, prompt, images, model), REQUEST_WAIT)
        .await
    {
        Ok(started) => started,
        Err(error) => return TurnEnd { result: Err(rpc_failure("turn/start", &error)), started: false, completed: false },
    };
    let ended = |result, completed| TurnEnd { result, started: true, completed };
    let mut turn: Turn<H::Ticket> = Turn {
        thread_id: thread_id.to_string(),
        turn_id: proto::turn_id_of(&started).unwrap_or_else(|| {
            tracing::debug!("Codex turn/start answered without an id; using an empty id");
            String::new()
        }),
        items: HashMap::new(),
        asked: HashSet::new(),
        tickets: HashMap::new(),
        text: String::new(),
        streamed: HashSet::new(),
        last_error: None,
    };
    let mut stop_tick = tokio::time::interval(STOP_POLL);
    let mut interrupted: Option<tokio::time::Instant> = None;
    loop {
        let next = tokio::select! {
            next = inbox.next() => next,
            _ = stop_tick.tick() => {
                if let Some(at) = interrupted {
                    if at.elapsed() > INTERRUPT_GRACE {
                        return ended(Ok(turn.text), false);
                    }
                } else if handler.stopped() {
                    interrupt(client, &turn).await;
                    interrupted = Some(tokio::time::Instant::now());
                }
                continue;
            }
        };
        match next {
            None | Some(Incoming::Ended(_)) => {
                return ended(Err(BridgeError::failed("codex app-server ended before the turn completed")), false)
            }
            Some(Incoming::Unparsed(_)) => {}
            Some(Incoming::Request(request)) => {
                if let Err(error) = approve(client, request, &mut turn, handler).await {
                    return ended(Err(error), false);
                }
            }
            Some(Incoming::Notification { method, params }) => {
                if let Some(done) = notified(client, &method, &params, &mut turn, handler, interrupted.is_some()).await {
                    let completed = method == "turn/completed";
                    return ended(done, completed);
                }
            }
        }
    }
}

async fn interrupt<T>(client: &RpcClient, turn: &Turn<T>) {
    let params = json!({"threadId": turn.thread_id, "turnId": turn.turn_id});
    if let Err(error) = client.request("turn/interrupt", params, REQUEST_WAIT).await {
        tracing::info!("codex bridge: turn/interrupt not acknowledged: {error}");
    }
}

/// Waits for `decision` unless the session ends first; dropping the
/// decision withdraws a card that is still open.
async fn until_ended<F: Future>(client: &RpcClient, decision: F) -> Result<F::Output, BridgeError> {
    tokio::select! {
        decided = decision => Ok(decided),
        _ = client.sender().wait_ended() => {
            Err(BridgeError::failed("codex app-server ended while a permission request was open"))
        }
    }
}

fn answer_of<T>(verdict: &Verdict<T>) -> Answer {
    match verdict {
        Verdict::Allow(_) => Answer::Accept,
        Verdict::Decline => Answer::Decline,
        Verdict::Cancel => Answer::Cancel,
    }
}

async fn approve<H: Approvals>(
    client: &RpcClient,
    request: ServerRequest,
    turn: &mut Turn<H::Ticket>,
    handler: &H,
) -> Result<(), BridgeError> {
    let item_id = request.params.get("itemId").and_then(Value::as_str).unwrap_or("").to_string();
    turn.asked.insert(item_id.clone());
    let answer = if request.method == COMMAND_APPROVAL {
        let command = proto::shown_command(request.params.get("command").and_then(Value::as_str).unwrap_or(""));
        if command.is_empty() {
            Answer::Decline
        } else {
            let cwd = request.params.get("cwd").and_then(Value::as_str);
            let verdict = until_ended(client, handler.command(command, cwd)).await?;
            let answer = answer_of(&verdict);
            if let Verdict::Allow(ticket) = verdict {
                turn.tickets.entry(item_id).or_default().push(ticket);
            }
            answer
        }
    } else if request.method == FILE_APPROVAL {
        approve_files(client, &item_id, turn, handler).await?
    } else {
        Answer::Decline
    };
    if let Err(error) = request.respond(answer.result()) {
        tracing::info!("codex bridge: approval answer not delivered: {error}");
    }
    Ok(())
}

/// Every path of the change must be allowed; the first refusal declines the
/// whole change and closes the steps of the paths allowed before it.
async fn approve_files<H: Approvals>(
    client: &RpcClient,
    item_id: &str,
    turn: &mut Turn<H::Ticket>,
    handler: &H,
) -> Result<Answer, BridgeError> {
    let changes = turn.items.get(item_id).map(proto::file_changes).unwrap_or_else(|| {
        tracing::debug!("Codex file change {item_id} is not in the turn; no paths to approve");
        Vec::new()
    });
    if changes.is_empty() {
        tracing::warn!("codex bridge: file change without known paths declined");
        return Ok(Answer::Decline);
    }
    let mut allowed = Vec::new();
    for change in &changes {
        match until_ended(client, handler.file(&change.path, change.delete)).await? {
            Verdict::Allow(ticket) => allowed.push((ticket, change.path.clone())),
            refused => {
                for (ticket, path) in allowed {
                    handler.finished(ticket, ActionOutcome::not_run(&path)).await;
                }
                return Ok(answer_of(&refused));
            }
        }
    }
    turn.tickets
        .entry(item_id.to_string())
        .or_default()
        .extend(allowed.into_iter().map(|(ticket, _)| ticket));
    Ok(Answer::Accept)
}

fn is_action(item: &Value) -> bool {
    matches!(item.get("type").and_then(Value::as_str), Some("commandExecution" | "fileChange"))
}

/// Handles one notification; `Some` ends the turn with that result.
async fn notified<H: Approvals>(
    client: &RpcClient,
    method: &str,
    params: &Value,
    turn: &mut Turn<H::Ticket>,
    handler: &H,
    stopped: bool,
) -> Option<Result<String, BridgeError>> {
    let item = params.get("item").cloned().unwrap_or(Value::Null);
    let item_id = item.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    match method {
        "item/started" if is_action(&item) => {
            turn.items.insert(item_id, item);
        }
        "item/agentMessage/delta" => {
            let id = params.get("itemId").and_then(Value::as_str).unwrap_or("").to_string();
            let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
            turn.push_text(handler, &id, delta, true).await;
        }
        "item/reasoning/summaryTextDelta" | "item/reasoning/textDelta" => {
            if let Some(delta) = params.get("delta").and_then(Value::as_str).filter(|d| !d.is_empty()) {
                handler.reasoning(delta).await;
            }
        }
        "item/completed" if item.get("type").and_then(Value::as_str) == Some("agentMessage") => {
            if !turn.streamed.contains(&item_id) {
                let full = item.get("text").and_then(Value::as_str).unwrap_or("").to_string();
                turn.push_text(handler, &item_id, &full, false).await;
            }
        }
        "item/completed" if is_action(&item) => {
            let status = item.get("status").and_then(Value::as_str).unwrap_or("");
            if let Some(tickets) = turn.tickets.remove(&item_id) {
                let mut outcome = ActionOutcome::of_item(&item);
                if outcome.files.is_empty() {
                    outcome.files = turn.items.get(&item_id).map(proto::file_changes).unwrap_or_else(|| {
                        tracing::debug!("Codex file change {item_id} is not in the turn; no paths to report");
                        Vec::new()
                    })
                        .into_iter().map(|change| change.path).collect();
                }
                for ticket in tickets {
                    handler.finished(ticket, outcome.clone()).await;
                }
            } else if !turn.asked.contains(&item_id) && status != "declined" {
                tracing::error!("codex bridge: an action finished without asking (status {status}); turn stopped");
                interrupt(client, turn).await;
                return Some(Err(BridgeError::Unasked));
            }
        }
        "item/completed" => {
            if let Some((kind, text, ok)) = super::unasked::noted_step(&item) {
                handler.noted(kind, &text, ok).await;
            }
        }
        "error" => {
            let retrying = params.get("willRetry").and_then(Value::as_bool).unwrap_or(false);
            let message = params.get("error").and_then(|e| e.get("message")).and_then(Value::as_str);
            if let (false, Some(message)) = (retrying, message) {
                turn.last_error = Some(message.to_string());
            }
        }
        "turn/completed" => return Some(finish(params.get("turn").unwrap_or(&Value::Null), turn, stopped)),
        _ => {}
    }
    None
}

fn finish<T>(done: &Value, turn: &mut Turn<T>, stopped: bool) -> Result<String, BridgeError> {
    let text = std::mem::take(&mut turn.text);
    match done.get("status").and_then(Value::as_str).unwrap_or("") {
        "completed" => Ok(text),
        "interrupted" if stopped || !text.trim().is_empty() => Ok(text),
        "interrupted" => Err(BridgeError::Failed(
            StatusCode::REQUEST_TIMEOUT,
            "Codex stopped: a permission request was not answered".to_string(),
        )),
        _ => {
            let reason = proto::turn_failure(done, turn.last_error.as_deref());
            match crate::cli_turn::plan_limit("", &reason) {
                Some((code, message)) => Err(BridgeError::Failed(code, message)),
                None => Err(BridgeError::failed(format!("codex turn failed: {reason}"))),
            }
        }
    }
}
