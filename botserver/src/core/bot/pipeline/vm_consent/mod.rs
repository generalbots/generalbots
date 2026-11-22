//! The one gate for agent actions in the person's virtual machine.
//!
//! Nothing an agent asks for runs in the VM, or leaves it, until the person
//! decides on a card: "Permitir uma vez" (this exact action), "Permitir nesta
//! conversa" (this kind of action, this agent, this conversation), "Permitir
//! tudo" (every in-VM action of this agent in this conversation) or "Negar".
//! A stored conversation grant answers without a card. Every action is one
//! step in the answer's steps block, whatever answered it.
//!
//! Callers: the native tools (`vm_exec` and its background and repaired
//! runs, `terminal_run`, `drive_fetch`, `drive_save`,
//! `save_to_external_drive`), the turn's automatic publish to Drive, and the
//! agent CLI bridges, which call [`ask`] for each pending approval and
//! [`report`] when the action finished. The VM exec functions in
//! `agent_vm_hook` take the [`VmConsent`] ticket that only this module
//! issues, so a new call site cannot run anything without a decision.

mod action;
mod bridge;
#[cfg(feature = "consent")]
mod forget;
mod gate;
pub mod reason;
mod steps;
pub mod tools;
#[cfg(feature = "consent")]
mod wait;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

pub use action::{VmAction, VmActionKind, VmAgent};
pub use bridge::BridgeIdentity;
#[cfg(feature = "consent")]
pub use forget::forget_conversation;
pub use gate::ask;
pub use steps::{noted, report, VmOutcome};

/// How long the person has to answer a card (the consent request TTL).
pub const DECISION_WAIT: Duration = Duration::from_secs(300);

/// What allowed an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowSource {
    /// "Permitir uma vez" on this action's card.
    Once,
    /// "Permitir nesta conversa": now, or a grant stored earlier.
    Conversation,
    /// "Permitir tudo" for this agent in this conversation.
    AllowAll,
    /// The person's own message named it (files they referenced).
    PersonMessage,
}

impl AllowSource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Conversation => "conversation",
            Self::AllowAll => "all",
            Self::PersonMessage => "person",
        }
    }
}

/// Why an action did not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmDenial {
    /// The person pressed "Negar".
    ByPerson,
    /// Nobody answered within [`DECISION_WAIT`].
    NoAnswer,
    /// The turn was stopped while the card was open.
    Cancelled,
    /// The consent service is missing or failed; the gate fails closed.
    Unavailable,
}

impl VmDenial {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ByPerson => "denied",
            Self::NoAnswer => "no_answer",
            Self::Cancelled => "cancelled",
            Self::Unavailable => "unavailable",
        }
    }

    /// The tool result the model receives instead of the action's output.
    #[must_use]
    pub fn model_text(self, action: &VmAction) -> String {
        match self {
            Self::ByPerson => format!(
                "PERMISSION DENIED by the person: {} \"{}\". It did not run. Do not run it again \
                 and do not try another way to get the same effect. Tell the person what you \
                 could not do and ask how to continue.",
                action.kind_key(),
                action.preview(120)
            ),
            Self::NoAnswer => "NO ANSWER: the person did not answer the permission request in 5 \
                minutes; the action did not run. Do not retry; summarise what is pending and stop."
                .to_string(),
            Self::Cancelled => "CANCELLED: the turn was stopped before the person answered the \
                permission request; the action did not run. Do not retry."
                .to_string(),
            Self::Unavailable => "NOT RUN: the permission service is unavailable, so the action \
                did not run. Do not retry; tell the person it could not be asked."
                .to_string(),
        }
    }
}

/// The answer of [`ask`].
#[derive(Debug)]
pub enum VmDecision {
    /// Run it, presenting this ticket to the VM exec function.
    Allowed(VmConsent),
    /// Do not run it; tell the model [`VmDenial::model_text`].
    Denied(VmDenial),
}

/// Proof that one action was allowed. Only this module creates it (its field
/// is private and it is neither `Clone` nor `Copy`); the VM exec functions
/// take it by value, so each ticket runs one action.
#[derive(Debug)]
#[must_use]
pub struct VmConsent {
    receipt: VmReceipt,
}

/// What a ticket recorded, kept by the caller to [`report`] the result after
/// the ticket was spent.
#[derive(Debug, Clone)]
pub struct VmReceipt {
    pub thread_id: uuid::Uuid,
    pub source: AllowSource,
    pub action_hash: String,
    pub step_id: String,
    started: Instant,
}

impl VmReceipt {
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

impl VmConsent {
    /// The allowed action counts as turn progress until [`report`] (`turn_progress`).
    fn issue(thread_id: uuid::Uuid, source: AllowSource, action_hash: String, step_id: String) -> Self {
        crate::core::bot::pipeline::turn_progress::busy(&thread_id.to_string(), true);
        Self {
            receipt: VmReceipt {
                thread_id,
                source,
                action_hash,
                step_id,
                started: Instant::now(),
            },
        }
    }

    /// A ticket for copying files the person named in their own message into
    /// this conversation's folder: the message is the decision. It is shown
    /// in the steps block like any other action.
    pub(crate) fn for_person_request(thread_id: uuid::Uuid, what: &str) -> Self {
        let hash = action::sha256_hex(what);
        botlog::safe_info!(
            "vm consent: thread={thread_id} agent=chat kind=file_write decision=allow source=person arg_sha256={}",
            &hash[..8]
        );
        Self::issue(thread_id, AllowSource::PersonMessage, hash, String::new())
    }

    #[must_use]
    pub fn receipt(&self) -> VmReceipt {
        self.receipt.clone()
    }

    /// Whether this ticket was issued for `session_id`'s conversation.
    #[must_use]
    pub fn covers(&self, session_id: uuid::Uuid) -> bool {
        self.receipt.thread_id == session_id
    }
}
