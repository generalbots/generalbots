//! Typed approval records, routes, SLAs, delegations and signed decisions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle of one approval. `Draft` is also the destination after a reject
/// or a request-for-changes: the document goes back to its author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Draft,
    InReview,
    AwaitingSignature,
    Approved,
    Signed,
    Published,
    Invalidated,
}

impl ApprovalStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in_review",
            Self::AwaitingSignature => "awaiting_signature",
            Self::Approved => "approved",
            Self::Signed => "signed",
            Self::Published => "published",
            Self::Invalidated => "invalidated",
        }
    }

    /// Only a signed, unpublished record may be published; a published record
    /// is immutable and an invalidated one needs a fresh route.
    #[must_use]
    pub fn is_immutable(self) -> bool {
        matches!(self, Self::Published)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepMode {
    /// Every listed approver may decide independently; `quorum` approvals
    /// are required before the step advances.
    Parallel,
    /// Approvers decide one at a time, in list order.
    Sequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Review,
    Approval,
    Signature,
}

impl StepKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Approval => "approval",
            Self::Signature => "signature",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalStep {
    pub id: String,
    pub title: String,
    pub mode: StepMode,
    pub kind: StepKind,
    #[serde(default)]
    pub approvers: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    /// Per-step service level, in seconds. `0` means no SLA.
    #[serde(default)]
    pub sla_secs: i64,
    #[serde(default)]
    pub escalate_to: Option<String>,
    /// Minimum distinct approvals for a parallel step (defaults to 1).
    #[serde(default)]
    pub quorum: usize,
}

impl ApprovalStep {
    #[must_use]
    pub fn quorum_size(&self) -> usize {
        match self.mode {
            StepMode::Parallel => {
                if self.quorum == 0 {
                    self.approvers.len().max(1)
                } else {
                    self.quorum.min(self.approvers.len().max(1))
                }
            }
            StepMode::Sequential => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Delegation {
    #[must_use]
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        match self.expires_at {
            Some(expires) => expires > now && self.to != self.from,
            None => self.to != self.from,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    Approve,
    Reject,
    RequestChanges,
}

impl DecisionKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::RequestChanges => "request_changes",
        }
    }
}

/// One signed decision event. `prev_hash` and `signature` make the trail
/// tamper-evident; `version_hash` binds the decision to a document version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDecision {
    pub id: Uuid,
    pub step_id: String,
    pub actor: String,
    pub decision: DecisionKind,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub comment: String,
    pub version_hash: String,
    pub prev_hash: String,
    pub signature: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedArtifact {
    pub artifact_id: Uuid,
    pub version_hash: String,
    pub signed_hash: String,
    pub records_entry: String,
    pub immutable: bool,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: Uuid,
    pub tenant: String,
    pub doc_id: String,
    pub doc_class: String,
    pub title: String,
    pub owner: String,
    pub version_hash: String,
    pub route: Vec<ApprovalStep>,
    pub status: ApprovalStatus,
    pub current_step: usize,
    /// Progress inside the current step: approvals collected (parallel) or
    /// the index of the next approver (sequential).
    #[serde(default)]
    pub step_progress: usize,
    #[serde(default)]
    pub message: String,
    pub due_at: Option<DateTime<Utc>>,
    pub step_started_at: DateTime<Utc>,
    #[serde(default)]
    pub delegations: Vec<Delegation>,
    #[serde(default)]
    pub decisions: Vec<SignedDecision>,
    #[serde(default)]
    pub reminder_sent: bool,
    #[serde(default)]
    pub escalation_sent: bool,
    pub published: Option<PublishedArtifact>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApprovalRecord {
    /// Effective assignees of the current step after applying active
    /// delegations (`from` is replaced by `to`).
    #[must_use]
    pub fn effective_assignees(&self, now: DateTime<Utc>) -> Vec<String> {
        let Some(step) = self.route.get(self.current_step) else {
            return Vec::new();
        };
        let mut names = step.approvers.clone();
        for name in step.groups.iter().cloned() {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        let mapped: Vec<String> = names
            .iter()
            .map(|name| {
                let mut target = name.clone();
                for delegation in self.delegations.iter().filter(|d| d.is_active(now)) {
                    if delegation.from == target {
                        target = delegation.to.clone();
                    }
                }
                target
            })
            .collect();
        let mut unique = Vec::new();
        for name in mapped {
            if !unique.contains(&name) {
                unique.push(name);
            }
        }
        unique
    }

    /// The single next actor for a sequential step, or `None` for parallel.
    #[must_use]
    pub fn next_actor(&self, now: DateTime<Utc>) -> Option<String> {
        let step = self.route.get(self.current_step)?;
        match step.mode {
            StepMode::Sequential => self.effective_assignees(now).get(self.step_progress).cloned(),
            StepMode::Parallel => None,
        }
    }

    #[must_use]
    pub fn current_step(&self) -> Option<&ApprovalStep> {
        self.route.get(self.current_step)
    }

    #[must_use]
    pub fn decision_headers(&self) -> Vec<String> {
        self.decisions.iter().map(|d| d.signature.clone()).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlaEvent {
    pub kind: SlaEventKind,
    pub approval_id: Uuid,
    pub step_id: String,
    pub assignee: String,
    pub escalate_to: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlaEventKind {
    Reminder,
    Escalation,
}

impl SlaEventKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reminder => "reminder",
            Self::Escalation => "escalation",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(mode: StepMode, approvers: &[&str]) -> ApprovalStep {
        ApprovalStep {
            id: "s".to_string(),
            title: "Step".to_string(),
            mode,
            kind: StepKind::Review,
            approvers: approvers.iter().map(|s| (*s).to_string()).collect(),
            groups: Vec::new(),
            sla_secs: 0,
            escalate_to: None,
            quorum: 0,
        }
    }

    #[test]
    fn parallel_quorum_defaults_to_all_approvers() {
        assert_eq!(step(StepMode::Parallel, &["a", "b"]).quorum_size(), 2);
        assert_eq!(step(StepMode::Sequential, &["a", "b"]).quorum_size(), 1);
        let mut explicit = step(StepMode::Parallel, &["a", "b", "c"]);
        explicit.quorum = 2;
        assert_eq!(explicit.quorum_size(), 2);
    }

    #[test]
    fn sequential_next_actor_follows_step_progress() {
        let mut step = step(StepMode::Sequential, &["alice", "cfo"]);
        step.id = "s1".to_string();
        let mut record = crate::approvals::engine::blank_record("t", "doc", "contract", "owner", "hash");
        record.route = vec![step];
        assert_eq!(record.next_actor(Utc::now()).as_deref(), Some("alice"));
        record.step_progress = 1;
        assert_eq!(record.next_actor(Utc::now()).as_deref(), Some("cfo"));
        record.step_progress = 2;
        assert_eq!(record.next_actor(Utc::now()), None);
    }
}
