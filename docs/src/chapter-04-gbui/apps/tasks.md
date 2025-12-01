# Tasks - To-Do Management

> **Track what needs to be done**

<img src="../../assets/suite/tasks-screen.svg" alt="Tasks Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Tasks is your to-do list manager within General Bots Suite. Create tasks, set priorities, organize by category, and track your progress. Built with HTMX for instant updates without page reloads.

---

## Features

### Adding Tasks

**Quick Add:**
1. Type task description in the input box
2. Press **Enter** or click **+ Add**

**With Details:**
1. Type task description
2. Select a category (optional)
3. Pick a due date (optional)
4. Click **+ Add**

### Priority Levels

| Priority | Icon | When to Use |
|----------|------|-------------|
| **High** | 🔴 | Must do today |
| **Medium** | 🟡 | Important but not urgent |
| **Low** | 🟢 | Can wait |
| **None** | ⚪ | No deadline |

### Categories

| Category | Icon |
|----------|------|
| Work | 💼 |
| Personal | 🏠 |
| Shopping | 🛒 |
| Health | ❤️ |
| Custom | 🏷️ |

### Filter Tabs

| Tab | Shows |
|-----|-------|
| **All** | All tasks |
| **Active** | Uncompleted tasks |
| **Completed** | Done tasks |
| **Priority** | High priority only |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Add task (when in input) |
| `Space` | Toggle task completion |
| `Delete` | Remove selected task |
| `Tab` | Move to next field |
| `Escape` | Cancel editing |
| `↑` / `↓` | Navigate tasks |

---

## Creating Tasks from Chat

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a task to review the budget by Friday</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Task created:</p>
      <p>📋 Review the budget</p>
      <p>📅 Due: Friday, March 22</p>
      <p>🏷️ Category: Work</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

### Managing Tasks via Chat

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What tasks do I have today?</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 Today's Tasks (3 active):</p>
      <p>🔴 Review quarterly report</p>
      <p>🟡 Call client about proposal</p>
      <p>🟢 Update project documentation</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Mark the quarterly report as done</p>
      <div class="wa-time">11:45</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Task completed:</p>
      <p>📋 Review quarterly report</p>
      <p>You have 2 tasks remaining today.</p>
      <div class="wa-time">11:45</div>
    </div>
  </div>
</div>

### Setting Reminders

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Remind me to call John tomorrow at 3pm</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Task created with reminder:</p>
      <p>📋 Call John</p>
      <p>📅 Tomorrow at 3:00 PM</p>
      <p>🔔 Reminder set for 2:55 PM</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/tasks` | GET | List all tasks |
| `/api/tasks` | POST | Create new task |
| `/api/tasks/:id` | GET | Get single task |
| `/api/tasks/:id` | PATCH | Update task |
| `/api/tasks/:id` | DELETE | Delete task |
| `/api/tasks/stats` | GET | Get task statistics |

### Query Parameters

| Parameter | Values | Default |
|-----------|--------|---------|
| `filter` | `all`, `active`, `completed`, `priority` | `all` |
| `category` | `work`, `personal`, `shopping`, `health` | none |
| `sort` | `created`, `dueDate`, `priority`, `text` | `created` |
| `order` | `asc`, `desc` | `desc` |

### Request Body (Create/Update)

```json
{
    "text": "Review quarterly report",
    "category": "work",
    "dueDate": "2024-03-20",
    "priority": "high",
    "completed": false
}
```

### Response Format

```json
{
    "id": 123,
    "text": "Review quarterly report",
    "category": "work",
    "dueDate": "2024-03-20",
    "priority": "high",
    "completed": false,
    "createdAt": "2024-03-18T10:30:00Z",
    "updatedAt": "2024-03-18T10:30:00Z"
}
```

---

## Integration with Calendar

Tasks with due dates automatically appear in your Calendar view, helping you visualize your workload across days and weeks.

---

## Troubleshooting

### Tasks Not Saving

1. Check network connection
2. Verify API endpoint is accessible
3. Check browser console for errors
4. Try refreshing the page

### Filters Not Working

1. Click the filter tab again
2. Check if tasks exist for that filter
3. Clear browser cache

### Stats Not Updating

1. Reload the page
2. Check for JavaScript errors in console

---

## See Also

- [Suite Manual](../suite-manual.md) - Complete user guide
- [Chat App](./chat.md) - Create tasks from chat
- [Calendar App](./calendar.md) - View tasks in calendar
- [Tasks API](../../chapter-10-api/tasks-api.md) - API reference