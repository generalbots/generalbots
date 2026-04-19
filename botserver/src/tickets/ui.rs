use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{support_tickets, ticket_comments};
use crate::core::shared::state::AppState;
use crate::tickets::{SupportTicket, TicketComment};

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "urgent" => "<span class=\"badge badge-danger\">Urgent</span>",
        "high" => "<span class=\"badge badge-warning\">High</span>",
        "medium" => "<span class=\"badge badge-info\">Medium</span>",
        "low" => "<span class=\"badge badge-secondary\">Low</span>",
        _ => "<span class=\"badge\">Unknown</span>",
    }
}

fn status_badge(status: &str) -> &'static str {
    match status {
        "open" => "<span class=\"badge badge-primary\">Open</span>",
        "pending" => "<span class=\"badge badge-warning\">Pending</span>",
        "resolved" => "<span class=\"badge badge-success\">Resolved</span>",
        "closed" => "<span class=\"badge badge-secondary\">Closed</span>",
        _ => "<span class=\"badge\">Unknown</span>",
    }
}

fn render_empty_state(icon: &str, title: &str, description: &str) -> String {
    format!(
        "<div class=\"empty-state\">\
            <div class=\"empty-icon\">{}</div>\
            <h3>{}</h3>\
            <p>{}</p>\
        </div>",
        icon, title, description
    )
}

fn render_ticket_row(ticket: &SupportTicket) -> String {
    let requester = ticket
        .requester_name
        .as_deref()
        .or(ticket.requester_email.as_deref())
        .unwrap_or("Unknown");

    let assignee = ticket
        .assignee_id
        .map(|_| "Assigned")
        .unwrap_or("Unassigned");

    let created = ticket.created_at.format("%Y-%m-%d %H:%M").to_string();
    let hash = "#";

    format!(
        "<tr class=\"ticket-row\" data-id=\"{id}\">\
            <td class=\"ticket-number\">{number}</td>\
            <td class=\"ticket-subject\">\
                <a href=\"{hash}\" hx-get=\"/api/ui/tickets/{id}\" hx-target=\"{hash}ticket-detail\" hx-swap=\"innerHTML\">{subject}</a>\
            </td>\
            <td class=\"ticket-requester\">{requester}</td>\
            <td class=\"ticket-status\">{status}</td>\
            <td class=\"ticket-priority\">{priority}</td>\
            <td class=\"ticket-assignee\">{assignee}</td>\
            <td class=\"ticket-created\">{created}</td>\
            <td class=\"ticket-actions\">\
                <button class=\"btn-icon\" hx-put=\"/api/tickets/{id}/resolve\" hx-swap=\"none\" title=\"Resolve\">✓</button>\
                <button class=\"btn-icon\" hx-delete=\"/api/tickets/{id}\" hx-confirm=\"Delete this ticket?\" hx-swap=\"none\" title=\"Delete\">×</button>\
            </td>\
        </tr>",
        id = ticket.id,
        hash = hash,
        number = html_escape(&ticket.ticket_number),
        subject = html_escape(&ticket.subject),
        requester = html_escape(requester),
        status = status_badge(&ticket.status),
        priority = priority_badge(&ticket.priority),
        assignee = html_escape(assignee),
        created = created,
    )
}

fn render_ticket_card(ticket: &SupportTicket) -> String {
    let requester = ticket
        .requester_name
        .as_deref()
        .or(ticket.requester_email.as_deref())
        .unwrap_or("Unknown");

    let hash = "#";

    format!(
        "<div class=\"ticket-card\" data-id=\"{id}\">\
            <div class=\"ticket-card-header\">\
                <span class=\"ticket-number\">{number}</span>\
                {status}\
                {priority}\
            </div>\
            <div class=\"ticket-card-body\">\
                <h4 class=\"ticket-subject\">{subject}</h4>\
                <p class=\"ticket-requester\">From: {requester}</p>\
            </div>\
            <div class=\"ticket-card-footer\">\
                <button class=\"btn-sm\" hx-get=\"/api/ui/tickets/{id}\" hx-target=\"{hash}ticket-detail\">View</button>\
                <button class=\"btn-sm btn-success\" hx-put=\"/api/tickets/{id}/resolve\" hx-swap=\"none\">Resolve</button>\
            </div>\
        </div>",
        id = ticket.id,
        hash = hash,
        number = html_escape(&ticket.ticket_number),
        subject = html_escape(&ticket.subject),
        requester = html_escape(requester),
        status = status_badge(&ticket.status),
        priority = priority_badge(&ticket.priority),
    )
}

pub fn configure_tickets_ui_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/tickets", get(handle_tickets_list))
        .route("/api/ui/tickets/count", get(handle_tickets_count))
        .route("/api/ui/tickets/open-count", get(handle_open_count))
        .route("/api/ui/tickets/overdue-count", get(handle_overdue_count))
        .route("/api/ui/tickets/cards", get(handle_tickets_cards))
        .route("/api/ui/tickets/search", get(handle_tickets_search))
        .route("/api/ui/tickets/:id", get(handle_ticket_detail))
        .route("/api/ui/tickets/:id/comments", get(handle_ticket_comments))
        .route("/api/ui/tickets/stats/by-status", get(handle_stats_by_status))
        .route("/api/ui/tickets/stats/by-priority", get(handle_stats_by_priority))
        .route("/api/ui/tickets/stats/avg-resolution", get(handle_avg_resolution))
}

async fn handle_tickets_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("🎫", "No tickets", "Unable to load tickets"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let mut q = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(support_tickets::status.eq(status));
        }
    }

    let tickets: Vec<SupportTicket> = q
        .order(support_tickets::created_at.desc())
        .limit(50)
        .load(&mut conn)
        .unwrap_or_default();

    if tickets.is_empty() {
        return Html(render_empty_state(
            "🎫",
            "No tickets yet",
            "Create your first ticket to get started",
        ));
    }

    let mut html = String::from(
        "<table class=\"tickets-table\">\
            <thead>\
                <tr>\
                    <th>Number</th>\
                    <th>Subject</th>\
                    <th>Requester</th>\
                    <th>Status</th>\
                    <th>Priority</th>\
                    <th>Assignee</th>\
                    <th>Created</th>\
                    <th>Actions</th>\
                </tr>\
            </thead>\
            <tbody>",
    );

    for ticket in &tickets {
        html.push_str(&render_ticket_row(ticket));
    }

    html.push_str("</tbody></table>");
    Html(html)
}

async fn handle_tickets_count(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = if let Some(status) = query.status {
        if status == "all" {
            support_tickets::table
                .filter(support_tickets::org_id.eq(org_id))
                .filter(support_tickets::bot_id.eq(bot_id))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0)
        } else {
            support_tickets::table
                .filter(support_tickets::org_id.eq(org_id))
                .filter(support_tickets::bot_id.eq(bot_id))
                .filter(support_tickets::status.eq(status))
                .count()
                .get_result(&mut conn)
                .unwrap_or(0)
        }
    } else {
        support_tickets::table
            .filter(support_tickets::org_id.eq(org_id))
            .filter(support_tickets::bot_id.eq(bot_id))
            .count()
            .get_result(&mut conn)
            .unwrap_or(0)
    };

    Html(count.to_string())
}

async fn handle_open_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let count: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("open"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

async fn handle_overdue_count(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("0".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);
    let now = chrono::Utc::now();

    let count: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.ne("closed"))
        .filter(support_tickets::status.ne("resolved"))
        .filter(support_tickets::due_date.lt(now))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Html(count.to_string())
}

async fn handle_tickets_cards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatusQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("🎫", "No tickets", "Unable to load tickets"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let mut q = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(support_tickets::status.eq(status));
        }
    }

    let tickets: Vec<SupportTicket> = q
        .order(support_tickets::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if tickets.is_empty() {
        return Html(render_empty_state(
            "🎫",
            "No tickets",
            "No tickets match your criteria",
        ));
    }

    let mut html = String::from("<div class=\"tickets-grid\">");
    for ticket in &tickets {
        html.push_str(&render_ticket_card(ticket));
    }
    html.push_str("</div>");

    Html(html)
}

async fn handle_tickets_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("🔍", "Search error", "Unable to search tickets"));
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let search_term = query.q.unwrap_or_default();
    if search_term.is_empty() {
        return Html(render_empty_state(
            "🔍",
            "Enter search term",
            "Type to search tickets",
        ));
    }

    let pattern = format!("%{search_term}%");

    let tickets: Vec<SupportTicket> = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(
            support_tickets::subject
                .ilike(pattern.clone())
                .or(support_tickets::description.ilike(pattern.clone()))
                .or(support_tickets::ticket_number.ilike(pattern)),
        )
        .order(support_tickets::created_at.desc())
        .limit(20)
        .load(&mut conn)
        .unwrap_or_default();

    if tickets.is_empty() {
        return Html(render_empty_state(
            "🔍",
            "No results",
            "No tickets match your search",
        ));
    }

    let mut html = String::from("<div class=\"search-results\">");
    for ticket in &tickets {
        html.push_str(&render_ticket_card(ticket));
    }
    html.push_str("</div>");

    Html(html)
}

async fn handle_ticket_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html(render_empty_state("❌", "Error", "Unable to load ticket"));
    };

    let ticket: Result<SupportTicket, _> = support_tickets::table
        .filter(support_tickets::id.eq(id))
        .first(&mut conn);

    let Ok(ticket) = ticket else {
        return Html(render_empty_state("❌", "Not found", "Ticket not found"));
    };

    let comments: Vec<TicketComment> = ticket_comments::table
        .filter(ticket_comments::ticket_id.eq(id))
        .order(ticket_comments::created_at.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let requester = ticket
        .requester_name
        .as_deref()
        .or(ticket.requester_email.as_deref())
        .unwrap_or("Unknown");

    let description = ticket
        .description
        .as_deref()
        .unwrap_or("No description provided");

    let created = ticket.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated = ticket.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let category = ticket.category.as_deref().unwrap_or("-");

    let mut comments_html = String::new();
    for comment in &comments {
        let author = comment
            .author_name
            .as_deref()
            .or(comment.author_email.as_deref())
            .unwrap_or("Unknown");
        let comment_time = comment.created_at.format("%Y-%m-%d %H:%M").to_string();
        let internal_class = if comment.is_internal { " internal" } else { "" };
        let internal_badge = if comment.is_internal {
            "<span class=\"badge badge-warning\">Internal</span>"
        } else {
            ""
        };

        comments_html.push_str(&format!(
            "<div class=\"comment{}\">\
                <div class=\"comment-header\">\
                    <span class=\"comment-author\">{}</span>\
                    <span class=\"comment-time\">{}</span>\
                    {}\
                </div>\
                <div class=\"comment-body\">{}</div>\
            </div>",
            internal_class,
            html_escape(author),
            comment_time,
            internal_badge,
            html_escape(&comment.content),
        ));
    }

    let html = format!(
        "<div class=\"ticket-detail\">\
            <div class=\"ticket-detail-header\">\
                <h2>{}: {}</h2>\
                <div class=\"ticket-badges\">\
                    {}\
                    {}\
                </div>\
            </div>\
            <div class=\"ticket-detail-meta\">\
                <div class=\"meta-item\">\
                    <label>Requester</label>\
                    <span>{}</span>\
                </div>\
                <div class=\"meta-item\">\
                    <label>Created</label>\
                    <span>{}</span>\
                </div>\
                <div class=\"meta-item\">\
                    <label>Updated</label>\
                    <span>{}</span>\
                </div>\
                <div class=\"meta-item\">\
                    <label>Category</label>\
                    <span>{}</span>\
                </div>\
            </div>\
            <div class=\"ticket-detail-description\">\
                <h3>Description</h3>\
                <p>{}</p>\
            </div>\
            <div class=\"ticket-detail-actions\">\
                <button class=\"btn btn-success\" hx-put=\"/api/tickets/{}/resolve\" hx-swap=\"none\">Resolve</button>\
                <button class=\"btn btn-secondary\" hx-put=\"/api/tickets/{}/close\" hx-swap=\"none\">Close</button>\
                <button class=\"btn btn-warning\" hx-put=\"/api/tickets/{}/reopen\" hx-swap=\"none\">Reopen</button>\
            </div>\
            <div class=\"ticket-comments\">\
                <h3>Comments ({})</h3>\
                {}\
                <form class=\"comment-form\" hx-post=\"/api/tickets/{}/comments\" hx-target=\"#ticket-detail\" hx-swap=\"innerHTML\">\
                    <textarea name=\"content\" placeholder=\"Add a comment...\" required></textarea>\
                    <div class=\"comment-form-actions\">\
                        <label>\
                            <input type=\"checkbox\" name=\"is_internal\" value=\"true\">\
                            Internal note\
                        </label>\
                        <button type=\"submit\" class=\"btn btn-primary\">Add Comment</button>\
                    </div>\
                </form>\
            </div>\
        </div>",
        html_escape(&ticket.ticket_number),
        html_escape(&ticket.subject),
        status_badge(&ticket.status),
        priority_badge(&ticket.priority),
        html_escape(requester),
        created,
        updated,
        html_escape(category),
        html_escape(description),
        ticket.id,
        ticket.id,
        ticket.id,
        comments.len(),
        comments_html,
        ticket.id,
    );

    Html(html)
}

async fn handle_ticket_comments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("<p>Unable to load comments</p>".to_string());
    };

    let comments: Vec<TicketComment> = ticket_comments::table
        .filter(ticket_comments::ticket_id.eq(id))
        .order(ticket_comments::created_at.asc())
        .load(&mut conn)
        .unwrap_or_default();

    if comments.is_empty() {
        return Html("<p class=\"no-comments\">No comments yet</p>".to_string());
    }

    let mut html = String::new();
    for comment in &comments {
        let author = comment
            .author_name
            .as_deref()
            .or(comment.author_email.as_deref())
            .unwrap_or("Unknown");
        let comment_time = comment.created_at.format("%Y-%m-%d %H:%M").to_string();
        let internal_class = if comment.is_internal { " internal" } else { "" };

        html.push_str(&format!(
            "<div class=\"comment{}\">\
                <div class=\"comment-header\">\
                    <span class=\"comment-author\">{}</span>\
                    <span class=\"comment-time\">{}</span>\
                </div>\
                <div class=\"comment-body\">{}</div>\
            </div>",
            internal_class,
            html_escape(author),
            comment_time,
            html_escape(&comment.content),
        ));
    }

    Html(html)
}

async fn handle_stats_by_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("<p>Unable to load stats</p>".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let open: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("open"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let pending: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("pending"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let resolved: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("resolved"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let closed: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::status.eq("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let html = format!(
        "<div class=\"stats-grid\">\
            <div class=\"stat-card stat-open\">\
                <div class=\"stat-value\">{}</div>\
                <div class=\"stat-label\">Open</div>\
            </div>\
            <div class=\"stat-card stat-pending\">\
                <div class=\"stat-value\">{}</div>\
                <div class=\"stat-label\">Pending</div>\
            </div>\
            <div class=\"stat-card stat-resolved\">\
                <div class=\"stat-value\">{}</div>\
                <div class=\"stat-label\">Resolved</div>\
            </div>\
            <div class=\"stat-card stat-closed\">\
                <div class=\"stat-value\">{}</div>\
                <div class=\"stat-label\">Closed</div>\
            </div>\
        </div>",
        open, pending, resolved, closed
    );

    Html(html)
}

async fn handle_stats_by_priority(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("<p>Unable to load stats</p>".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let urgent: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::priority.eq("urgent"))
        .filter(support_tickets::status.ne("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let high: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::priority.eq("high"))
        .filter(support_tickets::status.ne("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let medium: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::priority.eq("medium"))
        .filter(support_tickets::status.ne("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let low: i64 = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::priority.eq("low"))
        .filter(support_tickets::status.ne("closed"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let html = format!(
        "<div class=\"priority-stats\">\
            <div class=\"priority-bar\">\
                <div class=\"priority-segment urgent\" style=\"flex: {};\" title=\"Urgent: {}\"></div>\
                <div class=\"priority-segment high\" style=\"flex: {};\" title=\"High: {}\"></div>\
                <div class=\"priority-segment medium\" style=\"flex: {};\" title=\"Medium: {}\"></div>\
                <div class=\"priority-segment low\" style=\"flex: {};\" title=\"Low: {}\"></div>\
            </div>\
            <div class=\"priority-legend\">\
                <span class=\"legend-item\"><span class=\"dot urgent\"></span>Urgent ({})</span>\
                <span class=\"legend-item\"><span class=\"dot high\"></span>High ({})</span>\
                <span class=\"legend-item\"><span class=\"dot medium\"></span>Medium ({})</span>\
                <span class=\"legend-item\"><span class=\"dot low\"></span>Low ({})</span>\
            </div>\
        </div>",
        urgent, urgent, high, high, medium, medium, low, low,
        urgent, high, medium, low
    );

    Html(html)
}

async fn handle_avg_resolution(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(mut conn) = state.conn.get() else {
        return Html("-".to_string());
    };

    let (org_id, bot_id) = get_bot_context(&state);

    let resolved_tickets: Vec<SupportTicket> = support_tickets::table
        .filter(support_tickets::org_id.eq(org_id))
        .filter(support_tickets::bot_id.eq(bot_id))
        .filter(support_tickets::resolved_at.is_not_null())
        .limit(100)
        .load(&mut conn)
        .unwrap_or_default();

    if resolved_tickets.is_empty() {
        return Html("-".to_string());
    }

    let total_hours: f64 = resolved_tickets
        .iter()
        .filter_map(|t| {
            t.resolved_at.map(|resolved| {
                let duration = resolved - t.created_at;
                duration.num_hours() as f64
            })
        })
        .sum();

    let avg_hours = total_hours / resolved_tickets.len() as f64;

    if avg_hours < 24.0 {
        Html(format!("{:.1}h", avg_hours))
    } else {
        Html(format!("{:.1}d", avg_hours / 24.0))
    }
}
