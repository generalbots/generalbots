# BOOK / BOOK_MEETING / CHECK_AVAILABILITY Keywords

The `BOOK` family of keywords provides calendar and scheduling functionality, allowing bots to create appointments, schedule meetings with attendees, and check availability.

## Keywords Overview

| Keyword | Purpose |
|---------|---------|
| `BOOK` | Create a simple calendar appointment |
| `BOOK_MEETING` | Schedule a meeting with multiple attendees |
| `CHECK_AVAILABILITY` | Find available time slots |

## BOOK

Creates a calendar appointment for the current user.

### Syntax

```basic
result = BOOK title, description, start_time, duration_minutes, location
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | String | Title/subject of the appointment |
| `description` | String | Detailed description of the appointment |
| `start_time` | String | When the appointment starts (see Time Formats) |
| `duration_minutes` | Integer | Duration in minutes (default: 30) |
| `location` | String | Location or meeting room |

### Example

```basic
' Book a dentist appointment
result = BOOK "Dentist Appointment", "Annual checkup", "2024-03-15 14:00", 60, "123 Medical Center"
TALK "Your appointment has been booked: " + result

' Book a quick meeting
result = BOOK "Team Sync", "Weekly standup", "tomorrow 10:00", 30, "Conference Room A"
```

## BOOK_MEETING

Schedules a meeting with multiple attendees, sending calendar invites.

### Syntax

```basic
result = BOOK_MEETING meeting_details, attendees
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `meeting_details` | JSON String | Meeting configuration object |
| `attendees` | Array | List of attendee email addresses |

### Meeting Details Object

```json
{
    "title": "Meeting Title",
    "description": "Meeting description",
    "start_time": "2024-03-15 14:00",
    "duration": 60,
    "location": "Conference Room B",
    "reminder_minutes": 15,
    "recurrence": "weekly"
}
```

### Example

```basic
' Schedule a team meeting
meeting = '{
    "title": "Sprint Planning",
    "description": "Plan next sprint tasks and priorities",
    "start_time": "Monday 09:00",
    "duration": 90,
    "location": "Main Conference Room",
    "reminder_minutes": 30
}'

attendees = ["alice@company.com", "bob@company.com", "carol@company.com"]

result = BOOK_MEETING meeting, attendees
TALK "Meeting scheduled with " + LEN(attendees) + " attendees"
```

## CHECK_AVAILABILITY

Finds available time slots for a given date and duration.

### Syntax

```basic
available_slots = CHECK_AVAILABILITY date, duration_minutes
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `date` | String | The date to check availability |
| `duration_minutes` | Integer | Required duration for the meeting |

### Example

```basic
' Check availability for a 1-hour meeting tomorrow
slots = CHECK_AVAILABILITY "tomorrow", 60

TALK "Available time slots:"
FOR EACH slot IN slots
    TALK "  - " + slot
NEXT
```

## Time Formats

The BOOK keywords support flexible time formats:

### Absolute Formats

| Format | Example |
|--------|---------|
| ISO 8601 | `"2024-03-15T14:00:00"` |
| Date + Time | `"2024-03-15 14:00"` |
| Date + Time (12h) | `"2024-03-15 2:00 PM"` |

### Relative Formats

| Format | Example |
|--------|---------|
| Day name | `"Monday 10:00"` |
| Relative day | `"tomorrow 14:00"` |
| Next week | `"next Tuesday 09:00"` |

## Complete Example: Appointment Scheduling Bot

```basic
' appointment-bot.bas
' A complete appointment scheduling workflow

TALK "Welcome to our scheduling assistant!"
TALK "What type of appointment would you like to book?"

HEAR appointment_type

SWITCH appointment_type
    CASE "consultation"
        duration = 60
        description = "Initial consultation meeting"
    CASE "follow-up"
        duration = 30
        description = "Follow-up discussion"
    CASE "review"
        duration = 45
        description = "Project review session"
    DEFAULT
        duration = 30
        description = appointment_type
END SWITCH

TALK "When would you like to schedule this?"
HEAR preferred_date

' Check available slots
slots = CHECK_AVAILABILITY preferred_date, duration

IF LEN(slots) = 0 THEN
    TALK "Sorry, no availability on that date. Please try another day."
ELSE
    TALK "Available times:"
    index = 1
    FOR EACH slot IN slots
        TALK index + ". " + slot
        index = index + 1
    NEXT
    
    TALK "Which time slot would you prefer? (enter number)"
    HEAR choice
    
    selected_time = slots[choice - 1]
    
    TALK "Where would you like the meeting to take place?"
    HEAR location
    
    ' Book the appointment
    result = BOOK appointment_type, description, selected_time, duration, location
    
    TALK "✅ Your appointment has been booked!"
    TALK "Details: " + result
END IF
```

## Meeting with Recurrence

```basic
' Schedule a recurring weekly meeting
meeting = '{
    "title": "Weekly Team Standup",
    "description": "Daily sync on project progress",
    "start_time": "Monday 09:00",
    "duration": 15,
    "location": "Virtual - Teams",
    "reminder_minutes": 5,
    "recurrence": {
        "frequency": "weekly",
        "interval": 1,
        "count": 12,
        "by_day": ["MO", "WE", "FR"]
    }
}'

attendees = ["team@company.com"]
result = BOOK_MEETING meeting, attendees
```

## Event Status

Calendar events can have the following statuses:

| Status | Description |
|--------|-------------|
| `Confirmed` | Event is confirmed and scheduled |
| `Tentative` | Event is tentatively scheduled |
| `Cancelled` | Event has been cancelled |

## Calendar Event Structure

When an event is created, it contains:

```json
{
    "id": "uuid",
    "title": "Meeting Title",
    "description": "Description",
    "start_time": "2024-03-15T14:00:00Z",
    "end_time": "2024-03-15T15:00:00Z",
    "location": "Conference Room",
    "organizer": "user@example.com",
    "attendees": ["attendee1@example.com"],
    "reminder_minutes": 15,
    "recurrence_rule": null,
    "status": "Confirmed",
    "created_at": "2024-03-10T10:00:00Z",
    "updated_at": "2024-03-10T10:00:00Z"
}
```

## Configuration

To enable calendar functionality, configure the following in `config.csv`:

| Key | Description |
|-----|-------------|
| `calendar-provider` | Calendar service (google, outlook, caldav) |
| `calendar-client-id` | OAuth client ID |
| `calendar-client-secret` | OAuth client secret |
| `calendar-default-reminder` | Default reminder time in minutes |

## Error Handling

```basic
' Handle booking errors gracefully
ON ERROR GOTO handle_error

result = BOOK "Meeting", "Description", "invalid-date", 30, "Location"
TALK "Booked: " + result
END

handle_error:
    TALK "Sorry, I couldn't book that appointment. Please check the date and time format."
    TALK "Error: " + ERROR_MESSAGE
END
```

## Best Practices

1. **Always check availability first**: Before booking, use `CHECK_AVAILABILITY` to ensure the time slot is free.

2. **Use descriptive titles**: Make appointment titles clear and searchable.

3. **Set appropriate reminders**: Configure reminder times based on appointment importance.

4. **Handle time zones**: Be explicit about time zones when scheduling across regions.

5. **Validate inputs**: Check user-provided dates and times before attempting to book.

## Related Keywords

- [SET SCHEDULE](./keyword-set-schedule.md) - Schedule recurring bot tasks
- [WAIT](./keyword-wait.md) - Pause execution for a duration
- [SEND MAIL](./keyword-send-mail.md) - Send meeting confirmations via email

## See Also

- [Calendar Integration](../appendix-external-services/calendar.md)
- [Google Calendar Setup](../appendix-external-services/google-calendar.md)
- [Microsoft Outlook Integration](../appendix-external-services/outlook.md)