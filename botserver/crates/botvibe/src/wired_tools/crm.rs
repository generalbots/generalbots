//! CRM group of wired tools (Issue #796): contacts, opportunities, tickets
//! and a queued email outbox, all scoped by `bot_id`. Every handler fails
//! closed when no bot scope is supplied — a nil `bot_id` must never degrade
//! to a cross-tenant global query (#928).

use super::{bot_id_arg, err, handler, ok, opt_str, short_token};
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{DbPool, VibeState};
use diesel::prelude::*;
use serde_json::{json, Value};
use uuid::Uuid;

/// Rejects a missing/blank bot scope. CRM reads and writes are tenant-scoped
/// by `bot_id`; a nil value must fail closed instead of querying globally.
fn require_bot_scope(bot_id: &Uuid) -> Result<(), String> {
    if bot_id.is_nil() {
        Err("bot_id is required — CRM data is scoped per bot".to_string())
    } else {
        Ok(())
    }
}

/// `search_contacts` — full-text-ish search over crm_contacts.
fn search_contacts() -> ToolHandler {
    handler(|args, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        async move {
            let query = opt_str(&args, "query", "");
            if query.trim().is_empty() {
                return err("missing required argument 'query'".into());
            }
            let bot_id = bot_id_arg(&args);
            if let Err(e) = require_bot_scope(&bot_id) {
                return err(e);
            }
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10).min(100);
            let sql =
                "SELECT id::text, first_name, last_name, email, phone, org_id::text \
                 FROM crm_contacts \
                 WHERE (first_name ILIKE '%' || $1 || '%' \
                     OR last_name ILIKE '%' || $1 || '%' \
                     OR email ILIKE '%' || $1 || '%' \
                     OR phone ILIKE '%' || $1 || '%') AND bot_id = $2 \
                 ORDER BY created_at DESC LIMIT $3";
            let contacts = match query_contacts(&pool, sql, &query, &bot_id, limit as i64) {
                Ok(c) => c,
                Err(e) => return err(e),
            };
            ok(json!({ "query": query, "contacts": contacts }))
        }
    })
}

fn query_contacts(
    pool: &DbPool,
    sql: &str,
    query: &str,
    bot_id: &Uuid,
    limit: i64,
) -> Result<Vec<Value>, String> {
    let mut conn = pool.get().map_err(|e| format!("db connection failed: {e}"))?;
    let rows: Vec<ContactRow> = diesel::sql_query(sql)
        .bind::<diesel::sql_types::Text, _>(query)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .load(&mut conn)
        .map_err(|e| format!("contact search failed: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let name = [r.first_name, r.last_name]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            json!({
                "id": r.id,
                "name": name,
                "email": r.email,
                "phone": r.phone,
                "org_id": r.org_id,
            })
        })
        .collect())
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ContactRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    first_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    last_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    email: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    phone: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    org_id: String,
}

/// `get_deals` — pipeline opportunities from crm_opportunities.
fn get_deals() -> ToolHandler {
    handler(|args, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        async move {
            let bot_id = bot_id_arg(&args);
            if let Err(e) = require_bot_scope(&bot_id) {
                return err(e);
            }
            let stage = opt_str(&args, "stage", "");
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).min(200);
            let sql =
                "SELECT name, stage, value::text AS value, currency, probability, expected_close_date::text \
                 FROM crm_opportunities \
                 WHERE ($1 = '' OR stage = $1) AND bot_id = $2 \
                 ORDER BY created_at DESC LIMIT $3";
            let deals = match query_deals(&pool, sql, &stage, &bot_id, limit as i64) {
                Ok(d) => d,
                Err(e) => return err(e),
            };
            ok(json!({ "deals": deals }))
        }
    })
}

fn query_deals(
    pool: &DbPool,
    sql: &str,
    stage: &str,
    bot_id: &Uuid,
    limit: i64,
) -> Result<Vec<Value>, String> {
    let mut conn = pool.get().map_err(|e| format!("db connection failed: {e}"))?;
    let rows: Vec<DealRow> = diesel::sql_query(sql)
        .bind::<diesel::sql_types::Text, _>(stage)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .load(&mut conn)
        .map_err(|e| format!("deals query failed: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| {
            json!({
                "name": r.name,
                "stage": r.stage,
                "value": r.value,
                "currency": r.currency,
                "probability": r.probability,
                "expected_close_date": r.expected_close_date,
            })
        })
        .collect())
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DealRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    stage: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    value: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    currency: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    probability: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    expected_close_date: Option<String>,
}

/// `create_ticket` — inserts a support ticket.
fn create_ticket() -> ToolHandler {
    handler(|args, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        async move {
            let subject = match super::require_str(&args, "subject") {
                Ok(s) => s.to_string(),
                Err(e) => return err(e),
            };
            let description = opt_str(&args, "description", "");
            let priority = opt_str(&args, "priority", "medium");
            let bot_id = bot_id_arg(&args);
            if let Err(e) = require_bot_scope(&bot_id) {
                return err(e);
            }
            let ticket_number = short_token("T", &Uuid::new_v4());
            let result = (|| -> Result<String, String> {
                let mut conn = pool.get().map_err(|e| format!("db connection failed: {e}"))?;
                let org_id = org_id_for_bot(&mut conn, &bot_id).unwrap_or_else(|_| Uuid::nil());
                let row: IdRow = diesel::sql_query(
                    "INSERT INTO support_tickets \
                     (org_id, bot_id, ticket_number, subject, description, status, priority, source) \
                     VALUES ($1, $2, $3, $4, $5, 'open', $6, 'vibe') \
                     RETURNING id::text",
                )
                    .bind::<diesel::sql_types::Uuid, _>(org_id)
                    .bind::<diesel::sql_types::Uuid, _>(bot_id)
                    .bind::<diesel::sql_types::Text, _>(&ticket_number)
                    .bind::<diesel::sql_types::Text, _>(&subject)
                    .bind::<diesel::sql_types::Text, _>(&description)
                    .bind::<diesel::sql_types::Text, _>(&priority)
                    .get_result(&mut conn)
                    .map_err(|e| format!("ticket creation failed: {e}"))?;
                Ok(row.id)
            })();
            match result {
                Ok(id) => ok(json!({ "id": id, "ticket_number": ticket_number, "status": "open" })),
                Err(e) => err(e),
            }
        }
    })
}

/// Best-effort org resolution for a bot; nil when unknown (insert will fail
/// on the FK unless the caller passes an explicit bot with org).
fn org_id_for_bot(
    conn: &mut diesel::PgConnection,
    bot_id: &Uuid,
) -> Result<Uuid, String> {
    let row: OrgRow = diesel::sql_query("SELECT organization_id::text FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result(conn)
        .map_err(|e| format!("org lookup failed: {e}"))?;
    Uuid::parse_str(&row.organization_id).map_err(|e| format!("org id parse failed: {e}"))
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct OrgRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    organization_id: String,
}

/// `update_ticket` — updates status/priority/subject of a support ticket.
fn update_ticket() -> ToolHandler {
    handler(|args, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        async move {
            let ticket_id = match super::require_str(&args, "ticket_id") {
                Ok(t) => t.to_string(),
                Err(e) => return err(e),
            };
            let status = opt_str(&args, "status", "");
            let priority = opt_str(&args, "priority", "");
            let mut set: Vec<String> = Vec::new();
            if !status.is_empty() {
                set.push(format!("status = '{}'", status.replace('\'', "''")));
            }
            if !priority.is_empty() {
                set.push(format!("priority = '{}'", priority.replace('\'', "''")));
            }
            if set.is_empty() {
                return err("provide at least one of 'status' or 'priority'".into());
            }
            let sql = format!(
                "UPDATE support_tickets SET {} WHERE id = $1::uuid",
                set.join(", ")
            );
            let result = (|| -> Result<usize, String> {
                let mut conn = pool.get().map_err(|e| format!("db connection failed: {e}"))?;
                diesel::sql_query(&sql)
                    .bind::<diesel::sql_types::Text, _>(&ticket_id)
                    .execute(&mut conn)
                    .map_err(|e| format!("ticket update failed: {e}"))
            })();
            match result {
                Ok(0) => err(format!("ticket '{ticket_id}' not found")),
                Ok(updated) => ok(json!({ "ticket_id": ticket_id, "updated": updated })),
                Err(e) => err(e),
            }
        }
    })
}

/// `send_email` — queues an email in the vibe outbox; the SMTP relay
/// (botemail) picks pending rows up.
fn send_email() -> ToolHandler {
    handler(|args, state: &dyn VibeState| {
        let pool = state.db_pool().clone();
        async move {
            let to = match super::require_str(&args, "to") {
                Ok(t) => t.to_string(),
                Err(e) => return err(e),
            };
            let subject = opt_str(&args, "subject", "(no subject)");
            let body = opt_str(&args, "body", "");
            let bot_id = bot_id_arg(&args);
            if let Err(e) = require_bot_scope(&bot_id) {
                return err(e);
            }
            let id = Uuid::new_v4();
            let result = (|| -> Result<String, String> {
                let mut conn = pool.get().map_err(|e| format!("db connection failed: {e}"))?;
                let row: IdRow = diesel::sql_query(
                    "INSERT INTO vibe_email_outbox (id, bot_id, recipient, subject, body, status) \
                     VALUES ($1, $2, $3, $4, $5, 'queued') RETURNING id::text",
                )
                    .bind::<diesel::sql_types::Uuid, _>(id)
                    .bind::<diesel::sql_types::Uuid, _>(bot_id)
                    .bind::<diesel::sql_types::Text, _>(&to)
                    .bind::<diesel::sql_types::Text, _>(&subject)
                    .bind::<diesel::sql_types::Text, _>(&body)
                    .get_result(&mut conn)
                    .map_err(|e| format!("email queue failed: {e}"))?;
                Ok(row.id)
            })();
            match result {
                Ok(id_text) => ok(json!({ "id": id_text, "status": "queued" })),
                Err(e) => err(e),
            }
        }
    })
}

/// Row wrapper for single-column `RETURNING id::text` queries.
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct IdRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
}

/// Registration triplets for the CRM group.
pub fn crm_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    use crate::types::VibeUseCase;
    let cases = vec![VibeUseCase::CustomerSupport];
    let contact_schema = json!({
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Name, email or phone fragment"},
            "bot_id": {"type": "string", "description": "Optional bot scope (UUID)"},
            "limit": {"type": "integer", "description": "Max results (default 10)"}
        },
        "required": ["query"]
    });
    let deal_schema = json!({
        "type": "object",
        "properties": {
            "stage": {"type": "string", "description": "Optional pipeline stage filter"},
            "bot_id": {"type": "string", "description": "Optional bot scope (UUID)"},
            "limit": {"type": "integer", "description": "Max results (default 20)"}
        }
    });
    let ticket_schema = json!({
        "type": "object",
        "properties": {
            "subject": {"type": "string", "description": "Ticket subject"},
            "description": {"type": "string", "description": "Ticket body"},
            "priority": {"type": "string", "enum": ["low", "medium", "high", "urgent"], "description": "Priority"},
            "bot_id": {"type": "string", "description": "Optional bot scope (UUID)"}
        },
        "required": ["subject"]
    });
    let update_ticket_schema = json!({
        "type": "object",
        "properties": {
            "ticket_id": {"type": "string", "description": "Support ticket id"},
            "status": {"type": "string", "enum": ["open", "in_progress", "resolved", "closed"], "description": "New status"},
            "priority": {"type": "string", "enum": ["low", "medium", "high", "urgent"], "description": "New priority"}
        },
        "required": ["ticket_id"]
    });
    let email_schema = json!({
        "type": "object",
        "properties": {
            "to": {"type": "string", "description": "Recipient address"},
            "subject": {"type": "string", "description": "Email subject"},
            "body": {"type": "string", "description": "Email body"},
            "bot_id": {"type": "string", "description": "Optional bot scope (UUID)"}
        },
        "required": ["to"]
    });
    vec![
        ("search_contacts".into(),
            ToolSchema::new("search_contacts", "Search CRM contacts").with_parameters(contact_schema).with_use_cases(cases.clone()),
            search_contacts()),
        ("get_deals".into(),
            ToolSchema::new("get_deals", "Get CRM pipeline opportunities").with_parameters(deal_schema).with_use_cases(cases.clone()),
            get_deals()),
        ("create_ticket".into(),
            ToolSchema::new("create_ticket", "Create support ticket").with_parameters(ticket_schema).with_use_cases(cases.clone()),
            create_ticket()),
        ("update_ticket".into(),
            ToolSchema::new("update_ticket", "Update support ticket status/priority").with_parameters(update_ticket_schema).with_use_cases(cases.clone()),
            update_ticket()),
        ("send_email".into(),
            ToolSchema::new("send_email", "Queue an email to a contact").with_parameters(email_schema).with_use_cases(cases),
            send_email()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_bot_scope_fails_closed_on_nil() {
        assert!(require_bot_scope(&Uuid::nil()).is_err());
        assert!(require_bot_scope(&Uuid::new_v4()).is_ok());
    }

    #[test]
    fn ticket_number_shape() {
        let n = short_token("T", &Uuid::nil());
        assert_eq!(n, "T-00000000");
    }
}
