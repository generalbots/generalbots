# Calendar API

The Calendar API provides endpoints for managing events, schedules, and time-based activities within BotServer.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in BotServer.

## Planned Features

The Calendar API will enable:
- Event creation and management
- Meeting scheduling
- Availability checking
- Recurring events
- Calendar synchronization
- Reminders and notifications

## Planned Endpoints

### Event Management
- `POST /api/v1/calendar/events` - Create event
- `GET /api/v1/calendar/events` - List events
- `GET /api/v1/calendar/events/{event_id}` - Get event details
- `PATCH /api/v1/calendar/events/{event_id}` - Update event
- `DELETE /api/v1/calendar/events/{event_id}` - Delete event

### Scheduling
- `POST /api/v1/calendar/schedule` - Find available time slots
- `POST /api/v1/calendar/meeting` - Schedule meeting
- `GET /api/v1/calendar/availability` - Check availability

### Recurring Events
- `POST /api/v1/calendar/events/recurring` - Create recurring event
- `PATCH /api/v1/calendar/events/{event_id}/recurrence` - Update recurrence

### Reminders
- `POST /api/v1/calendar/events/{event_id}/reminders` - Add reminder
- `GET /api/v1/calendar/reminders` - List upcoming reminders

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
- **Single Events**: One-time occurrences
- **Recurring Events**: Daily, weekly, monthly patterns
- **All-day Events**: Full day events without specific times
- **Multi-day Events**: Events spanning multiple days

### Notification Methods
- In-app notifications
- Email reminders
- SMS alerts (when configured)
- Bot messages

### Calendar Views
- Day view
- Week view
- Month view
- Agenda view

### Time Zone Support
- User-specific time zones
- Automatic DST handling
- Cross-timezone meeting coordination

### Integration Points
- External calendar systems (Google, Outlook)
- Video conferencing platforms
- Task management system
- Notification system

## Implementation Considerations

When implemented, the Calendar API will:

1. **Use PostgreSQL** for event storage
2. **Support iCal format** for import/export
3. **Handle time zones** properly
4. **Provide conflict detection** for scheduling
5. **Include RBAC** for event management
6. **Support delegation** for assistants
7. **Enable calendar sharing** between users

## Alternative Solutions

Until the Calendar API is implemented, consider:

1. **External Calendar Services**
   - Integrate with Google Calendar API
   - Use Microsoft Graph API for Outlook
   - Connect to CalDAV servers

2. **Simple Scheduling in BASIC**
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

3. **Task-based Scheduling**
   - Use the Tasks API with due dates
   - Create tasks for time-sensitive items
   - Set reminders via scheduled BASIC scripts

## Future Integration

The Calendar API will integrate with:
- [Tasks API](./tasks-api.md) - Link tasks to calendar events
- [Notifications API](./notifications-api.md) - Event reminders
- [User API](./user-security.md) - User availability
- Meeting API (future) - Video conferencing

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