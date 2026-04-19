# Tasks - Autonomous App Builder

> **Describe what you want, get a working application**

<img src="../../assets/suite/tasks-screen.svg" alt="Tasks Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Tasks is the command center for autonomous application creation in General Bots Suite. Instead of coding, you describe what you need in plain language, and the system builds complete, working applications with databases, UI, and automation.

This is not a simple to-do list—it's an AI-powered development environment that understands intent and executes multi-step plans to create real applications.

---

## Features

### Natural Language App Creation

Describe your application in plain English:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a CRM for my cellphone repair shop</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Plan created:</strong></p>
      <p>1. Create customers table</p>
      <p>2. Create repairs table with status workflow</p>
      <p>3. Create inventory table</p>
      <p>4. Generate HTMX application</p>
      <p></p>
      <p>Confidence: 94% | ETA: 2 minutes</p>
      <p></p>
      <p>[▶️ Execute] [✏️ Modify] [❌ Cancel]</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

### Intent Classification

The system automatically classifies your request and routes it appropriately:

| Intent Type | Example Request | What Gets Created |
|-------------|-----------------|-------------------|
| `APP_CREATE` | "Create app for clinic" | Full HTMX application with tables |
| `TODO` | "Call John tomorrow at 3pm" | Task entry with reminder |
| `MONITOR` | "Alert me when stock is low" | ON CHANGE event handler |
| `ACTION` | "Email all customers now" | Immediate execution |
| `SCHEDULE` | "Daily sales report at 9am" | SET SCHEDULE automation |
| `GOAL` | "Increase response time by 20%" | Autonomous optimization loop |
| `TOOL` | "When I say 'check stock', show inventory" | Voice/chat command |

### Real-Time Progress Tracking

Watch your application being built step by step:

| Status | Icon | Meaning |
|--------|------|---------|
| Completed | ✓ | Step finished successfully |
| Running | ◐ | Currently executing |
| Pending | ○ | Waiting to start |
| Attention | ⚠ | Needs user input |
| Failed | ✕ | Error occurred |

### Execution Modes

Choose how much control you want:

| Mode | Description | Best For |
|------|-------------|----------|
| **Automatic** | Executes all steps without confirmation | Trusted, simple tasks |
| **Supervised** | Asks approval before each step | Learning, reviewing changes |
| **Dry Run** | Shows what would happen without executing | Testing, validation |

---

## Task Types

### To-Do Tasks

Simple reminders and action items:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Remind me to follow up with the client tomorrow</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Task created:</p>
      <p>📋 Follow up with client</p>
      <p>📅 Tomorrow at 9:00 AM</p>
      <p>🔔 Notification set</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

### Application Tasks

Full application generation:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need an inventory system with products, suppliers, and purchase orders</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Building inventory system...</strong></p>
      <p></p>
      <p>[████████████████░░░░] 80%</p>
      <p></p>
      <p>✓ products table created</p>
      <p>✓ suppliers table created</p>
      <p>✓ purchase_orders table created</p>
      <p>◐ Generating UI components...</p>
      <div class="wa-time">09:16</div>
    </div>
  </div>
</div>

### Scheduled Tasks

Recurring automations:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Every Monday at 8am, send me a summary of pending repairs</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Scheduler created:</p>
      <p>📄 weekly-repairs-summary.bas</p>
      <p>⏰ Every Monday at 8:00 AM</p>
      <p>📧 Will email summary to you</p>
      <div class="wa-time">11:00</div>
    </div>
  </div>
</div>

### Monitor Tasks

Event-driven triggers:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Alert me when any product stock falls below 10 units</p>
      <div class="wa-time">15:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Monitor created:</p>
      <p>📄 low-stock-alert.bas</p>
      <p>👁️ Watching: products.stock</p>
      <p>⚡ Triggers when: stock &lt; 10</p>
      <div class="wa-time">15:30</div>
    </div>
  </div>
</div>

### Goal Tasks

Autonomous optimization with metrics:

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Reduce average repair time by 15%</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Goal registered:</p>
      <p>🎯 Target: -15% repair time</p>
      <p>📊 Current avg: 3.2 days</p>
      <p>📈 Will analyze patterns and suggest improvements</p>
      <p>📅 Weekly progress reports enabled</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
</div>

---

## Generated Applications

When you create an app, it generates a complete structure:

```
.gbdrive/apps/cellphone-crm/
├── index.html          # Main HTMX application
├── _assets/
│   ├── htmx.min.js     # HTMX library
│   └── styles.css      # Application styles
└── schema.json         # Table definitions
```

### Direct API Access

Generated apps use HTMX to communicate directly with botserver:

| Operation | HTMX Attribute | Example |
|-----------|----------------|---------|
| List records | `hx-get` | `hx-get="/api/db/customers"` |
| Create record | `hx-post` | `hx-post="/api/db/customers"` |
| Update record | `hx-put` | `hx-put="/api/db/customers/123"` |
| Delete record | `hx-delete` | `hx-delete="/api/db/customers/123"` |
| Search | `hx-get` with trigger | `hx-trigger="keyup changed delay:300ms"` |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Add new task |
| `Space` | Toggle task complete |
| `Delete` | Delete selected task |
| `S` | Star/unstar task |
| `E` | Edit task |
| `P` | Set priority |
| `D` | Set due date |
| `/` | Search tasks |

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/tasks` | GET | List all tasks |
| `/api/tasks` | POST | Create new task |
| `/api/tasks/:id` | GET | Get task details |
| `/api/tasks/:id` | PUT | Update task |
| `/api/tasks/:id` | DELETE | Delete task |
| `/api/tasks/:id/execute` | POST | Execute task plan |
| `/api/tasks/:id/pause` | POST | Pause running task |
| `/api/tasks/:id/resume` | POST | Resume paused task |
| `/api/tasks/:id/cancel` | POST | Cancel task |
| `/api/tasks/:id/steps` | GET | Get task steps |

---

## Task Actions

| Action | When Available | What It Does |
|--------|----------------|--------------|
| **Execute** | Task planned | Start executing the plan |
| **Pause** | Task running | Temporarily stop execution |
| **Resume** | Task paused | Continue from last step |
| **Cancel** | Anytime | Stop and discard changes |
| **Retry** | Step failed | Retry the failed step |
| **Modify** | Task planned | Edit the plan before executing |

---

## Writing Effective Requests

### Be Specific

| ✅ Good Request | ❌ Vague Request |
|-----------------|------------------|
| "CRM for cellphone store with customer tracking, repair status, and inventory" | "Make an app" |
| "Inventory system with low stock alerts when below 10 units" | "Track stuff" |
| "Daily sales report emailed at 9am with revenue chart" | "Send reports" |
| "Alert when any customer hasn't been contacted in 30 days" | "Monitor customers" |

### Include Context

- **What data?** Customers, products, orders, etc.
- **What workflow?** Status changes, approvals, notifications
- **What output?** Reports, alerts, dashboards
- **What schedule?** Daily, weekly, on-change

---

## Examples

### Cellphone Repair Shop

```
"CRM for my repair shop with:
- Customers (name, phone, email)
- Repairs with status: received, diagnosing, waiting parts, repairing, ready, delivered
- Parts inventory with low stock alerts
- Daily summary of pending repairs"
```

### Restaurant Reservations

```
"Reservation system with:
- Tables (number, capacity, location)
- Reservations (date, time, party size, notes)
- Waitlist when fully booked
- SMS confirmation to customers"
```

### Project Management

```
"Project tracker with:
- Projects (name, client, deadline)
- Tasks with assignees and status
- Time tracking per task
- Weekly progress report"
```

---

## Troubleshooting

### Task Stuck on "Running"

1. Check the step details for errors
2. Try pausing and resuming
3. Check server logs for issues
4. Cancel and retry with modified request

### Generated App Not Working

1. Verify tables were created in database
2. Check browser console for JavaScript errors
3. Ensure API endpoints are accessible
4. Review generated HTML for issues

### Intent Misclassified

1. Be more explicit in your request
2. Use keywords like "create app", "remind me", "every day"
3. Break complex requests into smaller parts

---

## See Also

- [Autonomous Tasks - Complete Guide](../../17-autonomous-tasks/README.md) — Full documentation
- [Task Workflow](../../17-autonomous-tasks/workflow.md) — How tasks execute step by step
- [App Generation](../../17-autonomous-tasks/app-generation.md) — Understanding generated apps
- [Data Model](../../17-autonomous-tasks/data-model.md) — How tables work
- [Examples](../../17-autonomous-tasks/examples.md) — Real-world use cases
- [Designer](./designer.md) — Modify apps through conversation
- [HTMX Architecture](../htmx-architecture.md) — How the UI works