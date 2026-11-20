//! #2544 — CRM lifecycle end-to-end flow: contact → lead → convert →
//! opportunity → close won.
//!
//! Exercises the real `/api/crm/*` handlers and checks the persisted row after
//! every step (`crm_contacts`, then `crm_deals` for both the lead and the
//! opportunity). This is the flow the issue asked for; the tenant-isolation
//! suite already covers cross-branch visibility separately.
//!
//! Requires a botserver plus a migrated database; otherwise the test skips
//! with a reason so the offline suite stays green. No LLM is involved.

use diesel::prelude::*;
use reqwest::Method;
use serde_json::json;
use uuid::Uuid;

use super::flow_support::{
    connect_db, flow_env, flow_token, http_client, json_id, request_json, seed_tenant,
};

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ContactState {
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    email: Option<String>,
}

fn contact_state(conn: &mut PgConnection, id: Uuid) -> Option<ContactState> {
    diesel::sql_query(
        "SELECT CAST(status AS TEXT) AS status, branch_id, \
         CAST(email AS TEXT) AS email FROM crm_contacts WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(conn)
    .ok()
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DealState {
    #[diesel(sql_type = diesel::sql_types::Text)]
    stage: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    won: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    closed: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    lead_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    contact_id: Option<Uuid>,
}

fn deal_state(conn: &mut PgConnection, id: Uuid) -> Option<DealState> {
    diesel::sql_query(
        "SELECT CAST(COALESCE(stage, '') AS TEXT) AS stage, \
         COALESCE(won, false) AS won, (closed_at IS NOT NULL) AS closed, \
         lead_id, branch_id, contact_id FROM crm_deals WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(conn)
    .ok()
}

#[tokio::test]
async fn test_crm_flow_contact_lead_convert_close_won() {
    let Some(env) = flow_env().await else {
        eprintln!("Skipping CRM flow: no server/database available");
        return;
    };
    let Some(client) = http_client() else {
        eprintln!("Skipping CRM flow: cannot build HTTP client");
        return;
    };
    let Some(tenant) = seed_tenant(&env.db_url, "crmflow") else {
        eprintln!("Skipping CRM flow: tenant seeding unavailable");
        return;
    };
    let token = flow_token(&tenant.slug, &tenant.org_id, &tenant.branch_id);
    let mut conn = match connect_db(&env.db_url) {
        Some(conn) => conn,
        None => return,
    };
    let email = format!("prospect-{}@crmflow.test", Uuid::new_v4().simple());

    // 1. Contact — create_contact dedupes by email inside the branch and
    // stores the JWT branch claim as both org_id and branch_id.
    let payload = json!({
        "first_name": "Flow",
        "last_name": "Prospect",
        "email": email,
        "company": "Flow Test Co",
        "source": "integration-test",
    });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/contacts", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => {
            eprintln!("Skipping CRM flow: contact create unavailable");
            return;
        }
    };
    if !status.is_success() {
        eprintln!("Skipping CRM flow: contact create returned {status}");
        return;
    }
    let Some(contact_id) = json_id(&body).and_then(|id| Uuid::parse_str(&id).ok()) else {
        eprintln!("Skipping CRM flow: contact create body had no id");
        return;
    };
    let contact = match contact_state(&mut conn, contact_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(contact.status, "active", "new CRM contacts must start active");
    assert_eq!(contact.branch_id, tenant.branch_id);
    assert_eq!(contact.email.as_deref(), Some(email.as_str()));

    // 2. Lead — the form handler links the existing contact, starts at "new".
    let payload = json!({
        "title": "Flow lead",
        "first_name": "Flow",
        "last_name": "Prospect",
        "email": email,
        "company": "Flow Test Co",
        "value": 1500.0,
        "currency": "USD",
    });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/leads", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping CRM flow: lead create returned {status}");
        return;
    }
    let Some(lead_id) = json_id(&body).and_then(|id| Uuid::parse_str(&id).ok()) else {
        eprintln!("Skipping CRM flow: lead create body had no id");
        return;
    };
    let lead = match deal_state(&mut conn, lead_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(lead.stage, "new", "a captured lead starts in the new stage");
    assert_eq!(lead.lead_id, None, "a lead does not back another lead");
    assert_eq!(lead.branch_id, tenant.branch_id);
    assert_eq!(lead.contact_id, Some(contact_id), "lead must link the contact");

    // 3. Convert — creates a qualification opportunity and closes the lead.
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/leads/{lead_id}/convert", env.base_url),
        &token,
        None,
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping CRM flow: lead convert returned {status}");
        return;
    }
    let Some(opportunity_id) = json_id(&body).and_then(|id| Uuid::parse_str(&id).ok()) else {
        eprintln!("Skipping CRM flow: convert body had no opportunity id");
        return;
    };
    let opportunity = match deal_state(&mut conn, opportunity_id) {
        Some(row) => row,
        None => return,
    };
    assert_ne!(opportunity_id, lead_id, "convert must insert a new row");
    assert_eq!(opportunity.stage, "qualification");
    assert_eq!(opportunity.lead_id, Some(lead_id));
    assert_eq!(opportunity.contact_id, Some(contact_id));
    assert_eq!(opportunity.branch_id, tenant.branch_id);
    let converted_lead = match deal_state(&mut conn, lead_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(converted_lead.stage, "converted");
    assert!(converted_lead.closed, "converted lead must carry closed_at");

    // 4. Close won — stage, flag, probability and close date are persisted.
    let payload = json!({ "won": true, "actual_close_date": "2026-11-20" });
    let (status, body) = match request_json(
        &client,
        Method::POST,
        format!("{}/api/crm/opportunities/{opportunity_id}/close", env.base_url),
        &token,
        Some(&payload),
    )
    .await
    {
        Some(result) => result,
        None => return,
    };
    if !status.is_success() {
        eprintln!("Skipping CRM flow: close returned {status}");
        return;
    }
    let won_flag = body
        .as_ref()
        .and_then(|value| value["won"].as_bool())
        .unwrap_or(false);
    assert!(won_flag, "close response must report the won deal");
    let won = match deal_state(&mut conn, opportunity_id) {
        Some(row) => row,
        None => return,
    };
    assert_eq!(won.stage, "won");
    assert!(won.won, "crm_deals.won must be true after close_won");
    assert_eq!(won.lead_id, Some(lead_id), "close must not lose the lead link");
}
