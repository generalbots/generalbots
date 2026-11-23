use super::*;

use botsecurity_core::tenant::{org_from_claims, user_id_from_claims};
use diesel::prelude::*;

/// Tenant scope of a calendar write. Every scope column is `NOT NULL` without
/// a default, so none of them may ever be `Uuid::nil()` (issue #2895).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CalendarScope {
    org_id: Uuid,
    bot_id: Uuid,
    branch_id: Uuid,
}

/// Pure tenant-ownership predicate for a calendar reference (issue #2895).
///
/// Every `calendars` row carries the three scope columns. A write may only
/// follow a client-supplied `calendar_id` when all three match the caller's
/// verified scope, otherwise the event would be attributed to a calendar in a
/// different tenant. Kept free of any DB/IO so the cross-tenant rule can be
/// unit-tested without a running server.
fn calendar_in_scope(org_id: Uuid, bot_id: Uuid, branch_id: Uuid, scope: &CalendarScope) -> bool {
    org_id == scope.org_id && bot_id == scope.bot_id && branch_id == scope.branch_id
}

/// Owning organization of a verified branch, used only for legacy tokens that
/// carry no `org_id` claim. The branch itself always comes from the signed JWT.
fn org_for_branch(conn: &mut diesel::PgConnection, branch_id: Uuid) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<OrgRow>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|row| row.org_id)
}

/// Resolves (or provisions) the workspace's scoped primary calendar.
fn default_calendar_id(
    conn: &mut diesel::PgConnection,
    scope: &CalendarScope,
    owner_id: Uuid,
) -> Result<Uuid, String> {
    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let existing = diesel::sql_query(
        "SELECT id FROM calendars WHERE org_id = $1 AND bot_id = $2 AND branch_id = $3 \
         ORDER BY is_primary DESC, created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .get_result::<IdRow>(conn)
    .optional()
    .map_err(|e| format!("Calendar lookup failed: {e}"))?;
    if let Some(row) = existing {
        return Ok(row.id);
    }

    let calendar_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO calendars \
         (id, org_id, bot_id, branch_id, owner_id, name, description, color, timezone, \
          is_primary, is_visible, is_shared) \
         VALUES ($1, $2, $3, $4, $5, 'Default', 'Your default calendar', '#3b82f6', 'UTC', \
                 true, true, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(calendar_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(owner_id)
    .execute(conn)
    .map_err(|e| format!("Calendar provisioning failed: {e}"))?;
    Ok(calendar_id)
}

/// Resolves the calendar an event may be written to (issue #2895).
///
/// A client-supplied id is followed only when the stored row's full scope
/// matches the caller's verified scope; a nil/absent id selects the workspace's
/// scoped primary calendar. No value ever comes from an untrusted source.
fn scoped_calendar_id(
    conn: &mut diesel::PgConnection,
    scope: &CalendarScope,
    requested: Option<Uuid>,
    owner_id: Uuid,
) -> Result<Uuid, String> {
    #[derive(diesel::QueryableByName)]
    struct ScopeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    match requested.filter(|id| !id.is_nil()) {
        Some(id) => {
            let row = diesel::sql_query(
                "SELECT org_id, bot_id, branch_id FROM calendars WHERE id = $1 LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result::<ScopeRow>(conn)
            .optional()
            .map_err(|e| format!("Calendar lookup failed: {e}"))?;
            match row {
                Some(row) if calendar_in_scope(row.org_id, row.bot_id, row.branch_id, scope) => {
                    Ok(id)
                }
                // Hide existence: an id from another tenant is never followed.
                _ => Err("This calendar is not available in your workspace.".to_string()),
            }
        }
        None => default_calendar_id(conn, scope, owner_id),
    }
}

pub async fn handle_calendar_event_save(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Form(payload): Form<SaveEventRequest>,
) -> impl IntoResponse {
    let title = payload.title.unwrap_or_default();
    if title.trim().is_empty() {
        return Html("<div class=\"calendar-error\">Title is required</div>".to_string());
    }

    // #2895 — branch and org come from verified server-side identity, never
    // from the form body and never from a global "oldest branch" lookup. The
    // canonical resolver accepts the signed `branch_id` claim or, for opaque
    // suite tokens, a verified email → server-side tenant binding. A request
    // with no verified workspace cannot write into anybody's calendar.
    let id = Uuid::new_v4();
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => {
            return Html(format!(
                "<div class=\"calendar-error\">{}</div>",
                html_escape(&e.1)
            ))
        }
    };

    let Some(branch) =
        botsecurity_core::scope::branch_from_jwt(&headers, &mut conn).filter(|id| !id.is_nil())
    else {
        return Html(
            "<div class=\"calendar-error\">Sign in to a workspace before saving an event.</div>"
                .to_string(),
        );
    };

    let org = match org_from_claims(&headers).filter(|id| !id.is_nil()) {
        Some(org) => org,
        None => match org_for_branch(&mut conn, branch).filter(|id| !id.is_nil()) {
            Some(org) => org,
            None => {
                return Html(
                    "<div class=\"calendar-error\">Your workspace could not be resolved.</div>"
                        .to_string(),
                )
            }
        },
    };
    let bot = bot_id_for(&mut conn, branch);
    if bot.is_nil() {
        return Html(
            "<div class=\"calendar-error\">Your workspace has no bot to own this event.</div>"
                .to_string(),
        );
    }
    let scope = CalendarScope {
        org_id: org,
        bot_id: bot,
        branch_id: branch,
    };

    let owner_id = user_id_from_claims(&headers)
        .and_then(|sub| Uuid::parse_str(&sub).ok())
        .unwrap_or(Uuid::nil());
    let requested_calendar = payload
        .calendar_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| Uuid::parse_str(value).ok())
        .filter(|id| !id.is_nil());
    let calendar = match scoped_calendar_id(&mut conn, &scope, requested_calendar, owner_id) {
        Ok(calendar) => calendar,
        Err(e) => {
            log::error!("Calendar scope check failed: {e}");
            return Html(format!(
                "<div class=\"calendar-error\">{}</div>",
                html_escape(&e)
            ));
        }
    };

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

    // All four scope references (org_id, bot_id, branch_id, calendar_id) are
    // derived from verified claims; none of them is a nil UUID or client input.
    let insert = diesel::sql_query(
        "INSERT INTO calendar_events \
         (id, org_id, bot_id, branch_id, calendar_id, owner_id, title, description, location, \
          start_time, end_time, all_day, status, visibility, busy_status, reminders, attendees, \
          metadata) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'confirmed', 'default', \
                 'busy', '[]', '[]', '{}')",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Uuid, _>(&scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(&scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(&scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(&calendar)
    .bind::<diesel::sql_types::Uuid, _>(&owner_id)
    .bind::<diesel::sql_types::Text, _>(&title)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.description)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.location)
    .bind::<diesel::sql_types::Timestamptz, _>(&start)
    .bind::<diesel::sql_types::Timestamptz, _>(&end)
    .bind::<diesel::sql_types::Bool, _>(all_day)
    .execute(&mut conn);

    // The result used to be discarded, so the form confirmed a save that had
    // failed. Surface the failure instead of reporting a phantom event.
    if let Err(e) = insert {
        log::error!("Failed to save calendar event '{title}': {e}");
        return Html(
            "<div class=\"calendar-error\">The event could not be saved. Please try again.</div>"
                .to_string(),
        );
    }

    Html(format!(
        r#"<div class="event-saved" data-event-id="{id}"><p>Event "{title}" saved</p></div>"#,
        id = id,
        title = html_escape(&title),
    ))
}

#[cfg(test)]
mod calendar_scope_tests {
    use super::*;

    fn scope() -> CalendarScope {
        CalendarScope {
            org_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            branch_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn accepts_a_calendar_whose_full_scope_matches() {
        let s = scope();
        assert!(calendar_in_scope(s.org_id, s.bot_id, s.branch_id, &s));
    }

    #[test]
    fn rejects_a_calendar_from_another_org() {
        let s = scope();
        assert!(!calendar_in_scope(
            Uuid::new_v4(),
            s.bot_id,
            s.branch_id,
            &s
        ));
    }

    #[test]
    fn rejects_a_calendar_from_another_bot() {
        let s = scope();
        assert!(!calendar_in_scope(
            s.org_id,
            Uuid::new_v4(),
            s.branch_id,
            &s
        ));
    }

    #[test]
    fn rejects_a_calendar_from_another_branch_of_the_same_org() {
        let s = scope();
        assert!(!calendar_in_scope(s.org_id, s.bot_id, Uuid::new_v4(), &s));
    }

    #[test]
    fn rejects_a_real_calendar_when_no_workspace_is_verified() {
        assert!(!calendar_in_scope(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            &CalendarScope {
                org_id: Uuid::nil(),
                bot_id: Uuid::nil(),
                branch_id: Uuid::nil(),
            }
        ));
    }
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
