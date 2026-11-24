//! The Codex app server kept warm between the turns of one conversation:
//! the process, its open thread and its version. The next turn of the same
//! conversation sends `turn/start` on it instead of starting Codex again.
//!
//! Every approval request is still read and answered inside a turn, with
//! that turn's gate: a request queued while the process waited between
//! turns is refused ([`refuse_idle`]) before the next turn starts.

use std::sync::LazyLock;
use std::time::Duration;

use crate::duplex::{RpcClient, RpcInbox};
use crate::warm::observe::TurnClock;
use crate::warm::{self, WarmPool, WarmProcess};

use super::app_server::ActionOutcome;
use super::bridge::{Approvals, Verdict};
use super::bridge_turn::BRIDGE_DEADLINE;

/// Longest life of one app server process (its session deadline).
pub(crate) const LIFETIME: Duration = Duration::from_secs(2 * 60 * 60);

/// One warm `codex app-server` with its thread open.
pub struct CodexWarm {
    pub(crate) client: RpcClient,
    pub(crate) inbox: RpcInbox,
    pub(crate) thread_id: String,
    pub(crate) version: String,
}

impl WarmProcess for CodexWarm {
    fn alive(&self) -> bool {
        self.client.sender().ended().is_none()
    }

    fn shut_down(self) {
        let Self { client, .. } = self;
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    client.close().await;
                });
            }
            // Dropping the client closes the session too.
            Err(_) => drop(client),
        }
    }
}

static POOL: LazyLock<WarmPool<CodexWarm>> =
    LazyLock::new(|| WarmPool::new("codex", warm::limits_for(LIFETIME, BRIDGE_DEADLINE)));

/// The Codex warm processes.
pub fn warm_pool() -> &'static WarmPool<CodexWarm> {
    &POOL
}

/// Refuses what the process asked while no turn ran (see [`warm::refuse_idle_rpc`]).
pub(crate) fn refuse_idle(inbox: &mut RpcInbox) -> bool {
    warm::refuse_idle_rpc(inbox, "codex")
}

/// The turn's own gate, with the turn's timing and whether anything of the
/// turn reached the chat.
pub(crate) struct Observed<'a, H> {
    pub(crate) inner: &'a H,
    pub(crate) clock: &'a TurnClock,
}

impl<H: Approvals> Approvals for Observed<'_, H> {
    type Ticket = H::Ticket;

    async fn command(&self, command: &str, cwd: Option<&str>) -> Verdict<H::Ticket> {
        self.clock.touch("permission request");
        self.inner.command(command, cwd).await
    }

    async fn file(&self, path: &str, delete: bool) -> Verdict<H::Ticket> {
        self.clock.touch("permission request");
        self.inner.file(path, delete).await
    }

    async fn finished(&self, ticket: H::Ticket, outcome: ActionOutcome) {
        self.inner.finished(ticket, outcome).await;
    }

    async fn text(&self, chunk: &str) {
        self.clock.touch("text");
        self.inner.text(chunk).await;
    }

    async fn noted(&self, kind: &str, text: &str, ok: bool) {
        self.clock.touch("step");
        self.inner.noted(kind, text, ok).await;
    }

    async fn reasoning(&self, chunk: &str) {
        self.clock.touch("thought");
        self.inner.reasoning(chunk).await;
    }

    fn stopped(&self) -> bool {
        self.inner.stopped()
    }
}
