use super::*;

/// Tenant scope written alongside every calendar row. `calendars` and
/// `calendar_events` declare all three columns as `NOT NULL` without a default,
/// so a write that omits any of them is rejected by PostgreSQL.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CalendarScope {
    pub(crate) org_id: Uuid,
    pub(crate) bot_id: Uuid,
    pub(crate) branch_id: Uuid,
}

impl CalendarScope {
    /// The global scope, used when no branch can be resolved (anonymous caller
    /// or unreachable database). The read paths have always fallen back to it,
    /// and the write paths still need concrete values for the `NOT NULL`
    /// columns rather than a failed insert.
    pub(crate) const NIL: Self = Self {
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        branch_id: Uuid::nil(),
    };
}

/// Resolves the caller's org, bot and branch scope from the Authorization
/// header. The branch comes from the server-minted JWT claim or the
/// user→org binding fallback (botsecurity-core); the org is the branch's
/// owning tenant and the bot is the branch's default bot. Falls back to nil
/// (global/default scope) for anonymous callers — matching the legacy
/// behavior while making authenticated requests see their own data.
pub(crate) fn resolve_calendar_scope(
    headers: &axum::http::HeaderMap,
    conn: &mut diesel::PgConnection,
) -> CalendarScope {
    use diesel::sql_query;
    let Some(branch_id) = botsecurity_core::tenant::branch_from_claims(headers)
        .or_else(|| {
            botsecurity_core::tenant::email_from_claims(headers)
                .or_else(|| session_email(headers))
                .and_then(|email| botsecurity_core::tenant::branch_from_user_binding(conn, &email))
        })
    else {
        return CalendarScope::NIL;
    };

    #[derive(diesel::QueryableByName)]
    struct ScopeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
    }
    let row = sql_query(
        "SELECT b.org_id, COALESCE((SELECT id FROM bots WHERE branch_id = b.id \
         ORDER BY is_default_for_branch DESC, created_at ASC LIMIT 1), '00000000-0000-0000-0000-000000000000') AS bot_id \
         FROM branches b WHERE b.id = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<ScopeRow>(conn)
    .optional()
    .ok()
    .flatten();
    match row {
        Some(r) => CalendarScope {
            org_id: r.org_id,
            bot_id: r.bot_id,
            branch_id,
        },
        None => CalendarScope::NIL,
    }
}

/// Resolves the caller's (org_id, bot_id) scope for the read paths, which
/// filter on those two columns only.
pub(crate) fn resolve_scope(headers: &axum::http::HeaderMap, conn: &mut diesel::PgConnection) -> (Uuid, Uuid) {
    let scope = resolve_calendar_scope(headers, conn);
    (scope.org_id, scope.bot_id)
}

/// Scope used by the write paths, resolved from request headers with a graceful
/// fallback to the global scope when the database is unreachable.
pub(crate) fn write_scope_from_headers(state: &Arc<DbPool>, headers: &axum::http::HeaderMap) -> CalendarScope {
    match state.get() {
        Ok(mut conn) => resolve_calendar_scope(headers, &mut conn),
        Err(_) => CalendarScope::NIL,
    }
}

/// Resolves the user email from an opaque suite session token (`gb_*`) in the
/// Authorization header via the shared session cache lookup.
pub(crate) fn session_email(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth = headers.get("authorization").and_then(|v| v.to_str().ok())?;
    let token = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))?;
    botsecurity_core::lookup_session_cache(token).map(|u| u.email)
}

pub trait SecretsProvider: Send + Sync + 'static {
    fn get_value(&self, path: &str, key: &str) -> Option<String>;
}

pub trait DefaultBotProvider: Send + Sync + 'static {
    fn get_default_bot(&self, conn: &mut diesel::PgConnection) -> (Uuid, String);
}

pub trait CalendarEngineProvider: Send + Sync + 'static {}
