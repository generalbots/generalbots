# Tasks - To-Do Management

> **Track what needs to be done**

<img src="../../assets/suite/tasks-flow.svg" alt="Tasks Flow Diagram" style="max-width: 100%; height: auto;">

---

## Overview

Tasks is your to-do list manager within General Bots Suite. Create tasks, set priorities, organize by category, and track your progress. Built with HTMX for instant updates without page reloads.

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  ✓ Tasks                          Total: 12  Active: 5  Done: 7│
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ What needs to be done?         [Category ▼] [📅] [+ Add]│   │
│  └─────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  [📋 All (12)] [⏳ Active (5)] [✓ Completed (7)] [⚡ Priority] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ☐ Review quarterly report                    📅 Today    🔴   │
│  ☐ Call client about proposal                 📅 Today    🟡   │
│  ☐ Update project documentation               📅 Tomorrow 🟢   │
│  ☐ Schedule team meeting                      📅 Mar 20   ⚪   │
│  ────────────────────────────────────────────────────────────── │
│  ☑ Send meeting notes                         ✓ Done           │
│  ☑ Complete expense report                    ✓ Done           │
│  ☑ Review pull request                        ✓ Done           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

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

```html
<form class="add-task-form"
      hx-post="/api/tasks"
      hx-target="#task-list"
      hx-swap="afterbegin"
      hx-on::after-request="this.reset()">
    <input type="text" 
           name="text" 
           placeholder="What needs to be done?" 
           required>
    <select name="category">
        <option value="">No Category</option>
        <option value="work">Work</option>
        <option value="personal">Personal</option>
        <option value="shopping">Shopping</option>
        <option value="health">Health</option>
    </select>
    <input type="date" name="dueDate">
    <button type="submit">+ Add Task</button>
</form>
```

### Completing Tasks

Click the checkbox to mark a task complete:

```html
<div class="task-item" id="task-123">
    <input type="checkbox"
           hx-patch="/api/tasks/123"
           hx-vals='{"completed": true}'
           hx-target="#task-123"
           hx-swap="outerHTML">
    <span class="task-text">Review quarterly report</span>
    <span class="task-due">📅 Today</span>
    <span class="task-priority high">🔴</span>
</div>
```

### Priority Levels

| Priority | Color | Icon | When to Use |
|----------|-------|------|-------------|
| **High** | Red | 🔴 | Must do today |
| **Medium** | Yellow | 🟡 | Important but not urgent |
| **Low** | Green | 🟢 | Can wait |
| **None** | Gray | ⚪ | No deadline |

### Categories

Organize tasks by category:

| Category | Color | Icon |
|----------|-------|------|
| Work | Blue | 💼 |
| Personal | Green | 🏠 |
| Shopping | Orange | 🛒 |
| Health | Red | ❤️ |
| Custom | Purple | 🏷️ |

### Filter Tabs

| Tab | Shows | HTMX Trigger |
|-----|-------|--------------|
| **All** | All tasks | `hx-get="/api/tasks?filter=all"` |
| **Active** | Uncompleted tasks | `hx-get="/api/tasks?filter=active"` |
| **Completed** | Done tasks | `hx-get="/api/tasks?filter=completed"` |
| **Priority** | High priority only | `hx-get="/api/tasks?filter=priority"` |

```html
<div class="filter-tabs">
    <button class="filter-tab active"
            hx-get="/api/tasks?filter=all"
            hx-target="#task-list"
            hx-swap="innerHTML">
        📋 All
        <span class="tab-count" id="count-all">12</span>
    </button>
    <button class="filter-tab"
            hx-get="/api/tasks?filter=active"
            hx-target="#task-list"
            hx-swap="innerHTML">
        ⏳ Active
        <span class="tab-count" id="count-active">5</span>
    </button>
    <button class="filter-tab"
            hx-get="/api/tasks?filter=completed"
            hx-target="#task-list"
            hx-swap="innerHTML">
        ✓ Completed
        <span class="tab-count" id="count-completed">7</span>
    </button>
    <button class="filter-tab"
            hx-get="/api/tasks?filter=priority"
            hx-target="#task-list"
            hx-swap="innerHTML">
        ⚡ Priority
    </button>
</div>
```

### Stats Dashboard

Real-time statistics shown in the header:

```html
<div class="header-stats" id="task-stats"
     hx-get="/api/tasks/stats"
     hx-trigger="load, taskUpdated from:body"
     hx-swap="innerHTML">
    <span class="stat-item">
        <span class="stat-value">12</span>
        <span class="stat-label">Total</span>
    </span>
    <span class="stat-item">
        <span class="stat-value">5</span>
        <span class="stat-label">Active</span>
    </span>
    <span class="stat-item">
        <span class="stat-value">7</span>
        <span class="stat-label">Completed</span>
    </span>
</div>
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Add task (when in input) |
| `Space` | Toggle task completion |
| `Delete` | Remove selected task |
| `Tab` | Move to next field |
| `Escape` | Cancel editing |
| `↑` / `↓` | Navigate tasks |

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

```
GET /api/tasks?filter=active&category=work&sort=dueDate&order=asc
```

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

## HTMX Integration

### Task Creation

```html
<form hx-post="/api/tasks"
      hx-target="#task-list"
      hx-swap="afterbegin"
      hx-on::after-request="this.reset(); htmx.trigger('body', 'taskUpdated')">
    <input type="text" name="text" required>
    <button type="submit">Add</button>
</form>
```

### Task Toggle

```html
<input type="checkbox"
       hx-patch="/api/tasks/123"
       hx-vals='{"completed": true}'
       hx-target="closest .task-item"
       hx-swap="outerHTML"
       hx-on::after-request="htmx.trigger('body', 'taskUpdated')">
```

### Task Deletion

```html
<button hx-delete="/api/tasks/123"
        hx-target="closest .task-item"
        hx-swap="outerHTML swap:0.3s"
        hx-confirm="Delete this task?">
    🗑
</button>
```

### Inline Editing

```html
<span class="task-text"
      hx-get="/api/tasks/123/edit"
      hx-trigger="dblclick"
      hx-target="this"
      hx-swap="outerHTML">
    Review quarterly report
</span>
```

## CSS Classes

```css
.tasks-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
}

.tasks-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.header-stats {
    display: flex;
    gap: 1.5rem;
}

.stat-item {
    text-align: center;
}

.stat-value {
    display: block;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--primary);
}

.stat-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
}

.add-task-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
}

.task-input {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 2px solid var(--border);
    border-radius: 8px;
    font-size: 1rem;
}

.task-input:focus {
    border-color: var(--primary);
    outline: none;
}

.filter-tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
}

.filter-tab {
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.filter-tab:hover {
    background: var(--surface-hover);
}

.filter-tab.active {
    background: var(--primary);
    color: white;
}

.tab-count {
    background: rgba(255,255,255,0.2);
    padding: 0.125rem 0.5rem;
    border-radius: 10px;
    font-size: 0.75rem;
}

.task-list {
    flex: 1;
    overflow-y: auto;
}

.task-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
    transition: all 0.2s;
}

.task-item:hover {
    background: var(--surface-hover);
}

.task-item.completed {
    opacity: 0.6;
}

.task-item.completed .task-text {
    text-decoration: line-through;
    color: var(--text-secondary);
}

.task-checkbox {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    appearance: none;
}

.task-checkbox:checked {
    background: var(--success);
    border-color: var(--success);
}

.task-checkbox:checked::after {
    content: '✓';
    display: block;
    text-align: center;
    color: white;
    font-size: 14px;
    line-height: 16px;
}

.task-text {
    flex: 1;
}

.task-due {
    font-size: 0.875rem;
    color: var(--text-secondary);
}

.task-due.overdue {
    color: var(--error);
}

.task-priority {
    font-size: 0.75rem;
}

.task-priority.high {
    color: var(--error);
}

.task-priority.medium {
    color: var(--warning);
}

.task-priority.low {
    color: var(--success);
}

.task-delete {
    opacity: 0;
    padding: 0.25rem;
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--error);
}

.task-item:hover .task-delete {
    opacity: 1;
}

/* Animation for task completion */
.task-item.completing {
    animation: complete 0.3s ease-out;
}

@keyframes complete {
    0% { transform: scale(1); }
    50% { transform: scale(1.02); background: var(--success-light); }
    100% { transform: scale(1); }
}
```

## JavaScript Handlers

```javascript
// Tab switching
function setActiveTab(button) {
    document.querySelectorAll('.filter-tab').forEach(tab => {
        tab.classList.remove('active');
    });
    button.classList.add('active');
}

// Task completion animation
document.body.addEventListener('htmx:beforeSwap', (e) => {
    if (e.detail.target.classList.contains('task-item')) {
        e.detail.target.classList.add('completing');
    }
});

// Update counts after any task change
document.body.addEventListener('taskUpdated', () => {
    htmx.ajax('GET', '/api/tasks/stats', {
        target: '#task-stats',
        swap: 'innerHTML'
    });
});

// Keyboard navigation
document.addEventListener('keydown', (e) => {
    const taskItems = document.querySelectorAll('.task-item');
    const focused = document.activeElement.closest('.task-item');
    
    if (e.key === 'ArrowDown' && focused) {
        const index = [...taskItems].indexOf(focused);
        if (index < taskItems.length - 1) {
            taskItems[index + 1].querySelector('input').focus();
        }
    }
    
    if (e.key === 'ArrowUp' && focused) {
        const index = [...taskItems].indexOf(focused);
        if (index > 0) {
            taskItems[index - 1].querySelector('input').focus();
        }
    }
    
    if (e.key === ' ' && focused) {
        e.preventDefault();
        focused.querySelector('.task-checkbox').click();
    }
});

// Due date formatting
function formatDueDate(dateString) {
    const date = new Date(dateString);
    const today = new Date();
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);
    
    if (date.toDateString() === today.toDateString()) {
        return 'Today';
    } else if (date.toDateString() === tomorrow.toDateString()) {
        return 'Tomorrow';
    } else {
        return date.toLocaleDateString('en-US', { 
            month: 'short', 
            day: 'numeric' 
        });
    }
}
```

## Creating Tasks from Chat

In the Chat app, you can create tasks naturally:

```
You: Create a task to review the budget by Friday
Bot: ✅ Task created:
     📋 Review the budget
     📅 Due: Friday, March 22
     🏷️ Category: Work
     
     [View Tasks]
```

## Integration with Calendar

Tasks with due dates appear in your Calendar view:

```html
<div class="calendar-task"
     hx-get="/api/tasks/123"
     hx-target="#task-detail"
     hx-trigger="click">
    📋 Review quarterly report
</div>
```

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
4. Verify HTMX is loaded

### Stats Not Updating

1. Check for JavaScript errors
2. Verify `taskUpdated` event is firing
3. Inspect network requests
4. Reload the page

## See Also

- [HTMX Architecture](../htmx-architecture.md) - How Tasks uses HTMX
- [Suite Manual](../suite-manual.md) - Complete user guide
- [Chat App](./chat.md) - Create tasks from chat
- [Calendar App](./calendar.md) - View tasks in calendar
- [Tasks API](../../chapter-10-api/tasks-api.md) - API reference