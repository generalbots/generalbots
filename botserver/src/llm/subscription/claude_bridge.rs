//! The Claude Code chat provider: each answer is one bridged stream-json
//! session (`botagent::claude::bridge`). Every tool use Claude Code wants in
//! the person's workspace (commands, reads, file writes and edits, web
//! fetches) goes through the permission gate (`pipeline::vm_consent`): the
//! card, the conversation's grants and "Permitir tudo" live there, and each
//! action is a step of the answer with its result. The model's text streams
//! into the answer. Claude's session id is kept per conversation, so the
//! next turn resumes it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use async_trait::async_trait;
use axum::http::StatusCode;
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use botagent::claude::bridge::{
    self, BridgeGate, BridgedTurn, ClaudeAction, GateAnswer, ToolOutcome, ToolRequest, CLAUDE_CODE_VERSION,
    UNSAFE_VERSION_MARKER,
};
use botagent::AgentService;

use crate::core::ai_accounts::{provider_label, CLAUDE_CODE};
use crate::core::bot::pipeline::llm::refusal::{coded, AGENT_NOT_LINKED, AGENT_UNSAFE_VERSION};
use crate::core::bot::pipeline::vm_consent::{
    self, BridgeIdentity, VmAction, VmActionKind, VmAgent, VmDecision, VmOutcome, VmReceipt,
};
use crate::core::bot::runs::bus;

type ProviderError = Box<dyn std::error::Error + Send + Sync>;

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
        let transcript = super::flatten_prompt(prompt, config);
        let latest = latest_message(prompt, config);
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
        };
        let conversation = self.session_id.to_string();
        let remembered = sessions().get(&self.session_id).cloned();
        let resume = match remembered {
            Some(id) => Some(id),
            None => bridge::stored_session(&self.service, &self.vm_name, &conversation).await,
        };
        let turn = BridgedTurn { message: &latest, transcript: &transcript, model: selected, resume: resume.as_deref() };
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
            Err((_, message)) if message.starts_with(UNSAFE_VERSION_MARKER) => Err(super::boxed(unsafe_refusal())),
            Err((StatusCode::UNAUTHORIZED, message)) => {
                let label = provider_label(CLAUDE_CODE);
                Err(super::boxed(coded(AGENT_NOT_LINKED, &message, &[("agent", CLAUDE_CODE), ("label", label)])))
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

/// The person's latest message: all a resumed session still lacks.
fn latest_message(prompt: &str, config: &Value) -> String {
    config
        .as_array()
        .and_then(|messages| {
            messages.iter().rev().find(|m| m.get("role").and_then(Value::as_str) == Some("user"))
        })
        .and_then(|message| message.get("content").and_then(Value::as_str))
        .filter(|content| !content.trim().is_empty())
        .unwrap_or(prompt)
        .to_string()
}

/// The action the card shows for one Claude Code tool use. A read is its own
/// kind (`vm_file_read`): allowing reads never allows commands.
fn vm_action(action: &ClaudeAction) -> VmAction {
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
    };
    VmAction { agent: VmAgent::ClaudeCode, kind }
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
}

impl BridgeGate for Gate<'_> {
    async fn decide(&self, request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        let vm = vm_action(action);
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
    use super::{latest_message, vm_action};
    use botagent::claude::bridge::ClaudeAction;
    use serde_json::json;

    #[test]
    fn a_resumed_session_gets_only_the_latest_message() {
        let config = json!([
            {"role": "system", "content": "be brief"},
            {"role": "user", "content": "first"},
            {"role": "assistant", "content": "ok"},
            {"role": "user", "content": "second"}
        ]);
        assert_eq!(latest_message("fallback", &config), "second");
        assert_eq!(latest_message("fallback", &json!(null)), "fallback");
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
