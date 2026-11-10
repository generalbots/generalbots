//! Contact and account find-or-create plus `crm.contact.create`.

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};
use super::rows::ContactIdRow;

/// #1441 C8 — `crm.contact.create`: capture a person from chat, deduped by
/// e-mail inside the branch (A2).
pub(crate) async fn crm_create_contact_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let str_p = |k: &str| {
        params
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
    };
    let email = str_p("email").filter(|e| !e.is_empty());
    let first = str_p("first_name");
    let last = str_p("last_name");
    if email.is_none() && first.is_none() && last.is_none() {
        return Err("params.email or params.first_name/last_name is required".to_string());
    }

    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let contact_id = ensure_contact(state, &mut conn, branch, *bot_uuid, &email, params)?;

    let label = [first.as_deref(), last.as_deref()]
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<&str>>()
        .join(" ");
    let label = if label.is_empty() {
        email.clone().unwrap_or_else(|| "contact".to_string())
    } else {
        label
    };
    Ok(json!({
        "summary": format!("Contact '{label}' is on file."),
        "contact_id": contact_id,
        "email": email,
        "deduplicated": true,
    }))
}
/// Find-or-create the contact for `email` inside the branch — the same dedupe
/// key the suite lead form uses (A2).
pub(super) fn ensure_contact(
    state: &Arc<AppState>,
    conn: &mut diesel::PgConnection,
    branch: Uuid,
    bot_uuid: Uuid,
    email: &Option<String>,
    params: &serde_json::Map<String, Value>,
) -> Result<Uuid, String> {
    let str_p = |k: &str| {
        params
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    let Some(mail) = email.clone() else {
        return Err("params.email is required".to_string());
    };
    let existing: Option<ContactIdRow> = diesel::sql_query(
        "SELECT id AS contact_id FROM crm_contacts WHERE branch_id = $1 AND email ILIKE $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&mail)
    .get_result(conn)
    .ok();
    if let Some(row) = existing {
        return Ok(row.contact_id);
    }

    let cid = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO crm_contacts (id, org_id, bot_id, branch_id, first_name, last_name, email, phone, company, job_title, status, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(cid)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("first_name"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("last_name"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(mail))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("phone"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("company"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("job_title"))
    .execute(conn)
    .map_err(|e| format!("insert contact: {e}"))?;
    let _ = state;
    Ok(cid)
}

/// Find-or-create the account for `company` inside the branch (A2).
pub(super) fn ensure_account(
    conn: &mut diesel::PgConnection,
    branch: Uuid,
    company: &str,
) -> Result<Uuid, String> {
    let existing: Option<ContactIdRow> = diesel::sql_query(
        "SELECT id AS contact_id FROM crm_accounts WHERE branch_id = $1 AND name ILIKE $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(company)
    .get_result(conn)
    .ok();
    if let Some(row) = existing {
        return Ok(row.contact_id);
    }
    let aid = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO crm_accounts (id, org_id, bot_id, branch_id, name, tags, custom_fields, created_at, updated_at) \
         VALUES ($1, $2, (SELECT id FROM bots WHERE branch_id = $2 AND is_active = true ORDER BY is_default_for_branch DESC LIMIT 1), $2, $3, '{}', '{}', NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(aid)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(company)
    .execute(conn)
    .map_err(|e| format!("insert account: {e}"))?;
    Ok(aid)
}
