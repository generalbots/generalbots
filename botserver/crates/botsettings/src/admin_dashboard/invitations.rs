use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::get_conn;

pub async fn dashboard_invitations(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct InviteRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        role: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
    }

    let invites: Vec<InviteRow> = diesel::sql_query(
        "SELECT id, email, role, created_at, expires_at FROM organization_invitations
         WHERE status = 'pending' ORDER BY created_at DESC LIMIT 10",
    )
    .load::<InviteRow>(&mut conn)
    .unwrap_or_default();

    if invites.is_empty() {
        return Html(
            r##"<div class="invitation-empty"><p>No pending invitations</p></div>"##.to_string(),
        );
    }

    let now = Utc::now();
    let mut items = String::new();
    for invite in invites {
        let expires = invite
            .expires_at
            .unwrap_or(invite.created_at + ChronoDuration::days(7));
        let expires_in = (expires - now).num_days().max(0);
        let exp_class = if expires_in <= 2 { " warning" } else { "" };
        items.push_str(&format!(
            r##"<div class="invitation-item">
    <div class="invitation-info">
        <span class="invitation-email">{email}</span>
        <span class="invitation-role">{role}</span>
    </div>
    <div class="invitation-meta">
        <span class="invitation-sent">{sent}</span>
        <span class="invitation-expires{exp_class}">Expires in {expires_in} days</span>
    </div>
    <div class="invitation-actions">
        <button class="btn-icon" title="Resend invitation" hx-post="/api/admin/invitations/resend/{id}" hx-swap="none">
            <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <polyline points="23 4 23 10 17 10"></polyline>
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
            </svg>
        </button>
        <button class="btn-icon danger" title="Revoke invitation" hx-delete="/api/admin/invitations/{id}" hx-swap="none" hx-confirm="Revoke this invitation?">
            <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
        </button>
    </div>
</div>"##,
            email = invite.email,
            role = invite.role,
            sent = format!("Sent {}", relative_time(invite.created_at)),
            id = invite.id,
        ));
    }

    Html(items)
}

pub async fn resend_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> StatusCode {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    diesel::sql_query(
        "UPDATE organization_invitations SET updated_at = NOW() WHERE id = $1 AND status = 'pending'",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn)
    .map(|n| if n > 0 { StatusCode::OK } else { StatusCode::NOT_FOUND })
    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn revoke_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<uuid::Uuid>,
) -> StatusCode {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    diesel::sql_query(
        "UPDATE organization_invitations SET status = 'revoked', updated_at = NOW() WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .execute(&mut conn)
    .map(|n| if n > 0 { StatusCode::OK } else { StatusCode::NOT_FOUND })
    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn admin_invitations(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return axum::Json(serde_json::json!([])),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct InviteRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        role: String,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: DateTime<Utc>,
    }

    let rows: Vec<InviteRow> = diesel::sql_query(
        "SELECT id, email, role, status, created_at FROM organization_invitations ORDER BY created_at DESC LIMIT 50",
    )
    .load::<InviteRow>(&mut conn)
    .unwrap_or_default();

    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "email": i.email,
                "role": i.role,
                "status": i.status,
                "created_at": i.created_at,
            })
        })
        .collect();

    axum::Json(serde_json::json!(items))
}

fn relative_time(time: DateTime<Utc>) -> String {
    let diff = Utc::now() - time;
    if diff.num_days() > 0 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_hours() > 0 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_minutes() > 0 {
        format!("{}m ago", diff.num_minutes())
    } else {
        "just now".to_string()
    }
}
