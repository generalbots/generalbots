use super::*;

impl CalendarEvent {
    pub fn to_ical(&self) -> IcalEvent {
        let mut event = IcalEvent::new();
        event.uid(&self.id.to_string());
        event.summary(&self.title);
        event.starts(self.start_time);
        event.ends(self.end_time);

        if let Some(ref desc) = self.description {
            event.description(desc);
        }
        if let Some(ref loc) = self.location {
            event.location(loc);
        }

        event.add_property("ORGANIZER", format!("mailto:{}", self.organizer));

        for attendee in &self.attendees {
            event.add_property("ATTENDEE", format!("mailto:{attendee}"));
        }

        if let Some(ref rrule) = self.recurrence {
            event.add_property("RRULE", rrule);
        }

        if let Some(minutes) = self.reminder_minutes {
            event.add_property("VALARM", format!("-PT{minutes}M"));
        }

        event.done()
    }

    pub fn from_ical(ical: &IcalEvent, organizer: &str, calendar_id: Uuid) -> Option<Self> {
        let uid = ical.get_uid()?;
        let summary = ical.get_summary()?;

        let start_time = date_perhaps_time_to_utc(ical.get_start()?)?;
        let end_time = date_perhaps_time_to_utc(ical.get_end()?)?;

        let id = Uuid::parse_str(uid).unwrap_or_else(|_| Uuid::new_v4());

        Some(Self {
            id,
            calendar_id,
            title: summary.to_string(),
            description: ical.get_description().map(String::from),
            start_time,
            end_time,
            location: ical.get_location().map(String::from),
            attendees: Vec::new(),
            organizer: organizer.to_string(),
            reminder_minutes: None,
            recurrence: None,
            all_day: false,
            status: "confirmed".to_string(),
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

pub(crate) fn date_perhaps_time_to_utc(dpt: DatePerhapsTime) -> Option<DateTime<Utc>> {
    match dpt {
        DatePerhapsTime::DateTime(cal_dt) => match cal_dt {
            CalendarDateTime::Utc(dt) => Some(dt),
            CalendarDateTime::Floating(naive) => Some(Utc.from_utc_datetime(&naive)),
            CalendarDateTime::WithTimezone { date_time, .. } => {
                Some(Utc.from_utc_datetime(&date_time))
            }
        },
        DatePerhapsTime::Date(date) => {
            let naive = NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(0, 0, 0)?);
            Some(Utc.from_utc_datetime(&naive))
        }
    }
}

pub(crate) fn record_to_event(record: CalendarEventRecord) -> CalendarEvent {
    let attendees: Vec<String> =
        serde_json::from_value(record.attendees.clone()).unwrap_or_default();
    let reminders: Vec<serde_json::Value> =
        serde_json::from_value(record.reminders.clone()).unwrap_or_default();
    let reminder_minutes = reminders
        .first()
        .and_then(|r| r.get("minutes_before"))
        .and_then(|m| m.as_i64())
        .map(|m| m as i32);

    CalendarEvent {
        id: record.id,
        calendar_id: record.calendar_id,
        title: record.title,
        description: record.description,
        start_time: record.start_time,
        end_time: record.end_time,
        location: record.location,
        attendees,
        organizer: record.owner_id.to_string(),
        reminder_minutes,
        recurrence: record.recurrence_rule,
        all_day: record.all_day,
        status: record.status,
        color: record.color,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

pub fn export_to_ical(events: &[CalendarEvent], calendar_name: &str) -> String {
    let mut calendar = Calendar::new();
    calendar.name(calendar_name);
    calendar.append_property(Property::new("PRODID", "-//GeneralBots//Calendar//EN"));

    for event in events {
        calendar.push(event.to_ical());
    }

    calendar.done().to_string()
}

pub fn import_from_ical(ical_str: &str, organizer: &str, calendar_id: Uuid) -> Vec<CalendarEvent> {
    let Ok(calendar) = ical_str.parse::<Calendar>() else {
        return Vec::new();
    };

    calendar
        .components
        .iter()
        .filter_map(|c| {
            if let icalendar::CalendarComponent::Event(e) = c {
                CalendarEvent::from_ical(e, organizer, calendar_id)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_bot_context_from_secrets(secrets: &dyn SecretsProvider) -> (Uuid, Uuid) {
    let org_id = secrets
        .get_value("gbo/analytics", "default_org_id")
        .unwrap_or_else(|| "system".to_string());
    let bot_id = secrets
        .get_value("gbo/analytics", "default_bot_id")
        .unwrap_or_else(|| "system".to_string());
    (
        Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::nil()),
        Uuid::parse_str(&bot_id).unwrap_or_else(|_| Uuid::nil()),
    )
}


/// Resolves the caller's (org_id, bot_id) scope from request headers with a
/// graceful fallback to the global nil scope when the DB is unreachable —
/// keeps read-only calendar UI fragments working for anonymous sessions.
pub(crate) fn scope_from_headers(state: &Arc<DbPool>, headers: &axum::http::HeaderMap) -> (Uuid, Uuid) {
    match state.get() {
        Ok(mut conn) => resolve_scope(headers, &mut conn),
        Err(_) => (Uuid::nil(), Uuid::nil()),
    }
}
