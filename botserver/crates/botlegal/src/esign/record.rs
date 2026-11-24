//! Deterministic publish-to-records export payload.

use serde::{Deserialize, Serialize};

use super::hash_hex;
use super::model::{ApprovalError, ApprovalStep, DecisionEvent, RouteMode, Signature, WorkflowState};
use super::workflow::ApprovalWorkflow;

/// Immutable snapshot of a signed approval, suitable for filing in records.
/// Field order is fixed by the struct, so serializing the same workflow twice
/// yields the same canonical string and therefore the same `export_hash`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordsExport {
    pub schema_version: u32,
    pub document_id: String,
    pub digest: String,
    pub route_mode: RouteMode,
    pub workflow_state: WorkflowState,
    pub steps: Vec<ApprovalStep>,
    pub signatures: Vec<Signature>,
    pub trail: Vec<DecisionEvent>,
    /// Deterministic hash of every other field (this field is empty when the
    /// hash is computed).
    pub export_hash: String,
}

impl RecordsExport {
    /// Build an export from a signed workflow, or refuse if not yet publishable.
    pub fn from_workflow(workflow: &ApprovalWorkflow) -> Result<Self, ApprovalError> {
        if workflow.state() != WorkflowState::Signed {
            return Err(ApprovalError::NotPublished);
        }
        let mut export = Self {
            schema_version: 1,
            document_id: workflow.document_id.clone(),
            digest: workflow.digest.clone(),
            route_mode: workflow.mode,
            workflow_state: workflow.state(),
            steps: workflow.steps.clone(),
            signatures: workflow.signatures.clone(),
            trail: workflow.trail().to_vec(),
            export_hash: String::new(),
        };
        let canonical =
            serde_json::to_string(&export).map_err(|_| ApprovalError::SerializationFailed)?;
        export.export_hash = hash_hex(&canonical);
        Ok(export)
    }

    /// Recompute the canonical hash and compare, so a filed record can detect
    /// tampering without re-running the workflow.
    #[must_use]
    pub fn is_hash_valid(&self) -> bool {
        let mut copy = self.clone();
        copy.export_hash = String::new();
        match serde_json::to_string(&copy) {
            Ok(canonical) => hash_hex(&canonical) == self.export_hash,
            Err(_) => false,
        }
    }
}
