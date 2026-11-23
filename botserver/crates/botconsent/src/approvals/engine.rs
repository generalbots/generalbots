//! Pure workflow transitions for typed approval records. No I/O and no locks:
//! the store owns persistence and calls these helpers while holding a write
//! guard. Every decision is signed before it is appended to the trail.

use chrono::{DateTime, Duration, Utc};

use super::signing::{chain_hash, sign_decision, version_hash};
use super::types::{
    ApprovalRecord, ApprovalStatus, ApprovalStep, DecisionKind, PublishedArtifact, SignedDecision,
    SlaEvent, SlaEventKind, StepKind, StepMode,
};
use uuid::Uuid;

/// Escalation fires at half the SLA window (at least five minutes) after the
/// first reminder. Both events are emitted at most once per step.
fn escalation_offset(sla_secs: i64) -> Duration {
    Duration::seconds((sla_secs / 2).max(300))
}

pub fn blank_record(
    tenant: &str,
    doc_id: &str,
    doc_class: &str,
    owner: &str,
    content_hash: &str,
) -> ApprovalRecord {
    let now = Utc::now();
    ApprovalRecord {
        id: Uuid::new_v4(),
        tenant: tenant.to_string(),
        doc_id: doc_id.to_string(),
        doc_class: doc_class.to_string(),
        title: String::new(),
        owner: owner.to_string(),
        version_hash: content_hash.to_string(),
        route: Vec::new(),
        status: ApprovalStatus::Draft,
        current_step: 0,
        step_progress: 0,
        message: String::new(),
        due_at: None,
        step_started_at: now,
        delegations: Vec::new(),
        decisions: Vec::new(),
        reminder_sent: false,
        escalation_sent: false,
        published: None,
        created_at: now,
        updated_at: now,
    }
}

/// Starts a workflow: stores the ordered route and moves the record into
/// review. This is the only place a route may be changed after creation.
pub fn start_route(
    record: &mut ApprovalRecord,
    route: Vec<ApprovalStep>,
    message: &str,
    due_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<(), String> {
    if route.is_empty() {
        return Err("route must contain at least one step".to_string());
    }
    if record.status.is_immutable() {
        return Err("published approval is immutable".to_string());
    }
    record.route = route;
    record.message = message.to_string();
    record.due_at = due_at;
    record.status = if record.current_step().map(|step| step.kind) == Some(StepKind::Signature) {
        ApprovalStatus::AwaitingSignature
    } else {
        ApprovalStatus::InReview
    };
    record.current_step = 0;
    record.step_progress = 0;
    record.step_started_at = now;
    record.reminder_sent = false;
    record.escalation_sent = false;
    record.updated_at = now;
    Ok(())
}

/// Canonical bytes bound to a decision. Both the signer and any verifier must
/// use this exact function so the HMAC is reproducible.
#[must_use]
pub fn decision_body(
    approval_id: Uuid,
    step_id: &str,
    actor: &str,
    kind: DecisionKind,
    reason: &str,
    comment: &str,
    version_hash_value: &str,
    prev_hash: &str,
    timestamp: i64,
) -> Vec<u8> {
    serde_json::json!({
        "approval_id": approval_id,
        "step_id": step_id,
        "actor": actor,
        "decision": kind.as_str(),
        "reason": reason,
        "comment": comment,
        "version_hash": version_hash_value,
        "prev_hash": prev_hash,
        "decided_at": timestamp,
    })
    .to_string()
    .into_bytes()
}

fn reset_step(record: &mut ApprovalRecord, now: DateTime<Utc>) {
    record.current_step += 1;
    record.step_progress = 0;
    record.step_started_at = now;
    record.reminder_sent = false;
    record.escalation_sent = false;
    record.updated_at = now;
}

fn complete_route(record: &mut ApprovalRecord, now: DateTime<Utc>) {
    let signed = record
        .route
        .last()
        .map(|step| step.kind == StepKind::Signature)
        == Some(true);
    record.status = if signed {
        ApprovalStatus::Signed
    } else {
        ApprovalStatus::Approved
    };
    record.current_step = record.route.len();
    record.step_progress = 0;
    record.updated_at = now;
}

/// Applies one decision. `expected_hash` binds the decision to the version the
/// actor saw; `None` is accepted by internal callers that already checked.
pub fn apply_decision(
    record: &mut ApprovalRecord,
    secret: &[u8],
    actor: &str,
    kind: DecisionKind,
    reason: &str,
    comment: &str,
    expected_hash: Option<&str>,
    now: DateTime<Utc>,
) -> Result<SignedDecision, String> {
    if record.status.is_immutable() {
        return Err("published approval is immutable".to_string());
    }
    if !matches!(
        record.status,
        ApprovalStatus::InReview | ApprovalStatus::AwaitingSignature
    ) {
        return Err("approval is not in review".to_string());
    }
    // A creator may never approve their own request, even if they listed
    // themselves in the route or obtained a delegation. `actor` is the
    // server-derived principal, so this cannot be evaded by renaming fields.
    if actor == record.owner.as_str() {
        return Err("owner_cannot_approve".to_string());
    }
    if let Some(hash) = expected_hash {
        if hash != record.version_hash {
            return Err("version_mismatch".to_string());
        }
    }
    let step = match record.current_step() {
        Some(step) => step.clone(),
        None => return Err("no active step".to_string()),
    };
    if !record.effective_assignees(now).iter().any(|a| a == actor) {
        return Err("not_an_assignee".to_string());
    }
    if step.mode == StepMode::Sequential && record.next_actor(now).as_deref() != Some(actor) {
        return Err("out_of_turn".to_string());
    }
    if kind != DecisionKind::Approve && reason.trim().is_empty() {
        return Err("reason_required".to_string());
    }
    if record
        .decisions
        .iter()
        .any(|d| d.step_id == step.id && d.actor == actor)
    {
        return Err("already_decided".to_string());
    }

    let timestamp = now.timestamp();
    let prev_hash = record
        .decisions
        .last()
        .map(|d| d.signature.clone())
        .unwrap_or_else(|| record.version_hash.clone());
    let body = decision_body(
        record.id,
        &step.id,
        actor,
        kind,
        reason,
        comment,
        &record.version_hash,
        &prev_hash,
        timestamp,
    );
    let decision = SignedDecision {
        id: Uuid::new_v4(),
        step_id: step.id.clone(),
        actor: actor.to_string(),
        decision: kind,
        reason: reason.to_string(),
        comment: comment.to_string(),
        version_hash: record.version_hash.clone(),
        prev_hash,
        signature: sign_decision(secret, timestamp, &body),
        decided_at: now,
    };
    record.decisions.push(decision.clone());
    record.updated_at = now;

    match kind {
        DecisionKind::Reject | DecisionKind::RequestChanges => {
            // Rejection returns the document to draft; the full trail remains.
            record.status = ApprovalStatus::Draft;
            record.current_step = 0;
            record.step_progress = 0;
            record.step_started_at = now;
            record.reminder_sent = false;
            record.escalation_sent = false;
        }
        DecisionKind::Approve => {
            record.step_progress += 1;
            let done = match step.mode {
                StepMode::Parallel => record.step_progress >= step.quorum_size(),
                StepMode::Sequential => record.step_progress >= step.approvers.len().max(1),
            };
            let is_signature = step.kind == StepKind::Signature;
            if done {
                if record.current_step + 1 >= record.route.len() {
                    complete_route(record, now);
                    if is_signature {
                        record.status = ApprovalStatus::Signed;
                    }
                } else {
                    reset_step(record, now);
                    if is_signature {
                        record.status = ApprovalStatus::AwaitingSignature;
                    }
                }
            } else if is_signature {
                record.status = ApprovalStatus::AwaitingSignature;
            }
        }
    }
    Ok(decision)
}

/// Computes due SLA events. Flags on the record are updated by the store after
/// the caller has delivered/recorded the returned events, so each event is
/// emitted once per step.
#[must_use]
pub fn check_sla(record: &ApprovalRecord, now: DateTime<Utc>) -> Vec<SlaEvent> {
    if !matches!(
        record.status,
        ApprovalStatus::InReview | ApprovalStatus::AwaitingSignature
    ) {
        return Vec::new();
    }
    let Some(step) = record.current_step() else {
        return Vec::new();
    };
    if step.sla_secs <= 0 {
        return Vec::new();
    }
    let due = record.step_started_at + Duration::seconds(step.sla_secs);
    if now <= due {
        return Vec::new();
    }
    let assignee = match record
        .next_actor(now)
        .or_else(|| record.effective_assignees(now).into_iter().next())
    {
        Some(name) => name,
        None => String::new(),
    };
    let mut events = Vec::new();
    if !record.reminder_sent {
        events.push(SlaEvent {
            kind: SlaEventKind::Reminder,
            approval_id: record.id,
            step_id: step.id.clone(),
            assignee: assignee.clone(),
            escalate_to: None,
        });
    }
    if !record.escalation_sent && now >= due + escalation_offset(step.sla_secs) {
        events.push(SlaEvent {
            kind: SlaEventKind::Escalation,
            approval_id: record.id,
            step_id: step.id.clone(),
            assignee,
            escalate_to: step.escalate_to.clone(),
        });
    }
    events
}

/// Creates the immutable published artefact and its records entry. Only a
/// signed record at the current version may be published.
pub fn publish(
    record: &mut ApprovalRecord,
    now: DateTime<Utc>,
) -> Result<PublishedArtifact, String> {
    if record.status != ApprovalStatus::Signed {
        return Err("only signed approvals can be published".to_string());
    }
    if record.published.is_some() || record.status.is_immutable() {
        return Err("already published".to_string());
    }
    let artifact = PublishedArtifact {
        artifact_id: Uuid::new_v4(),
        version_hash: record.version_hash.clone(),
        signed_hash: chain_hash(&record.decision_headers()),
        records_entry: format!(
            "records://{}/{}/{}",
            record.tenant, record.doc_id, record.version_hash
        ),
        immutable: true,
        published_at: now,
    };
    record.published = Some(artifact.clone());
    record.status = ApprovalStatus::Published;
    record.updated_at = now;
    Ok(artifact)
}

/// A content edit after routing invalidates an in-flight or signed approval
/// visibly. Drafts simply adopt the new version hash.
pub fn invalidate_if_changed(record: &mut ApprovalRecord, new_hash: &str) -> bool {
    if record.status.is_immutable() || new_hash == record.version_hash {
        return false;
    }
    match record.status {
        ApprovalStatus::Draft => {
            record.version_hash = new_hash.to_string();
            false
        }
        _ => {
            record.status = ApprovalStatus::Invalidated;
            record.updated_at = Utc::now();
            true
        }
    }
}

/// SHA-256 helper re-exported so callers can compute the hash the approval
/// must bind to before creating a record.
#[must_use]
pub fn content_hash(content: &[u8]) -> String {
    version_hash(content)
}
