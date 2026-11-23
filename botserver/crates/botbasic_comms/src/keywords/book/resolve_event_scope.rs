use super::*;

/// Why a booking could not be scoped to the caller's tenant (issue #2895).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeError {
    MissingBranch,
    MissingBot,
    MissingOrg,
    UnknownBranch,
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::MissingBranch => "the caller's session has no branch",
            Self::MissingBot => "the caller's session has no bot",
            Self::MissingOrg => "the caller's session has no organization",
            Self::UnknownBranch => "the caller's branch is not registered",
        };
        f.write_str(text)
    }
}

impl std::error::Error for ScopeError {}

/// Pure scope resolver for calendar writes (issue #2895).
///
/// Builds the tenant scope exclusively from values the server verified when it
/// created the caller's session. It performs no I/O and reads no client body,
/// so the cross-tenant rules can be unit-tested offline. A nil branch/bot or a
/// missing org is refused: the previous resolver silently picked the oldest
/// branch in the whole installation, attributing every `BOOK` to one global
/// scope regardless of who made the request.
fn event_scope_from_claims(
    branch_id: Uuid,
    org_id: Option<Uuid>,
    bot_id: Uuid,
) -> Result<EventScope, ScopeError> {
    if branch_id.is_nil() {
        return Err(ScopeError::MissingBranch);
    }
    if bot_id.is_nil() {
        return Err(ScopeError::MissingBot);
    }
    let org_id = org_id
        .filter(|id| !id.is_nil())
        .ok_or(ScopeError::MissingOrg)?;
    Ok(EventScope {
        org_id,
        bot_id,
        branch_id,
    })
}

/// Resolves the caller's booking scope from the verified session (issue #2895).
///
/// `branch_id` and `bot_id` are server-minted session claims; `org_id` comes
/// from the session's verified tenant data, falling back to the owning branch
/// row only for legacy sessions that do not carry it. The branch is never
/// guessed from "the oldest branch" — that wrote every booking into a single
/// global scope shared by all tenants.
pub(super) fn resolve_event_scope(
    conn: &mut diesel::PgConnection,
    user: &UserSession,
) -> Result<EventScope, Box<dyn std::error::Error>> {
    if user.branch_id.is_nil() {
        return Err(ScopeError::MissingBranch.into());
    }
    let org_id = match user.org_id().filter(|id| !id.is_nil()) {
        Some(org_id) => org_id,
        None => org_for_branch(conn, user.branch_id)?,
    };
    Ok(event_scope_from_claims(
        user.branch_id,
        Some(org_id),
        user.bot_id,
    )?)
}

/// Owning organization of a verified branch. Only used for legacy sessions
/// without an `org_id` claim; the branch itself is always the caller's.
fn org_for_branch(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    #[derive(diesel::QueryableByName)]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }

    diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<OrgRow>(conn)
        .optional()?
        .map(|row| row.org_id)
        .ok_or_else(|| ScopeError::UnknownBranch.into())
}

/// Returns the identifier of the scope's primary calendar, provisioning one
/// when the scope has none. `calendar_events.calendar_id` is `NOT NULL`, so a
/// booking cannot be stored until a calendar exists. The lookup and the insert
/// are both constrained to the verified scope, so a calendar belonging to
/// another tenant can never be followed by a write.
pub(super) fn ensure_default_calendar(
    conn: &mut diesel::PgConnection,
    scope: &EventScope,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    #[derive(diesel::QueryableByName)]
    struct CalendarIdRow {
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
    .get_result::<CalendarIdRow>(conn)
    .optional()?;

    if let Some(row) = existing {
        return Ok(row.id);
    }

    let calendar_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO calendars \
         (id, org_id, bot_id, branch_id, owner_id, name, description, color, timezone, \
          is_primary, is_visible, is_shared) \
         VALUES ($1, $2, $3, $4, '00000000-0000-0000-0000-000000000000', 'Default', \
                 'Your default calendar', '#3b82f6', 'UTC', true, true, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(calendar_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .execute(conn)?;

    info!(
        "Provisioned default calendar {calendar_id} for branch {}",
        scope.branch_id
    );
    Ok(calendar_id)
}
/// Renders the in-memory recurrence rule as an iCalendar `RRULE` value, which
/// is the representation stored in `calendar_events.recurrence_rule`.
pub(super) fn recurrence_rule_to_string(rule: &RecurrenceRule) -> String {
    let mut parts = vec![format!("FREQ={}", rule.frequency.to_uppercase())];
    if rule.interval > 0 {
        parts.push(format!("INTERVAL={}", rule.interval));
    }
    if let Some(count) = rule.count {
        parts.push(format!("COUNT={count}"));
    }
    if let Some(until) = rule.until {
        parts.push(format!("UNTIL={}", until.format("%Y%m%dT%H%M%SZ")));
    }
    if let Some(days) = rule.by_day.as_ref().filter(|days| !days.is_empty()) {
        parts.push(format!("BYDAY={}", days.join(",")));
    }
    parts.join(";")
}
pub(super) fn send_meeting_invite(
    _state: &dyn BasicRuntime,
    event: &CalendarEvent,
    attendee: &str,
) -> Result<(), String> {
    info!(
        "Would send meeting invite for '{}' to {}",
        event.title, attendee
    );
    Ok(())
}

#[cfg(test)]
mod event_scope_resolver_tests {
    use super::*;

    fn scope() -> EventScope {
        EventScope {
            org_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            branch_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn accepts_a_fully_verified_scope() {
        let expected = scope();
        assert_eq!(
            event_scope_from_claims(expected.branch_id, Some(expected.org_id), expected.bot_id),
            Ok(expected)
        );
    }

    #[test]
    fn rejects_a_nil_branch() {
        let s = scope();
        assert_eq!(
            event_scope_from_claims(Uuid::nil(), Some(s.org_id), s.bot_id),
            Err(ScopeError::MissingBranch)
        );
    }

    #[test]
    fn rejects_a_nil_bot() {
        let s = scope();
        assert_eq!(
            event_scope_from_claims(s.branch_id, Some(s.org_id), Uuid::nil()),
            Err(ScopeError::MissingBot)
        );
    }

    #[test]
    fn rejects_a_missing_org() {
        let s = scope();
        assert_eq!(
            event_scope_from_claims(s.branch_id, None, s.bot_id),
            Err(ScopeError::MissingOrg)
        );
    }

    #[test]
    fn rejects_a_nil_org() {
        let s = scope();
        assert_eq!(
            event_scope_from_claims(s.branch_id, Some(Uuid::nil()), s.bot_id),
            Err(ScopeError::MissingOrg)
        );
    }

    #[test]
    fn never_invents_a_scope_from_missing_claims() {
        assert_eq!(
            event_scope_from_claims(Uuid::nil(), None, Uuid::nil()),
            Err(ScopeError::MissingBranch)
        );
    }
}
