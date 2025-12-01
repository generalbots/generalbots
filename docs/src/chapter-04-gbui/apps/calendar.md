# Calendar - Scheduling

> **Your personal scheduling assistant**

![Calendar Flow](../../assets/suite/calendar-flow.svg)

---

## Overview

Calendar is your scheduling hub in General Bots Suite. Create events, manage appointments, schedule meetings, and let the AI help you find the perfect time. Calendar syncs with your other apps so you never miss an important date.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Calendar                                    ◄ May 2025 ►    [+ Event]  │
├──────────────┬──────────────────────────────────────────────────────────┤
│              │  Mon    Tue    Wed    Thu    Fri    Sat    Sun          │
│  CALENDARS   │ ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┐      │
│  ───────────  │ │      │      │      │  1   │  2   │  3   │  4   │      │
│  ☑ Personal  │ │      │      │      │      │      │      │      │      │
│  ☑ Work      │ ├──────┼──────┼──────┼──────┼──────┼──────┼──────┤      │
│  ☑ Team      │ │  5   │  6   │  7   │  8   │  9   │ 10   │ 11   │      │
│  ☐ Holidays  │ │      │ 9AM  │      │10AM  │      │      │      │      │
│              │ │      │Team  │      │Review│      │      │      │      │
│  ───────────  │ ├──────┼──────┼──────┼──────┼──────┼──────┼──────┤      │
│              │ │ 12   │ 13   │ 14   │ 15   │ 16   │ 17   │ 18   │      │
│  QUICK ADD   │ │      │ 2PM  │      │      │ ALL  │      │      │      │
│  ───────────  │ │      │Call  │      │      │ DAY  │      │      │      │
│  [ Meeting ] │ ├──────┼──────┼──────┼──────┼──────┼──────┼──────┤      │
│  [ Call    ] │ │ 19   │ 20   │ 21   │ 22   │ 23   │ 24   │ 25   │      │
│  [ Reminder] │ │ 3PM  │      │ 11AM │      │      │      │      │      │
│              │ │Demo  │      │Lunch │      │      │      │      │      │
│              │ ├──────┼──────┼──────┼──────┼──────┼──────┼──────┤      │
│              │ │ 26   │ 27   │ 28   │ 29   │ 30   │ 31   │      │      │
│              │ │      │      │      │      │      │      │      │      │
│              │ └──────┴──────┴──────┴──────┴──────┴──────┴──────┘      │
├──────────────┴──────────────────────────────────────────────────────────┤
│  Today: May 15, 2025                                    [Day][Week][Mo] │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Features

### Creating Events

**Method 1: Click on a Date**

1. Click any date on the calendar
2. A quick-add dialog appears
3. Type the event name
4. Press **Enter** or click **Create**

```
┌─────────────────────────────────────────┐
│  New Event - May 15, 2025         [×]   │
├─────────────────────────────────────────┤
│                                         │
│  Event Name:                            │
│  ┌─────────────────────────────────┐    │
│  │ Team Planning Meeting           │    │
│  └─────────────────────────────────┘    │
│                                         │
│  Time: [10:00 AM ▼] to [11:00 AM ▼]     │
│                                         │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   Create    │  │  More Options   │   │
│  └─────────────┘  └─────────────────┘   │
│                                         │
└─────────────────────────────────────────┘
```

**Method 2: Use the + Event Button**

1. Click **+ Event** in the top right corner
2. Fill in the full event details
3. Click **Save**

**Method 3: Ask the Bot**

Simply tell the bot what you want to schedule:

```
You: Schedule a meeting with Sarah tomorrow at 2pm
Bot: ✅ Event created:
     📅 Meeting with Sarah
     🕐 Tomorrow, 2:00 PM - 3:00 PM
     
     Would you like to send her an invitation?
```

---

### Event Details

When creating or editing an event, you can set:

| Field | Description | Example |
|-------|-------------|---------|
| **Title** | Event name | Team Standup |
| **Date** | When it occurs | May 15, 2025 |
| **Time** | Start and end time | 10:00 AM - 10:30 AM |
| **Location** | Where it takes place | Conference Room A |
| **Description** | Additional details | Weekly sync meeting |
| **Calendar** | Which calendar | Work |
| **Reminder** | Alert before event | 15 minutes before |
| **Repeat** | Recurring pattern | Every Monday |
| **Attendees** | People to invite | sarah@company.com |

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Edit Event                                                       [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Title:                                                                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Weekly Team Standup                                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Date: [May 15, 2025    ▼]                                              │
│                                                                         │
│  Time: [10:00 AM ▼] to [10:30 AM ▼]    ☐ All day                        │
│                                                                         │
│  Location:                                                              │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Conference Room B                                               │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Calendar: [Work            ▼]                                          │
│                                                                         │
│  Reminder: [15 minutes before ▼]                                        │
│                                                                         │
│  Repeat:   [Every Monday      ▼]                                        │
│                                                                         │
│  Attendees:                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ sarah@company.com, john@company.com                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Description:                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Daily sync to discuss progress and blockers                     │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐     │
│  │    Save     │  │   Cancel    │  │   Delete Event              │     │
│  └─────────────┘  └─────────────┘  └─────────────────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### View Modes

Switch between different calendar views using the buttons in the bottom right:

**Day View** - See one day in detail with hourly slots

```
┌─────────────────────────────────────────────────────────────────┐
│  Thursday, May 15, 2025                              [◄] [►]    │
├─────────────────────────────────────────────────────────────────┤
│   8:00 │                                                        │
│   9:00 │                                                        │
│  10:00 │ ████████████████████████████████                       │
│        │ █ Team Standup                █                       │
│  11:00 │ ████████████████████████████████                       │
│  12:00 │                                                        │
│   1:00 │                                                        │
│   2:00 │ ██████████████████████████████████████████████████    │
│        │ █ Project Review Meeting                         █    │
│   3:00 │ █                                                █    │
│        │ ██████████████████████████████████████████████████    │
│   4:00 │                                                        │
│   5:00 │                                                        │
└─────────────────────────────────────────────────────────────────┘
```

**Week View** - See a full week at a glance

**Month View** - See the entire month (default view)

---

### Managing Calendars

You can have multiple calendars for different purposes:

| Calendar | Color | Purpose |
|----------|-------|---------|
| Personal | 🔵 Blue | Personal appointments |
| Work | 🟢 Green | Work meetings and tasks |
| Team | 🟣 Purple | Shared team events |
| Holidays | 🔴 Red | Public holidays |

**To show/hide a calendar:**

1. Look at the **Calendars** panel on the left
2. Click the checkbox next to a calendar name
3. ☑ = visible, ☐ = hidden

**To create a new calendar:**

1. Click the **+** button next to "Calendars"
2. Enter a name for the calendar
3. Choose a color
4. Click **Create**

---

### AI Scheduling Assistant ✨

Let the AI help you find the best time for meetings:

**Finding Free Time**

```
You: When am I free this week?
Bot: You have the following available times this week:
     
     📅 Monday: 11 AM - 12 PM, 3 PM - 5 PM
     📅 Tuesday: All day
     📅 Wednesday: 9 AM - 10 AM, 2 PM - 5 PM
     📅 Thursday: 9 AM - 10 AM, 3 PM - 5 PM
     📅 Friday: 9 AM - 12 PM
     
     Would you like to schedule something?
```

**Smart Scheduling**

```
You: Find a time for a 1-hour meeting with Sarah next week
Bot: I found these times when both you and Sarah are free:
     
     1. Monday at 2:00 PM
     2. Wednesday at 10:00 AM  
     3. Thursday at 3:00 PM
     
     Which works best? (Reply with 1, 2, or 3)
```

**Rescheduling**

```
You: Move my 2pm meeting to tomorrow
Bot: ✅ Moved "Project Review" from today 2 PM to 
     tomorrow (May 16) at 2 PM.
     
     Should I notify the attendees?
```

---

### Reminders

Set reminders to get notified before events:

| Reminder Option | When You're Notified |
|-----------------|---------------------|
| At time of event | When event starts |
| 5 minutes before | 5 min early |
| 15 minutes before | 15 min early |
| 30 minutes before | 30 min early |
| 1 hour before | 1 hour early |
| 1 day before | Day before |
| 1 week before | Week before |

Reminders appear as:
- 🔔 Pop-up notification in the browser
- 📧 Email (if configured)
- 💬 Chat message from the bot

---

### Recurring Events

Set events to repeat automatically:

| Pattern | Example |
|---------|---------|
| Daily | Every day at 9 AM |
| Weekly | Every Monday |
| Bi-weekly | Every other Tuesday |
| Monthly | First Friday of each month |
| Yearly | Every March 15 |
| Custom | Mon, Wed, Fri |

```
┌─────────────────────────────────────────┐
│  Repeat                           [×]   │
├─────────────────────────────────────────┤
│                                         │
│  Repeat: [Weekly           ▼]           │
│                                         │
│  On:                                    │
│  [Mon] [Tue] [Wed] [Thu] [Fri]         │
│   ☑     ☐     ☑     ☐     ☑            │
│                                         │
│  Ends:                                  │
│  ○ Never                                │
│  ○ After [10] occurrences               │
│  ● On [December 31, 2025]               │
│                                         │
│  ┌─────────────┐  ┌─────────────┐       │
│  │    Done     │  │   Cancel    │       │
│  └─────────────┘  └─────────────┘       │
│                                         │
└─────────────────────────────────────────┘
```

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `T` | Go to today |
| `D` | Switch to day view |
| `W` | Switch to week view |
| `M` | Switch to month view |
| `←` | Previous day/week/month |
| `→` | Next day/week/month |
| `C` | Create new event |
| `E` | Edit selected event |
| `Delete` | Delete selected event |
| `/` | Search events |
| `Escape` | Close dialog |

---

## Tips & Tricks

### Quick Tips

💡 **Double-click** a date to create an all-day event

💡 **Drag and drop** events to reschedule them

💡 **Drag the bottom edge** of an event to change its duration

💡 **Hold Ctrl and click** to select multiple events

### Natural Language Input

The AI understands natural language for scheduling:

| Say This | Creates |
|----------|---------|
| "Lunch tomorrow at noon" | Event tomorrow, 12:00 PM |
| "Meeting in 2 hours" | Event 2 hours from now |
| "Dentist next Tuesday 3pm" | Event next Tuesday, 3:00 PM |
| "Weekly standup every Monday 9am" | Recurring event |
| "Birthday party May 20" | All-day event on May 20 |

### Color Coding

Use different calendars to color-code your schedule:

- 🔵 **Blue** - Regular meetings
- 🟢 **Green** - Personal time / breaks
- 🟡 **Yellow** - Deadlines
- 🔴 **Red** - Important / urgent
- 🟣 **Purple** - Tentative / pending

---

## Troubleshooting

### Event not showing up

**Possible causes:**
1. Calendar is hidden in the sidebar
2. Wrong date/time was entered
3. Event is on a different calendar

**Solution:**
1. Check all calendar checkboxes are ☑ enabled
2. Use the search function to find the event
3. Try refreshing the page

---

### Can't edit an event

**Possible causes:**
1. Event was created by someone else
2. Event is from a read-only calendar
3. You don't have edit permissions

**Solution:**
1. Contact the event organizer to make changes
2. Check if the calendar allows editing
3. Ask your administrator for permissions

---

### Reminders not working

**Possible causes:**
1. Browser notifications are blocked
2. Reminder wasn't set on the event
3. Browser tab is not open

**Solution:**
1. Allow notifications in browser settings:
   - Click the 🔒 icon in the address bar
   - Set Notifications to "Allow"
2. Edit the event and add a reminder
3. Keep the General Bots tab open or enable email reminders

---

### Calendar sync issues

**Possible causes:**
1. Network connection problem
2. External calendar not linked
3. Sync is paused

**Solution:**
1. Check your internet connection
2. Go to Settings → Integrations → Calendar
3. Click "Sync Now" to force a refresh

---

## BASIC Integration

Control the calendar from your bot dialogs:

### Create an Event

```basic
event = NEW OBJECT
event.title = "Team Meeting"
event.date = "2025-05-15"
event.startTime = "10:00"
event.endTime = "11:00"
event.location = "Room A"

result = CREATE EVENT event
TALK "Event created: " + result.id
```

### Get Today's Events

```basic
today = GET EVENTS TODAY

FOR EACH event IN today
    TALK event.time + " - " + event.title
NEXT
```

### Find Free Slots

```basic
slots = FIND FREE TIME "next week", 60

TALK "Available 1-hour slots:"
FOR EACH slot IN slots
    TALK slot.date + " at " + slot.time
NEXT
```

### Schedule with Attendees

```basic
HEAR email AS EMAIL "Who should I invite?"

event = NEW OBJECT
event.title = "Project Discussion"
event.date = TOMORROW
event.startTime = "14:00"
event.attendees = [email]

CREATE EVENT event
SEND INVITATION event
TALK "Meeting scheduled and invitation sent!"
```

---

## See Also

- [Meet App](./meet.md) - Video calls for your meetings
- [Tasks App](./tasks.md) - Turn events into actionable tasks
- [Mail App](./mail.md) - Send meeting invitations
- [How To: Create Your First Bot](../how-to/create-first-bot.md)