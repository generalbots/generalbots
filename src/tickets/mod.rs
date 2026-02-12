pub mod ui;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{
    support_tickets, ticket_canned_responses, ticket_categories, ticket_comments,
    ticket_sla_policies, ticket_tags,
};
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = support_tickets)]
pub struct SupportTicket {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub ticket_number: String,
    pub subject: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub category: Option<String>,
    pub source: String,
    pub requester_id: Option<Uuid>,
    pub requester_email: Option<String>,
    pub requester_name: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub first_response_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub satisfaction_rating: Option<i32>,
    pub tags: Vec<String>,
    pub custom_fields: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = ticket_comments)]
pub struct TicketComment {
    pub id: Uuid,
    pub ticket_id: Uuid,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub content: String,
    pub is_internal: bool,
    pub attachments: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = ticket_sla_policies)]
pub struct TicketSlaPolicy {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub priority: String,
    pub first_response_hours: i32,
    pub resolution_hours: i32,
    pub business_hours_only: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = ticket_canned_responses)]
pub struct TicketCannedResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub shortcut: Option<String>,
    pub created_by: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = ticket_categories)]
pub struct TicketCategory {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = ticket_tags)]
pub struct TicketTag {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub requester_email: Option<String>,
    pub requester_name: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketRequest {
    pub subject: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTicketRequest {
    pub assignee_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusRequest {
    pub status: String,
    pub resolution: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub is_internal: Option<bool>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCannedResponseRequest {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub shortcut: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub requester_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TicketStats {
    pub total_tickets: i64,
    pub open_tickets: i64,
    pub pending_tickets: i64,
    pub resolved_tickets: i64,
    pub closed_tickets: i64,
    pub avg_resolution_hours: f64,
    pub overdue_tickets: i64,
}

#[derive(Debug, Serialize)]
pub struct TicketWithComments {
    pub ticket: SupportTicket,
    pub comments: Vec<TicketComment>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn generate_ticket_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("TKT-{:06}", count + 1)
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let ticket_number = generate_ticket_number(&mut conn, org_id);

    let due_date = req
        .due_date
        .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
        .map(|d| d.with_timezone(&Utc));

    let ticket = SupportTicket {
        id,
        org_id,
        bot_id,
        ticket_number,
        subject: req.subject,
        description: req.description,
        status: "open".to_string(),
        priority: req.priority.unwrap_or_else(|| "medium".to_string()),
        category: req.category,
        source: req.source.unwrap_or_else(|| "web".to_string()),
        requester_id: None,
        requester_email: req.requester_email,
        requester_name: req.requester_name,
        assignee_id: req.assignee_id,
        team_id: None,
        due_date,
        first_response_at: None,
        resolved_at: None,
        closed_at: None,
        satisfaction_rating: None,
        tags: req.tags.unwrap_or_default(),
        custom_fields: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(support_tickets::table)
        .values(&ticket)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(ticket))
}

pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SupportTicket>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(status) = query.status {
        q = q.filter(support_tickets::status.eq(status));
    }

    if let Some(priority) = query.priority {
        q = q.filter(support_tickets::priority.eq(priority));
    }

    if let Some(category) = query.category {
        q = q.filter(support_tickets::category.eq(category));
    }

    if let Some(assignee_id) = query.assignee_id {
        q = q.filter(support_tickets::assignee_id.eq(assignee_id));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            support_tickets::subject
                .ilike(pattern.clone())
                .or(support_tickets::description.ilike(pattern.clone()))
                .or(support_tickets::ticket_number.ilike(pattern)),
        );
    }

    let tickets: Vec<SupportTicket> = q
        .order(support_tickets::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(tickets))
}

pub async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let ticket: SupportTicket = support_tickets::table
        .filter(support_tickets::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    Ok(Json(ticket))
}

pub async fn get_ticket_with_comments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TicketWithComments>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let ticket: SupportTicket = support_tickets::table
        .filter(support_tickets::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    let comments: Vec<TicketComment> = ticket_comments::table
        .filter(ticket_comments::ticket_id.eq(id))
        .order(ticket_comments::created_at.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(TicketWithComments { ticket, comments }))
}

pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTicketRequest>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
        .set(support_tickets::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(subject) = req.subject {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::subject.eq(subject))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(description) = req.description {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::description.eq(description))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(priority) = req.priority {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::priority.eq(priority))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(category) = req.category {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::category.eq(category))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(assignee_id) = req.assignee_id {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::assignee_id.eq(Some(assignee_id)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(tags) = req.tags {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::tags.eq(tags))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_ticket(State(state), Path(id)).await
}

pub async fn assign_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignTicketRequest>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
        .set((
            support_tickets::assignee_id.eq(Some(req.assignee_id)),
            support_tickets::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_ticket(State(state), Path(id)).await
}

pub async fn change_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
        .set((
            support_tickets::status.eq(&req.status),
            support_tickets::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if req.status == "resolved" {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::resolved_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if req.status == "closed" {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
            .set(support_tickets::closed_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_ticket(State(state), Path(id)).await
}

pub async fn resolve_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    change_status(
        State(state),
        Path(id),
        Json(ChangeStatusRequest {
            status: "resolved".to_string(),
            resolution: None,
        }),
    )
    .await
}

pub async fn close_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    change_status(
        State(state),
        Path(id),
        Json(ChangeStatusRequest {
            status: "closed".to_string(),
            resolution: None,
        }),
    )
    .await
}

pub async fn reopen_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupportTicket>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(support_tickets::table.filter(support_tickets::id.eq(id)))
        .set((
            support_tickets::status.eq("open"),
            support_tickets::resolved_at.eq(None::<DateTime<Utc>>),
            support_tickets::closed_at.eq(None::<DateTime<Utc>>),
            support_tickets::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_ticket(State(state), Path(id)).await
}

pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(support_tickets::table.filter(support_tickets::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_comment(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<TicketComment>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let comment = TicketComment {
        id,
        ticket_id,
        author_id: None,
        author_name: req.author_name,
        author_email: req.author_email,
        content: req.content,
        is_internal: req.is_internal.unwrap_or(false),
        attachments: serde_json::json!([]),
        created_at: now,
    };

    diesel::insert_into(ticket_comments::table)
        .values(&comment)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    let ticket: SupportTicket = support_tickets::table
        .filter(support_tickets::id.eq(ticket_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Ticket not found".to_string()))?;

    if ticket.first_response_at.is_none() && !comment.is_internal {
        diesel::update(support_tickets::table.filter(support_tickets::id.eq(ticket_id)))
            .set(support_tickets::first_response_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    diesel::update(support_tickets::table.filter(support_tickets::id.eq(ticket_id)))
        .set(support_tickets::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    Ok(Json(comment))
}

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Path(ticket_id): Path<Uuid>,
) -> Result<Json<Vec<TicketComment>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let comments: Vec<TicketComment> = ticket_comments::table
        .filter(ticket_comments::ticket_id.eq(ticket_id))
        .order(ticket_comments::created_at.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(comments))
}

pub async fn get_ticket_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TicketStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let total_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let open_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("open"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let pending_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("pending"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let resolved_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("resolved"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let closed_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let now = Utc::now();
    let overdue_tickets: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.ne("closed"))
        .filter(support_tickets::status.ne("resolved"))
        .filter(support_tickets::due_date.lt(now))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let stats = TicketStats {
        total_tickets,
        open_tickets,
        pending_tickets,
        resolved_tickets,
        closed_tickets,
        avg_resolution_hours: 0.0,
        overdue_tickets,
    };

    Ok(Json(stats))
}

pub async fn list_overdue_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SupportTicket>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let now = Utc::now();

    let tickets: Vec<SupportTicket> = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.ne("closed"))
        .filter(support_tickets::status.ne("resolved"))
        .filter(support_tickets::due_date.lt(now))
        .order(support_tickets::due_date.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(tickets))
}

pub async fn list_canned_responses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketCannedResponse>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let responses: Vec<TicketCannedResponse> = ticket_canned_responses::table
        .filter(ticket_canned_responses::org_id.eq(org_id))
        .filter(ticket_canned_responses::bot_id.eq(bot_id))
        .filter(ticket_canned_responses::is_active.eq(true))
        .order(ticket_canned_responses::title.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(responses))
}

pub async fn create_canned_response(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCannedResponseRequest>,
) -> Result<Json<TicketCannedResponse>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let response = TicketCannedResponse {
        id,
        org_id,
        bot_id,
        title: req.title,
        content: req.content,
        category: req.category,
        shortcut: req.shortcut,
        created_by: None,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(ticket_canned_responses::table)
        .values(&response)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(response))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketCategory>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let categories: Vec<TicketCategory> = ticket_categories::table
        .filter(ticket_categories::org_id.eq(org_id))
        .filter(ticket_categories::bot_id.eq(bot_id))
        .filter(ticket_categories::is_active.eq(true))
        .order(ticket_categories::sort_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(categories))
}

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<TicketCategory>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let max_order: Option<i32> = ticket_categories::table
        .filter(ticket_categories::org_id.eq(org_id))
        .filter(ticket_categories::bot_id.eq(bot_id))
        .select(diesel::dsl::max(ticket_categories::sort_order))
        .first(&mut conn)
        .unwrap_or(None);

    let category = TicketCategory {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        parent_id: req.parent_id,
        color: req.color,
        icon: req.icon,
        sort_order: max_order.unwrap_or(0) + 1,
        is_active: true,
        created_at: now,
    };

    diesel::insert_into(ticket_categories::table)
        .values(&category)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(category))
}

pub async fn list_sla_policies(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketSlaPolicy>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let policies: Vec<TicketSlaPolicy> = ticket_sla_policies::table
        .filter(ticket_sla_policies::org_id.eq(org_id))
        .filter(ticket_sla_policies::bot_id.eq(bot_id))
        .filter(ticket_sla_policies::is_active.eq(true))
        .order(ticket_sla_policies::priority.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(policies))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TicketTag>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let tags: Vec<TicketTag> = ticket_tags::table
        .filter(ticket_tags::org_id.eq(org_id))
        .filter(ticket_tags::bot_id.eq(bot_id))
        .order(ticket_tags::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(tags))
}

pub fn configure_tickets_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tickets", get(list_tickets).post(create_ticket))
        .route("/api/tickets/stats", get(get_ticket_stats))
        .route("/api/tickets/overdue", get(list_overdue_tickets))
        .route("/api/tickets/:id", get(get_ticket).put(update_ticket).delete(delete_ticket))
        .route("/api/tickets/:id/full", get(get_ticket_with_comments))
        .route("/api/tickets/:id/assign", put(assign_ticket))
        .route("/api/tickets/:id/status", put(change_status))
        .route("/api/tickets/:id/resolve", put(resolve_ticket))
        .route("/api/tickets/:id/close", put(close_ticket))
        .route("/api/tickets/:id/reopen", put(reopen_ticket))
        .route("/api/tickets/:id/comments", get(list_comments).post(add_comment))
        .route("/api/tickets/canned", get(list_canned_responses).post(create_canned_response))
        .route("/api/tickets/categories", get(list_categories).post(create_category))
        .route("/api/tickets/sla", get(list_sla_policies))
        .route("/api/tickets/tags", get(list_tags))
}
