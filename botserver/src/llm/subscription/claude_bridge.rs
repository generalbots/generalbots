//! The Claude Code chat provider: each answer is one bridged stream-json
//! session (`botagent::claude::bridge`). Every tool use Claude Code wants in
//! the person's workspace (commands, reads, file writes and edits, web
//! fetches) goes through the permission gate (`pipeline::vm_consent`): the
//! card, the conversation's grants and "Permitir tudo" live there, and each
//! action is a step of the answer with its result. The model's text streams
//! into the answer. Claude's session id is kept per conversation, so the
//! next turn resumes it. The files the person attached travel inside the
//! message (`claude_attached`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use botagent::claude::bridge::{
    self, BridgeGate, BridgedTurn, ClaudeAction, GateAnswer, ToolOutcome, ToolRequest, CLAUDE_CODE_VERSION,
    UNSAFE_VERSION_MARKER,
};
use botagent::AgentService;

use crate::core::ai_accounts::{provider_label, CLAUDE_CODE};
use crate::core::bot::pipeline::native_tools::web_steps::{self, WebStep};
use crate::core::bot::pipeline::llm::refusal::{coded, AGENT_NOT_LINKED, AGENT_UNSAFE_VERSION};
use crate::core::bot::pipeline::vm_consent::{
    self, BridgeIdentity, VmAction, VmActionKind, VmAgent, VmDecision, VmOutcome, VmReceipt,
};
use crate::core::bot::runs::bus;

type ProviderError = Box<dyn std::error::Error + Send + Sync>;

/// The model reads this when a tool use has no card to ask with.
const UNDESCRIBED: &str = "This action cannot be asked for here; nothing ran.";

/// Claude's session id per conversation, resumed on its next turn. The
/// workspace keeps a copy (`bridge::store_session`), read when this map has
/// none, so a server restart still resumes the conversation.
static SESSIONS: OnceLock<Mutex<HashMap<Uuid, String>>> = OnceLock::new();
const MAX_SESSIONS: usize = 4096;

fn sessions() -> std::sync::MutexGuard<'static, HashMap<Uuid, String>> {
    SESSIONS.get_or_init(Mutex::default).lock().unwrap_or_else(PoisonError::into_inner)
}

fn remember(conversation: Uuid, session: String) {
    let mut map = sessions();
    if map.len() >= MAX_SESSIONS && !map.contains_key(&conversation) {
        map.clear();
    }
    map.insert(conversation, session);
}

/// Runs chat turns through Claude Code in the person's agent workspace.
pub struct ClaudeBridgeProvider {
    service: Arc<AgentService>,
    vm_name: String,
    model: String,
    session_id: Uuid,
    turn: BridgeIdentity,
}

impl std::fmt::Debug for ClaudeBridgeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeBridgeProvider")
            .field("vm_name", &self.vm_name)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl ClaudeBridgeProvider {
    pub fn new(service: Arc<AgentService>, vm_name: String, model: String, turn: BridgeIdentity, session_id: Uuid) -> Self {
        Self { service, vm_name, model, session_id, turn }
    }

    async fn answer(
        &self,
        prompt: &str,
        config: &Value,
        model: &str,
        tx: Option<&mpsc::Sender<String>>,
    ) -> Result<String, ProviderError> {
        let config = &botagent::deliverables::with_note(config, &self.session_id.to_string());
        let transcript = super::flatten_prompt(prompt, config);
        let latest = super::turn_message::for_resumed(prompt, config);
        let selected = if model.trim().is_empty() { self.model.as_str() } else { model };
        let thread = bus::thread_id(self.session_id);
        let run_bus = bus::bus();
        let gate = Gate {
            turn: &self.turn,
            tx,
            run_at_start: run_bus.current_run_id(&thread),
            cancelled_at_start: run_bus.is_cancelled(&thread),
            thread,
            open: Mutex::default(),
            web: Mutex::default(),
            staged: super::claude_attached::staged_paths(self.session_id),
        };
        let quoted = transcript.contains(super::antigravity_bridge::QUOTED_MARKER);
        let attachments =
            super::claude_attached::attachments(&self.service, &self.vm_name, self.session_id, quoted).await;
        let conversation = self.session_id.to_string();
        let remembered = sessions().get(&self.session_id).cloned();
        let resume = match remembered {
            Some(id) => Some(id),
            None => bridge::stored_session(&self.service, &self.vm_name, &conversation).await,
        };
        let turn = BridgedTurn {
            message: &latest,
            transcript: &transcript,
            model: selected,
            resume: resume.as_deref(),
            conversation: Some(&conversation),
            attachments: &attachments,
        };
        match bridge::run_bridged_turn(&self.service, &self.vm_name, turn, &gate).await {
            Ok(answer) => {
                if let Some(id) = answer.session_id {
                    if resume.as_deref() != Some(id.as_str()) {
                        bridge::store_session(&self.service, &self.vm_name, &conversation, &id).await;
                    }
                    remember(self.session_id, id);
                }
                Ok(answer.answer)
            }
            Err(problem) if problem.message.starts_with(UNSAFE_VERSION_MARKER) => Err(super::boxed(unsafe_refusal())),
            Err(problem) if problem.error == "unauthorized" => {
                let label = provider_label(CLAUDE_CODE);
                Err(super::boxed(coded(AGENT_NOT_LINKED, &problem.message, &[("agent", CLAUDE_CODE), ("label", label)])))
            }
            Err(error) => super::plan_limit::settle(Err(error), &self.service, &self.vm_name, CLAUDE_CODE).await,
        }
    }
}

fn unsafe_refusal() -> String {
    let label = provider_label(CLAUDE_CODE);
    coded(
        AGENT_UNSAFE_VERSION,
        &format!("This version of {label} did not ask for permission before acting, so it is blocked."),
        &[("agent", CLAUDE_CODE), ("label", label), ("version", CLAUDE_CODE_VERSION)],
    )
}

/// The action the card shows for one Claude Code tool use. A read is its own
/// kind (`vm_file_read`): allowing reads never allows commands.
/// A web search, or (in this edition) a page read: a step of the answer,
/// allowed without a card.
fn web_step(action: &ClaudeAction) -> Option<(&'static str, &str)> {
    match action {
        ClaudeAction::Search { query } => Some((web_steps::SEARCH, query.as_str())),
        ClaudeAction::Fetch { url } if web_steps::ALWAYS_ON => Some((web_steps::FETCH, url.as_str())),
        _ => None,
    }
}

/// The card's action, or `None` for a web search (it has no card: see
/// [`web_step`]).
fn vm_action(action: &ClaudeAction) -> Option<VmAction> {
    let command = |command: String| VmActionKind::Command {
        command,
        cwd: Some(botagent::claude::CLAUDE_HOME.to_string()),
        language: None,
        packages: Vec::new(),
        background: false,
        repair: false,
    };
    let kind = match action {
        ClaudeAction::Command { command: line } => command(line.clone()),
        ClaudeAction::Install { manager, packages, .. } => VmActionKind::NetworkInstall {
            packages: packages.clone(),
            manager: Some(manager.clone()),
        },
        ClaudeAction::ReadFile { path } => VmActionKind::FileRead { path: path.clone() },
        ClaudeAction::WriteFile { path, .. } => VmActionKind::FileWrite { path: path.clone(), from: None },
        ClaudeAction::Fetch { url } => VmActionKind::OpenUrl { url: url.clone() },
        ClaudeAction::Search { .. } => return None,
    };
    Some(VmAction { agent: VmAgent::ClaudeCode, kind })
}

/// The gate side of one turn.
struct Gate<'a> {
    turn: &'a BridgeIdentity,
    tx: Option<&'a mpsc::Sender<String>>,
    thread: String,
    run_at_start: Option<String>,
    cancelled_at_start: bool,
    /// Allowed tool uses waiting for their result: id → receipt, files.
    open: Mutex<HashMap<String, (VmReceipt, Vec<String>)>>,
    /// Web searches and reads waiting for their result: id → step.
    web: Mutex<HashMap<String, WebStep>>,
    /// The workspace copies of the files attached to this message.
    staged: Vec<String>,
}

impl BridgeGate for Gate<'_> {
    async fn decide(&self, request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        if let ClaudeAction::ReadFile { path } = action {
            if super::claude_attached::is_own_attachment(path, &self.staged) {
                botlog::safe_info!("claude bridge: read of an attachment of this message, allowed by the message");
                return GateAnswer::Allow;
            }
        }
        if let Some((tool, text)) = web_step(action) {
            botlog::safe_info!("claude bridge: {tool} allowed without a card {}", web_steps::query_fingerprint(text));
            let step = WebStep::begin(&self.turn.identity(), tool, text).await;
            if let (Some(step), Some(id)) = (step, &request.tool_use_id) {
                self.web.lock().unwrap_or_else(PoisonError::into_inner).insert(id.clone(), step);
            }
            return GateAnswer::Allow;
        }
        let Some(vm) = vm_action(action) else {
            return GateAnswer::Deny(UNDESCRIBED.to_string());
        };
        match vm_consent::ask(&self.turn.identity(), &vm).await {
            VmDecision::Allowed(ticket) => {
                if let Some(id) = &request.tool_use_id {
                    self.open
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(id.clone(), (ticket.receipt(), action.files()));
                }
                GateAnswer::Allow
            }
            VmDecision::Denied(denial) => GateAnswer::Deny(denial.model_text(&vm)),
        }
    }

    async fn finished(&self, tool_use_id: &str, outcome: &ToolOutcome) {
        let step = self.web.lock().unwrap_or_else(PoisonError::into_inner).remove(tool_use_id);
        if let Some(step) = step {
            step.end(self.turn.identity().sink, outcome.ok, &[]).await;
            return;
        }
        let entry = self.open.lock().unwrap_or_else(PoisonError::into_inner).remove(tool_use_id);
        let Some((receipt, files)) = entry else {
            return;
        };
        let outcome = VmOutcome {
            ok: outcome.ok,
            exit_code: outcome.exit_code,
            output: outcome.output.clone(),
            files: if outcome.ok { files } else { Vec::new() },
            reason: None,
        };
        vm_consent::report(&self.turn.identity(), &receipt, &outcome).await;
    }

    async fn text(&self, chunk: &str) -> bool {
        match self.tx {
            Some(tx) => tx.send(chunk.to_string()).await.is_ok(),
            None => true,
        }
    }

    async fn thinking(&self, chunk: &str) {
        if let Some(tx) = self.tx {
            if let Err(send_err) = tx.send(serde_json::json!({ "__reasoning__": chunk }).to_string()).await
            {
                log::debug!("claude reasoning chunk send failed: {send_err}");
            }
        }
    }

    fn stopped(&self) -> bool {
        if self.tx.is_some_and(mpsc::Sender::is_closed) {
            return true;
        }
        let run_bus = bus::bus();
        (run_bus.is_cancelled(&self.thread) && !self.cancelled_at_start)
            || (self.run_at_start.is_some() && run_bus.current_run_id(&self.thread) != self.run_at_start)
    }
}

#[async_trait]
impl botlib::traits::LLMProvider for ClaudeBridgeProvider {
    async fn generate(&self, prompt: &str, config: &Value, model: &str, _key: &str) -> Result<String, ProviderError> {
        self.answer(prompt, config, model, None).await
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        config: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        _key: &str,
        _tools: Option<&Vec<Value>>,
    ) -> Result<(), ProviderError> {
        self.answer(prompt, config, model, Some(&tx)).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::{web_step, web_steps};
    use crate::core::bot::pipeline::vm_consent::VmAction;
    use botagent::claude::bridge::ClaudeAction;

    fn vm_action(action: &ClaudeAction) -> VmAction {
        super::vm_action(action).expect("this action has a card")
    }

    #[test]
    fn a_web_search_is_a_step_without_a_card() {
        let search = ClaudeAction::Search { query: "rust 2026".into() };
        assert_eq!(web_step(&search), Some((web_steps::SEARCH, "rust 2026")));
        assert!(super::vm_action(&search).is_none());
        let fetch = ClaudeAction::Fetch { url: "https://example.com/".into() };
        assert_eq!(web_step(&fetch).is_some(), web_steps::ALWAYS_ON);
        assert!(web_step(&ClaudeAction::Command { command: "curl https://example.com".into() }).is_none());
    }

    #[test]
    fn the_card_carries_the_exact_command_path_or_url() {
        let read = vm_action(&ClaudeAction::ReadFile { path: "/home/claude/.claude/.credentials.json".into() });
        assert_eq!(read.text(), "/home/claude/.claude/.credentials.json");
        assert!(read.protected());
        assert_eq!(read.offers(), vec!["allow_once", "deny"]);
        let notes = vm_action(&ClaudeAction::ReadFile { path: "/home/claude/notes.md".into() });
        assert_eq!((notes.class(), notes.kind_key()), ("vm_file_read", "file_read"));
        assert_eq!(notes.card_detail()["path"], "/home/claude/notes.md");
        assert_ne!(notes.class(), vm_action(&ClaudeAction::Command { command: "ls".into() }).class());
        let write = vm_action(&ClaudeAction::WriteFile { path: "/home/claude/a.txt".into(), edit: true });
        assert_eq!((write.class(), write.text().as_str()), ("vm_file_write", "/home/claude/a.txt"));
        let fetch = vm_action(&ClaudeAction::Fetch { url: "https://example.com/".into() });
        assert_eq!(fetch.class(), "vm_browser");
        let install = vm_action(&ClaudeAction::Install {
            manager: "pip".into(),
            packages: vec!["requests".into()],
            command: "pip install requests".into(),
        });
        assert_eq!(install.class(), "vm_network");
        assert_eq!(vm_action(&ClaudeAction::Command { command: "ls".into() }).agent.as_str(), "claude-code");
    }
}
