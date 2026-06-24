# Integrated Suite — Conversational Interface Plan

> **Pattern:** Every suite app exposes its own `PROMPT.md` + internal tools.
> The shared chat bar activates app-specific context when the user is inside that app.
> WhatsApp campaigns is the first full example.
a common chat window stay fixed right like pane colapsable, except for chat... all other ui must be controled by chat, via api/ pompt common mechanismo.
---

## Architecture

```
User (WhatsApp / Suite chat bar)
        ↓
BotOrchestrator (core/bot/mod.rs)
        ↓
  detect active app context
        ↓
  load app PROMPT.md  +  app InternalTools
        ↓
  LLM with tools → tool_executor.rs
        ↓
  app data / actions
```

### Key existing pieces
| File | Role |
|---|---|
| `core/bot/mod.rs` | `get_session_tools()` + `ToolExecutor::execute_tool_call()` |
| `tasks/PROMPT.md` | Pattern for app-level LLM prompt |
| `marketing/whatsapp.rs` | WhatsApp campaign send/metrics |
| `marketing/campaigns.rs` | Campaign CRUD |
| `marketing/lists.rs` | Recipient lists |
| `botui/ui/suite/campaigns/` | Campaigns UI |

---

## Standard: Every Suite App

### 1. `PROMPT.md` per app folder
Location: `botserver/src/<app>/PROMPT.md`

```markdown
# <App> — Internal Tools Guide

You are the <App> assistant. When the user is in <App>, you have access to:
- tool: list_<entities>
- tool: create_<entity>
- tool: search_<entity>
- tool: <app_specific_action>

Rules:
- Always confirm destructive actions before executing
- Show results as structured summaries, not raw JSON
- If user uploads a file, parse it and confirm before acting
```

### 2. `tools.rs` per app
Location: `botserver/src/<app>/tools.rs`

Registers `Vec<Tool>` (LLM function-calling schema) + handler mapping.
Loaded by `get_session_tools()` when session's active app = this app.

### 3. App context detection
`core/bot/mod.rs` reads `session.active_app` (set by UI via `POST /api/session/context`).
Loads `<app>/PROMPT.md` as system prompt prefix + `<app>/tools.rs` tools.

---

## WhatsApp Campaigns — Full Conversational Flow

### Meta Rules (enforced in tools)
- Only approved Message Templates for marketing (non-session-initiated)
- 24h session window for free-form after user replies
- Media: image/video/document via Media Upload API before send
- Opt-out: always honor STOP, add to suppression list immediately
- Rate: respect per-phone-number rate limits (1000 msg/s business tier)
- Template category: MARKETING requires explicit opt-in from recipient

### Conversation Flow (WhatsApp → campaign creation)

```
User sends to bot number:
  "I want to send a campaign"
        ↓
Bot: "Great! Send me:
  1. Your contact list (.xlsx or .csv)
  2. The message text
  3. An image (optional)
  4. When to send (or 'now')"
        ↓
User uploads contacts.xlsx
        ↓
[tool: parse_contact_file]
  → extract phone numbers, names
  → validate E.164 format
  → show preview: "Found 342 contacts. First 3: +55..."
        ↓
User sends message text
        ↓
[tool: check_template_compliance]
  → check if free-form or needs approved template
  → if template needed: list available approved templates
  → suggest closest match
        ↓
User sends image (optional)
        ↓
[tool: upload_media]
  → upload to Meta Media API
  → return media_id
        ↓
Bot: "Ready to send to 342 contacts at 14:00 today.
  Preview: [image] Hello {name}, ...
  Estimated cost: $X
  Confirm? (yes/no)"
        ↓
User: "yes"
        ↓
[tool: create_and_schedule_campaign]
  → create campaign record
  → apply warmup limit if IP warming
  → schedule via TaskScheduler
```

### WhatsApp Campaign Tools (`marketing/whatsapp_tools.rs`)

```rust
// Tool definitions for LLM function calling
pub fn whatsapp_campaign_tools() -> Vec<Tool> {
    vec![
        Tool::new("parse_contact_file", "Parse uploaded xlsx/csv into contact list"),
        Tool::new("list_templates", "List approved WhatsApp message templates"),
        Tool::new("check_template_compliance", "Check if message needs approved template"),
        Tool::new("upload_media", "Upload image/video to Meta Media API"),
        Tool::new("preview_campaign", "Show campaign preview with cost estimate"),
        Tool::new("create_and_schedule_campaign", "Create campaign and schedule send"),
        Tool::new("get_campaign_status", "Get delivery/read metrics for a campaign"),
        Tool::new("pause_campaign", "Pause an in-progress campaign"),
        Tool::new("list_campaigns", "List recent campaigns with metrics"),
        Tool::new("add_to_suppression", "Add number to opt-out list"),
    ]
}
```

### WhatsApp PROMPT.md (`marketing/WHATSAPP_PROMPT.md`)

```markdown
# WhatsApp Campaign Assistant

You help users create and manage WhatsApp marketing campaigns.

## Meta Platform Rules (MANDATORY)
- Marketing messages MUST use pre-approved templates outside 24h session window
- Always check opt-in status before adding to campaign
- Honor STOP/unsubscribe immediately via add_to_suppression tool
- Never send more than warmup daily limit if IP is warming up
- Image must be uploaded via upload_media before referencing in campaign

## Conversation Style
- Guide step by step: contacts → message → media → schedule → confirm
- Show cost estimate before confirming
- After send: proactively share open/read rates when available

## File Handling
- .xlsx/.csv → use parse_contact_file tool
- Images → use upload_media tool
- Always confirm parsed data before proceeding
```

---

## Integrated Suite Chat Bar — Standard

### How it works
1. User opens any suite app (CRM, Campaigns, Drive, etc.)
2. Chat bar at bottom activates with app context
3. `POST /api/session/context { app: "campaigns" }` sets `session.active_app`
4. BotOrchestrator loads `campaigns/PROMPT.md` + `campaigns/tools.rs`
5. User can ask natural language questions or trigger actions

### Examples per app

| App | Example query | Tool activated |
|---|---|---|
| **Campaigns** | "How did last week's campaign perform?" | `get_campaign_metrics` |
| **CRM** | "Show deals closing this month" | `list_deals` with filter |
| **Drive** | "Find the Q1 report" | `search_files` |
| **Tasks** | "Create a task to follow up with Acme" | `create_task` |
| **People** | "Who hasn't been contacted in 30 days?" | `list_contacts` with filter |
| **Mail** | "Summarize unread emails from clients" | `list_emails` + LLM summary |
| **Sheet** | "What's the total revenue in column D?" | `query_sheet` |
| **Learn** | "What does our refund policy say?" | `search_kb` |

---

## Implementation Plan

### Phase 1 — Infrastructure (1 sprint)
- [ ] `core/bot/mod.rs` — read `session.active_app`, load app PROMPT + tools
- [ ] `core/tool_context.rs` — app tool registry: `register_app_tools(app_name) -> Vec<Tool>`
- [ ] `POST /api/session/context` — set active app from UI
- [ ] Suite chat bar UI component (`botui/ui/suite/partials/chatbar.html`)

### Phase 2 — WhatsApp Campaigns (1 sprint)
- [ ] `marketing/whatsapp_tools.rs` — 10 tools above
- [ ] `marketing/WHATSAPP_PROMPT.md`
- [ ] `marketing/file_parser.rs` — xlsx/csv → contact list
- [ ] Meta warmup enforcement in send path
- [ ] Conversational campaign creation flow (state machine in session)

### Phase 3 — App-by-app rollout (1 app/sprint)
Priority order based on value:
1. CRM (deals, contacts, pipeline queries)
2. Campaigns (email + WhatsApp)
3. Tasks (create, assign, status)
4. Drive (search, summarize docs)
5. Mail (summarize, draft reply)
6. People (segment, find contacts)
7. Sheet (query, calculate)
8. Learn (KB search)

### Phase 4 — Cross-app intelligence
- [ ] Global search across all apps via single query
- [ ] "What happened today?" — aggregates activity across CRM + Mail + Tasks
- [ ] Proactive suggestions: "You have 3 deals closing this week and no follow-up tasks"

---

## File Structure to Create

```
botserver/src/
├── marketing/
│   ├── whatsapp_tools.rs      ← NEW: LLM tool definitions + handlers
│   ├── WHATSAPP_PROMPT.md     ← NEW: WhatsApp assistant system prompt
│   ├── file_parser.rs         ← NEW: xlsx/csv → contacts
│   └── warmup.rs              ← NEW: (from campaigns.md plan)
├── core/
│   ├── tool_registry.rs       ← NEW: app → tools mapping
│   └── bot/
│       └── app_context.rs     ← NEW: load app prompt + tools per session
├── crm/
│   ├── tools.rs               ← NEW
│   └── PROMPT.md              ← NEW
├── tasks/
│   └── tools.rs               ← NEW (PROMPT.md exists)
└── <each app>/
    ├── tools.rs               ← NEW per app
    └── PROMPT.md              ← NEW per app

botui/ui/suite/
└── partials/
    └── chatbar.html           ← NEW: shared chat bar component
```

---

## Chat Bar UI (`partials/chatbar.html`)

```html
<div id="suite-chatbar" class="chatbar">
  <div id="chatbar-messages" hx-ext="ws" ws-connect="/ws/suite-chat"></div>
  <form ws-send>
    <input type="hidden" name="app_context" value="{{ active_app }}">
    <input type="file" id="chatbar-file" name="file" accept=".xlsx,.csv,.png,.jpg,.pdf" style="display:none">
    <button type="button" onclick="document.getElementById('chatbar-file').click()">📎</button>
    <input type="text" name="message" placeholder="Ask anything about {{ active_app }}...">
    <button type="submit">→</button>
  </form>
</div>
```

File uploads go to `POST /api/suite/upload` → stored in Drive → media_id passed to tool.

---

## Compilation & Validation Log

**Last Validated:** 2026-04-17

**Status:** ✅ Design Document - No Code to Compile

**Validation Checklist:**
- [x] Architecture diagram is clear
- [x] App pattern is defined
- [x] WhatsApp campaign flow is documented
- [x] Tool definitions are complete
- [x] HTML structure is valid
- [x] Rust code examples are syntactically correct
- [x] Implementation phases are planned
- [x] File structure is organized

**Notes:**
- This is a design document for integrated suite conversational interface
- No compilation required - it's planning documentation
- Ready for implementation when developer is available
- References existing code in botserver/src/marketing/ and botserver/src/core/bot/
