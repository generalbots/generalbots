//! Document approval / e-signature state machine (issue #2501).
//!
//! This is the pure domain layer: it owns the lifecycle of an approval, the
//! guards on each transition, the actor roles that may perform them, and an
//! append-only audit trail. It deliberately knows nothing about HTTP routes,
//! UI fragments, notifications or persistence — callers drive it by handing
//! it a [`TransitionCommand`] and a timestamp.
//!
//! Lifecycle:
//!
//! ```text
//! draft -> submitted -> in_review -> approved -> signed
//!                           |
//!                           +------> rejected -> draft
//! ```
//!
//! Every decision is recorded as an [`AuditEvent`] carrying the actor, role,
//! time, the version hash the decision was made against, a comment and (for a
//! signature) a deterministic signature hash. "Deterministic" here means the
//! same inputs always produce the same string — this layer does not do real
//! cryptography (PAdES / signature-page verification belongs to a later,
//! outbound slice), it only binds a decision to a version in a reproducible
//! and testable way.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The lifecycle state of an approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Draft,
    Submitted,
    InReview,
    Approved,
    Rejected,
    Signed,
}

impl ApprovalState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::InReview => "in_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Signed => "signed",
        }
    }
}

impl fmt::Display for ApprovalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A named actor class. Guards use the role, never the free-form actor name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorRole {
    /// Authors the document and submits it / returns a rejected one to draft.
    Author,
    /// Moves a submitted document into review.
    Reviewer,
    /// Approves or rejects a document under review.
    Approver,
    /// Signs an approved document.
    Signer,
    /// Trusted system actor (routing, SLA escalation, delegation).
    System,
}

impl ActorRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Reviewer => "reviewer",
            Self::Approver => "approver",
            Self::Signer => "signer",
            Self::System => "system",
        }
    }
}

/// The transition attempted. This is the verb of a [`TransitionCommand`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Submit,
    StartReview,
    Approve,
    Reject,
    Sign,
    ReturnToDraft,
}

impl TransitionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::StartReview => "start_review",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Sign => "sign",
            Self::ReturnToDraft => "return_to_draft",
        }
    }
}

impl fmt::Display for TransitionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a transition was refused. Carries enough context for a caller to turn
/// it into a precise message without string-matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    InvalidTransition {
        from: ApprovalState,
        action: TransitionKind,
    },
    RoleNotAllowed {
        role: ActorRole,
        action: TransitionKind,
    },
    EmptyActor,
    ReasonRequired,
    VersionHashRequired,
    VersionMismatch {
        expected: String,
        got: String,
    },
}

impl fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, action } => {
                write!(f, "cannot {action} from state {from}")
            }
            Self::RoleNotAllowed { role, action } => {
                write!(f, "role {} may not {action}", role.as_str())
            }
            Self::EmptyActor => f.write_str("an actor is required"),
            Self::ReasonRequired => f.write_str("a reason is required for this decision"),
            Self::VersionHashRequired => f.write_str("a version hash is required"),
            Self::VersionMismatch { expected, got } => {
                write!(f, "version hash mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for ApprovalError {}

/// A caller's request to perform one transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionCommand {
    pub kind: TransitionKind,
    pub actor: String,
    pub role: ActorRole,
    /// Free-form comment; mandatory (as a reason) for [`TransitionKind::Reject`].
    pub comment: String,
    /// The version hash being submitted / approved / signed. Required for
    /// those three actions; ignored for the others.
    pub version_hash: String,
}

impl TransitionCommand {
    #[must_use]
    pub fn new(
        kind: TransitionKind,
        actor: impl Into<String>,
        role: ActorRole,
        version_hash: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            actor: actor.into(),
            role,
            comment: String::new(),
            version_hash: version_hash.into(),
        }
    }

    #[must_use]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = comment.into();
        self
    }
}

/// One immutable entry in the audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Zero-based, monotonically increasing sequence number.
    pub seq: u64,
    pub action: TransitionKind,
    pub from: ApprovalState,
    pub to: ApprovalState,
    pub actor: String,
    pub role: ActorRole,
    /// Unix timestamp supplied by the caller (no clock access in this layer).
    pub at_epoch: i64,
    pub comment: String,
    /// Version hash the decision was bound to.
    pub version_hash: String,
    /// Non-empty only for [`TransitionKind::Sign`].
    pub signature_hash: String,
}

/// The approval flow for a single document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalFlow {
    pub document_id: String,
    pub state: ApprovalState,
    /// Hash of the version bound at submit; an approval/signature is only
    /// valid against this exact value.
    pub bound_version_hash: String,
    trail: Vec<AuditEvent>,
}

impl ApprovalFlow {
    #[must_use]
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            state: ApprovalState::Draft,
            bound_version_hash: String::new(),
            trail: Vec::new(),
        }
    }

    /// Read-only view of the audit trail.
    #[must_use]
    pub fn trail(&self) -> &[AuditEvent] {
        &self.trail
    }

    /// Cheap state+role check for callers that want to hide/reveal an action
    /// before attempting it. It does not validate reasons or hashes — call
    /// [`Self::transition`] for the authoritative guard.
    #[must_use]
    pub fn can_transition(&self, action: TransitionKind, role: ActorRole) -> bool {
        expected_from(action) == Some(self.state) && allowed_roles(action).contains(&role)
    }

    /// True when an approved/signed flow no longer matches the document's
    /// current content — i.e. edits happened after the decision was bound.
    #[must_use]
    pub fn is_stale(&self, current_version_hash: &str) -> bool {
        matches!(self.state, ApprovalState::Approved | ApprovalState::Signed)
            && current_version_hash != self.bound_version_hash
    }

    /// Attempt a transition. On success the state is advanced, an audit event
    /// is appended and returned; on failure the flow is left untouched.
    pub fn transition(
        &mut self,
        cmd: TransitionCommand,
        now_epoch: i64,
    ) -> Result<AuditEvent, ApprovalError> {
        let action = cmd.kind;
        let from = self.state;

        if expected_from(action) != Some(from) {
            return Err(ApprovalError::InvalidTransition { from, action });
        }
        if !allowed_roles(action).contains(&cmd.role) {
            return Err(ApprovalError::RoleNotAllowed {
                role: cmd.role,
                action,
            });
        }
        if cmd.actor.trim().is_empty() {
            return Err(ApprovalError::EmptyActor);
        }
        if action == TransitionKind::Reject && cmd.comment.trim().is_empty() {
            return Err(ApprovalError::ReasonRequired);
        }

        let binds_hash = matches!(
            action,
            TransitionKind::Submit | TransitionKind::Approve | TransitionKind::Sign
        );
        let event_version = if binds_hash {
            if cmd.version_hash.trim().is_empty() {
                return Err(ApprovalError::VersionHashRequired);
            }
            cmd.version_hash
        } else {
            self.bound_version_hash.clone()
        };

        if matches!(action, TransitionKind::Approve | TransitionKind::Sign)
            && event_version != self.bound_version_hash
        {
            return Err(ApprovalError::VersionMismatch {
                expected: self.bound_version_hash.clone(),
                got: event_version,
            });
        }

        let signature_hash = if action == TransitionKind::Sign {
            compute_signature_hash(&event_version, &cmd.actor, now_epoch)
        } else {
            String::new()
        };

        let to = next_state(action);
        let event = AuditEvent {
            seq: self.trail.len() as u64,
            action,
            from,
            to,
            actor: cmd.actor,
            role: cmd.role,
            at_epoch: now_epoch,
            comment: cmd.comment,
            version_hash: event_version.clone(),
            signature_hash,
        };

        self.state = to;
        match action {
            TransitionKind::Submit => self.bound_version_hash = event_version,
            TransitionKind::ReturnToDraft => self.bound_version_hash.clear(),
            _ => {}
        }
        self.trail.push(event.clone());
        Ok(event)
    }
}

/// Deterministic, dependency-free signature hash over the bound version, the
/// signer and the timestamp. FNV-1a is used purely as a stable digest — this
/// is a state-layer binding, not a cryptographic signature.
#[must_use]
pub fn compute_signature_hash(version_hash: &str, signer: &str, at_epoch: i64) -> String {
    let mut input = String::with_capacity(version_hash.len() + signer.len() + 24);
    input.push_str(version_hash);
    input.push('|');
    input.push_str(signer);
    input.push('|');
    input.push_str(&at_epoch.to_string());

    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn expected_from(action: TransitionKind) -> Option<ApprovalState> {
    match action {
        TransitionKind::Submit => Some(ApprovalState::Draft),
        TransitionKind::StartReview => Some(ApprovalState::Submitted),
        TransitionKind::Approve | TransitionKind::Reject => Some(ApprovalState::InReview),
        TransitionKind::Sign => Some(ApprovalState::Approved),
        TransitionKind::ReturnToDraft => Some(ApprovalState::Rejected),
    }
}

fn next_state(action: TransitionKind) -> ApprovalState {
    match action {
        TransitionKind::Submit => ApprovalState::Submitted,
        TransitionKind::StartReview => ApprovalState::InReview,
        TransitionKind::Approve => ApprovalState::Approved,
        TransitionKind::Reject => ApprovalState::Rejected,
        TransitionKind::Sign => ApprovalState::Signed,
        TransitionKind::ReturnToDraft => ApprovalState::Draft,
    }
}

fn allowed_roles(action: TransitionKind) -> &'static [ActorRole] {
    match action {
        TransitionKind::Submit => &[ActorRole::Author],
        TransitionKind::StartReview => &[ActorRole::Reviewer, ActorRole::System],
        TransitionKind::Approve => &[ActorRole::Approver],
        TransitionKind::Reject => &[ActorRole::Approver],
        TransitionKind::Sign => &[ActorRole::Signer],
        TransitionKind::ReturnToDraft => &[ActorRole::Author, ActorRole::System],
    }
}
