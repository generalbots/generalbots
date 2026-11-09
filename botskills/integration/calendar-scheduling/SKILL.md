---
name: calendar-scheduling
description: |
  Find availability and create invites without double-booking. Use for scheduling requests.
version: 1.0.0
source: in-repo
---

# calendar-scheduling

## Workflow

1. Read free/busy for the attendees in the target window and timezone.
2. Propose 2-3 slots; create the event only after the user picks.
3. Include duration, timezone, attendees, location/link and agenda.
4. Send invites with reminders matching the meeting type.
5. Handle conflicts by proposing alternatives, never by force-overwriting.
6. Record the event id for later updates/cancellations.

## Quality checks

- No attendee is double-booked.
- The event time is unambiguous across timezones (explicit TZ).

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
