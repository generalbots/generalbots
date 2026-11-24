//! Workflow orchestration: routing, decisions, SLA sweep, signing and export.

use serde::{Deserialize, Serialize};

use super::model::{
    ApprovalError, ApprovalStep, Decision, DecisionEvent, DecisionKind, RouteMode, Signature,
    StepState, WorkflowState,
};
use super::record::RecordsExport;

/// The approval flow for one document version. Steps are routed up front;
/// callers then drive decisions through [`Self::decide`]. The workflow owns the
/// append-only trail and enforces route ordering and chain-halting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    pub document_id: String,
    /// Digest of the version bound at creation; approval/signature must match.
    pub digest: String,
    pub mode: RouteMode,
    pub steps: Vec<ApprovalStep>,
    pub signatures: Vec<Signature>,
    trail: Vec<DecisionEvent>,
}

impl ApprovalWorkflow {
    #[must_use]
    pub fn new(document_id: impl Into<String>, digest: impl Into<String>, mode: RouteMode) -> Self {
        Self {
            document_id: document_id.into(),
            digest: digest.into(),
            mode,
            steps: Vec::new(),
            signatures: Vec::new(),
            trail: Vec::new(),
        }
    }

    /// Attach ordered steps to the route.
    pub fn route(&mut self, steps: Vec<ApprovalStep>) {
        self.steps.extend(steps);
    }

    #[must_use]
    pub fn trail(&self) -> &[DecisionEvent] {
        &self.trail
    }

    /// True when a rejection or expiry has halted the whole route.
    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.steps.iter().any(|s| {
            matches!(s.state, StepState::Rejected | StepState::Expired)
        })
    }

    /// Ids of the currently actionable step(s). A halted route has none.
    #[must_use]
    pub fn active_step_ids(&self) -> Vec<String> {
        if self.is_halted() {
            return Vec::new();
        }
        match self.mode {
            RouteMode::Sequential => self
                .steps
                .iter()
                .find(|s| s.state.is_active())
                .map(|s| vec![s.id.clone()])
                .unwrap_or_else(Vec::new),
            RouteMode::Parallel => self
                .steps
                .iter()
                .filter(|s| s.state.is_active())
                .map(|s| s.id.clone())
                .collect(),
        }
    }

    /// Overall state, derived from step states and signatures.
    #[must_use]
    pub fn state(&self) -> WorkflowState {
        if self.steps.iter().any(|s| s.state == StepState::Rejected) {
            return WorkflowState::Rejected;
        }
        if self.steps.iter().any(|s| s.state == StepState::Expired) {
            return WorkflowState::Expired;
        }
        if !self.steps.is_empty()
            && self.steps.iter().all(|s| s.state == StepState::Approved)
        {
            return if self.signatures.is_empty() {
                WorkflowState::Approved
            } else {
                WorkflowState::Signed
            };
        }
        WorkflowState::Pending
    }

    /// True when the document has changed since this workflow was bound.
    #[must_use]
    pub fn is_stale(&self, current_digest: &str) -> bool {
        current_digest != self.digest
    }

    /// Apply a decision to one step, enforcing route order and the halted-chain
    /// guard before delegating to the step's own transition guards.
    pub fn decide(
        &mut self,
        step_id: &str,
        decision: Decision,
        now_epoch: i64,
    ) -> Result<DecisionEvent, ApprovalError> {
        if self.is_halted() {
            return Err(ApprovalError::ChainHalted);
        }
        let index = self
            .steps
            .iter()
            .position(|s| s.id == step_id)
            .ok_or_else(|| ApprovalError::StepNotFound {
                id: step_id.to_string(),
            })?;

        if self.mode == RouteMode::Sequential {
            if self.active_step_ids().first().map(String::as_str) != Some(step_id) {
                return Err(ApprovalError::StepOutOfOrder {
                    step_id: step_id.to_string(),
                });
            }
        } else if !self.steps[index].state.is_active() {
            return Err(ApprovalError::StepNotActive {
                state: self.steps[index].state,
            });
        }

        if decision.kind == DecisionKind::Approve && decision.digest != self.digest {
            return Err(ApprovalError::DigestMismatch {
                expected: self.digest.clone(),
                got: decision.digest.clone(),
            });
        }

        let mut event = self.steps[index].apply(&decision, now_epoch)?;
        event.seq = self.trail.len() as u64;
        self.trail.push(event.clone());
        Ok(event)
    }

    /// Sign an approved workflow against the exact bound digest.
    pub fn sign(
        &mut self,
        signer: &str,
        digest: &str,
        signed_at_epoch: i64,
        reason: &str,
    ) -> Result<Signature, ApprovalError> {
        if self.state() != WorkflowState::Approved {
            return Err(ApprovalError::NotApproved);
        }
        if signer.trim().is_empty() {
            return Err(ApprovalError::EmptyActor);
        }
        if digest != self.digest {
            return Err(ApprovalError::DigestMismatch {
                expected: self.digest.clone(),
                got: digest.to_string(),
            });
        }
        let signature = Signature::new(signer, digest, signed_at_epoch, reason);
        self.signatures.push(signature.clone());
        Ok(signature)
    }

    /// Expire every active step whose SLA deadline has passed, emitting exactly
    /// one escalated event per step. Repeated sweeps emit nothing new.
    pub fn sweep_sla(&mut self, now_epoch: i64) -> Vec<DecisionEvent> {
        let mut emitted = Vec::new();
        if self.is_halted() {
            return emitted;
        }
        let overdue: Vec<String> = self
            .steps
            .iter()
            .filter(|s| {
                s.state.is_active() && !s.escalation_sent && now_epoch > s.sla_deadline_epoch
            })
            .map(|s| s.id.clone())
            .collect();

        for step_id in overdue {
            let index = match self.steps.iter().position(|s| s.id == step_id) {
                Some(index) => index,
                None => continue,
            };
            let expire = Decision::expire("system");
            if let Ok(mut event) = self.steps[index].apply(&expire, now_epoch) {
                event.seq = self.trail.len() as u64;
                event.escalated = true;
                self.trail.push(event.clone());
                emitted.push(event);
            }
        }
        emitted
    }

    /// Produce the immutable, hash-bound publish-to-records payload.
    pub fn export_record(&self) -> Result<RecordsExport, ApprovalError> {
        RecordsExport::from_workflow(self)
    }
}
