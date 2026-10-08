use super::*;


#[cfg(test)]
mod calendar_scope_tests {
    use super::*;

    /// Scope columns that both calendar tables declare as `NOT NULL` with no
    /// default. A definition that omits any of them makes every write fail with
    /// a constraint violation while the handler still reports success, which is
    /// the defect reported in issue #1338.
    const SCOPE_COLUMNS: [&str; 3] = ["org_id", "bot_id", "branch_id"];

    fn fixture_scope() -> (Uuid, Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4())
    }

    fn sample_calendar(scope: (Uuid, Uuid, Uuid)) -> CalendarRecord {
        let (org_id, bot_id, branch_id) = scope;
        let now = Utc::now();
        CalendarRecord {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            branch_id,
            owner_id: Uuid::new_v4(),
            name: "Fixture".to_string(),
            description: Some("Fixture calendar".to_string()),
            color: Some("#3b82f6".to_string()),
            timezone: Some("UTC".to_string()),
            is_primary: true,
            is_visible: true,
            is_shared: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn sample_event(scope: (Uuid, Uuid, Uuid), calendar_id: Uuid) -> CalendarEventRecord {
        let (org_id, bot_id, branch_id) = scope;
        let now = Utc::now();
        CalendarEventRecord {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            branch_id,
            calendar_id,
            owner_id: Uuid::new_v4(),
            title: "Fixture event".to_string(),
            description: None,
            location: None,
            start_time: now,
            end_time: now + chrono::Duration::hours(1),
            all_day: false,
            recurrence_rule: None,
            recurrence_id: None,
            color: None,
            status: "confirmed".to_string(),
            visibility: "default".to_string(),
            busy_status: "busy".to_string(),
            reminders: serde_json::json!([]),
            attendees: serde_json::json!([]),
            conference_data: None,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    fn insert_sql(calendar: &CalendarRecord) -> String {
        diesel::debug_query::<diesel::pg::Pg, _>(
            &diesel::insert_into(calendars::table).values(calendar),
        )
        .to_string()
    }

    fn insert_event_sql(event: &CalendarEventRecord) -> String {
        diesel::debug_query::<diesel::pg::Pg, _>(
            &diesel::insert_into(calendar_events::table).values(event),
        )
        .to_string()
    }

    #[test]
    fn calendar_insert_names_every_scope_column() {
        let sql = insert_sql(&sample_calendar(fixture_scope()));
        for column in SCOPE_COLUMNS {
            assert!(sql.contains(column), "calendars insert omits {column}: {sql}");
        }
    }

    #[test]
    fn calendar_event_insert_names_every_scope_column() {
        let scope = fixture_scope();
        let sql = insert_event_sql(&sample_event(scope, Uuid::new_v4()));
        for column in SCOPE_COLUMNS {
            assert!(
                sql.contains(column),
                "calendar_events insert omits {column}: {sql}"
            );
        }
    }

    /// Both tables are read with `all_columns`, so the shared definition must
    /// expose the three scope columns as well.
    #[test]
    fn shared_schema_exposes_every_scope_column() {
        let selects = [
            diesel::debug_query::<diesel::pg::Pg, _>(&calendars::table.select(calendars::all_columns))
                .to_string(),
            diesel::debug_query::<diesel::pg::Pg, _>(
                &calendar_events::table.select(calendar_events::all_columns),
            )
            .to_string(),
        ];
        for sql in selects {
            for column in SCOPE_COLUMNS {
                assert!(
                    sql.contains(column),
                    "shared schema omits {column}: {sql}"
                );
            }
        }
    }

    /// The fallback scope must still be writable: an anonymous caller resolves
    /// to nil rather than to a missing value, so the insert satisfies the
    /// `NOT NULL` columns instead of failing the request.
    #[test]
    fn nil_scope_is_a_complete_write_scope() {
        let scope = CalendarScope::NIL;
        assert_eq!(scope.org_id, Uuid::nil());
        assert_eq!(scope.bot_id, Uuid::nil());
        assert_eq!(scope.branch_id, Uuid::nil());
        let sql = insert_sql(&sample_calendar((scope.org_id, scope.bot_id, scope.branch_id)));
        for column in SCOPE_COLUMNS {
            assert!(sql.contains(column), "nil-scope insert omits {column}");
        }
    }

    /// Round trip against a live database: a calendar and an event written for
    /// the resolved scope must both be readable again. Skipped unless
    /// `DATABASE_URL` points at a database carrying the calendar schema; the
    /// fixture rows are removed afterwards.
    #[test]
    fn calendar_and_event_round_trip_for_resolved_scope() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let Ok(mut conn) = diesel::PgConnection::establish(&database_url) else {
            log::warn!("Skipping the calendar round trip: DATABASE_URL is unreachable");
            return;
        };
        let Some(scope) = fixture_scope_from_database(&mut conn) else {
            log::warn!("Skipping the calendar round trip: no branch is configured");
            return;
        };

        let calendar = sample_calendar(scope);
        diesel::insert_into(calendars::table)
            .values(&calendar)
            .execute(&mut conn)
            .expect("a calendar must be insertable for a resolved scope");

        let event = sample_event(scope, calendar.id);
        diesel::insert_into(calendar_events::table)
            .values(&event)
            .execute(&mut conn)
            .expect("an event must be insertable for a resolved scope");

        let stored_calendars = calendars::table
            .filter(calendars::branch_id.eq(scope.2))
            .load::<CalendarRecord>(&mut conn)
            .expect("the calendar must be readable back");
        assert!(
            stored_calendars.iter().any(|row| row.id == calendar.id),
            "the calendar written for the resolved scope must be readable"
        );

        let stored_events = calendar_events::table
            .filter(calendar_events::branch_id.eq(scope.2))
            .load::<CalendarEventRecord>(&mut conn)
            .expect("the event must be readable back");
        assert!(
            stored_events.iter().any(|row| row.id == event.id),
            "the event written for the resolved scope must be readable"
        );

        diesel::delete(calendar_events::table.filter(calendar_events::id.eq(event.id)))
            .execute(&mut conn)
            .ok();
        diesel::delete(calendars::table.filter(calendars::id.eq(calendar.id)))
            .execute(&mut conn)
            .ok();
    }

    /// Resolves the oldest branch and its default bot, matching the scope the
    /// request handlers compute for an authenticated caller.
    fn fixture_scope_from_database(conn: &mut diesel::PgConnection) -> Option<(Uuid, Uuid, Uuid)> {
        #[derive(diesel::QueryableByName)]
        struct ScopeRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }

        let row = diesel::sql_query(
            "SELECT id AS branch_id, org_id FROM branches ORDER BY created_at ASC LIMIT 1",
        )
        .get_result::<ScopeRow>(conn)
        .optional()
        .ok()
        .flatten()?;

        #[derive(diesel::QueryableByName)]
        struct BotRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let bot = diesel::sql_query(
            "SELECT id FROM bots WHERE branch_id = $1 \
             ORDER BY is_default_for_branch DESC, created_at ASC LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(row.branch_id)
        .get_result::<BotRow>(conn)
        .optional()
        .ok()
        .flatten()?;

        Some((row.org_id, bot.id, row.branch_id))
    }
}
