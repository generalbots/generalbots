use axum::{extract::State, response::Html};
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::{count_query, get_conn};

pub async fn dashboard_stats(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    let members = count_query(&mut conn, "SELECT COUNT(*) as count FROM users");
    let bots = count_query(
        &mut conn,
        "SELECT COUNT(*) as count FROM bots WHERE COALESCE(is_active, true) = true",
    );
    let conversations = count_query(
        &mut conn,
        "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '7 days'",
    );
    let active_today = count_query(
        &mut conn,
        "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at > NOW() - INTERVAL '24 hours'",
    );
    let last_week = count_query(
        &mut conn,
        "SELECT COUNT(DISTINCT session_id) as count FROM message_history WHERE created_at BETWEEN NOW() - INTERVAL '14 days' AND NOW() - INTERVAL '7 days'",
    );

    let change = if last_week > 0 {
        ((active_today as f64 - last_week as f64) / last_week as f64 * 100.0) as i64
    } else {
        0
    };
    let convo_change = if change >= 0 {
        format!("+{change}% this week")
    } else {
        format!("{change}% this week")
    };
    let convo_class = if convo_change.starts_with('-') {
        "negative"
    } else {
        "positive"
    };

    Html(format!(
        r##"<div class="stat-card members">
    <div class="stat-icon">
        <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
    </div>
    <div class="stat-content">
        <span class="stat-value">{members}</span>
        <span class="stat-label">Team Members</span>
        <span class="stat-change neutral">Registered users</span>
    </div>
</div>
<div class="stat-card bots">
    <div class="stat-icon">
        <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none">
            <rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle>
            <path d="M12 7v4"></path><line x1="8" y1="16" x2="8" y2="16"></line><line x1="16" y1="16" x2="16" y2="16"></line>
        </svg>
    </div>
    <div class="stat-content">
        <span class="stat-value">{bots}</span>
        <span class="stat-label">Active Bots</span>
        <span class="stat-change neutral">Deployed</span>
    </div>
</div>
<div class="stat-card conversations">
    <div class="stat-icon">
        <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        </svg>
    </div>
    <div class="stat-content">
        <span class="stat-value">{active_today}</span>
        <span class="stat-label">Conversations Today</span>
        <span class="stat-change {convo_class}">{convo_change}</span>
    </div>
</div>
<div class="stat-card uptime">
    <div class="stat-icon">
        <svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
        </svg>
    </div>
    <div class="stat-content">
        <span class="stat-value">{conversations}</span>
        <span class="stat-label">Conversations (7d)</span>
        <span class="stat-change neutral">All sessions</span>
    </div>
</div>"##
    ))
}
