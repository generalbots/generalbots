//! Typed data model for the approval / e-signature state machine.

use serde::{Deserialize, Serialize};
use std::fmt;

pub use super::error::ApprovalError;
use super::hash_hex;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Lifecycle state of a single approval step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Pending,
    Approved,
    Rejected,
    Delegated,
    Expired,
}

impl StepState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Delegated => "delegated",
            Self::Expired => "expired",
        }
    }

    /// Only pending (including delegated) steps accept decisions.
    #[must_use]
    pub fn is_active(self) -> bool {
        matches!(self, Self::Pending | Self::Delegated)
    }
}

impl fmt::Display for StepState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a route's steps are activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteMode {
    /// One step at a time, in declaration order.
    Sequential,
    /// Every step is actionable at once; a rejection still halts the route.
    Parallel,
}

impl RouteMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::Parallel => "parallel",
        }
    }
}

impl fmt::Display for RouteMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Overall state of a workflow, derived from its steps and signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Pending,
    Approved,
    Rejected,
    Expired,
    Signed,
}

impl WorkflowState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Signed => "signed",
        }
    }
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The decision verb applied to a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    Approve,
    Reject,
    Delegate,
    Expire,
}

impl DecisionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Delegate => "delegate",
            Self::Expire => "expire",
        }
    }
}

impl fmt::Display for DecisionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Commands and records
// ---------------------------------------------------------------------------

/// A caller's request to decide one step. Callers never mutate step state
/// directly; they hand one of these to the workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub kind: DecisionKind,
    pub actor: String,
    /// Document digest the decision is bound to. Mandatory for `Approve`.
    pub digest: String,
    /// Reason; mandatory for `Reject`, optional otherwise.
    pub reason: String,
    /// New approver; mandatory for `Delegate`.
    pub delegate_to: String,
}

impl Decision {
    #[must_use]
    pub fn approve(actor: impl Into<String>, digest: impl Into<String>) -> Self {
        Self {
            kind: DecisionKind::Approve,
            actor: actor.into(),
            digest: digest.into(),
            reason: String::new(),
            delegate_to: String::new(),
        }
    }

    #[must_use]
    pub fn reject(actor: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            kind: DecisionKind::Reject,
            actor: actor.into(),
            digest: String::new(),
            reason: reason.into(),
            delegate_to: String::new(),
        }
    }

    #[must_use]
    pub fn delegate(actor: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            kind: DecisionKind::Delegate,
            actor: actor.into(),
            digest: String::new(),
            reason: String::new(),
            delegate_to: to.into(),
        }
    }

    #[must_use]
    pub fn expire(actor: impl Into<String>) -> Self {
        Self {
            kind: DecisionKind::Expire,
            actor: actor.into(),
            digest: String::new(),
            reason: String::new(),
            delegate_to: String::new(),
        }
    }

    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }
}

/// Immutable record of a hand-off from one approver to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub from: String,
    pub to: String,
    pub at_epoch: i64,
    pub reason: String,
}

/// One append-only entry describing a decision and its effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionEvent {
    /// Position in the workflow trail, assigned by the workflow.
    pub seq: u64,
    pub step_id: String,
    pub action: DecisionKind,
    pub from: StepState,
    pub to: StepState,
    pub actor: String,
    /// Caller-supplied Unix timestamp.
    pub at_epoch: i64,
    pub reason: String,
    /// Digest the decision was made against.
    pub digest: String,
    /// True when this event is the single SLA escalation for its step.
    pub escalated: bool,
}

// ---------------------------------------------------------------------------
// Approval step — the per-step state machine
// ---------------------------------------------------------------------------

/// One approver's step in a route, with its SLA deadline and optional
/// delegation. The step owns the explicit transition guards; the workflow only
/// decides *which* step is actionable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalStep {
    pub id: String,
    /// Currently assigned approver (changes on delegation).
    pub approver: String,
    pub group: Option<String>,
    pub sla_deadline_epoch: i64,
    pub state: StepState,
    /// Digest bound when the step was routed.
    pub bound_digest: String,
    pub delegation: Option<Delegation>,
    pub decided_at_epoch: Option<i64>,
    pub reason: String,
    /// Guards against emitting more than one SLA escalation per step.
    pub escalation_sent: bool,
}

impl ApprovalStep {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        approver: impl Into<String>,
        sla_deadline_epoch: i64,
        bound_digest: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            approver: approver.into(),
            group: None,
            sla_deadline_epoch,
            state: StepState::Pending,
            bound_digest: bound_digest.into(),
            delegation: None,
            decided_at_epoch: None,
            reason: String::new(),
            escalation_sent: false,
        }
    }

    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Apply one decision to this step. On success the state advances and a
    /// `DecisionEvent` is returned for the caller to append to its trail. On
    /// failure the step is left untouched.
    pub(crate) fn apply(
        &mut self,
        decision: &Decision,
        now_epoch: i64,
    ) -> Result<DecisionEvent, ApprovalError> {
        let from = self.state;
        if !from.is_active() {
            return Err(ApprovalError::StepNotActive { state: from });
        }
        if decision.actor.trim().is_empty() {
            return Err(ApprovalError::EmptyActor);
        }

        // Explicit per-action guards.
        match decision.kind {
            DecisionKind::Approve => {
                if decision.digest.trim().is_empty() {
                    return Err(ApprovalError::DigestRequired);
                }
                if decision.digest != self.bound_digest {
                    return Err(ApprovalError::DigestMismatch {
                        expected: self.bound_digest.clone(),
                        got: decision.digest.clone(),
                    });
                }
            }
            DecisionKind::Reject => {
                if decision.reason.trim().is_empty() {
                    return Err(ApprovalError::ReasonRequired);
                }
            }
            DecisionKind::Delegate => {
                if decision.delegate_to.trim().is_empty() {
                    return Err(ApprovalError::DelegateRequired);
                }
            }
            DecisionKind::Expire => {
                if now_epoch <= self.sla_deadline_epoch {
                    return Err(ApprovalError::SlaNotElapsed {
                        deadline: self.sla_deadline_epoch,
                        now: now_epoch,
                    });
                }
            }
        }

        let to = match decision.kind {
            DecisionKind::Approve => StepState::Approved,
            DecisionKind::Reject => StepState::Rejected,
            DecisionKind::Delegate => StepState::Delegated,
            DecisionKind::Expire => StepState::Expired,
        };

        match decision.kind {
            DecisionKind::Delegate => {
                self.delegation = Some(Delegation {
                    from: self.approver.clone(),
                    to: decision.delegate_to.clone(),
                    at_epoch: now_epoch,
                    reason: decision.reason.clone(),
                });
                self.approver = decision.delegate_to.clone();
            }
            DecisionKind::Approve | DecisionKind::Reject => {
                self.decided_at_epoch = Some(now_epoch);
            }
            DecisionKind::Expire => {
                self.escalation_sent = true;
            }
        }
        if !decision.reason.is_empty() {
            self.reason = decision.reason.clone();
        }

        self.state = to;
        Ok(DecisionEvent {
            seq: 0,
            step_id: self.id.clone(),
            action: decision.kind,
            from,
            to,
            actor: decision.actor.clone(),
            at_epoch: now_epoch,
            reason: decision.reason.clone(),
            digest: decision.digest.clone(),
            escalated: decision.kind == DecisionKind::Expire,
        })
    }
}

// ---------------------------------------------------------------------------
// Signature
// ---------------------------------------------------------------------------

/// A signed decision bound to one document digest. The `signature_hash` is a
/// deterministic binding over (digest, signer, time, reason); real PAdES /
/// signature-page cryptography belongs to an outbound slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub signer: String,
    pub signed_at_epoch: i64,
    pub document_digest: String,
    pub reason: String,
    pub signature_hash: String,
}

impl Signature {
    #[must_use]
    pub fn new(
        signer: &str,
        document_digest: &str,
        signed_at_epoch: i64,
        reason: &str,
    ) -> Self {
        let signature_hash = hash_hex(&format!(
            "sig|{document_digest}|{signer}|{signed_at_epoch}|{reason}"
        ));
        Self {
            signer: signer.to_string(),
            signed_at_epoch,
            document_digest: document_digest.to_string(),
            reason: reason.to_string(),
            signature_hash,
        }
    }
}
