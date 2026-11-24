//! HTTP wiring for the approval / e-signature core (issue #2501).
//!
//! The core in [`super::esign`] is deliberately persistence-free: callers
//! supply timestamps and document digests, and drive the state machine through
//! explicit commands. This handler is that boundary. It accepts a workflow
//! snapshot plus one command, applies the core transition (including the digest
//! binding guards), and returns the updated snapshot as typed JSON. No
//! cryptography is invented here — the signature/export hashes are the
//! deterministic bindings produced by the core itself.

use super::*;

use super::esign::{
    ApprovalError, ApprovalWorkflow, Decision, DecisionEvent, RecordsExport, Signature,
    WorkflowState,
};

/// One workflow snapshot and the single command to apply to it.
#[derive(Debug, Deserialize)]
pub struct EsignTransitionRequest {
    pub workflow: ApprovalWorkflow,
    pub command: ApprovalCommand,
}

/// The command applied to the supplied workflow. Internally tagged so the wire
/// shape is `{"action":"decide", ...}` and clients cannot send ambiguous input.
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ApprovalCommand {
    /// Report current state and, optionally, whether the document has changed
    /// since the workflow was bound to its digest.
    Status {
        current_digest: Option<String>,
    },
    /// Decide one step. The core enforces route order and the digest binding.
    Decide {
        step_id: String,
        decision: Decision,
        at_epoch: i64,
    },
    /// Expire every overdue active step, emitting at most one escalation each.
    SweepSla {
        now_epoch: i64,
    },
    /// Sign an approved workflow against its exact bound digest.
    Sign {
        signer: String,
        digest: String,
        at_epoch: i64,
        reason: Option<String>,
    },
    /// Produce the immutable, hash-bound publish-to-records payload.
    Export,
}

impl ApprovalCommand {
    #[must_use]
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Status { .. } => "status",
            Self::Decide { .. } => "decide",
            Self::SweepSla { .. } => "sweep_sla",
            Self::Sign { .. } => "sign",
            Self::Export => "export",
        }
    }
}

/// Typed result of one transition. Operation-specific fields are `null` for
/// commands that do not produce them.
#[derive(Debug, Serialize)]
pub struct EsignTransitionResponse {
    pub action: &'static str,
    pub document_id: String,
    pub state: WorkflowState,
    pub active_step_ids: Vec<String>,
    /// Only populated by the `status` command with a `current_digest`.
    pub stale: bool,
    pub workflow: ApprovalWorkflow,
    pub event: Option<DecisionEvent>,
    pub events: Vec<DecisionEvent>,
    pub signature: Option<Signature>,
    pub record: Option<RecordsExport>,
}

/// Map a core refusal to a stable, client-safe code through the existing
/// [`LegalError`] response mapping. The core's `Display` text (which embeds
/// expected/got digests and step ids) is intentionally not forwarded verbatim.
fn map_approval_error(error: ApprovalError) -> LegalError {
    let code = match &error {
        ApprovalError::StepNotActive { .. } => "step_not_active",
        ApprovalError::StepNotFound { .. } => "step_not_found",
        ApprovalError::StepOutOfOrder { .. } => "step_out_of_order",
        ApprovalError::EmptyActor => "empty_actor",
        ApprovalError::DigestRequired => "digest_required",
        ApprovalError::DigestMismatch { .. } => "digest_mismatch",
        ApprovalError::ReasonRequired => "reason_required",
        ApprovalError::DelegateRequired => "delegate_required",
        ApprovalError::SlaNotElapsed { .. } => "sla_not_elapsed",
        ApprovalError::ChainHalted => "chain_halted",
        ApprovalError::NotApproved => "not_approved",
        ApprovalError::NotPublished => "not_published",
        ApprovalError::SerializationFailed => "serialization_failed",
    };
    if error == ApprovalError::SerializationFailed {
        LegalError::Internal(code.to_string())
    } else {
        LegalError::Validation(code.to_string())
    }
}

/// Drive one approval/e-signature transition.
///
/// `document_id` in the path is an integrity check against the supplied
/// workflow snapshot, not a persistence lookup: the core owns no storage, so
/// the authoritative state travels with the request and is returned in the
/// response. Callers persist the returned snapshot where appropriate.
pub async fn handle_esign_transition(
    State(_pool): State<Arc<DbPool>>,
    Path(document_id): Path<String>,
    Json(req): Json<EsignTransitionRequest>,
) -> Result<Json<EsignTransitionResponse>, LegalError> {
    let EsignTransitionRequest {
        mut workflow,
        command,
    } = req;

    if document_id != workflow.document_id {
        return Err(LegalError::Validation("document_id_mismatch".to_string()));
    }

    let action = command.as_str();
    let mut event = None;
    let mut events = Vec::new();
    let mut signature = None;
    let mut record = None;
    let mut stale = false;

    match command {
        ApprovalCommand::Status { current_digest } => {
            if let Some(digest) = current_digest {
                stale = workflow.is_stale(&digest);
            }
        }
        ApprovalCommand::Decide {
            step_id,
            decision,
            at_epoch,
        } => {
            event = Some(
                workflow
                    .decide(&step_id, decision, at_epoch)
                    .map_err(map_approval_error)?,
            );
        }
        ApprovalCommand::SweepSla { now_epoch } => {
            events = workflow.sweep_sla(now_epoch);
        }
        ApprovalCommand::Sign {
            signer,
            digest,
            at_epoch,
            reason,
        } => {
            signature = Some(
                workflow
                    .sign(&signer, &digest, at_epoch, &reason.unwrap_or_default())
                    .map_err(map_approval_error)?,
            );
        }
        ApprovalCommand::Export => {
            record = Some(workflow.export_record().map_err(map_approval_error)?);
        }
    }

    Ok(Json(EsignTransitionResponse {
        action,
        document_id: workflow.document_id.clone(),
        state: workflow.state(),
        active_step_ids: workflow.active_step_ids(),
        stale,
        workflow,
        event,
        events,
        signature,
        record,
    }))
}
