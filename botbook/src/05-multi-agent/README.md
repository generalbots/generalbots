# Chapter 5: Multi-Agent Orchestration

Build complete applications through natural conversation. Describe what you want, and the system creates it automatically.

The AutoTask system uses an LLM-powered intent classifier to understand your request and route it to the appropriate handler. Whether you need a full web application, a simple reminder, or automated monitoring, you describe it in plain language.

![AutoTask Architecture](../assets/05-multi-agent/autotask-architecture.svg)

---

## Intent Types

| Type | Example | What Gets Created |
|------|---------|-------------------|
| `APP_CREATE` | "create app for clinic" | HTMX pages, tools, schedulers |
| `TODO` | "call John tomorrow" | Task saved to tasks table |
| `MONITOR` | "alert when IBM changes" | ON CHANGE event handler |
| `ACTION` | "email all customers" | Executes immediately |
| `SCHEDULE` | "daily 9am summary" | SET SCHEDULE automation |
| `GOAL` | "increase sales 20%" | Autonomous LLM loop with metrics |
| `TOOL` | "when I say X, do Y" | Voice/chat command |

---

## Quick Start

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create an app for my clinic</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Done:</strong></p>
      <p>patients table created</p>
      <p>appointments table created</p>
      <p>App available at /apps/clinic</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

---

## Architecture Overview

![Bot Database Architecture](../assets/05-multi-agent/bot-database-architecture.svg)

One bot equals one database. All applications within a bot share the same data tables, tools, and schedulers.

---

## File Structure

| Path | Description |
|------|-------------|
| `.gbdrive/apps/{name}/` | Generated web application |
| `.gbdrive/apps/{name}/index.html` | Main HTMX page |
| `.gbdrive/apps/{name}/assets/` | CSS, images |
| `.gbdialog/tables.bas` | Database schema definitions |
| `.gbdialog/tools/` | Voice and chat commands |
| `.gbdialog/schedulers/` | Timed automations |
| `.gbdialog/events/` | Event triggers (ON CHANGE, ON EMAIL) |

---

## Creating an Application

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need an app to track customers and orders</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p><strong>Done:</strong></p>
      <p>customers table created</p>
      <p>orders table created</p>
      <p>App available at /apps/store</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

---

## Modifying Your Application

Use Designer to change anything about your app through conversation.

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Add a phone field to customers</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Phone field added to customers table.</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Change the submit button to blue</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Button color updated to blue.</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>
</div>

---

## Adding Automation

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Every day at 9am, send me new orders by email</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Scheduler created: daily-orders-summary.bas</p>
      <p>Schedule: Every day at 9:00 AM</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

---

## Keywords Reference

| Keyword | Purpose |
|---------|---------|
| `TABLE` | Define data structure |
| `FIND` | Search records |
| `SAVE` | Create record |
| `UPDATE` | Modify record |
| `DELETE` | Remove record |
| `TALK` | Send message |
| `HEAR` | Wait for input |
| `SET SCHEDULE` | Create automation |
| `ON CHANGE` | Monitor for changes |
| `ON EMAIL` | Trigger on email received |

---

## Next Steps

- [Designer Guide](./designer.md) — Edit apps through conversation
- [Data Model](./data-model.md) — Understanding tables
- [Task Workflow](./workflow.md) — How tasks execute
- [Examples](./examples.md) — Real-world applications