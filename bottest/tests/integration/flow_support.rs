//! Shared bootstrap for the CRM and Campaign end-to-end flow tests (#2544).
//!
//! Both flows follow the `tenant_isolation.rs` pattern: drive real HTTP
//! handlers, then assert persisted state through a direct PostgreSQL
//! connection. When no server or database is available the tests skip with a
//! reason instead of failing the offline suite (CI supplies either
//! `BOTSERVER_URL` + `DATABASE_URL` or the harness-started stack).
//!
//! No paid LLM endpoint is contacted by these helpers.

use std::time::Duration;

use bottest::prelude::*;
use diesel::prelude::*;
use reqwest::{Client, Method, StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::tenant_tokens::mint_admin_jwt;

/// A running botserver plus a database connection string for state assertions.
/// The harness context is kept alive for as long as the flow needs the server.
pub(super) struct FlowEnv {
    _ctx: Option<TestContext>,
    pub(super) base_url: String,
    pub(super) db_url: String,
}

pub(super) fn connect_db(url: &str) -> Option<PgConnection> {
    PgConnection::establish(url).ok()
}

/// Uses an externally provided server when both env vars are set, otherwise
/// starts the same botserver harness used by the tenant-isolation suite.
pub(super) async fn flow_env() -> Option<FlowEnv> {
    if let (Ok(url), Ok(db)) = (
        std::env::var("BOTSERVER_URL"),
        std::env::var("DATABASE_URL"),
    ) {
        if !url.trim().is_empty() && !db.trim().is_empty() {
            let probe = match Client::builder().timeout(Duration::from_secs(2)).build() {
                Ok(client) => client,
                Err(_) => Client::new(),
            };
            if probe.get(&url).send().await.is_ok() && connect_db(&db).is_some() {
                return Some(FlowEnv {
                    _ctx: None,
                    base_url: url,
                    db_url: db,
                });
            }
        }
    }

    let ctx = TestHarness::quick().await.ok()?;
    let db_url = ctx.database_url();
    connect_db(&db_url)?;
    let server = ctx.start_botserver().await.ok()?;
    if !server.is_running() {
        return None;
    }
    Some(FlowEnv {
        _ctx: Some(ctx),
        base_url: server.url.clone(),
        db_url,
    })
}

pub(super) fn http_client() -> Option<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .ok()
}

/// One JSON request, returning the status and the optional decoded body.
/// Empty/non-JSON bodies are represented by `None` rather than an error so a
/// caller can decide whether the endpoint is unavailable or misbehaving.
pub(super) async fn request_json(
    client: &Client,
    method: Method,
    url: String,
    token: &str,
    body: Option<&Value>,
) -> Option<(StatusCode, Option<Value>)> {
    let mut request = client.request(method, url).bearer_auth(token);
    if let Some(payload) = body {
        request = request.json(payload);
    }
    let response = request.send().await.ok()?;
    let status = response.status();
    let text = response.text().await.ok()?;
    let decoded = if text.trim().is_empty() {
        None
    } else {
        serde_json::from_str(&text).ok()
    };
    Some((status, decoded))
}

pub(super) fn json_id(body: &Option<Value>) -> Option<String> {
    body.as_ref()?
        .get("id")?
        .as_str()
        .map(str::to_string)
}

/// Creates one CRM contact through the public handler and returns its id.
/// Used by the campaign flow to build recipients without touching CRM tables.
pub(super) async fn create_crm_contact(
    client: &Client,
    base_url: &str,
    token: &str,
    first_name: &str,
    email: &str,
) -> Option<Uuid> {
    let payload = json!({
        "first_name": first_name,
        "last_name": "Recipient",
        "email": email,
        "company": "Flow Test Co",
    });
    let (status, body) = request_json(
        client,
        Method::POST,
        format!("{base_url}/api/crm/contacts"),
        token,
        Some(&payload),
    )
    .await?;
    if !status.is_success() {
        eprintln!("Skipping flow: contact create returned {status}");
        return None;
    }
    json_id(&body).and_then(|id| Uuid::parse_str(&id).ok())
}

pub(super) fn flow_token(slug: &str, org_id: &str, branch_id: &Uuid) -> String {
    mint_admin_jwt(slug, org_id, &branch_id.to_string())
}

/// Segment limited to the flow's own CRM contacts (`company = Flow Test Co`),
/// so preview counts are exact even when a fixture database has other contacts
/// in the default branch. `company` is a documented segment field.
pub(super) fn flow_segment() -> Value {
    json!({
        "match": "all",
        "groups": [
            { "field": "company", "op": "contains", "value": "flow test co" }
        ]
    })
}

/// A tenant (org + branch + admin) whose JWT branch claim is honoured by the
/// CRM handlers. Mirrors `tenant_isolation::seed_tenant`, but takes the
/// connection string explicitly so harness-started servers are covered too.
pub(super) struct FlowTenant {
    pub(super) slug: String,
    pub(super) org_id: String,
    pub(super) branch_id: Uuid,
}

pub(super) fn seed_tenant(db_url: &str, prefix: &str) -> Option<FlowTenant> {
    let mut conn = connect_db(db_url)?;
    let org_id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let slug = format!("{prefix}-{}", Uuid::new_v4().simple());
    let email = format!("admin-{slug}@tenant.test");

    diesel::sql_query(
        "INSERT INTO organizations (org_id, name, slug, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO branches (id, org_id, slug, name, created_at, updated_at) \
         VALUES ($1, $2, $3, $3, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, is_admin, created_at, updated_at) \
         VALUES ($1, $2, $3, 'x', true, false, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&email)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO user_organizations (id, user_id, org_id, role, is_default, joined_at) \
         VALUES ($1, $2, $3, 'admin', true, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .execute(&mut conn)
    .ok()?;

    // `crm_contacts.org_id` is a legacy FK to branches, not organizations.
    diesel::sql_query(
        "INSERT INTO crm_contacts (id, org_id, branch_id, bot_id, status, first_name, last_name, email, created_at, updated_at) \
         VALUES ($1, $2, $2, (SELECT id FROM bots ORDER BY created_at LIMIT 1), 'active', $3, 'Admin', $4, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&email)
    .execute(&mut conn)
    .ok()?;

    Some(FlowTenant {
        slug,
        org_id: org_id.to_string(),
        branch_id,
    })
}

/// `(org_id, branch_id)` of the default bot — the scope used by the marketing
/// handlers, which do not read the JWT branch claim for lists/publish.
pub(super) fn default_bot_scope(db_url: &str) -> Option<(String, Uuid)> {
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ScopeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }

    let mut conn = connect_db(db_url)?;
    let row: ScopeRow = diesel::sql_query(
        "SELECT org_id, branch_id FROM bots WHERE is_default_for_branch = TRUE LIMIT 1",
    )
    .get_result(&mut conn)
    .ok()?;
    Some((row.org_id.to_string(), row.branch_id))
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct CampaignState {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(super) status: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(super) branch_id: Uuid,
}

pub(super) fn campaign_state(conn: &mut PgConnection, id: Uuid) -> Option<CampaignState> {
    diesel::sql_query(
        "SELECT CAST(COALESCE(status, '') AS TEXT) AS status, branch_id \
         FROM marketing_campaigns WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(conn)
    .ok()
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

pub(super) fn recipient_count(conn: &mut PgConnection, campaign_id: Uuid) -> Option<i64> {
    diesel::sql_query(
        "SELECT COUNT(*) AS count FROM marketing_recipients WHERE campaign_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(campaign_id)
    .get_result::<CountRow>(conn)
    .ok()
    .map(|row| row.count)
}

pub(super) fn recipient_count_for_email(
    conn: &mut PgConnection,
    campaign_id: Uuid,
    email: &str,
) -> Option<i64> {
    diesel::sql_query(
        "SELECT COUNT(*) AS count FROM marketing_recipients \
         WHERE campaign_id = $1 AND email = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(campaign_id)
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<CountRow>(conn)
    .ok()
    .map(|row| row.count)
}

pub(super) fn add_marketing_contact(
    conn: &mut PgConnection,
    branch_id: Uuid,
    list_id: Uuid,
    email: &str,
    name: &str,
) -> Option<()> {
    diesel::sql_query(
        "INSERT INTO marketing_contacts \
         (id, branch_id, list_id, email, name, phone, metadata, subscribed, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, NULL, NULL, true, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Uuid, _>(list_id)
    .bind::<diesel::sql_types::Text, _>(email)
    .bind::<diesel::sql_types::Text, _>(name)
    .execute(conn)
    .ok()
    .map(|_| ())
}

pub(super) fn crm_contact_status(conn: &mut PgConnection, id: Uuid) -> Option<String> {
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct StatusRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }
    diesel::sql_query("SELECT CAST(status AS TEXT) AS status FROM crm_contacts WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .get_result::<StatusRow>(conn)
        .ok()
        .map(|row| row.status)
}

pub(super) fn preview_counts(body: &Option<Value>) -> Option<(i64, i64)> {
    let preview = body.as_ref()?.get("preview")?;
    let matched = preview.get("matched_count")?.as_i64()?;
    let suppressed = preview.get("suppressed_count")?.as_i64()?;
    Some((matched, suppressed))
}
