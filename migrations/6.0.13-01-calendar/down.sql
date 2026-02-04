DROP INDEX IF EXISTS idx_calendar_shares_email;
DROP INDEX IF EXISTS idx_calendar_shares_user;
DROP INDEX IF EXISTS idx_calendar_shares_calendar;

DROP INDEX IF EXISTS idx_calendar_event_reminders_pending;
DROP INDEX IF EXISTS idx_calendar_event_reminders_event;

DROP INDEX IF EXISTS idx_calendar_event_attendees_email;
DROP INDEX IF EXISTS idx_calendar_event_attendees_event;

DROP INDEX IF EXISTS idx_calendar_events_recurrence;
DROP INDEX IF EXISTS idx_calendar_events_status;
DROP INDEX IF EXISTS idx_calendar_events_time_range;
DROP INDEX IF EXISTS idx_calendar_events_owner;
DROP INDEX IF EXISTS idx_calendar_events_calendar;
DROP INDEX IF EXISTS idx_calendar_events_org_bot;

DROP INDEX IF EXISTS idx_calendars_primary;
DROP INDEX IF EXISTS idx_calendars_owner;
DROP INDEX IF EXISTS idx_calendars_org_bot;

DROP TABLE IF EXISTS calendar_shares;
DROP TABLE IF EXISTS calendar_event_reminders;
DROP TABLE IF EXISTS calendar_event_attendees;
DROP TABLE IF EXISTS calendar_events;
DROP TABLE IF EXISTS calendars;
