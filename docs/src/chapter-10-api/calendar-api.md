# Calendar API

The Calendar API provides endpoints for managing events, schedules, and time-based activities within BotServer.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in BotServer.

## Planned Features

The Calendar API will enable event creation and management, meeting scheduling, availability checking, recurring events, calendar synchronization, and reminders with notifications.

## Planned Endpoints

### Event Management

Event management endpoints handle the lifecycle of calendar events. Create events with `POST /api/v1/calendar/events`, list events with `GET /api/v1/calendar/events`, retrieve specific event details with `GET /api/v1/calendar/events/{event_id}`, update events with `PATCH /api/v1/calendar/events/{event_id}`, and delete events with `DELETE /api/v1/calendar/events/{event_id}`.

### Scheduling

Scheduling endpoints help coordinate meetings. Find available time slots with `POST /api/v1/calendar/schedule`, schedule meetings with `POST /api/v1/calendar/meeting`, and check availability with `GET /api/v1/calendar/availability`.

### Recurring Events

Recurring event endpoints manage events that repeat on a schedule. Create recurring events with `POST /api/v1/calendar/events/recurring` and update recurrence patterns with `PATCH /api/v1/calendar/events/{event_id}/recurrence`.

### Reminders

Reminder endpoints manage notifications for upcoming events. Add reminders with `POST /api/v1/calendar/events/{event_id}/reminders` and list upcoming reminders with `GET /api/v1/calendar/reminders`.

## Planned Integration with BASIC

When implemented, calendar features will be accessible via BASIC keywords:

```basic
' Create event (not yet available)
event_id = CREATE EVENT "Team Meeting", "2024-02-01 14:00"
SET EVENT DURATION event_id, 60  ' 60 minutes

' Check availability (not yet available)
available = CHECK AVAILABILITY "user123", "2024-02-01"
IF available THEN
    TALK "User is available"
END IF

' Schedule meeting (not yet available)
meeting_id = SCHEDULE MEETING participants, datetime, duration
SEND INVITES meeting_id
```

## Planned Data Models

### Event

```json
{
  "event_id": "evt_123",
  "title": "Team Meeting",
  "description": "Weekly sync",
  "start_time": "2024-02-01T14:00:00Z",
  "end_time": "2024-02-01T15:00:00Z",
  "location": "Conference Room A",
  "attendees": ["user123", "user456"],
  "recurrence": {
    "frequency": "weekly",
    "interval": 1,
    "days_of_week": ["monday"],
    "end_date": "2024-12-31"
  },
  "reminders": [
    {"minutes_before": 15, "method": "notification"},
    {"minutes_before": 60, "method": "email"}
  ]
}
```

### Availability

```json
{
  "user_id": "user123",
  "date": "2024-02-01",
  "time_slots": [
    {"start": "09:00", "end": "10:00", "available": true},
    {"start": "10:00", "end": "11:00", "available": false},
    {"start": "11:00", "end": "12:00", "available": true}
  ]
}
```

## Planned Features Detail

### Event Types

The API will support several event types. Single events are one-time occurrences. Recurring events follow daily, weekly, or monthly patterns. All-day events span the full day without specific start and end times. Multi-day events extend across multiple consecutive days.

### Notification Methods

Notifications can be delivered through in-app notifications, email reminders, SMS alerts when configured, and bot messages through the chat interface.

### Calendar Views

The API will support multiple calendar views including day view for detailed hourly scheduling, week view for weekly planning, month view for long-term visibility, and agenda view for a list-based perspective.

### Time Zone Support

Time zone handling will include user-specific time zones, automatic daylight saving time adjustments, and cross-timezone meeting coordination to ensure events display correctly for all participants.

### Integration Points

The calendar system will integrate with external calendar systems like Google Calendar and Outlook, video conferencing platforms, the task management system, and the notification system for reminders.

## Implementation Considerations

When implemented, the Calendar API will use PostgreSQL for event storage, support iCal format for import and export, handle time zones properly across all operations, provide conflict detection for scheduling, include role-based access control for event management, support delegation for assistants, and enable calendar sharing between users.

## Alternative Solutions

Until the Calendar API is implemented, consider these alternatives.

### External Calendar Services

You can integrate with external providers such as Google Calendar API, Microsoft Graph API for Outlook, or CalDAV servers for standards-based calendar access.

### Simple Scheduling in BASIC

For basic appointment tracking, you can store appointments in bot memory:

```basic
' Store appointments in bot memory
appointment = "Meeting with client at 2 PM"
SET BOT MEMORY "appointment_" + date, appointment

' Retrieve appointments
today_appointment = GET BOT MEMORY "appointment_" + TODAY()
IF today_appointment <> "" THEN
    TALK "Today's appointment: " + today_appointment
END IF
```

### Task-based Scheduling

An alternative approach uses the Tasks API with due dates, creates tasks for time-sensitive items, and sets reminders via scheduled BASIC scripts.

## Future Integration

The Calendar API will integrate with the [Tasks API](./tasks-api.md) to link tasks to calendar events, the [Notifications API](./notifications-api.md) for event reminders, the [User API](./user-security.md) for user availability, and the Meeting API for video conferencing.

## Workaround Example

Until the Calendar API is available, you can implement basic scheduling:

```basic
' Simple appointment booking system
FUNCTION BookAppointment(date, time, description)
    key = "appointment_" + date + "_" + time
    existing = GET BOT MEMORY key
    
    IF existing = "" THEN
        SET BOT MEMORY key, description
        TALK "Appointment booked for " + date + " at " + time
        RETURN TRUE
    ELSE
        TALK "That time slot is already taken"
        RETURN FALSE
    END IF
END FUNCTION

' Check availability
FUNCTION CheckAvailability(date)
    slots = ["09:00", "10:00", "11:00", "14:00", "15:00", "16:00"]
    available = []
    
    FOR EACH slot IN slots
        key = "appointment_" + date + "_" + slot
        appointment = GET BOT MEMORY key
        IF appointment = "" THEN
            available = APPEND(available, slot)
        END IF
    NEXT
    
    RETURN available
END FUNCTION
```

## Status Updates

Check the [GitHub repository](https://github.com/generalbots/botserver) for updates on Calendar API implementation status.