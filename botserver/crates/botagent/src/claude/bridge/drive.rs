//! One stream-json session, from the prompt to the `result` line: every
//! `can_use_tool` goes to the gate, every allowed tool's result back to it,
//! the model's text to the answer. Anything else the CLI asks is refused.

use std::collections::HashMap;
use std::future::Future;
use std::time::{Duration, Instant};

use crate::duplex::{DuplexEvent, DuplexSession, SessionEnd};

use super::actions::{exit_code, output_text, ClaudeAction};
use super::protocol::{self, Incoming, ToolRequest, TurnResult};

/// How often Stop is looked at while the model works.
const STOP_POLL: Duration = Duration::from_millis(500);
/// After an interrupt, how long the CLI has to end the turn on its own.
const STOP_GRACE: Duration = Duration::from_secs(10);

/// Told to the model for a tool the bridge does not describe.
pub const UNKNOWN_TOOL: &str = "NOT RUN: this tool is not available in the workspace. Use Bash, \
Read, Write, Edit or WebFetch instead.";
/// Told to the model when Stop was pressed.
pub const STOPPED: &str = "CANCELLED: the turn was stopped; the action did not run.";

/// The person's answer to one tool use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateAnswer {
    Allow,
    /// Deny, telling the model this text.
    Deny(String),
}

/// How an allowed tool use ended.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolOutcome {
    pub ok: bool,
    /// For `Bash`: 0, or the code its result states.
    pub exit_code: Option<i32>,
    pub output: String,
}

/// The chat side of a bridged turn: the card, the steps and the answer.
pub trait BridgeGate: Sync {
    /// Asks whether `action` may run (raising the card) and waits.
    fn decide(&self, request: &ToolRequest, action: &ClaudeAction) -> impl Future<Output = GateAnswer> + Send;
    /// The result of a tool use [`Self::decide`] allowed.
    fn finished(&self, tool_use_id: &str, outcome: &ToolOutcome) -> impl Future<Output = ()> + Send;
    /// Model text for the answer; `false` when nobody reads it any more.
    fn text(&self, chunk: &str) -> impl Future<Output = bool> + Send;
    /// Whether the person pressed Stop.
    fn stopped(&self) -> bool;
}

/// What one session did.
#[derive(Debug, Default)]
pub struct DriveReport {
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    /// The model text sent to [`BridgeGate::text`].
    pub answer: String,
    /// What the CLI wrote itself (sign-in, plan messages).
    pub cli_text: String,
    pub result: Option<TurnResult>,
    pub end: Option<SessionEnd>,
    /// Tools asked about, by name, in order.
    pub asked: Vec<String>,
    /// Tools whose result came back successful without being asked.
    pub unasked: Vec<String>,
    pub stopped: bool,
    /// The `rate_limit_info` objects the CLI printed, in order.
    pub rate_limits: Vec<serde_json::Value>,
}

/// A name the CLI sent, safe for a log line.
fn label(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').take(40).collect()
}

struct Turn<'g, G: BridgeGate> {
    gate: &'g G,
    report: DriveReport,
    /// Tool uses seen in the model's messages: id → tool name.
    uses: HashMap<String, String>,
    /// Allowed tool uses waiting for their result.
    pending: HashMap<String, String>,
    asked_ids: Vec<String>,
    reader_gone: bool,
}

impl<'g, G: BridgeGate> Turn<'g, G> {
    fn cwd(&self) -> &str {
        self.report.cwd.as_deref().unwrap_or(crate::claude::CLAUDE_HOME)
    }

    async fn text(&mut self, text: &str) {
        let chunk = if self.report.answer.is_empty() { text.to_string() } else { format!("\n\n{text}") };
        self.report.answer.push_str(&chunk);
        if !self.reader_gone && !self.gate.text(&chunk).await {
            self.reader_gone = true;
        }
    }

    /// The control response for one `can_use_tool`.
    async fn answer(&mut self, request_id: &str, request: &ToolRequest, stopping: bool) -> serde_json::Value {
        self.report.asked.push(request.tool_name.clone());
        if let Some(id) = &request.tool_use_id {
            self.asked_ids.push(id.clone());
        }
        let Some(action) = ClaudeAction::from_request(request, self.cwd()) else {
            tracing::warn!("claude bridge: refused unknown tool {}", label(&request.tool_name));
            return protocol::deny(request_id, UNKNOWN_TOOL);
        };
        if stopping {
            return protocol::deny(request_id, STOPPED);
        }
        match self.gate.decide(request, &action).await {
            GateAnswer::Allow => {
                if let Some(id) = &request.tool_use_id {
                    self.pending.insert(id.clone(), request.tool_name.clone());
                }
                protocol::allow(request_id, &request.input)
            }
            GateAnswer::Deny(message) => protocol::deny(request_id, &message),
        }
    }

    async fn tool_result(&mut self, tool_use_id: &str, is_error: bool, text: &str) {
        let Some(tool) = self.pending.remove(tool_use_id) else {
            if !is_error && !self.asked_ids.iter().any(|id| id == tool_use_id) {
                let name = self.uses.get(tool_use_id).cloned().unwrap_or_default();
                tracing::warn!("claude bridge: {} ran without a permission request", label(&name));
                self.report.unasked.push(name);
            }
            return;
        };
        let outcome = ToolOutcome {
            ok: !is_error,
            exit_code: if tool == "Bash" { exit_code(is_error, text) } else { None },
            output: output_text(text),
        };
        self.gate.finished(tool_use_id, &outcome).await;
    }
}

/// Runs one turn on an open session: sends `prompt`, answers the CLI until
/// its `result`, then lets the process end. Every way out leaves no process
/// behind (the session is closed, or dropped, here).
pub async fn drive<G: BridgeGate>(session: DuplexSession, prompt: &str, gate: &G) -> DriveReport {
    let (sender, mut events) = session.into_parts();
    let mut turn = Turn {
        gate,
        report: DriveReport::default(),
        uses: HashMap::new(),
        pending: HashMap::new(),
        asked_ids: Vec::new(),
        reader_gone: false,
    };
    let sent = sender
        .send_json(&protocol::initialize())
        .and_then(|()| sender.send_json(&protocol::user_message(prompt)));
    if sent.is_err() {
        sender.close();
    }
    let mut stop_sent: Option<Instant> = None;
    let mut ticker = tokio::time::interval(STOP_POLL);
    loop {
        let event = tokio::select! {
            event = events.recv() => event,
            _ = ticker.tick() => {
                if stop_sent.is_none() && (gate.stopped() || turn.reader_gone) {
                    turn.report.stopped = true;
                    stop_sent = Some(Instant::now());
                    if sender.send_json(&protocol::interrupt()).is_err() {
                        sender.close();
                    }
                } else if stop_sent.is_some_and(|at| at.elapsed() > STOP_GRACE) {
                    sender.close();
                }
                continue;
            }
        };
        let line = match event {
            Some(DuplexEvent::Line(line)) => line,
            Some(DuplexEvent::Ended(end)) => {
                turn.report.end = Some(end);
                break;
            }
            None => break,
        };
        turn.report.rate_limits.extend(crate::plan_usage::claude_rate_limit(&line));
        for incoming in protocol::parse(&line) {
            match incoming {
                Incoming::Init { session_id, cwd } => {
                    turn.report.session_id = Some(session_id);
                    turn.report.cwd = cwd;
                }
                Incoming::Text(text) => turn.text(&text).await,
                Incoming::CliText(text) => {
                    if !turn.report.cli_text.is_empty() {
                        turn.report.cli_text.push('\n');
                    }
                    turn.report.cli_text.push_str(&text);
                }
                Incoming::ToolUse { id, name } => {
                    turn.uses.insert(id, name);
                }
                Incoming::ToolResult { tool_use_id, is_error, text } => {
                    turn.tool_result(&tool_use_id, is_error, &text).await;
                }
                Incoming::CanUseTool { request_id, request } => {
                    let reply = turn.answer(&request_id, &request, stop_sent.is_some()).await;
                    let _ = sender.send_json(&reply);
                }
                Incoming::OtherControl { request_id, subtype } => {
                    tracing::warn!("claude bridge: refused control request {}", label(&subtype));
                    let _ = sender.send_json(&protocol::unsupported(&request_id));
                }
                Incoming::Result(result) => {
                    if result.session_id.is_some() {
                        turn.report.session_id.clone_from(&result.session_id);
                    }
                    turn.report.result = Some(result);
                    sender.close_stdin();
                }
                Incoming::Ignored => {}
            }
        }
    }
    sender.close();
    let unfinished: Vec<String> = turn.pending.drain().map(|(id, _)| id).collect();
    for id in unfinished {
        let outcome = ToolOutcome {
            ok: false,
            exit_code: None,
            output: "The session ended before the result arrived.".to_string(),
        };
        gate.finished(&id, &outcome).await;
    }
    turn.report
}
