use super::*;

pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
/// Resolves the branch's default bot. `calendar_events.bot_id` is `NOT NULL`
/// without a default, so a write that omits it is rejected by PostgreSQL.
pub(super) fn bot_id_for(conn: &mut diesel::PgConnection, branch_id: Uuid) -> Uuid {
    use diesel::prelude::*;
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
    }
    diesel::sql_query(
        "SELECT id AS bot_id FROM bots WHERE branch_id = $1 \
         ORDER BY is_default_for_branch DESC, created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<BotRow>(conn)
    .map(|r| r.bot_id)
    .unwrap_or(Uuid::nil())
}
pub(super) fn pool_conn(
    state: &Arc<AppState>,
) -> Result<
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    (StatusCode, String),
> {
    state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))
}

// ---------------------------------------------------------------------------
// Calendar: /api/calendar/event/save
// ---------------------------------------------------------------------------
