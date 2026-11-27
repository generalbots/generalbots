//! The Codex chat provider: each answer is one bridged `codex app-server`
//! turn (`botagent::codex::run_bridged`). Every command and file change
//! Codex wants to make in the person's workspace goes through the
//! permission gate (`pipeline::vm_consent`): the card, the conversation's
//! grants and "Permitir tudo" all live there, and each action is a step of
//! the answer with its result. The agent's messages stream into the answer.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use botagent::codex::app_server::ActionOutcome;
use botagent::codex::{Approvals, BridgeError, CodexTurn, Verdict};
use botagent::AgentService;

use crate::core::ai_accounts::{provider_label, CODEX_CLI};
use crate::core::bot::pipeline::llm::refusal::{coded, AGENT_NOT_LINKED, AGENT_UNSAFE_VERSION};
use crate::core::bot::pipeline::native_tools::web_steps;
use crate::core::bot::pipeline::vm_consent::{
    self, BridgeIdentity, VmAction, VmActionKind, VmAgent, VmDecision, VmDenial, VmOutcome, VmReceipt,
};
use crate::core::bot::runs::bus;

type ProviderError = Box<dyn std::error::Error + Send + Sync>;

/// Runs chat turns through the Codex CLI in the person's agent workspace.
pub struct CodexBridgeProvider {
    service: Arc<AgentService>,
    vm_name: String,
    model: String,
    session_id: Uuid,
    turn: BridgeIdentity,
}

impl std::fmt::Debug for CodexBridgeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexBridgeProvider")
            .field("vm_name", &self.vm_name)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl CodexBridgeProvider {
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
        };
        let conversation = self.session_id.to_string();
        let instructions = super::turn_message::persona(config);
        #[cfg(feature = "drive")]
        let images = attached_images(self.session_id);
        #[cfg(not(feature = "drive"))]
        let images: Vec<String> = Vec::new();
        let turn = CodexTurn {
            conversation: &conversation,
            transcript: &transcript,
            latest: &latest,
            model: selected,
            instructions: instructions.as_deref(),
            images: &images,
        };
        let label = provider_label(CODEX_CLI);
        match botagent::codex::run_bridged(&self.service, &self.vm_name, turn, &gate).await {
            Ok(text) if text.trim().is_empty() => Err(super::boxed("Codex returned an empty response".to_string())),
            Ok(text) => Ok(text),
            Err(BridgeError::NotSignedIn) => Err(super::boxed(coded(
                AGENT_NOT_LINKED,
                "Sign in to use your Codex subscription in the agent workspace.",
                &[("agent", CODEX_CLI), ("label", label)],
            ))),
            Err(BridgeError::Unsafe { version }) => Err(super::boxed(unsafe_refusal(label, &version))),
            Err(BridgeError::Unasked) => Err(super::boxed(unsafe_refusal(label, "unknown"))),
            Err(BridgeError::Failed(problem)) => {
                super::plan_limit::settle(Err(problem), &self.service, &self.vm_name, CODEX_CLI).await
            }
        }
    }
}

/// Pictures the person attached to this message, already copied into the
/// workspace, for Codex to see as images (`localImage`).
#[cfg(feature = "drive")]
fn attached_images(session_id: Uuid) -> Vec<String> {
    crate::core::bot::pipeline::native_drive::attached::staged_for(session_id)
        .into_iter()
        .filter(|file| file.image)
        .map(|file| file.vm_path)
        .collect()
}

fn unsafe_refusal(label: &str, version: &str) -> String {
    botlog::safe_warn!("codex bridge: version {version} acts without asking; chat turns refused");
    coded(
        AGENT_UNSAFE_VERSION,
        &format!("This version of {label} ({version}) did not ask for permission before acting, so it is blocked."),
        &[("agent", CODEX_CLI), ("label", label), ("version", version)],
    )
}

/// The gate side of one turn.
struct Gate<'a> {
    turn: &'a BridgeIdentity,
    tx: Option<&'a mpsc::Sender<String>>,
    thread: String,
    run_at_start: Option<String>,
    cancelled_at_start: bool,
}

impl Gate<'_> {
    async fn ask(&self, kind: VmActionKind) -> Verdict<VmReceipt> {
        let action = VmAction { agent: VmAgent::CodexCli, kind };
        match vm_consent::ask(&self.turn.identity(), &action).await {
            VmDecision::Allowed(ticket) => Verdict::Allow(ticket.receipt()),
            VmDecision::Denied(VmDenial::ByPerson) => Verdict::Decline,
            VmDecision::Denied(_) => Verdict::Cancel,
        }
    }
}

impl Approvals for Gate<'_> {
    type Ticket = VmReceipt;

    async fn command(&self, command: &str, cwd: Option<&str>) -> Verdict<VmReceipt> {
        self.ask(VmActionKind::Command {
            command: command.to_string(),
            cwd: cwd.map(str::to_string),
            language: None,
            packages: Vec::new(),
            background: false,
            repair: false,
        })
        .await
    }

    async fn file(&self, path: &str, delete: bool) -> Verdict<VmReceipt> {
        let path = path.to_string();
        self.ask(if delete { VmActionKind::FileDelete { path } } else { VmActionKind::FileWrite { path, from: None } })
            .await
    }

    async fn finished(&self, receipt: VmReceipt, outcome: ActionOutcome) {
        let outcome = VmOutcome { ok: outcome.ok, exit_code: outcome.exit_code, output: outcome.output, files: outcome.files, reason: None };
        vm_consent::report(&self.turn.identity(), &receipt, &outcome).await;
    }

    async fn noted(&self, kind: &str, text: &str, ok: bool) {
        // A search or a page read is a web step ("Pesquisou na web: …").
        if let Some(tool) = web_steps::kind_of(kind) {
            botlog::safe_info!("codex bridge: {tool} ok={ok} {}", web_steps::query_fingerprint(text));
            web_steps::noted(&self.turn.identity(), tool, text, ok).await;
            return;
        }
        vm_consent::noted(&self.turn.identity(), VmAgent::CodexCli, kind, text, ok).await;
    }

    async fn text(&self, chunk: &str) {
        if let Some(tx) = self.tx {
            if let Err(send_err) = tx.send(chunk.to_string()).await
            {
                log::debug!("codex text chunk send failed: {send_err}");
            }
        }
    }

    async fn reasoning(&self, chunk: &str) {
        if let Some(tx) = self.tx {
            if let Err(send_err) = tx.send(serde_json::json!({ "__reasoning__": chunk }).to_string()).await
            {
                log::debug!("codex reasoning chunk send failed: {send_err}");
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
impl botlib::traits::LLMProvider for CodexBridgeProvider {
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
