//! Identity for an agent CLI bridge (Claude Code, Codex, Antigravity).
//!
//! A subscription turn runs inside its provider, outside the chat pipeline
//! that lends a borrowed [`SessionIdentity`] to the native tools.
//! `llm::subscription::build` already holds the four values a bridge needs
//! (state, user, conversation, bot): the provider keeps one
//! [`BridgeIdentity`] built from them for the turn, and lends
//! [`BridgeIdentity::identity`] to [`super::ask`] for each pending approval
//! and to [`super::report`] when the action finished. The frames (the step
//! row and the card's Permission activity) go to the conversation's run
//! channel, which every socket open on that conversation receives: the same
//! channel a background turn publishes to.
//!
//! ```ignore
//! let turn = vm_consent::BridgeIdentity::new(state.clone(), user_id, session_id, bot_uuid);
//! let action = VmAction {
//!     agent: VmAgent::ClaudeCode,
//!     kind: VmActionKind::Command {
//!         command, cwd, language: None, packages: Vec::new(), background: false, repair: false,
//!     },
//! };
//! match vm_consent::ask(&turn.identity(), &action).await {
//!     VmDecision::Allowed(ticket) => {
//!         let receipt = ticket.receipt();
//!         // Answer the CLI "allow"; once it reports the action's result:
//!         vm_consent::report(&turn.identity(), &receipt, &outcome).await;
//!     }
//!     VmDecision::Denied(denial) => {
//!         // Answer the CLI "deny" with `denial.model_text(&action)`.
//!     }
//! }
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use botcore::shared::state::AppState;
use botlib::models::BotResponse;
use uuid::Uuid;

use crate::core::bot::pipeline::types::{PipelineError, PipelineResult, SessionIdentity};
use crate::core::bot::pipeline::ChannelSink;
use crate::core::bot::runs::bus;

/// What a bridge holds for one subscription turn. Owned (no borrows), so a
/// provider can keep it for the whole turn; cheap to clone.
#[derive(Clone)]
pub struct BridgeIdentity {
    state: Arc<AppState>,
    sink: RunChannelSink,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
}

impl std::fmt::Debug for BridgeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgeIdentity")
            .field("session_id", &self.session_id)
            .field("bot_uuid", &self.bot_uuid)
            .finish_non_exhaustive()
    }
}

impl BridgeIdentity {
    /// The turn's identity, from the values `llm::subscription::build`
    /// receives: `session_id` is the conversation (its run channel and its
    /// grants), `user_id` the person whose card it is.
    #[must_use]
    pub fn new(state: Arc<AppState>, user_id: Uuid, session_id: Uuid, bot_uuid: Uuid) -> Self {
        Self {
            sink: RunChannelSink {
                thread_id: bus::thread_id(session_id),
            },
            state,
            bot_uuid,
            session_id,
            user_id,
        }
    }

    /// The identity [`super::ask`] and [`super::report`] take. A bridge only
    /// runs with the conversation's Agent switch on (`subscription::build`
    /// refuses otherwise), hence `agent_vm_enabled`. `bot_name` is empty:
    /// neither function reads it.
    #[must_use]
    pub fn identity(&self) -> SessionIdentity<'_> {
        SessionIdentity {
            sink: &self.sink,
            state: &self.state,
            bot_uuid: self.bot_uuid,
            session_id: self.session_id,
            user_id: self.user_id,
            bot_name: "",
            agent_vm_enabled: true,
        }
    }
}

/// Publishes to the conversation's run channel (`runs::bus`).
#[derive(Clone)]
struct RunChannelSink {
    thread_id: String,
}

#[async_trait]
impl ChannelSink for RunChannelSink {
    async fn send_bot_response(&self, response: &BotResponse) -> PipelineResult<()> {
        let frame = serde_json::to_value(response)
            .map_err(|error| PipelineError::Transport(format!("bridge frame encode: {error}")))?;
        bus::bus().publish(&self.thread_id, frame);
        Ok(())
    }

    async fn send_raw_json(&self, json: &serde_json::Value) -> PipelineResult<()> {
        bus::bus().publish(&self.thread_id, json.clone());
        Ok(())
    }

    async fn send_error(&self, session_id: &str, message: &str) -> PipelineResult<()> {
        let frame = serde_json::json!({
            "session_id": session_id,
            "channel": "web",
            "content": message,
            "is_complete": true,
            "is_error": true,
        });
        self.send_raw_json(&frame).await
    }

    fn channel_type(&self) -> &str {
        "web"
    }

    fn supports_raw_frames(&self) -> bool {
        true
    }
}
