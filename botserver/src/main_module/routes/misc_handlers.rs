use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Form, Router,
};
use chrono::{NaiveDateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn branch_id_for(conn: &mut diesel::PgConnection) -> Uuid {
    use diesel::prelude::*;
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    diesel::sql_query(
        "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<BranchRow>(conn)
    .map(|r| r.branch_id)
    .unwrap_or(Uuid::nil())
}

fn pool_conn(
    state: &Arc<AppState>,
) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, (StatusCode, String)>
{
    state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))
}

// ---------------------------------------------------------------------------
// Calendar: /api/calendar/event/save
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SaveEventRequest {
    pub title: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub all_day: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}

pub async fn handle_calendar_event_save(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<SaveEventRequest>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let title = payload.title.unwrap_or_default();
    if title.trim().is_empty() {
        return Html("<div class=\"calendar-error\">Title is required</div>".to_string());
    }

    let id = Uuid::new_v4();
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<div class=\"calendar-error\">{}</div>", html_escape(&e.1))),
    };
    let branch = branch_id_for(&mut conn);

    let start_str = payload.start.unwrap_or_default();
    let end_str = payload.end.unwrap_or_default();
    let parse = |s: &str| -> Option<chrono::DateTime<Utc>> {
        if s.is_empty() {
            return None;
        }
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
            .ok()
            .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    };
    let start = parse(&start_str).unwrap_or_else(Utc::now);
    let end = parse(&end_str).unwrap_or(start + chrono::Duration::hours(1));
    let all_day = matches!(payload.all_day.as_deref(), Some("on") | Some("true"));

    let _ = diesel::sql_query(
        "INSERT INTO calendar_events (id, org_id, branch_id, calendar_id, owner_id, title, location, start_time, end_time, all_day, status, visibility, busy_status, reminders, attendees, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'confirmed', 'default', 'busy', '[]', '[]', '{}')",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Uuid, _>(&branch)
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Uuid, _>(&Uuid::nil())
    .bind::<diesel::sql_types::Text, _>(&title)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.location)
    .bind::<diesel::sql_types::Timestamptz, _>(&start)
    .bind::<diesel::sql_types::Timestamptz, _>(&end)
    .bind::<diesel::sql_types::Bool, _>(all_day)
    .execute(&mut conn);

    Html(format!(
        r#"<div class="event-saved" data-event-id="{id}"><p>Event "{title}" saved</p></div>"#,
        id = id,
        title = html_escape(&title),
    ))
}

// ---------------------------------------------------------------------------
// Goals: /api/goals/objectives/new (form modal)
// ---------------------------------------------------------------------------

pub async fn handle_goals_objective_new() -> impl IntoResponse {
    Html(
        r##"<div class="goals-modal-content">
        <h3>New Objective</h3>
        <form hx-post="/api/goals/objectives" hx-target="#goals-list" hx-swap="afterbegin">
            <div class="form-group">
                <label>Objective title</label>
                <input type="text" name="title" required placeholder="e.g. Grow revenue 30%" />
            </div>
            <div class="form-group">
                <label>Period</label>
                <select name="period"><option value="Q1">Q1</option><option value="Q2">Q2</option><option value="Q3">Q3</option><option value="Q4">Q4</option><option value="year">Year</option></select>
            </div>
            <div class="form-group">
                <label>Description</label>
                <textarea name="description" rows="3" placeholder="Describe the objective"></textarea>
            </div>
            <button type="submit" class="btn-primary btn-sm">Create Objective</button>
        </form>
        </div>"##,
    )
}

// ---------------------------------------------------------------------------
// Autotask: /api/ui/autotask/create
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AutotaskRequest {
    pub intent: Option<String>,
}

pub async fn handle_autotask_create(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<AutotaskRequest>,
) -> impl IntoResponse {
    let intent = payload.intent.unwrap_or_default().trim().to_string();
    if intent.is_empty() {
        return Json(serde_json::json!({ "ok": false, "error": "intent is required" }));
    }

    let response = if let Some(llm) = state.llm_provider.clone() {
        match llm
            .generate_simple(&format!(
                "You are an automation planner. Break this user request into concrete steps using \
                 available tools (send email, create task, create report, query data). Return a short \
                 plan.\n\nRequest: {intent}"
            ))
            .await
        {
            Ok(plan) => plan,
            Err(e) => {
                log::error!("Autotask LLM error: {e}");
                format!("Could not plan '{intent}'.")
            }
        }
    } else {
        format!("Planned steps for: {intent}")
    };

    Json(serde_json::json!({
        "ok": true,
        "intent": intent,
        "plan": response,
    }))
}

// ---------------------------------------------------------------------------
// Services summary: /api/services/summary
// ---------------------------------------------------------------------------

pub async fn handle_services_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let mut conn = pool_conn(&state)?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ServiceRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }

    let rows: Vec<ServiceRow> = diesel::sql_query(
        "SELECT 'api' AS name, 'online' AS status",
    )
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let mut services = Vec::new();
    for r in &rows {
        services.push(serde_json::json!({ "name": r.name, "status": r.status }));
    }

    Ok(Json(serde_json::json!({ "services": services, "total": services.len() })))
}

// ---------------------------------------------------------------------------
// Products pricelists: /api/products/pricelists
// ---------------------------------------------------------------------------

pub async fn handle_products_pricelists(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let mut conn = pool_conn(&state)?;

    // Issue #738: pricelists are never anonymous. Anonymous callers are
    // already rejected by the auth middleware (this path is not public);
    // authenticated callers without a tenant binding resolve to the global
    // (nil) branch scope, mirroring campaigns and the seeded global catalog.
    let branch_id = botcontacts::scope::branch_from_jwt_pool(&headers, &state.conn)
        .unwrap_or(Uuid::nil());

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PriceRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        price: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        currency: String,
    }

    let rows: Vec<PriceRow> = diesel::sql_query(
        "SELECT id, name, price, currency FROM products \
         WHERE branch_id = $1 AND is_public = true \
         ORDER BY name ASC LIMIT 100",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let pricelists: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "price": r.price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
                "currency": r.currency,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "pricelists": pricelists })))
}

// ---------------------------------------------------------------------------
// Contacts search: /api/contacts/search
// ---------------------------------------------------------------------------

pub async fn handle_contacts_search(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let q = params.get("q").cloned().unwrap_or_default();
    let mut conn = pool_conn(&state)?;

    // Tenant scoping (issue #738): the search only ever returns contacts of
    // the authenticated user's workspace branch. The branch is derived from
    // the server-minted JWT claim, the session cache, or the crm_contacts
    // owner row — never from client input. Unauthenticated requests get no
    // rows (the middleware rejects them before this handler anyway).
    let branch_id = botcontacts::scope::branch_from_jwt_pool(&headers, &state.conn)
        .unwrap_or(Uuid::nil());

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ContactRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        first_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        last_name: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        email: Option<String>,
    }

    fn display_name(row: &ContactRow) -> String {
        let full = [row.first_name.as_deref(), row.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if full.is_empty() {
            row.email.clone().unwrap_or_else(|| "Unknown".to_string())
        } else {
            full
        }
    }

    let pattern = format!("%{q}%");
    let rows: Vec<ContactRow> = if q.is_empty() {
        diesel::sql_query(
            "SELECT id, first_name, last_name, email FROM crm_contacts \
             WHERE branch_id = $1 ORDER BY first_name ASC, last_name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?
    } else {
        diesel::sql_query(
            "SELECT id, first_name, last_name, email FROM crm_contacts \
             WHERE branch_id = $1 \
               AND (LOWER(first_name) LIKE LOWER($2) \
                    OR LOWER(last_name) LIKE LOWER($2) \
                    OR LOWER(email) LIKE LOWER($2)) \
             ORDER BY first_name ASC, last_name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(&pattern)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?
    };

    let contacts: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": display_name(r),
                "email": r.email,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "contacts": contacts })))
}

// ---------------------------------------------------------------------------
// DNS records: /api/dns/list, /api/dns/register, /api/dns/remove,
//              /api/dns/:id/edit, /api/dns/search
// ---------------------------------------------------------------------------

pub async fn handle_dns_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<tr><td colspan=\"7\">{}</td></tr>", html_escape(&e.1))),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct DnsRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        hostname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        record_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        target: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        ttl: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }

    let rows: Vec<DnsRow> = diesel::sql_query(
        "SELECT id, hostname, record_type, target, ttl, status FROM dns_records ORDER BY hostname ASC LIMIT 200",
    )
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html("<tr><td colspan=\"7\" class=\"empty-cell\">No DNS records yet</td></tr>".to_string());
    }

    let mut html = String::new();
    for r in &rows {
        html.push_str(&format!(
            r#"<tr>
    <td>{hostname}</td><td>{rtype}</td><td>{target}</td><td>{ttl}</td>
    <td><span class="status-badge {status}">{status}</span></td>
    <td>{created}</td>
    <td>
        <button class="btn-icon" onclick="openEditModal('{id}')">✏️</button>
        <button class="btn-icon" onclick="openRemoveModal('{id}')">🗑️</button>
    </td>
</tr>"#,
            hostname = html_escape(&r.hostname),
            rtype = html_escape(&r.record_type),
            target = html_escape(&r.target),
            ttl = r.ttl,
            status = html_escape(&r.status),
            created = "-",
            id = r.id,
        ));
    }
    Html(html)
}

pub async fn handle_dns_search(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let q = params.get("q").cloned().unwrap_or_default();
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<tr><td colspan=\"7\">{}</td></tr>", html_escape(&e.1))),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct DnsRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        hostname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        record_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        target: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        ttl: i32,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }

    let pattern = format!("%{q}%");
    let rows: Vec<DnsRow> = if q.is_empty() {
        diesel::sql_query("SELECT id, hostname, record_type, target, ttl, status FROM dns_records ORDER BY hostname ASC LIMIT 200")
            .load(&mut conn)
            .unwrap_or_default()
    } else {
        diesel::sql_query(
            "SELECT id, hostname, record_type, target, ttl, status FROM dns_records WHERE LOWER(hostname) LIKE LOWER($1) ORDER BY hostname ASC LIMIT 200",
        )
        .bind::<diesel::sql_types::Text, _>(&pattern)
        .load(&mut conn)
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return Html("<tr><td colspan=\"7\" class=\"empty-cell\">No DNS records match</td></tr>".to_string());
    }

    let mut html = String::new();
    for r in &rows {
        html.push_str(&format!(
            r#"<tr>
    <td>{hostname}</td><td>{rtype}</td><td>{target}</td><td>{ttl}</td>
    <td><span class="status-badge {status}">{status}</span></td>
    <td>-</td>
    <td>
        <button class="btn-icon" onclick="openEditModal('{id}')">✏️</button>
        <button class="btn-icon" onclick="openRemoveModal('{id}')">🗑️</button>
    </td>
</tr>"#,
            hostname = html_escape(&r.hostname),
            rtype = html_escape(&r.record_type),
            target = html_escape(&r.target),
            ttl = r.ttl,
            status = html_escape(&r.status),
            id = r.id,
        ));
    }
    Html(html)
}

#[derive(Deserialize)]
pub struct DnsRecordRequest {
    pub hostname: Option<String>,
    pub record_type: Option<String>,
    pub target: Option<String>,
    pub ttl: Option<String>,
}

pub async fn handle_dns_register(
    State(state): State<Arc<AppState>>,
    Form(form): Form<DnsRecordRequest>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let hostname = form.hostname.unwrap_or_default().trim().to_string();
    if hostname.is_empty() {
        return Html("<div class=\"dns-error\">Hostname is required</div>".to_string());
    }
    let record_type = form.record_type.unwrap_or_else(|| "A".to_string());
    let target = form.target.unwrap_or_default();
    let ttl = form.ttl.and_then(|t| t.parse::<i32>().ok()).unwrap_or(300);
    let id = Uuid::new_v4();

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<div class=\"dns-error\">{}</div>", html_escape(&e.1))),
    };

    let _ = diesel::sql_query(
        "INSERT INTO dns_records (id, hostname, record_type, target, ttl, status) VALUES ($1, $2, $3, $4, $5, 'active') ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Text, _>(&hostname)
    .bind::<diesel::sql_types::Text, _>(&record_type)
    .bind::<diesel::sql_types::Text, _>(&target)
    .bind::<diesel::sql_types::Int4, _>(ttl)
    .execute(&mut conn);

    Html(format!(
        r#"<div class="dns-success">DNS record <strong>{hostname}</strong> registered</div>"#,
        hostname = html_escape(&hostname),
    ))
}

pub async fn handle_dns_edit_form(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<p>{}</p>", html_escape(&e.1))),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct DnsRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        hostname: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        record_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        target: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        ttl: i32,
    }

    let row: Option<DnsRow> = diesel::sql_query(
        "SELECT hostname, record_type, target, ttl FROM dns_records WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .get_result(&mut conn)
    .ok();

    match row {
        Some(r) => Html(format!(
            r##"<form hx-post="/api/dns/register" hx-target="#edit-dns-form-container" hx-swap="innerHTML">
            <div class="form-group"><label>Hostname</label><input type="text" name="hostname" value="{hostname}" required /></div>
            <div class="form-group"><label>Type</label><input type="text" name="record_type" value="{rtype}" /></div>
            <div class="form-group"><label>Target/IP</label><input type="text" name="target" value="{target}" /></div>
            <div class="form-group"><label>TTL</label><input type="number" name="ttl" value="{ttl}" /></div>
            <button type="submit" class="btn-primary btn-sm">Save</button>
        </form>"##,
            hostname = html_escape(&r.hostname),
            rtype = html_escape(&r.record_type),
            target = html_escape(&r.target),
            ttl = r.ttl,
        )),
        None => Html("<p class=\"text-muted\">Record not found</p>".to_string()),
    }
}

#[derive(Deserialize)]
pub struct DnsRemoveRequest {
    pub id: Option<String>,
}

pub async fn handle_dns_remove(
    State(state): State<Arc<AppState>>,
    Form(form): Form<DnsRemoveRequest>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let id = form.id.and_then(|s| Uuid::parse_str(&s).ok());
    let Some(id) = id else {
        return Html("<div class=\"dns-error\">Invalid record id</div>".to_string());
    };

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return Html(format!("<div class=\"dns-error\">{}</div>", html_escape(&e.1))),
    };

    let _ = diesel::sql_query("DELETE FROM dns_records WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn);

    Html("<div class=\"dns-success\">DNS record removed</div>".to_string())
}

// ---------------------------------------------------------------------------
// User orgs: /api/user/current-org, /api/user/organizations
// ---------------------------------------------------------------------------

pub async fn handle_user_current_org(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let mut conn = pool_conn(&state)?;

    // Honor the persisted selection made via PUT /api/user/current-org.
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PrefRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        value: String,
    }
    let saved: Option<PrefRow> = diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences \
         WHERE preference_key = 'current_org_id' ORDER BY updated_at DESC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();
    if let Some(p) = saved {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&p.value) {
            if let Some(id_str) = parsed.as_str().or_else(|| parsed.get("org_id").and_then(|v| v.as_str())) {
                if let Ok(org_id) = Uuid::parse_str(id_str) {
                    #[derive(diesel::QueryableByName)]
                    #[diesel(check_for_backend(diesel::pg::Pg))]
                    struct OrgRow {
                        #[diesel(sql_type = diesel::sql_types::Uuid)]
                        id: Uuid,
                        #[diesel(sql_type = diesel::sql_types::Text)]
                        name: String,
                    }
                    let row: Option<OrgRow> = diesel::sql_query(
                        "SELECT org_id AS id, name FROM organizations WHERE org_id = $1 LIMIT 1",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(org_id)
                    .get_result(&mut conn)
                    .ok();
                    if let Some(r) = row {
                        return Ok(Json(serde_json::json!({ "id": r.id, "name": r.name })));
                    }
                }
            }
        }
    }

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let row: Option<OrgRow> = diesel::sql_query(
        "SELECT org_id AS id, name FROM organizations ORDER BY created_at ASC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();

    match row {
        Some(r) => Ok(Json(serde_json::json!({ "id": r.id, "name": r.name }))),
        None => Ok(Json(serde_json::json!({ "id": Uuid::nil(), "name": "Default Organization" }))),
    }
}

pub async fn handle_user_organizations(
    State(state): State<Arc<AppState>>,
) -> Result<Html<String>, (StatusCode, String)> {
    use diesel::prelude::*;

    let mut conn = pool_conn(&state)?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let rows: Vec<OrgRow> = diesel::sql_query(
        "SELECT org_id AS id, name FROM organizations ORDER BY created_at ASC LIMIT 50",
    )
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let mut items = String::new();
    for r in rows {
        items.push_str(&format!(
            r##"<div class="org-dropdown-item" onclick="selectOrganization('{id}', '{name}', 'Member')">
    <span class="org-item-name">{name}</span>
</div>"##,
            id = r.id,
            name = html_escape(&r.name),
        ));
    }

    Ok(Html(items))
}

/// PUT /api/user/current-org
///
/// Persists the user's selected organization in `user_preferences`
/// (key `current_org_id`). The user is resolved from the bearer session —
/// never from client input.
pub async fn handle_user_current_org_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let org_id = payload
        .get("org_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or((StatusCode::BAD_REQUEST, "org_id must be a valid UUID".to_string()))?;

    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|a| a.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let user_id = match token {
        Some(t) => {
            let session = {
                let cache = botcoredirectory::auth_routes::SESSION_CACHE.read().await;
                cache.get(&t).cloned()
            };
            match session {
                Some(s) => s.user_id,
                None => {
                    return Err((StatusCode::UNAUTHORIZED, "No valid session for this token".to_string()));
                }
            }
        }
        None => return Err((StatusCode::UNAUTHORIZED, "Missing bearer token".to_string())),
    };

    let mut conn = pool_conn(&state)?;

    let _ = diesel::sql_query(
        "INSERT INTO user_preferences (id, user_id, preference_key, preference_value, created_at, updated_at) \
         VALUES ($1, $2, 'current_org_id', $3::jsonb, NOW(), NOW()) \
         ON CONFLICT (user_id, preference_key) \
         DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&user_id)
    .bind::<diesel::sql_types::Text, _>(&org_id.to_string())
    .execute(&mut conn);

    Ok(Json(serde_json::json!({ "success": true, "org_id": org_id })))
}

// ---------------------------------------------------------------------------
// Sandbox connection details: /api/v2/sandbox/connection-details
// ---------------------------------------------------------------------------

pub async fn handle_sandbox_connection_details() -> impl IntoResponse {
    Json(serde_json::json!({
        "host": "localhost",
        "ports": { "api": 8080, "ws": 8080 },
        "secure": false,
        "sandbox": true,
    }))
}

pub fn configure_misc_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/calendar/event/save", post(handle_calendar_event_save))
        .route("/api/goals/objectives/new", get(handle_goals_objective_new))
        .route("/api/ui/autotask/create", post(handle_autotask_create))
        .route("/api/services/summary", get(handle_services_summary))
        .route("/api/products/pricelists", get(handle_products_pricelists))
        .route("/api/contacts/search", get(handle_contacts_search))
        .route("/api/dns/list", get(handle_dns_list))
        .route("/api/dns/search", get(handle_dns_search))
        .route("/api/dns/register", post(handle_dns_register))
        .route("/api/dns/remove", post(handle_dns_remove))
        .route("/api/dns/:id/edit", get(handle_dns_edit_form))
        .route("/api/user/current-org", get(handle_user_current_org).put(handle_user_current_org_put))
        .route("/api/user/organizations", get(handle_user_organizations))
        .route("/api/v2/sandbox/connection-details", get(handle_sandbox_connection_details))
}
