# Intelligent Email Triaging 🟡 BETA

> **LLM-powered email classification and automated calendar conflict resolution**

## Overview

The Intelligent Email Triaging system automatically classifies incoming emails, extracts tasks and meeting requests, and resolves calendar conflicts without human intervention. It integrates with the Mail app and Calendar app to provide an autonomous email management experience.

## Email Classification

When new email arrives via IMAP/SMTP, the system classifies it into one of four categories:

| Category | Description | Action |
|----------|-------------|--------|
| **Urgent Action Required** | Time-sensitive, requires immediate response | Flag as high priority, notify user |
| **Informational** | Newsletter, notification, status update | Archive or low-priority folder |
| **Meeting Request** | Calendar invitation or scheduling proposal | Extract datetime, check Calendar |
| **Spam/Ignore** | Unsolicited, promotional, or bulk | Auto-trash or spam folder |

## Calendar Conflict Resolution Flow

```
Meeting Request Email received
  │
  ▼
Extract proposal (date, time, attendees)
  │
  ▼
Cross-reference Calendar via botcalendar API
  │
  ├── No conflict → Auto-accept, send confirmation
  └── Conflict found →
       ├── Query LLM for nearby free slots
       ├── Draft response with alternative times
       └── Present to user for approval
```

## Calendar Conflict Detector

The conflict resolution engine in `botcalendar`:

- Parses `.ics`/iCalendar attachments
- Detects overlapping events within configurable working hours
- Uses LLM to rank alternative free slots by attendee availability
- Drafts a response email with optimal alternatives

## LLM-Assisted Draft Refinement

Users can refine email drafts interactively:

```basic
' In BASIC script
HEAR "Make this email more formal" AS command
' System regenerates draft body via LLM
REFINE DRAFT WITH "more formal"
```

From the Mail UI, users type natural language commands like:
- "Make this message more formal"
- "Suggest next Tuesday afternoon"
- "Summarize this thread"

## Configuration

| config.csv key | Description | Default |
|----------------|-------------|---------|
| `email-triage-enabled` | Enable triaging | `true` |
| `email-triage-model` | LLM model for classification | `gpt-4o` |
| `calendar-work-hours-start` | Start of work day | `09:00` |
| `calendar-work-hours-end` | End of work day | `18:00` |

## UI

See [Mail - Email Client](../07-user-interface/apps/mail.md) for the email interface and [Calendar - Scheduling](../07-user-interface/apps/calendar.md) for calendar integration.

## Feature Flag

Enable with `mail` feature flag:
```toml
botserver = { features = ["mail"] }
```
