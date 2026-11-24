//! The Antigravity chat provider: each answer is one bridged `agy acp`
//! prompt (`botagent::antigravity::acp`). Every tool call Antigravity wants
//! in the person's workspace goes through the permission gate
//! (`pipeline::vm_consent`): the card, the conversation's grants and
//! "Permitir tudo" live there, and each action is a step of the answer with
//! its result. The answer and the agent's reasoning stream as they arrive;
//! pictures the person attached are sent as images. A CLI that does not
//! speak ACP answers in print mode instead.

use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use serde_json::Value;
use tokio::sync::mpsc;
use uuid::Uuid;

use botagent::antigravity::acp::{self, AcpAction, AcpError, AcpGate, AcpTurn, ImagePart, TextResource};
use botagent::codex::app_server::ActionOutcome;
use botagent::codex::Verdict;
use botagent::AgentService;

use crate::core::ai_accounts::{provider_label, ANTIGRAVITY};
use crate::core::bot::pipeline::llm::refusal::{coded, AGENT_NOT_LINKED, AGENT_UNSAFE_VERSION};
use crate::core::bot::pipeline::vm_consent::{
    self, BridgeIdentity, VmAction, VmActionKind, VmAgent, VmDecision, VmDenial, VmOutcome, VmReceipt,
};
use crate::core::bot::runs::bus;

type ProviderError = Box<dyn std::error::Error + Send + Sync>;

/// Largest picture sent after shrinking (the providers' per-image cap).
const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
/// Largest picture read from the workspace to be shrunk.
const MAX_IMAGE_SOURCE: usize = 25 * 1024 * 1024;
/// Largest text file embedded whole.
const MAX_TEXT_BYTES: usize = 1024 * 1024;
const QUOTED_MARKER: &str = "--- BEGIN ATTACHMENT ";

/// Runs chat turns through the Antigravity CLI in the person's workspace.
pub struct AntigravityBridgeProvider {
    service: Arc<AgentService>,
    vm_name: String,
    model: String,
    session_id: Uuid,
    turn: BridgeIdentity,
}

impl std::fmt::Debug for AntigravityBridgeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AntigravityBridgeProvider")
            .field("vm_name", &self.vm_name)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl AntigravityBridgeProvider {
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
        let conversation = self.session_id.to_string();
        let config = super::acp_notes::for_acp(config, &conversation);
        let transcript = super::flatten_prompt(prompt, &config);
        let latest = super::turn_message::for_resumed(prompt, &config);
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
        let (images, resources) = self.attached(&conversation, transcript.contains(QUOTED_MARKER)).await;
        let turn = AcpTurn {
            conversation: &conversation,
            transcript: &transcript,
            latest: &latest,
            model: selected,
            images: &images,
            resources: &resources,
        };
        let label = provider_label(ANTIGRAVITY);
        match acp::run_acp_turn(&self.service, &self.vm_name, turn, &gate).await {
            Ok(text) if text.trim().is_empty() => Err(super::boxed("Antigravity returned an empty response".to_string())),
            Ok(text) => {
                botlog::safe_info!("antigravity turn: path=acp session={}", self.session_id);
                Ok(text)
            }
            Err(AcpError::NoAcp(reason)) => {
                botlog::safe_warn!("antigravity turn: path=print session={} (acp unavailable: {reason})", self.session_id);
                let result = botagent::antigravity::run_prompt(&self.service, &self.vm_name, &transcript, selected).await;
                let text = super::plan_limit::settle(result, &self.service, &self.vm_name, ANTIGRAVITY).await?;
                if let Some(tx) = tx {
                    super::send_in_chunks(tx, &text).await;
                }
                Ok(text)
            }
            Err(AcpError::NotSignedIn) => Err(super::boxed(coded(
                AGENT_NOT_LINKED,
                "Sign in to use your Antigravity account in the agent workspace.",
                &[("agent", ANTIGRAVITY), ("label", label)],
            ))),
            Err(AcpError::Unsafe { version }) => Err(super::boxed(unsafe_refusal(label, &version))),
            Err(AcpError::Unasked) => Err(super::boxed(unsafe_refusal(label, "unknown"))),
            Err(AcpError::Failed(code, message)) => {
                super::plan_limit::settle(Err((code, message)), &self.service, &self.vm_name, ANTIGRAVITY).await
            }
        }
    }

    /// Pictures the person attached to this message, shrunk like the API
    /// path does, and text files whose text the message does not already
    /// quote, read back from the workspace copies.
    async fn attached(&self, conversation: &str, quoted: bool) -> (Vec<ImagePart>, Vec<TextResource>) {
        let mut images = Vec::new();
        let mut resources = Vec::new();
        #[cfg(feature = "drive")]
        for file in crate::core::bot::pipeline::native_drive::attached::staged_for(self.session_id) {
            let Some(rel) = relative_path(&file.vm_path, conversation) else {
                continue;
            };
            let mime = image_mime(&rel);
            let wanted = if file.image { mime.is_some() } else { !quoted && is_text(&rel) };
            if !wanted {
                continue;
            }
            let bytes = match botagent::transfer::read_file(&self.service, &self.vm_name, conversation, &rel).await {
                Ok(bytes) => bytes,
                Err((_, error)) => {
                    botlog::safe_warn!("antigravity turn: attachment not read back: {error}");
                    continue;
                }
            };
            match (file.image, mime) {
                (true, Some(mime)) if bytes.len() <= MAX_IMAGE_SOURCE => {
                    let prepared = tokio::task::spawn_blocking(move || {
                        crate::core::bot::pipeline::turn_hooks::vision_image::prepare(&bytes, mime)
                    })
                    .await
                    .ok()
                    .flatten()
                    .filter(|(bytes, _)| bytes.len() <= MAX_IMAGE_BYTES);
                    if let Some((bytes, out_type)) = prepared {
                        images.push(ImagePart {
                            mime_type: out_type.to_string(),
                            data: base64::engine::general_purpose::STANDARD.encode(&bytes),
                        });
                    }
                }
                (false, _) if bytes.len() <= MAX_TEXT_BYTES => resources.push(TextResource {
                    path: file.vm_path.clone(),
                    mime_type: "text/plain".to_string(),
                    text: String::from_utf8_lossy(&bytes).into_owned(),
                }),
                _ => {}
            }
        }
        #[cfg(not(feature = "drive"))]
        let _ = (conversation, quoted);
        (images, resources)
    }
}

/// The path of a workspace copy relative to the conversation folder.
fn relative_path(vm_path: &str, conversation: &str) -> Option<String> {
    vm_path
        .strip_prefix(&format!("/opt/vibe/conversations/{conversation}/"))
        .filter(|rel| !rel.is_empty())
        .map(str::to_string)
}

fn extension(name: &str) -> String {
    name.rsplit('.').next().unwrap_or("").to_ascii_lowercase()
}

fn image_mime(name: &str) -> Option<&'static str> {
    match extension(name).as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        _ => None,
    }
}

fn is_text(name: &str) -> bool {
    matches!(extension(name).as_str(), "txt" | "md" | "csv" | "tsv" | "json" | "xml" | "yaml" | "yml" | "log" | "html" | "htm")
}

fn unsafe_refusal(label: &str, version: &str) -> String {
    botlog::safe_warn!("antigravity bridge: version {version} acts without asking; chat turns refused");
    coded(
        AGENT_UNSAFE_VERSION,
        &format!("This version of {label} ({version}) did not ask for permission before acting, so it is blocked."),
        &[("agent", ANTIGRAVITY), ("label", label), ("version", version)],
    )
}

/// The action the card shows for one Antigravity tool call.
fn vm_action(action: &AcpAction) -> VmAction {
    let command = |command: String, cwd: Option<String>| VmActionKind::Command {
        command,
        cwd,
        language: None,
        packages: Vec::new(),
        background: false,
        repair: false,
    };
    let kind = match action {
        AcpAction::Command { command: line, cwd } => {
            command(line.clone(), cwd.clone().or_else(|| Some(botagent::antigravity::ANTIGRAVITY_HOME.to_string())))
        }
        AcpAction::Read { path } => VmActionKind::FileRead { path: path.clone() },
        AcpAction::Write { path } => VmActionKind::FileWrite { path: path.clone(), from: None },
        AcpAction::Delete { path } => VmActionKind::FileDelete { path: path.clone() },
        AcpAction::Fetch { url } => VmActionKind::OpenUrl { url: url.clone() },
        AcpAction::Other { title } => command(title.clone(), Some(botagent::antigravity::ANTIGRAVITY_HOME.to_string())),
    };
    VmAction { agent: VmAgent::Antigravity, kind }
}

/// The gate side of one turn.
struct Gate<'a> {
    turn: &'a BridgeIdentity,
    tx: Option<&'a mpsc::Sender<String>>,
    thread: String,
    run_at_start: Option<String>,
    cancelled_at_start: bool,
}

impl AcpGate for Gate<'_> {
    type Ticket = VmReceipt;

    async fn decide(&self, action: &AcpAction) -> Verdict<VmReceipt> {
        match vm_consent::ask(&self.turn.identity(), &vm_action(action)).await {
            VmDecision::Allowed(ticket) => Verdict::Allow(ticket.receipt()),
            VmDecision::Denied(VmDenial::ByPerson) => Verdict::Decline,
            VmDecision::Denied(_) => Verdict::Cancel,
        }
    }

    async fn finished(&self, receipt: VmReceipt, outcome: ActionOutcome) {
        let outcome = VmOutcome { ok: outcome.ok, exit_code: outcome.exit_code, output: outcome.output, files: outcome.files, reason: None };
        vm_consent::report(&self.turn.identity(), &receipt, &outcome).await;
    }

    async fn text(&self, chunk: &str) {
        if let Some(tx) = self.tx {
            let _ = tx.send(chunk.to_string()).await;
        }
    }

    async fn thought(&self, chunk: &str) {
        if let Some(tx) = self.tx {
            let _ = tx.send(serde_json::json!({ "__reasoning__": chunk }).to_string()).await;
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
impl botlib::traits::LLMProvider for AntigravityBridgeProvider {
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
    use super::*;

    #[test]
    fn each_tool_call_shows_its_own_card_for_antigravity() {
        let run = vm_action(&AcpAction::Command { command: "ls".into(), cwd: None });
        assert_eq!(run.agent.as_str(), VmAgent::Antigravity.as_str());
        assert!(matches!(run.kind, VmActionKind::Command { ref cwd, .. } if cwd.as_deref() == Some("/home/antigravity")));
        let read = vm_action(&AcpAction::Read { path: "/opt/vibe/conversations/c/data/t.txt".into() });
        assert_eq!(read.class(), "vm_file_read");
        let token = vm_action(&AcpAction::Read { path: "/home/antigravity/.gemini/oauth_creds.json".into() });
        assert!(token.protected());
        assert_eq!(vm_action(&AcpAction::Write { path: "/a".into() }).class(), "vm_file_write");
        assert_eq!(vm_action(&AcpAction::Fetch { url: "https://example.org/".into() }).class(), "vm_browser");
        assert!(matches!(vm_action(&AcpAction::Delete { path: "/a".into() }).kind, VmActionKind::FileDelete { .. }));
    }

    #[test]
    fn workspace_copies_are_found_by_their_place_and_kind() {
        assert_eq!(relative_path("/opt/vibe/conversations/c1/data/a b.png", "c1").as_deref(), Some("data/a b.png"));
        assert_eq!(relative_path("/opt/vibe/conversations/c2/data/a.png", "c1"), None);
        assert_eq!(image_mime("data/Cartaz.PNG"), Some("image/png"));
        assert_eq!(image_mime("data/r.pdf"), None);
        assert!(is_text("data/texto-colado.txt") && !is_text("data/r.pdf"));
    }
}
