use axum::{extract::State, response::Html};
use diesel::RunQueryDsl;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::{format_number, get_conn};

pub async fn admin_bots(State(state): State<Arc<AppState>>) -> Html<String> {
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
        is_active: Option<bool>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
        is_public: Option<bool>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        chat_count: i64,
    }

    let bots: Vec<BotRow> = diesel::sql_query(
        "SELECT b.name, b.description, b.is_active, b.is_public,
         (SELECT COUNT(*) FROM message_history mh WHERE mh.session_id IN (SELECT id FROM user_sessions WHERE bot_id = b.id)) as chat_count
         FROM bots b ORDER BY chat_count DESC",
    )
    .load::<BotRow>(&mut conn)
    .unwrap_or_default();

    if bots.is_empty() {
        return Html(
            r#"<div class="empty-state"><p>No bots registered yet.</p></div>"#.to_string(),
        );
    }

    let mut rows = String::new();
    for bot in bots {
        let running = bot.is_active.unwrap_or(true);
        let status_class = if running { "running" } else { "paused" };
        let status_text = if running { "Running" } else { "Paused" };
        let visibility = if bot.is_public.unwrap_or(false) {
            "Public"
        } else {
            "Private"
        };
        let initials: String = bot
            .name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect();
        let desc = bot.description.unwrap_or_else(|| "—".to_string());
        rows.push_str(&format!(
            r##"<div class="bot-item">
    <div class="bot-avatar">{initials}</div>
    <div class="bot-info">
        <span class="bot-name">{name}</span>
        <span class="bot-desc">{desc}</span>
    </div>
    <div class="bot-metrics">
        <span class="bot-metric">{chats} chats</span>
        <span class="bot-metric">{visibility}</span>
    </div>
    <span class="bot-status {status_class}">{status_text}</span>
</div>"##,
            name = bot.name,
            desc = desc,
            chats = format_number(bot.chat_count),
            visibility = visibility,
        ));
    }

    Html(format!(
        r##"<div class="bots-page">
    <div class="page-header">
        <h1>Bots</h1>
        <p class="subtitle">All registered bots and their activity</p>
    </div>
    <div class="bots-list">{rows}</div>
</div>"##
    ))
}
