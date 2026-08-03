use axum::{extract::State, response::Html};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::{format_number, get_conn};

pub async fn dashboard_members(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        role: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: DateTime<Utc>,
    }

    let users: Vec<UserRow> = diesel::sql_query(
        "SELECT COALESCE(username, email) as name,
         CASE WHEN is_admin THEN 'Admin' ELSE 'Member' END as role,
         COALESCE(is_active, true) as active, updated_at
         FROM users ORDER BY updated_at DESC LIMIT 6",
    )
    .load::<UserRow>(&mut conn)
    .unwrap_or_default();

    let mut items = String::new();
    for user in users {
        let (indicator, status) = if !user.active {
            ("offline", "Inactive")
        } else if Utc::now().signed_duration_since(user.updated_at).num_minutes() < 30 {
            ("online", "Online")
        } else {
            ("away", "Away")
        };
        items.push_str(&format!(
            r##"<div class="member-item">
    <div class="member-avatar">
        <img src="/suite/assets/avatars/default.svg" alt="User avatar">
        <span class="status-indicator {indicator}"></span>
    </div>
    <div class="member-info">
        <span class="member-name">{name}</span>
        <span class="member-role">{role}</span>
    </div>
    <span class="member-status {indicator}">{status}</span>
</div>"##,
            name = user.name,
            role = user.role,
        ));
    }

    Html(items)
}

pub async fn dashboard_roles(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct RoleRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        role: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let rows: Vec<RoleRow> = diesel::sql_query(
        "SELECT CASE WHEN is_admin THEN 'Admin' ELSE 'Member' END as role, COUNT(*) as count
         FROM users GROUP BY role",
    )
    .load::<RoleRow>(&mut conn)
    .unwrap_or_default();

    let total: i64 = rows.iter().map(|r| r.count).sum();
    let colors = [
        "var(--chart-1)",
        "var(--chart-2)",
        "var(--chart-3)",
        "var(--chart-4)",
        "var(--chart-5)",
    ];

    let mut bars = String::new();
    for (i, row) in rows.iter().enumerate() {
        let pct = if total > 0 {
            (row.count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        bars.push_str(&format!(
            r##"<div class="role-bar-item">
    <div class="role-bar-label">
        <span class="role-name">{role}</span>
        <span class="role-count">{count}</span>
    </div>
    <div class="role-bar">
        <div class="role-bar-fill" style="width: {pct:.0}%; background: {color}"></div>
    </div>
</div>"##,
            role = row.role,
            count = row.count,
            color = colors[i % colors.len()],
        ));
    }

    Html(bars)
}

pub async fn dashboard_bots(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
        active: Option<bool>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        chat_count: i64,
    }

    let bots: Vec<BotRow> = diesel::sql_query(
        "SELECT b.name, b.description, b.is_active as active,
         (SELECT COUNT(*) FROM message_history mh WHERE mh.session_id IN (SELECT id FROM user_sessions WHERE bot_id = b.bot_id)) as chat_count
         FROM bots b ORDER BY chat_count DESC LIMIT 5",
    )
    .load::<BotRow>(&mut conn)
    .unwrap_or_default();

    let mut items = String::new();
    for bot in bots {
        let running = bot.active.unwrap_or(true);
        let initials: String = bot
            .name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect();
        let desc = bot.description.unwrap_or_else(|| "—".to_string());
        let status_class = if running { "running" } else { "paused" };
        let status_text = if running { "Running" } else { "Paused" };
        items.push_str(&format!(
            r##"<div class="bot-item">
    <div class="bot-avatar">{initials}</div>
    <div class="bot-info">
        <span class="bot-name">{name}</span>
        <span class="bot-desc">{desc}</span>
    </div>
    <div class="bot-metrics">
        <span class="bot-metric">{chats} chats today</span>
    </div>
    <span class="bot-status {status_class}">{status_text}</span>
</div>"##,
            name = bot.name,
            chats = format_number(bot.chat_count),
        ));
    }

    Html(items)
}
