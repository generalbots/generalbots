# BOOK

Schedule meetings and calendar events with attendees.

## Syntax

```basic
BOOK attendees, subject, duration
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `attendees` | String/Array | Email address(es) of attendees. Can be a single email or array of emails |
| `subject` | String | Meeting subject/title |
| `duration` | String | Duration of the meeting (e.g., "30m", "1h", "90m") |

## Description

The `BOOK` keyword creates calendar events and schedules meetings with specified attendees. It integrates with the calendar engine to:

- Find available time slots for all attendees
- Create calendar invitations
- Send meeting notifications
- Handle timezone conversions
- Support recurring meetings

## Examples

### Single Attendee Meeting
```basic
BOOK "john@example.com", "Project Review", "30m"
```

### Multiple Attendees
```basic
attendees = ["alice@example.com", "bob@example.com", "carol@example.com"]
BOOK attendees, "Team Standup", "15m"
```

### Dynamic Meeting Scheduling
```basic
email = GET_USER_EMAIL()
BOOK email, "1-on-1 Discussion", "45m"
```

## Integration Points

- **Calendar Engine**: Uses `calendar_engine` module for availability checking
- **Email Notifications**: Sends invitations via email integration
- **User Sessions**: Respects user timezone preferences
- **Meeting Rooms**: Can optionally reserve physical/virtual rooms

## Return Value

Returns a meeting object containing:
- `meeting_id`: Unique identifier for the meeting
- `start_time`: Scheduled start time
- `end_time`: Scheduled end time
- `meeting_link`: Virtual meeting URL (if applicable)
- `status`: Booking status ("confirmed", "tentative", "failed")

## Error Handling

- Returns error if no common availability found
- Validates email addresses before sending invites
- Checks for calendar permissions
- Handles timezone mismatches gracefully

## Best Practices

1. **Check Availability First**: Consider using availability checks for critical meetings
2. **Provide Context**: Include meaningful subject lines
3. **Respect Working Hours**: System checks business hours by default
4. **Handle Conflicts**: Implement retry logic for failed bookings

## See Also

- [CREATE_TASK](./keyword-create-task.md) - Create tasks instead of meetings
- [SET_SCHEDULE](./keyword-set-schedule.md) - Schedule recurring events
- [SEND_MAIL](./keyword-send-mail.md) - Send email notifications

## Implementation

Located in `src/basic/keywords/book.rs`
