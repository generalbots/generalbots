//! One prompt on an open ACP session: `session/prompt` runs while the
//! session's updates and permission requests are handled, until the prompt
//! answers with its stop reason.
//!
//! - `agent_message_chunk` → [`AcpGate::text`], `agent_thought_chunk` →
//!   [`AcpGate::thought`], as they arrive;
//! - `session/request_permission` → [`AcpGate::decide`] per action, answered
//!   with the agent's own "once" option (or `cancelled`);
//! - a finished tool call that was allowed → [`AcpGate::finished`];
//! - a web search or fetch that completes without having asked →
//!   [`AcpGate::noted`];
//! - a tool call that changes or runs something and completes without
//!   having asked ends the turn as unsafe.
//!
//! The person stopping the turn sends `session/cancel`. A session that stays
//! silent for [`QUIET_LIMIT`] while no card is open ends the turn. What the
//! session does is logged by [`super::trace`].

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::{json, Value};

use crate::codex::app_server::ActionOutcome;
use crate::codex::Verdict;
use crate::duplex::{Incoming, RpcClient, RpcError, RpcInbox, ServerRequest};

use super::proto::{self, AcpAction, AgentInfo, Choice, ImagePart, TextResource, Update, PERMISSION_REQUEST};
use super::{trace, AcpError};

const STOP_POLL: Duration = Duration::from_millis(500);
/// JSON-RPC "method not found".
const METHOD_NOT_FOUND: i64 = -32601;
/// After `session/cancel`, how long the prompt may take to answer.
const CANCEL_GRACE: Duration = Duration::from_secs(15);
/// Longest silence of the agent (no update, no request) while no card waits.
pub const QUIET_LIMIT: Duration = Duration::from_secs(5 * 60);

/// The server side of the bridge: the permission gate and the answer.
pub trait AcpGate: Send + Sync {
    type Ticket: Send;

    fn decide(&self, action: &AcpAction) -> impl Future<Output = Verdict<Self::Ticket>> + Send;
    fn finished(&self, ticket: Self::Ticket, outcome: ActionOutcome) -> impl Future<Output = ()> + Send;
    /// A web search or page read that finished without asking, shown as a step.
    fn noted(&self, action: &AcpAction, ok: bool) -> impl Future<Output = ()> + Send;
    /// A piece of the answer, in order.
    fn text(&self, chunk: &str) -> impl Future<Output = ()> + Send;
    /// A piece of the agent's reasoning, in order.
    fn thought(&self, chunk: &str) -> impl Future<Output = ()> + Send;
    /// The person stopped the turn (or nobody reads the answer any more).
    fn stopped(&self) -> bool;
}

/// What one prompt sends.
pub(super) struct PromptInput<'a> {
    pub session_id: &'a str,
    pub text: &'a str,
    pub images: &'a [ImagePart],
    pub resources: &'a [TextResource],
    pub info: &'a AgentInfo,
}

struct Turn<T> {
    session_id: String,
    /// What the updates said about each tool call, and when it appeared.
    calls: HashMap<String, (tokio::time::Instant, Value)>,
    asked: HashSet<String>,
    warned: HashSet<String>,
    notified: HashSet<String>,
    tickets: HashMap<String, Vec<(T, Vec<String>)>>,
    text: String,
    cancelled: bool,
}

fn failed(message: impl Into<String>) -> AcpError {
    AcpError::Failed(StatusCode::INTERNAL_SERVER_ERROR, message.into())
}

/// Runs one prompt and returns the answer.
pub(super) async fn run_prompt<H: AcpGate>(
    client: &RpcClient,
    inbox: &mut RpcInbox,
    input: PromptInput<'_>,
    handler: &H,
) -> Result<String, AcpError> {
    let params = proto::prompt_params(input.session_id, input.text, input.images, input.resources, input.info);
    let prompt = client.request("session/prompt", params, super::BRIDGE_DEADLINE);
    tokio::pin!(prompt);
    let mut turn: Turn<H::Ticket> = Turn {
        session_id: input.session_id.to_string(),
        calls: HashMap::new(),
        asked: HashSet::new(),
        warned: HashSet::new(),
        notified: HashSet::new(),
        tickets: HashMap::new(),
        text: String::new(),
        cancelled: false,
    };
    let mut stop_tick = tokio::time::interval(STOP_POLL);
    let mut cancelled_at: Option<tokio::time::Instant> = None;
    let started = tokio::time::Instant::now();
    let mut last_heard = started;
    tracing::info!("antigravity bridge: session/prompt sent");
    loop {
        tokio::select! {
            biased;
            next = inbox.next() => {
                match next {
                    None | Some(Incoming::Ended(_)) => return Err(failed("agy acp ended before the answer was complete")),
                    Some(Incoming::Unparsed(_)) => {}
                    Some(Incoming::Request(request)) => permission(client, request, &mut turn, handler).await?,
                    Some(Incoming::Notification { method, params }) => {
                        if method == "session/update" {
                            updated(client, &params, &mut turn, handler).await?;
                        } else {
                            trace::other_notification(&mut turn.notified, &method);
                        }
                    }
                }
                // Counted after the message: time on an open card is the
                // person's, not the agent's silence.
                last_heard = tokio::time::Instant::now();
            }
            answered = &mut prompt => {
                while let Some(message) = inbox.try_next() {
                    if let Incoming::Notification { method, params } = message {
                        if method == "session/update" {
                            updated(client, &params, &mut turn, handler).await?;
                        }
                    }
                }
                tracing::info!(
                    "antigravity bridge: session/prompt answered after {} ms (stop reason {})",
                    started.elapsed().as_millis(),
                    answered.as_ref().map_or_else(|_| "error".to_string(), proto::stop_reason)
                );
                return finish(answered, turn);
            }
            _ = stop_tick.tick() => {
                trace::waiting_unasked(&turn.calls, &turn.asked, &mut turn.warned);
                if let Some(at) = cancelled_at {
                    if at.elapsed() > CANCEL_GRACE {
                        return Ok(turn.text);
                    }
                } else if handler.stopped() {
                    cancel(client, &mut turn);
                    cancelled_at = Some(tokio::time::Instant::now());
                } else if last_heard.elapsed() > QUIET_LIMIT {
                    cancel(client, &mut turn);
                    return Err(AcpError::Failed(
                        StatusCode::GATEWAY_TIMEOUT,
                        "Antigravity stopped answering".to_string(),
                    ));
                }
            }
        }
    }
}

fn cancel<T>(client: &RpcClient, turn: &mut Turn<T>) {
    turn.cancelled = true;
    if let Err(error) = client.notify("session/cancel", json!({"sessionId": turn.session_id})) {
        tracing::info!("antigravity bridge: session/cancel not sent: {error}");
    }
}

fn finish<T>(answered: Result<Value, RpcError>, turn: Turn<T>) -> Result<String, AcpError> {
    match answered {
        Ok(result) => match proto::stop_reason(&result).as_str() {
            "refusal" if turn.text.trim().is_empty() => Err(failed("Antigravity refused to answer")),
            "cancelled" if !turn.cancelled && turn.text.trim().is_empty() => Err(AcpError::Failed(
                StatusCode::REQUEST_TIMEOUT,
                "Antigravity stopped: a permission request was not answered".to_string(),
            )),
            _ => Ok(turn.text),
        },
        Err(RpcError::Remote { message, .. }) if proto::needs_sign_in(&message) => Err(AcpError::NotSignedIn),
        Err(RpcError::Remote { message, .. }) => match crate::cli_turn::plan_limit("", &message) {
            Some((code, limit)) => Err(AcpError::Failed(code, limit)),
            None => Err(failed(format!("antigravity prompt failed: {message}"))),
        },
        Err(error) => Err(failed(format!("antigravity prompt failed: {error}"))),
    }
}

/// Waits for `decision` unless the session ends first; dropping the
/// decision withdraws a card that is still open.
async fn until_ended<F: Future>(client: &RpcClient, decision: F) -> Result<F::Output, AcpError> {
    tokio::select! {
        decided = decision => Ok(decided),
        _ = client.sender().wait_ended() => Err(failed("agy acp ended while a permission request was open")),
    }
}

async fn permission<H: AcpGate>(
    client: &RpcClient,
    request: ServerRequest,
    turn: &mut Turn<H::Ticket>,
    handler: &H,
) -> Result<(), AcpError> {
    if request.method != PERMISSION_REQUEST {
        trace::request_refused(&request.method);
        if let Err(error) = request.respond_error(METHOD_NOT_FOUND, "method not handled by this client") {
            tracing::info!("antigravity bridge: refusal not delivered: {error}");
        }
        return Ok(());
    }
    let asked_at = tokio::time::Instant::now();
    let tool_id = proto::permission_tool_id(&request.params);
    turn.asked.insert(tool_id.clone());
    let params = trace::with_known_call(&request.params, turn.calls.get(&tool_id).map(|(_, call)| call));
    let actions = proto::actions_of(&params);
    trace::permission_asked(&params, actions.len());
    let mut allowed: Vec<(H::Ticket, Vec<String>)> = Vec::new();
    let mut choice = Choice::Allow;
    for action in actions {
        match until_ended(client, handler.decide(&action)).await? {
            Verdict::Allow(ticket) => allowed.push((ticket, written_paths(&action))),
            Verdict::Decline => {
                choice = Choice::Reject;
                break;
            }
            Verdict::Cancel => {
                choice = Choice::Cancelled;
                break;
            }
        }
    }
    if choice == Choice::Allow {
        turn.tickets.entry(tool_id.clone()).or_default().extend(allowed);
    } else {
        for (ticket, files) in allowed {
            handler.finished(ticket, ActionOutcome::not_run(files.first().map_or("", String::as_str))).await;
        }
    }
    let result = proto::permission_result(&params, choice);
    let delivered = match request.respond(result) {
        Ok(()) => true,
        Err(error) => {
            tracing::info!("antigravity bridge: permission answer not delivered: {error}");
            false
        }
    };
    trace::permission_answered(&tool_id, choice, asked_at, delivered);
    if choice == Choice::Cancelled {
        cancel(client, turn);
    }
    Ok(())
}

/// The web searches and reads of a tool call that did not ask, from what
/// the updates said about it; `None` when it is not one.
fn web_actions<T>(turn: &Turn<T>, id: &str, call: &Value) -> Option<Vec<AcpAction>> {
    let known = turn.calls.get(id).map_or(call, |(_, known)| known);
    let web: Vec<AcpAction> =
        proto::actions_of(&json!({ "toolCall": known })).into_iter().filter(proto::is_web).collect();
    (!web.is_empty()).then_some(web)
}

fn written_paths(action: &AcpAction) -> Vec<String> {
    match action {
        AcpAction::Write { path } => vec![path.clone()],
        _ => Vec::new(),
    }
}

async fn updated<H: AcpGate>(
    client: &RpcClient,
    params: &Value,
    turn: &mut Turn<H::Ticket>,
    handler: &H,
) -> Result<(), AcpError> {
    match proto::update_of(params) {
        Update::Text(chunk) if !chunk.is_empty() => {
            turn.text.push_str(&chunk);
            handler.text(&chunk).await;
        }
        Update::Thought(chunk) if !chunk.is_empty() => handler.thought(&chunk).await,
        Update::Tool { id, kind, status, call } => {
            trace::tool_update(&call);
            trace::remember(&mut turn.calls, &id, &call);
            let status = status.unwrap_or_default();
            if !matches!(status.as_str(), "completed" | "failed") {
                return Ok(());
            }
            if let Some(tickets) = turn.tickets.remove(&id) {
                for (ticket, files) in tickets {
                    handler.finished(ticket, proto::outcome_of(&call, &status, files)).await;
                }
            } else if !turn.asked.contains(&id) {
                if let Some(web) = web_actions(turn, &id, &call) {
                    for action in web {
                        handler.noted(&action, status == "completed").await;
                    }
                } else if status == "completed"
                    && kind
                        .as_deref()
                        .or_else(|| turn.calls.get(&id).and_then(|(_, known)| known.get("kind")).and_then(Value::as_str))
                        .is_some_and(proto::acts)
                {
                    tracing::error!("antigravity bridge: an action finished without asking; turn stopped");
                    cancel(client, turn);
                    return Err(AcpError::Unasked);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
