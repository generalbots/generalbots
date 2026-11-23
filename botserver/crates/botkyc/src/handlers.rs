use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;

use crate::db;
use crate::storage::ensure_schema_sync;

use botcore::shared::tenant::branch_from_claims;

/// Resolves the caller's tenant branch from the JWT claims.
///
/// #2066 — this used to fall back to `Uuid::nil()`, so a request that carried no
/// valid claims wrote into the global branch instead of being refused. A KYC
/// write must never be globalized by accident.
fn resolve_branch(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    branch_from_claims(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "missing tenant claims: sign in again".to_string(),
        )
    })
}

/// Maps the verified JWT subject to the KYC audit actor (#2873).
///
/// The request body is deliberately NOT an input: a caller must not be able to
/// attribute a compliance decision to a reviewer who never acted. An absent or
/// blank claim degrades to `system`, never to a client-supplied value.
fn audit_actor_from_claim(claim: Option<String>) -> String {
    claim
        .map(|subject| subject.trim().to_string())
        .filter(|subject| !subject.is_empty())
        .unwrap_or_else(|| "system".to_string())
}

/// Builds the append-only audit event for a KYC state transition. Both the
/// actor and the timestamp come from the server, never from the client body.
fn build_state_change_audit(
    actor: &str,
    branch: Uuid,
    case_id: Uuid,
    from: &str,
    to: &str,
    at: chrono::DateTime<Utc>,
) -> botbiometry::audit::BiometricAuditEvent {
    botbiometry::audit::BiometricAuditEvent {
        id: Uuid::new_v4(),
        tenant_id: branch.to_string(),
        actor_id: actor.to_string(),
        action: botbiometry::audit::AuditAction::KycStateChanged,
        payload_json: format!(
            "{{\"verification_id\":\"{case_id}\",\"from\":\"{from}\",\"to\":\"{to}\"}}"
        ),
        at,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub kind: String,
    pub status: String,
    pub documents: Vec<String>,
    pub reviewed_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: Uuid,
    pub document_id: Uuid,
    pub signer_name: String,
    pub signer_email: String,
    pub status: String,
    pub signed_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Uuid,
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub valid_from: chrono::DateTime<Utc>,
    pub valid_until: chrono::DateTime<Utc>,
    pub status: String,
}

/// KYC state-change request wire type (#2873).
///
/// `reviewed_by` was removed on purpose: the audit actor is the verified JWT
/// subject, never a caller-supplied field echoed back in the response.
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVerification {
    pub status: String,
    pub documents: Option<Vec<String>>,
}

pub async fn list_verifications(headers: HeaderMap) -> Result<Json<Vec<Verification>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] profile_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] workflow_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] steps_completed: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] started_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] completed_at: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, profile_id, workflow_name, status, steps_completed, started_at, completed_at
         FROM identity_kyc_workflows WHERE branch_id = $1 ORDER BY started_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Verification {
        id: r.id,
        user_id: r.profile_id,
        kind: r.workflow_name,
        status: r.status,
        documents: crate::lifecycle::documents_from_steps(&r.steps_completed),
        reviewed_by: None,
        created_at: r.started_at,
        completed_at: r.completed_at,
    }).collect()))
}

pub async fn update_verification(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateVerification>,
) -> Result<Json<Verification>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    // The biometric KYC state machine owns the lifecycle (#1502): unknown
    // statuses are rejected here and terminal cases cannot be reopened.
    let state = crate::lifecycle::resolve_requested(&req.status)
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct CurrentRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }
    let current: Vec<CurrentRow> = diesel::sql_query(
        "SELECT status FROM identity_kyc_workflows WHERE id = $1 AND branch_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let current = match current.into_iter().next() {
        Some(row) => row.status,
        None => return Err((StatusCode::NOT_FOUND, format!("Verification {id} not found"))),
    };
    if let Err(error) = crate::lifecycle::check_transition(&current, state) {
        return Err((StatusCode::CONFLICT, format!("KYC case {id} is {error}")));
    }
    // Server clock drives every timestamp in this handler (completion and audit).
    let now = Utc::now();
    let completed_at = if state.is_terminal() {
        Some(now)
    } else {
        None
    };
    // Bind the audit actor to the authenticated principal, not to the PUT body.
    let actor = audit_actor_from_claim(botcore::shared::tenant::user_id_from_claims(&headers));
    let n = diesel::sql_query(
        "UPDATE identity_kyc_workflows SET status = $1, completed_at = COALESCE($2, completed_at) WHERE id = $3 AND branch_id = $4",
    )
    .bind::<diesel::sql_types::Text, _>(state.as_status())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(completed_at)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, format!("Verification {id} not found"))); }
    // Structured biometric audit trail for the state change. The actor is the
    // verified principal and the timestamp is the server clock — neither can be
    // forged by the request body (#2873).
    let event = build_state_change_audit(
        &actor,
        branch,
        parsed,
        &current,
        state.as_status(),
        now,
    );
    log::info!(
        "biometric audit: action={:?} id={} tenant={} actor={} payload={}",
        event.action,
        event.id,
        event.tenant_id,
        event.actor_id,
        event.payload_json
    );
    Ok(Json(Verification {
        id: parsed, user_id: Uuid::nil(), kind: "identity".to_string(), status: state.as_status().to_string(),
        documents: req.documents.unwrap_or_default(), reviewed_by: None,
        created_at: now, completed_at,
    }))
}

pub async fn list_signatures(headers: HeaderMap) -> Result<Json<Vec<Signature>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] document_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] signature_data: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] signed_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, document_id, signature_data, signed_at
         FROM identity_signatures WHERE branch_id = $1 ORDER BY signed_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Signature {
        id: r.id,
        document_id: r.document_id,
        signer_name: r.signature_data,
        signer_email: String::new(),
        status: "signed".to_string(),
        signed_at: Some(r.signed_at),
        created_at: r.signed_at,
    }).collect()))
}

pub async fn sign_document(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Signature>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let now = Utc::now();
    let n = diesel::sql_query(
        "UPDATE identity_signatures SET signed_at = $1 WHERE id = $2 AND branch_id = $3",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Signature {id} not found")));
    }
    Ok(Json(Signature {
        id: parsed, document_id: Uuid::nil(), signer_name: String::new(), signer_email: String::new(),
        status: "signed".to_string(), signed_at: Some(now), created_at: now,
    }))
}

pub async fn list_certificates(headers: HeaderMap) -> Result<Json<Vec<Certificate>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    // The identity_certificates table is not created by any migration yet,
    // so guard with to_regclass and return an empty list instead of 500ing.
    #[derive(diesel::QueryableByName)]
    struct RegRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        exists: Option<String>,
    }
    let rows: Vec<RegRow> = diesel::sql_query(
        "SELECT to_regclass('public.identity_certificates')::text AS exists",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    if rows.is_empty() || rows[0].exists.is_none() {
        return Ok(Json(Vec::<Certificate>::new()));
    }
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] subject: String,
        #[diesel(sql_type = diesel::sql_types::Text)] issuer: String,
        #[diesel(sql_type = diesel::sql_types::Text)] serial: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] valid_from: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] valid_until: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, subject, issuer, serial, valid_from, valid_until, status
         FROM identity_certificates WHERE branch_id = $1 ORDER BY subject ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Certificate {
        id: r.id, subject: r.subject, issuer: r.issuer, serial: r.serial,
        valid_from: r.valid_from, valid_until: r.valid_until, status: r.status,
    }).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn forged_body_reviewer_is_ignored_and_never_audited() {
        // An attacker attempts to attribute the approval to an arbitrary UUID.
        let forged = Uuid::new_v4().to_string();
        let body = json!({
            "status": "approved",
            "reviewed_by": forged,
            "documents": []
        });
        let req: UpdateVerification =
            serde_json::from_value(body).expect("unknown fields are ignored by serde");
        // Reviewer is no longer part of the wire contract, so the forged value
        // cannot be echoed back to the caller either.
        let echoed = serde_json::to_string(&req).expect("serialize request");
        assert!(!echoed.contains(&forged), "forged reviewer echoed: {echoed}");

        // The audit actor is derived from the verified claim, not the body.
        let verified_subject = "verified-user-123";
        let actor = audit_actor_from_claim(Some(verified_subject.to_string()));
        assert_eq!(actor, verified_subject, "actor must be the authenticated user");
        assert_ne!(actor, forged, "forged body actor must never be the audit actor");

        let branch = Uuid::new_v4();
        let case_id = Uuid::new_v4();
        let at = Utc::now();
        let event = build_state_change_audit(&actor, branch, case_id, "pending", "approved", at);
        assert_eq!(event.actor_id, verified_subject);
        assert_ne!(event.actor_id, forged);
        assert_eq!(event.at, at, "audit timestamp must come from the server clock");

        // No (or a blank) verified claim degrades to `system`, never to a value
        // supplied by the caller.
        assert_eq!(audit_actor_from_claim(None), "system");
        assert_eq!(audit_actor_from_claim(Some("  ".to_string())), "system");
    }
}
