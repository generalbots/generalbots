# General Bots AI Agent Guidelines
- stop saving .png on root! Use /tmp. never allow new files on root.
- never push to alm without asking first - because it is production!
- **❌ NEVER deploy to production manually — ALWAYS use CI/CD pipeline**
- **❌ NEVER include sensitive data (IPs, tokens, passwords, keys) in AGENTS.md or any documentation**
- **❌ NEVER use `scp`, direct SSH binary copy, or manual deployment to system container**
- **✅ ALWAYS push to ALM → CI builds on alm-ci → CI deploys to system container automatically**
- **❌ NEVER restart botserver for config.csv changes — DriveMonitor auto-reloads on ETag change (~10s)**
- **🌐 ALWAYS respond in English regardless of the user's language — answer directly and concisely**
8080 is the server port (botserver). Suite on **3000**, cloud on **4000**, login on **5000**.
if you are in trouble with some tool, please go to the official website to get proper install or instructions
To test suite: http://localhost:3000 | To test cloud: http://localhost:4000 | To test login: http://localhost:5000
> **Exclusive Login/Signup:** `login.pragmatismo.com.br` (port 5000) is the **only** domain that serves login and signup pages. Port 4000 (cloud) **does not** serve login or signup — any access to `/login` or `/signup` on port 4000 redirects to port 5000.



test login here http://localhost:5000/login
> **⚠️ CRITICAL SECURITY WARNING**
I AM IN DEV ENV, but sometimes, pasting from PROD, do not treat my env as prod! Just fix, to me and push to CI. So I can test in PROD, for a while.
>**🚨 MANDATORY RULE: ALL bot tests MUST be done via browser (Chrome CDP port 9222). ❌ FORBIDDEN to use WebSocket (node wscat, direct WS scripts) to test bots — only the browser reflects the real state of the chat, suggestions, buttons and network errors.**
> **🚨 ALSO: Every web-facing task (login, dashboard, settings, etc.) MUST be browser-tested before marking complete. Open one tab per use case, NEVER close the browser — tabs are living trace evidence.**
> **NEVER CREATE FILES WITH SECRETS IN THE REPOSITORY ROOT**
> - ❌ **NEVER** write internal IPs to logs or output
> - When debugging network issues, mask IPs (e.g., "10.x.x.x" instead of "10.0.0.1")
> - Use hostnames instead of IPs in configs and documentation
See botserver/src/main_module/drive_monitors.rs to see how bots are loaded from MinIO drive buckets (`.gbai` format). Bots are sourced exclusively from Drive (MinIO buckets), not from local filesystem paths.
- ❌ **NEVER** create `.bak`, `.old`, or backup directories in the repository — use `/tmp/` for all backups
- ❌ **NEVER** commit `*.bak`, `*.old`, or any temporary backup files to git
- ❌ **NEVER** commit `.bas` source files from production bots — only `.ast` (compiled) and `.json` files
- ✅ `.bas` source files for production bots belong in the `work/` folder (local development only)
- ✅ `.bas` template files in `bottemplates/` are part of the repository (source templates, not production)

>
> Secret files MUST be placed in `/tmp/` only:
> - ✅ `/tmp/vault-token-gb` - Vault root token
> - ✅ `/tmp/vault-unseal-key-gb` - Vault unseal key
> - ❌ `vault-unseal-keys` - FORBIDDEN (tracked by git)
> - ❌ `start-and-unseal.sh` - FORBIDDEN (contains secrets)
>
> **Why `/tmp/`?**
> - Cleared on reboot (ephemeral)
> - Not tracked by git
> - Standard Unix security practice
> - Prevents accidental commits

---

## 📁 WORKSPACE STRUCTURE

### Three Listeners (Ports)

| Port | Service | Domain | Content | Authentication | Routing |
|------|---------|---------|----------|--------------|------------|
| **3000** | Suite (botui) | `localhost:3000` | `ui/suite/*.html` — HTMX apps, chat, desktop | ✅ GB_LOGIN_URL injected | Reverse proxy → botserver `/api/*`, `/ws` |
| **4000** | Cloud (botui) | `localhost:4000` | `ui/cloud/*.html` — store, dashboard, plans, offers | ❌ **No login/signup** — redirects → 5000 | URL rewriting (`/store` → `store.html`), GB_LOGIN_URL injected |
| **5000** | Login (botui) | `login.pragmatismo.com.br` | `ui/login/*.html` — login, signup | ✅ **Only domain with auth** | Serves CSS/JS/images from cloud via proxy |
| **8080** | API (botserver) | `localhost:8080` | API endpoints + fragments | ✅ Bearer token | `/api/*`, `/cloud/partials/*`, `/ws` |
| **—** | Desktop (botapp) | Tauri 2 | Shell wrapper | N/A | N/A |

### Cloud UI Architecture — Who Serves What

| Port | Serves | Does Not Serve |
|-------|-------|-----------|
| **3000** (suite) | `ui/suite/*` — chat, apps, desktop | ❌ `/cloud/*` (explicit 404) |
| **4000** (cloud) | `ui/cloud/*.html` — store, dashboard, plans, offers | ❌ `/login`, `/signup` (redirects 307 → 5000) |
| **5000** (login) | `ui/login/*.html` — login, signup | Auth only |
| **8080** (botserver) | API (`/api/cloud/*`), fragments (`/cloud/partials/*`), WebSocket | ❌ Complete HTML pages |

**Rule:** botserver NEVER serves complete HTMX pages — only API endpoints and HTML fragments. Complete cloud pages are statically served by botui from `ui/cloud/`.

**`GB_LOGIN_URL` Injection:** Both port 3000 (suite) and port 4000 (cloud) inject `<script>window.GB_LOGIN_URL = "http://localhost:5000";</script>` into the `<head>` of HTML pages, allowing the frontend to redirect to port 5000 without hardcoding. The `LOGIN_URL` environment variable (default `http://localhost:5000`) controls the value.

### Key Paths
- **Binary:** `target/debug/botserver`
- **Run from:** `botserver/` directory
- **Env file:** `botserver/.env`
- **Suite UI Files:** `botui/ui/suite/`
- **Cloud UI Files:** `botui/ui/cloud/`
- **Login UI Files:** `botui/ui/login/`
- **Cloud API:** botserver `/api/cloud/*`
- **Cloud fragment:** botserver `/cloud/partials/sidebar.html`

### BotUI Development Mode
**IMPORTANT:** BotUI serves static HTML/JS/CSS files directly from `botui/ui/` - **NO recompilation needed** for frontend changes.
- Changes to `.html`, `.js`, `.css` files in `botui/ui/` take effect immediately on page refresh
- Only Rust code changes in `botui/src/` require rebuild with `cargo build -p botui`
- This is "gate off" mode - static assets served directly from filesystem

### ⚠️ Critical: Absolute Paths for HTMX Apps
**TODO:** Confirm path: When subdirectory apps (e.g. `/suite/social/social.html`) are loaded via launcher into `/suite/desktop.html`, their HTML is injected into the desktop via HTMX. Relative paths (e.g. `href="social.css"`) resolve against `/suite/desktop.html`, NOT against the app's actual directory. This causes 404s like `/suite/social.css` instead of correct `/suite/social/social.css`.

**Fix:** ALL resource references in subdirectory app HTMLs MUST use absolute paths starting with `/suite/`:
```html
<!-- ✅ CORRECT — works in both direct nav and HTMX injection -->
<link rel="stylesheet" href="/suite/social/social.css" />
<script src="/suite/social/social.js"></script>

<!-- ❌ WRONG — 404 when injected via desktop launcher -->
<link rel="stylesheet" href="social.css" />
<script src="social.js"></script>
```

---

## 🏗️ System Architecture Overview

### Chat Flow Architecture

```
User Message (WebSocket)
│
▼
┌─────────────────────────────────┐
│  1. WebSocket Connection        │  botserver/src/main_module/ws/handler.rs
│     - Session established       │  UserSession created
│     - Redis connection          │  session_id generated
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│ 2. start.bas Execution │ MinIO: {bot}.gbai/...
│ - Runs ONCE per session │ {bot}.gbdialog/start.bas
│     - ADD_SUGGESTION calls      │  Adds button suggestions
│     - Sets Redis flag           │  prevents re-run
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│  3. Message Processing          │  stream_response()
│     - IF message_type == 6      │  TOOL_EXEC (bypass LLM)
│     - ELSE: KB injection        │  USE_KB context
│     - LLM processing            │  generate_response()
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│  4. Tool Execution              │  TOOL_EXEC (type 6)
│     - Direct .ast execution     │  No LLM, no KB
│     - Rhai engine               │  ScriptService::run()
│     - Immediate response        │  Result in chat
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│  5. LLM Response (if not tool) │  Groq/OpenAI/etc
│     - Prompt with context       │  System + KB + History
│     - Streaming response        │  WebSocket chunks
│     - Tool suggestions          │  LLM suggests tools
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│  6. Frontend Display            │  botui HTMX/WebSocket
│     - Message appended          │  #chat-messages
│     - Suggestion buttons        │  From Redis suggestions:{bot}:{session}
│     - Tool buttons active       │  MessageType 6 triggers
└─────────────────────────────────┘
```

### Message Types Reference

| ID | Name | Purpose | LLM Used? |
|----|------|---------|-----------|
| 0 | EXTERNAL | External message | No |
| 1 | USER | User message | Yes |
| 2 | BOT_RESPONSE | Bot response | No |
| 3 | CONTINUE | Continue processing | No |
| 4 | SUGGESTION | Suggestion button | Yes |
| 5 | CONTEXT_CHANGE | Context change | No |
| 6 | **TOOL_EXEC** | **Direct tool execution** | **No - Bypasses LLM** |

**TOOL_EXEC (Type 6)**: When frontend sends `message_type: 6`, backend executes the tool `.ast` file directly via Rhai engine. NO KB injection, NO LLM call. Result appears immediately in chat.

---

## 📝 Bot Scripts Architecture

### start.bas - Session Entry Point

**Execution:**
- Runs on WebSocket connect
- Runs again on first user message (blocking, once per session)
- Sets Redis key: `session:{session_id}:initialized`
- Subsequent messages skip start.bas

**Purpose:**
- Load suggestion buttons via `ADD_SUGGESTION "text"`
- Initialize bot memory
- Set up context

**Example:**
```basic
' start.bas
ADD_SUGGESTION "Check inventory"
ADD_SUGGESTION "Create report"
ADD_SUGGESTION "Send email"

TALK "Hello! I'm your assistant. How can I help?"
```

### tables.bas - Database Schema

**SPECIAL FILE - DO NOT CALL WITH CALL**
- Parsed automatically at compile time
- Defines tables for `sync_bot_tables()`
- Creates/updates database schema

**Example:**
```basic
' tables.bas
BEGIN TABLE customers
    id UUID PRIMARY KEY
    name VARCHAR(255)
    email VARCHAR(255)
    created_at TIMESTAMP
END TABLE

BEGIN TABLE orders
    id UUID PRIMARY KEY
    customer_id UUID REFERENCES customers
    total DECIMAL(10,2)
    status VARCHAR(50)
END TABLE
```

### {tool}.bas - Tool Scripts

**Location:** `/opt/gbo/data/{bot}.gbai/{bot}.gbdialog/{tool}.bas`
**Compiled to:** `{tool}.ast` (in memory or `/opt/gbo/work/`)
**Execution:** Via `CALL "tool"` or TOOL_EXEC (type 6)

**Example:**
```basic
' detecta.bas - Inventory checker

items = GET FROM inventory WHERE quantity < 10

IF COUNT(items) = 0 THEN
    TALK "All items well stocked!"
ELSE
    response = "Low stock items:\n"
    FOR EACH item IN items
        response = response + "- " + item.name + ": " + item.quantity + "\n"
    NEXT
    TALK response
END IF
```

### CALL Keyword

```basic
' Call in-memory procedure or .bas script
CALL "script_name"
CALL "procedure_name"

' If not in memory, looks for {name}.bas in bot's gbdialog folder
```

### DETECT Keyword

```basic
' Analyze table for anomalies
' Requires table defined in tables.bas
result = DETECT "folha_salarios"

' Calls BotModels API at /api/anomaly/detect
```

---

## 🗄️ Bot File Operations - MANDATORY RULES

### All Bot Files Come From Drive (MinIO)

**❌ NEVER manipulate bot files on the local filesystem directly.** ALL bot files (`.bas`, `.gbkb`, `.gbdrive`, config, etc.) live exclusively in MinIO Drive buckets (`{bot}.gbai`). To read, modify, or test any bot file, you MUST use `mc` (MinIO Client) to interact with Drive.

**Workflow for ANY bot file operation:**
1. **Get credentials from Vault** — load `VAULT_*` variables from `botserver/.env`, then use the Vault binary to retrieve drive credentials from `secret/gbo/drive`
2. **Configure mc** — `/tmp/mc alias set local http://127.0.0.1:${DRIVE_PORT} ${DRIVE_ACCESSKEY} ${DRIVE_SECRET} --api s3v4`
3. **Pull files from Drive** — `/tmp/mc cp local/{bot}.gbai/{bot}.gbdialog/{file}.bas /tmp/`
4. **Edit locally in /tmp** — make changes to the pulled file
5. **Push back to Drive** — `/tmp/mc cp /tmp/{file}.bas local/{bot}.gbai/{bot}.gbdialog/{file}.bas`
6. **Verify** — botserver drive_monitor will auto-detect changes and reload

**Vault credential retrieval pattern:**
```bash
# Load ONLY VAULT_* variables from .env — NO other variables allowed
source <(grep -E '^VAULT_' ${WORKSPACE}/botserver/.env)
export VAULT_ADDR=$VAULT_ADDR
export VAULT_CACERT=${WORKSPACE}/botserver-stack/conf/system/certificates/ca/ca.crt
export VAULT_TOKEN=$(cat /tmp/vault-token-gb 2>/dev/null || echo $VAULT_TOKEN)

# Get drive credentials from Vault
VAULT_BIN=${WORKSPACE}/botserver-stack/bin/vault/vault
DRIVE_ACCESSKEY=$($VAULT_BIN kv get -field=accesskey secret/gbo/drive)
DRIVE_SECRET=$($VAULT_BIN kv get -field=secret secret/gbo/drive)
DRIVE_PORT=$($VAULT_BIN kv get -field=port secret/gbo/drive)

# Configure mc
/tmp/mc alias set local http://127.0.0.1:${DRIVE_PORT} ${DRIVE_ACCESSKEY} ${DRIVE_SECRET} --api s3v4
```

**Common mc operations for bot testing:**
```bash
# List all bots
/tmp/mc ls local/

# Inspect a bot's dialog files
/tmp/mc ls local/{bot}.gbai/{bot}.gbdialog/

# Read a bot's start.bas
/tmp/mc cp local/{bot}.gbai/{bot}.gbdialog/start.bas /tmp/start.bas && cat /tmp/start.bas

# Update a bot's start.bas after editing
/tmp/mc cp /tmp/start.bas local/{bot}.gbai/{bot}.gbdialog/start.bas

# List KB documents
/tmp/mc ls local/{bot}.gbai/{bot}.gbkb/docs/

# Upload a new KB document
/tmp/mc cp /tmp/document.pdf local/{bot}.gbai/{bot}.gbkb/docs/

# Remove a file from bot
/tmp/mc rm local/{bot}.gbai/{bot}.gbdialog/old_tool.bas
```

### 🔧 LLM Configuration — config.csv

Each bot has a `config.csv` file in `{bot}.gbai/{bot}.gbot/config.csv` that controls LLM settings.

**Location:** `local/{bot}.gbai/{bot}.gbot/config.csv`

**Key fields:**

| Field | Description | Example |
|-------|-------------|---------|
| `llm-url` | Full URL for chat completions | `https://integrate.api.nvidia.com/v1/chat/completions` |
| `llm-server` | Base server URL | `https://integrate.api.nvidia.com/v1` |
| `llm-key` | API key for the LLM provider | `nvapi-...` or `sk-...` |
| `llm-model` | Model identifier | `openai/gpt-oss-120b` |
| `llm-provider` | Provider type | `openai` |
| `system-prompt` | Bot personality/instructions | `You are the virtual assistant...` |
| `history-limit` | Conversation history turns | `6` |

**How it works:**
1. BotServer reads `config.csv` from Drive via `drive_monitor` on startup/change
2. LLM config is loaded per-bot from `ConfigManager::get_config()`
3. Falls back to env vars `LLM_URL`, `LLM_MODEL`, `LLM_KEY` if not in config
4. Provider is auto-detected from URL pattern (OpenAI-compatible, Anthropic, etc.)

**Updating bot LLM model:**
```bash
# 1. Pull config.csv from Drive
/tmp/mc cp local/{bot}.gbai/{bot}.gbot/config.csv /tmp/config.csv

# 2. Edit the llm-model field in /tmp/config.csv
sed -i 's/llm-model,.*/llm-model,<desired-model>/' /tmp/config.csv

# 3. Push back to Drive (drive_monitor auto-reloads)
/tmp/mc cp /tmp/config.csv local/{bot}.gbai/{bot}.gbot/config.csv
```

### Local Debugging with Chrome DevTools Protocol (CDP)

**🚨 MANDATORY RULE: ALL bot tests MUST be done via browser (Chrome CDP port 9222).**
**❌ FORBIDDEN to use WebSocket (node wscat, direct WS scripts) to test bots** — only the browser reflects the real state of the chat, suggestions, buttons and network errors.

**Use remote Chrome on port 9222 for visual debugging.** The agent starts and controls the browser via CDP, opening one tab per subject/use case.

**Required workflow:**

1. **Check if Chrome is already open with remote debugging:**
   ```bash
   ps aux | grep "chrome.*remote-debugging-port=9222" | grep -v grep
   ```
   If not running, start:
   ```bash
   export DISPLAY=:1
   google-chrome --no-sandbox --remote-debugging-port=9222 --remote-allow-origins=* \
     --user-data-dir=/tmp/chrome-debug --start-maximized &
   ```

2. **Each use case = a separate tab.** NEVER reuse tabs. Open a new tab via CDP:
   ```bash
   python3 -c "import requests; requests.put('http://localhost:9222/json/new?URL')"
   ```
   Or manually in the already-open Chrome.

3. **Navigate to the bot:** `http://localhost:3000/{bot}` (bot chat in suite) or `http://localhost:4000/cloud` (SaaS cloud management)

4. **Interact:** type messages in the chat, click suggestion buttons, execute tools.

5. **🚨 NEVER close the browser** — keep it open for user inspection. Each tab represents a different use case.

6. **Capture screenshots** at `/tmp/{bot}_case{N}_{desc}.png` for visual evidence.

### 🚨 Suite App Testing — ALWAYS Open Inside Desktop (NEVER Direct URL)

**❌ NEVER open a suite app page directly** — this includes `/suite/drive/drive.html`, `/suite/chat/chat.html`, `/suite/tasks/tasks.html`, or any other HTML file under `botui/ui/suite/`. Suite apps are **HTMX fragments** that require the desktop shell (`desktop.html`) to bootstrap their JS modules, security context, and window manager. Opening a fragment URL directly results in a broken layout with no JS modules loaded and no network/auth context.

**✅ Correct flow — always navigate to the desktop route, never to the HTML file:**
- `/drive` → desktop shell detects app name "drive" → HTMX loads `/suite/drive/drive.html` into the content area
- `/chat/<bot>` → desktop shell → HTMX loads `/suite/chat/chat.html`
- `/tasks` → desktop shell → HTMX loads `/suite/tasks/tasks.html`
- `/social` → desktop shell → HTMX loads `/suite/social/social.html`

**Why this matters for the Drive app specifically:** The Drive app now has 5 top-level tabs: **Bots** (admin), **My Files**, **Shared**, **Public**, **Root** (admin). When loaded via direct URL, none of the JS modules execute (no `01_state.js` → `99_init.js` chain), the tab bar stays empty, no API calls happen, and the page shows a broken empty shell. The desktop shell must load the HTML fragment into its content area for the sequential script loader (in `drive.html`) to fire.

**Verification:** After login/SSO hop lands on `/drive`, the desktop shell must be visible — window manager, taskbar, app icons, title bar with close/minimize/maximize. The Drive app appears as a **window inside the desktop**, NOT as a standalone full-page view. If you see the full browser window with only Drive content and no desktop chrome, you opened the wrong URL.

**How to test in Chrome:**
1. Open a new tab and navigate to `http://localhost:3000/` (the desktop login)
2. Log in (or redirect through port 5000)
3. Type `/drive` in the desktop launcher or navigate within the desktop
4. OR, using CDP directly: `http://localhost:9222/json/new?http://localhost:3000/drive` — this will hit the Zitadel SSO redirect chain and land on the desktop with Drive loaded inside

**Summary — the three pillars for bot testing:****
- **Drive (mc)** — all bot files come from MinIO, manipulated via `mc` with Vault credentials
- **Vault (.env)** — credentials are ALWAYS obtained from Vault; only `VAULT_*` variables may be in `.env`
- **Chrome CDP (9222)** — visual debugging is ALWAYS done via remote Chrome on port 9222, with separate tabs per use case. ❌ NEVER via direct WebSocket.

---

## 💬 Common BASIC Keywords Reference

### Language Guidelines
- Use formal language in all comments and documentation
- Avoid slang, neologisms, or informal expressions
- Maintain professional tone in code comments

### TALK - Bot Response

```basic
TALK "Hello, user!"
TALK "Result: " + result

' Multi-line with \n
TALK "Line 1\nLine 2\nLine 3"
```

### HEAR - Listen for Input

```basic
HEAR "What's your name?" AS name
HEAR "Enter amount:" AS amount

' Used in voice/chat triggered tools
HEAR "check inventory" AS request
```

### USE KB - Knowledge Base Context

```basic
' Inject KB content into LLM context
USE KB "manual"
USE KB "faq"
USE KB "cartas"

' Clear KB context
CLEAR KB

' Multiple KBs
USE KB "kb1"
USE KB "kb2"
```

**Flow:**
```
USE KB "manual"
↓
Bot searches .gbkb/ folder for documents
↓
Chunks text, creates embeddings
↓
Queries Qdrant for relevant chunks
↓
Injects into LLM prompt as context
↓
User question answered with KB context
```

### USE WEBSITE - Web Scraping Context

```basic
' Scrape website and inject into context
USE WEBSITE "https://example.com/docs"
USE WEBSITE "https://api.example.com/swagger"

' Combined with USE KB
USE KB "manual"
USE WEBSITE "https://company.com/updates"
TALK "How can I help with our product?"
```

### ADD SUGGESTION - Suggestion Buttons

```basic
' In start.bas - shown as quick reply buttons
ADD_SUGGESTION "Check status"
ADD_SUGGESTION "Create ticket"
ADD_SUGGESTION "Contact support"

' Deduplicated with Redis SADD
' Key: suggestions:{bot_id}:{session_id}
' Read with SMEMBERS
```

### Database Operations

```basic
' GET - Query records
customers = GET FROM customers WHERE status = "active"
order = GET FROM orders WHERE id = "123"

' SAVE - Insert/update
SAVE customer TO customers
SAVE order TO orders

' FIND - Search
result = FIND "term" IN products

' Array functions
first = FIRST(customers)
last = LAST(customers)
count = COUNT(customers)
```

### File Operations

```basic
' Create file in .gbdrive/
CREATE FILE "reports/sales.txt" WITH report_content

' Read file
content = READ FILE "data/config.txt"

' Write file
WRITE FILE "logs/activity.log" WITH log_entry

' Upload to MinIO
UPLOAD data TO "exports/data.json"
```

### HTTP Operations

```basic
' GET request
response = GET HTTP "https://api.example.com/data"

' POST request
result = POST HTTP "https://api.example.com/webhook" WITH json_data

' Webhook
WEBHOOK "https://callback.example.com" WITH payload
```

### Task & Scheduling

```basic
' Create task
CREATE_TASK "Review report", "john", "2024-01-15", project_id

' Wait
WAIT 5  ' seconds

' Event handlers
ON EMAIL FROM "@company.com" DO CALL "process_email"
ON CHANGE customers DO CALL "notify_admin"
```

### Memory & Context

```basic
' Bot-level memory (persists across sessions)
SET BOT MEMORY "company_name" = "Acme Corp"
name = GET BOT MEMORY "company_name"

' Session-level memory
REMEMBER "user_preference" = "dark_mode"
pref = RECALL "user_preference"

' Context variables
SET CONTEXT "current_order" = order_id
```

### Multi-Bot Operations

```basic
' Add sub-bot
ADD BOT "sales" WITH TRIGGER "talk to sales"

' Delegate
DELEGATE TO "sales"

' Send message to another bot
SEND TO BOT "sales" MESSAGE "New lead available"

' Broadcast
BROADCAST MESSAGE "System maintenance in 5 minutes"
```

### Control Flow

```basic
' IF/THEN/ELSE
IF condition THEN
    ' true branch
ELSE
    ' false branch
END IF

' FOR EACH loop
FOR EACH customer IN customers
    SEND MAIL TO customer.email WITH subj, body
    WAIT 1
NEXT

' SWITCH/CASE
SWITCH status
    CASE "active"
        TALK "Account active"
    CASE "inactive"
        TALK "Account inactive"
    DEFAULT
        TALK "Unknown status"
END SWITCH
```

### Built-in Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TODAY` | Current date | `IF created_at == TODAY THEN` |
| `NOW` | Current datetime | `SET last_seen = NOW` |
| `USER` | Current user object | `USER.email`, `USER.id` |
| `SESSION` | Current session object | `SESSION.id` |
| `BOT` | Current bot object | `BOT.name`, `BOT.id` |

---

## 🧭 LLM Navigation Guide

### Reading This Workspace
/opt/gbo/data is a place also for bots.
**For LLMs analyzing this codebase:**
0. Bots are in drive, each bucket is a bot. Respect LOAD_ONLY.
1. Start with **[Component Dependency Graph](../README.md#-component-dependency-graph)** in README to understand relationships
2. Review **[Module Responsibility Matrix](../README.md#-module-responsibility-matrix)** for what each module does
3. Study **[Data Flow Patterns](../README.md#-data-flow-patterns)** to understand execution flow
4. Reference **[Common Architectural Patterns](../README.md#-common-architectural-patterns)** before making changes
5. Check **[Security Rules](#-security-directives---mandatory)** below - violations are blocking issues
6. Follow **[Code Patterns](#-mandatory-code-patterns)** below - consistency is mandatory

---

## 🔄 Reset Process Notes

### reset.sh Behavior
- **Purpose**: Cleans and restarts the development environment
- **Timeouts**: The script can timeout during "Step 3/4: Waiting for BotServer to bootstrap"
- **Bootstrap Process**: Takes 3-5 minutes to install all components (Vault, PostgreSQL, Valkey, MinIO, Zitadel, LLM)

### Common Issues
1. **Script Timeout**: reset.sh waits for "Bootstrap complete: admin user" message
   - If Zitadel isn't ready within 60s, admin user creation fails
   - Script continues waiting indefinitely
   - **Solution**: Check botserver.log for "Bootstrap process completed!" message

2. **Zitadel Not Ready**: "Bootstrap check failed (Zitadel may not be ready)"
   - Directory service may need more than 60 seconds to start
   - Admin user creation deferred
   - Services still start successfully

3. **Services Exit After Start**: 
   - botserver/botui may exit after initial startup
   - Check logs for "dispatch failure" errors
   - Check Vault certificate errors: "tls: failed to verify certificate: x509"

### Manual Service Management
```bash
# If reset.sh times out, manually verify services:
ps aux | grep -E "(botserver|botui)" | grep -v grep
curl http://localhost:8080/health
tail -f botserver.log botui.log

# Restart services manually:
./restart.sh
```

### Reset Verification
After reset completes, verify:
- ✅ PostgreSQL running (port 5432)
- ✅ Valkey cache running (port 6379)
- ✅ BotServer listening on port 8080
- ✅ BotUI listening on ports 3000 (suite), 4000 (cloud), 5000 (login)
- ✅ No errors in botserver.log

---

## 🔐 Security Directives - MANDATORY

### 1. Error Handling - NO PANICS IN PRODUCTION

The `botserver` serves thousands of simultaneous sessions 24/7; any `panic!` crashes the process and interrupts all connected users. Therefore, every error path must propagate via `Result` or be handled locally — `unwrap`, `expect`, `panic!`, `todo!` and `unimplemented!` are strictly forbidden outside of tests. The pairs below contrast the code that aborts the process with the code that keeps it running.

```rust
// ❌ FORBIDDEN — any call that aborts the process is prohibited in production
fn load_config(path: &str) -> Config {
    let raw = std::fs::read_to_string(path).unwrap();            // panic if file missing
    let cfg: Config = serde_json::from_str(&raw).expect("parse"); // panic on invalid JSON
    if cfg.users.is_empty() {
        panic!("no users defined");                               // deliberate crash
    }
    cfg
}

fn save_user(user: &User) {
    let conn = POOL.get().unwrap();                               // panic if pool closed
    conn.execute(...).unwrap();                                   // panic on SQL error
    todo!("persistence not implemented yet");                     // forbidden placeholder
}
```

```rust
// ✅ REQUIRED — propagate via `?`, or handle locally with log
fn load_config(path: &str) -> Result<Config, ConfigError> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::Io(path.into(), e))?;
    let cfg: Config = serde_json::from_str(&raw)
        .map_err(ConfigError::Parse)?;
    if cfg.users.is_empty() {
        return Err(ConfigError::Empty);
    }
    Ok(cfg)
}

fn save_user(user: &User) -> Result<(), DbError> {
    let conn = POOL.get()
        .ok_or_else(|| DbError::PoolClosed)?;
    conn.execute(...).map_err(DbError::from)?;
    Ok(())
}

fn load_or_default(path: &str) -> Config {
    match load_config(path) {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("config load failed for {path}: {e}");
            Config::default()
        }
    }
}
```

**Quick translation table — whenever the left pattern appears, rewrite using the right one:**

| Forbidden pattern | Mandatory replacement |
|-----------------|--------------------------|
| `value.unwrap()` | `value?` (in function returning `Result`) or `value.ok_or_else(|| Error::X)?` |
| `value.expect("msg")` | `value.context("msg")?` (with `anyhow`) or `value.map_err(|e| Error::X(e))?` |
| `panic!("...")` | `return Err(Error::X.into());` |
| `todo!()` | actual function body or `unimplemented!()` documented in `#[cfg(test)]` |
| `unimplemented!()` | same — or `return Err(Error::NotImplemented.into());` |
| `if let Some(v) = x { ... }` loose | `match x { Some(v) => ..., None => return Err(...) }` for exhaustiveness |
| `match x { Ok(v) => v, Err(_) => default }` silent | `match x { Ok(v) => v, Err(e) => { log::error!(...); default } }` |

### 2. Command Execution - USE SafeCommand

```rust
// ❌ FORBIDDEN
Command::new("some_command").arg(user_input).output()

// ✅ REQUIRED
use crate::security::command_guard::SafeCommand;
SafeCommand::new("allowed_command")?
    .arg("safe_arg")?
    .execute()
```

### 3. Error Responses - USE ErrorSanitizer

```rust
// ❌ FORBIDDEN
Json(json!({ "error": e.to_string() }))
format!("Database error: {}", e)

// ✅ REQUIRED
use crate::security::error_sanitizer::log_and_sanitize;
let sanitized = log_and_sanitize(&e, "context", None);
(StatusCode::INTERNAL_SERVER_ERROR, sanitized)
```

### 4. SQL - USE sql_guard

```rust
// ❌ FORBIDDEN
format!("SELECT * FROM {}", user_table)

// ✅ REQUIRED
use crate::security::sql_guard::{sanitize_identifier, validate_table_name};
let safe_table = sanitize_identifier(&user_table);
validate_table_name(&safe_table)?;
```

### 5. Rate Limiting Strategy (IMP-07)

- **Default Limits:**
  - General: 100 req/s (global)
  - Auth: 10 req/s (login endpoints)
  - API: 50 req/s (per token)
- **Implementation:**
  - MUST use `governor` crate
  - MUST implement per-IP and per-User tracking
  - WebSocket connections MUST have message rate limits (e.g., 10 msgs/s)

### 6. CSRF Protection (IMP-08)

- **Requirement:** ALL state-changing endpoints (POST, PUT, DELETE, PATCH) MUST require a CSRF token.
- **Implementation:**
  - Use `tower_csrf` or similar middleware
  - Token MUST be bound to user session
  - Double-Submit Cookie pattern or Header-based token verification
  - **Exemptions:** API endpoints using Bearer Token authentication (stateless)

### 7. Security Headers (IMP-09)

- **Mandatory Headers on ALL Responses:**
  - `Content-Security-Policy`: "default-src 'self'; script-src 'self'; object-src 'none';"
  - `Strict-Transport-Security`: "max-age=63072000; includeSubDomains; preload"
  - `X-Frame-Options`: "DENY" or "SAMEORIGIN"
  - `X-Content-Type-Options`: "nosniff"
  - `Referrer-Policy`: "strict-origin-when-cross-origin"
  - `Permissions-Policy`: "geolocation=(), microphone=(), camera=()"

### 8. Dependency Management (IMP-10)

- **Pinning:**
  - Application crates (`botserver`, `botui`) MUST track `Cargo.lock`
  - Library crates (`botlib`) MUST NOT track `Cargo.lock`
- **Versions:**
  - Critical dependencies (crypto, security) MUST use exact versions (e.g., `=1.0.1`)
  - Regular dependencies MAY use caret (e.g., `1.0`)
- **Auditing:**
  - Run `cargo audit` weekly
  - Update dependencies only via PR with testing

---

## ✅ Mandatory Code Patterns

### Use Self in Impl Blocks
```rust
impl MyStruct {
    fn new() -> Self { Self { } }  // ✅ Not MyStruct
}
```

### Derive Eq with PartialEq
```rust
#[derive(PartialEq, Eq)]  // ✅ Always both
struct MyStruct { }
```

### Inline Format Args
```rust
format!("Hello {name}")  // ✅ Not format!("{}", name)
```

### Combine Match Arms
```rust
match x {
    A | B => do_thing(),  // ✅ Combine identical arms
    C => other(),
}
```

---

## ❌ Absolute Prohibitions
- NEVER search /target folder! It is binary compiled.
- ❌ **NEVER** hardcode passwords, tokens, API keys, or any credentials in source code — ALWAYS use `generate_random_string()` or environment variables
- ❌ **NEVER** build in release mode - ONLY debug builds allowed
- ❌ **NEVER** use `--release` flag on ANY cargo command
- ❌ **NEVER** run `cargo build` - use `cargo check` for syntax verification
- ❌ **NEVER** run `cargo check` synchronously - always `nohup cargo check > /tmp/<crate>_check.log 2>&1 &`
- ❌ **NEVER** compile directly for production - ALWAYS use push + CI/CD pipeline
- ❌ **NEVER** use `scp` or manual transfer to deploy - ONLY CI/CD ensures correct deployment
- ❌ **NEVER** manually copy binaries to production system container - ALWAYS push to ALM and let CI/CD build and deploy
- ❌ **NEVER** SSH into system container to deploy binaries - CI workflow handles build, transfer, and restart via alm-ci SSH
- ✅ **ALWAYS** push code to ALM → CI builds on alm-ci → CI deploys to system container via SSH from alm-ci
- ✅ **CI deploy path**: alm-ci builds at `/opt/gbo/data/botserver/target/debug/botserver` → tar+gzip via SSH → `/opt/gbo/bin/botserver` on system container → restart


- ❌ **NEVER** change git branches for any reason without explicit user approval
- ❌ **NEVER** use `panic!()`, `todo!()`, `unimplemented!()`
- ❌ **NEVER** use `Command::new()` directly - use `SafeCommand`
- ❌ **NEVER** return raw error strings to HTTP clients
- ❌ **NEVER** use `#[allow()]` in source code - FIX the code instead
- ❌ **NEVER** add lint exceptions to `Cargo.toml` - FIX the code instead
- ❌ **NEVER** use `_` prefix for unused variables - DELETE or USE them
- ❌ **NEVER** leave unused imports or dead code
- ❌ **NEVER** use CDN links - all assets must be local
- ❌ **NEVER** create `.md` documentation files without checking `botbook/` first
- ❌ **NEVER** comment out code - FIX it or DELETE it entirely

---

## 📏 File Size Limits - MANDATORY

### Maximum 450 Lines Per File

When a file grows beyond this limit:

1. **Identify logical groups** - Find related functions
2. **Create subdirectory module** - e.g., `handlers/` (Rust) or `modules/` (JS)
3. **Split by responsibility:**
   - `types.rs` - Structs, enums, type definitions
   - `handlers.rs` - HTTP handlers and routes
   - `operations.rs` - Core business logic
   - `utils.rs` - Helper functions
   - `mod.rs` - Re-exports and configuration
4. **Keep files focused** - Single responsibility
5. **Update mod.rs** - Re-export all public items

**NEVER let a single file exceed 450 lines - split proactively at 350 lines**

### JS Frontend Module Pattern

App JS files in `botui/ui/suite/<app>/` that exceed 450 lines are split into `modules/` subdirectory:

```
botui/ui/suite/<app>/
├── <app>.html          # Loads modules via <script> tags
├── <app>.js.orig       # Original monolithic file (backup)
└── modules/
    ├── 01_state.js     # Config, state, constants
    ├── 02_render.js    # Rendering logic
    ├── 03_events.js    # Event handlers
    ├── ...
    └── 99_init.js      # Startup/init code
```

**Rules:**
- Each module is a plain `<script>` file (NOT ES module) — functions are globally visible across modules via load order
- Module scripts are ordered in HTML from lowest-numbered to `99_init.js`
- `99_init.js` contains only the `DOMContentLoaded`/`init()` call
- Original monolithic file preserved as `<app>.js.orig`
- Each module file MUST begin with `"use strict";` after the header comment
- Module naming: `{order}_{firstFunctionName}.js`

---

## 🔥 Error Fixing Workflow

### Mode 1: OFFLINE Batch Fix (PREFERRED)

When given error output:

1. **Read ENTIRE error list first**
2. **Group errors by file**
3. **For EACH file with errors:**
   a. View file → understand context
   b. Fix ALL errors in that file
   c. Write once with all fixes
4. **Move to next file**
5. **REPEAT until ALL errors addressed**
6. **ONLY THEN → verify with build/diagnostics**

**NEVER run cargo build/check/clippy DURING fixing**
**Fix ALL errors OFFLINE first, verify ONCE at the end**

### Mode 2: Interactive Loop

```
LOOP UNTIL (0 warnings AND 0 errors):
  1. Run diagnostics → pick file with issues
  2. Read entire file
  3. Fix ALL issues in that file
  4. Write file once with all fixes
  5. Verify with diagnostics
  6. CONTINUE LOOP
END LOOP
```

### ⚡ Streaming Build Rule

**Do NOT wait for `cargo` to finish.** As soon as the first errors appear in output, cancel/interrupt the build, fix those errors immediately, then re-run. This avoids wasting time on a full compile when errors are already visible.

### 🔀 Parallel & Non-Blocking Execution Philosophy

**NEVER block the agent on long-running processes.** Always launch builds/checks in the background and continue working on code analysis, fixes, or planning while they run.

#### Core Principles
1. **NEVER WAIT** — Long-running tools (cargo check, cargo clippy, builds) must run in background via `nohup`
2. **ALWAYS PARALLEL** — Launch multiple independent checks simultaneously
3. **KEEP THINKING** — While processes run, analyze code, plan fixes, read files, write edits
4. **POLL LATER** — Check background process results when convenient, not immediately
5. **NEVER IDLE** — If a tool is running, start another task instead of waiting
6. **NEVER STOP THE LOOP** — The user only talks at the start; once tasked, the agent drives autonomously through the full cycle: launch → analyze → fix → verify → repeat. NEVER stop mid-loop to ask "should I continue?" — just keep going until 0 warnings/0 errors or a blocker that genuinely requires user input

#### Background Compile Pattern
```bash
# Launch compile checks in background — NEVER wait synchronously
nohup cargo check -p botcore > /tmp/botcore_check.log 2>&1 &
nohup cargo check -p botserver > /tmp/botserver_check.log 2>&1 &
nohup cargo check -p botlib > /tmp/botlib_check.log 2>&1 &
echo "Checks running in background — continue analyzing code"

# Later, when convenient, poll results:
tail -50 /tmp/botcore_check.log
tail -50 /tmp/botserver_check.log

# Check if still running:
ps aux | grep 'cargo check' | grep -v grep
```

#### Parallel Workflow
```
1. Launch: nohup cargo check -p <crate1> > /tmp/<crate1>_check.log 2>&1 &
2. Launch: nohup cargo check -p <crate2> > /tmp/<crate2>_check.log 2>&1 &
3. IMMEDIATELY: Read files, analyze code, plan fixes (do NOT wait)
4. Read more files, grep patterns, understand architecture
5. When ready: tail /tmp/<crate1>_check.log to see results
6. Fix errors offline (batch by file)
7. Re-launch checks in background
8. Continue working on next files while checks run
```

#### Rules
- ❌ **NEVER** run `cargo check` synchronously and wait — always `nohup ... &`
- ❌ **NEVER** run only one check when multiple crates need verification — launch all in parallel
- ❌ **NEVER** sit idle waiting for a process — start another analysis task
- ❌ **NEVER** stop the loop to ask the user "should I continue?" — drive autonomously until done
- ❌ **NEVER** fix errors one-by-one by hand when a Python script can batch-fix them — always generate a Python script
- ✅ **ALWAYS** write logs to `/tmp/` (never repo root)
- ✅ **ALWAYS** check if processes are still running before launching new ones (`ps aux | grep cargo`)
- ✅ **ALWAYS** kill stale processes before re-launching (`pkill -f "cargo check -p <crate>"`)
- ✅ **ALWAYS** continue code analysis/fixing while background processes run
- ✅ **ALWAYS** generate Python scripts for batch fixes — save to `/tmp/fix_*.py`, run with `python3 /tmp/fix_*.py`

### 🐍 Python Batch-Fix Scripts

**When 5+ errors share a pattern, ALWAYS generate a Python script instead of fixing by hand.**

The LLM agent writes a Python script that:
1. Reads the error log (`/tmp/<crate>_check.log`)
2. Parses error locations (file:line:col)
3. Reads each file, applies regex-based fixes
4. Writes all fixes at once
5. Reports what was fixed

**Pattern:**
```python
#!/usr/bin/env python3
"""Batch fix script: <description>"""
import re, os, sys

WORKSPACE = os.getenv("WORKSPACE", os.getcwd())

def fix_file(filepath, fixes):
    with open(filepath, 'r') as f:
        content = f.read()
    for old, new in fixes:
        content = content.replace(old, new)
    with open(filepath, 'w') as f:
        f.write(content)
    print(f"Fixed: {filepath}")

# Example: fix field renames across all files
for root, dirs, files in os.walk(WORKSPACE + "/botserver"):
    for fn in files:
        if fn.endswith('.rs'):
            path = os.path.join(root, fn)
            fix_file(path, [
                ("endpoint_url", "endpoint"),
                ("bucket_name", "bucket"),
            ])
```

**LLM-Enabled Python Scripts (for complex analysis):**
- For fixes requiring semantic understanding (e.g., "which type does this method return?"), use `openai` Python package
- Set `DEV_LLM_URL` and `DEV_LLM_KEY` in `.env` — these are DEV-only keys for the agent's Python scripts
- NEVER commit these keys — they are in `.env` only, loaded at runtime
- Pattern: script sends error context to LLM, LLM suggests fix, script applies it

```python
# LLM-assisted fix pattern
import openai, os
client = openai.OpenAI(
    base_url=os.getenv("DEV_LLM_URL"),
    api_key=os.getenv("DEV_LLM_KEY")
)
def ask_llm(error_msg, code_context):
    resp = client.chat.completions.create(
        model=os.getenv("DEV_LLM_MODEL", "default"),
        messages=[
            {"role": "system", "content": "You are a Rust fix generator. Return ONLY the fixed code, no explanations."},
            {"role": "user", "content": f"Error:\n{error_msg}\n\nCode:\n{code_context}"}
        ]
    )
    return resp.choices[0].message.content
```

---

## 🧠 Memory Management

When compilation fails due to memory issues (process "Killed"):

```bash
pkill -9 cargo; pkill -9 rustc; pkill -9 botserver
CARGO_BUILD_JOBS=1 cargo check -p botserver 2>&1 | tail -200
```

---

## 🧪 Testing Staging Environment (STAGE-GBO)

To test `chat.stage.pragmatismo.com.br` or other services in the STAGE-GBO environment:
- Use the `10.0.3.x` subnet for container IPs (e.g., `10.0.3.10` for the system container).
- Route testing via the host gateway at `10.0.0.1` or directly hit container IPs inside the staging host.
- Do NOT confuse staging IP ranges (`10.0.3.x`) with production ranges.

---

## 🎯 Automatic Bot Testing Workflow

 **When user says "test bot" or similar — do this autonomously:**

1. **Ask** "What bot would you like to test today?" (do NOT assume a specific bot name)
2. **Get Drive credentials from Vault** — follow the pattern in [Bot File Operations - MANDATORY RULES](#-bot-file-operations---mandatory-rules). Load ONLY `VAULT_*` from `.env`, retrieve credentials from `secret/gbo/drive`
3. **Run restart.sh** — `nohup ./restart.sh > /tmp/restart.log 2>&1 &`
4. **Wait for bootstrap** — poll `curl -s http://localhost:8080/health` until it responds 200 (up to 5 min)
5. **Find the bot** — check MinIO drive buckets via `mc`: `/tmp/mc ls local/` (each bucket = `{bot}.gbai`)
6. **If bot not in drive, ask user** — do NOT copy from work dir. Ask: "Where can I get a copy of the .gbai to work on?"
7. **Verify bot loaded** — check botserver logs for `[drive_monitor]` confirming bot sync
8. **Start Chrome CDP** — check if Chrome is running with `--remote-debugging-port=9222`; if not, start with `--remote-allow-origins=*` to allow WebSocket connections from any origin:
   ```bash
   export DISPLAY=:1
   google-chrome --no-sandbox --remote-debugging-port=9222 --remote-allow-origins=* \
     --user-data-dir=/tmp/chrome-debug --start-maximized &
   ```
9. **🚨 NEVER CLOSE THE BROWSER** — each open tab represents a use case and must remain open for user inspection. Close only when explicitly requested.
10. **Open tabs via Playwright CDP** (connects to already-open Chrome on 9222, tabs persist after the script):
    ```python
    from playwright.async_api import async_playwright
    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp("http://localhost:9222")
        default_ctx = browser.contexts[0]
        page = await default_ctx.new_page()
        await page.goto('http://localhost:4000/cloud/signup')
        # ... interact ...
        # DO NOT close the browser at the end — tabs remain open
    ```
11. **Test 3 use cases in separate tabs** — each tab is a different case:
    - **Case 1 (Greeting):** Send "Hello", check the welcome TALK and suggestion buttons
    - **Case 2 (Main service):** Send a message about the bot's main service, check data collection flow
    - **Case 3 (Second service or pending items):** Test the second service or list pending items
12. **Verify responses** — use `page.evaluate()` to capture the last `.message.bot .bot-message`
13. **Capture screenshots** — save to `/tmp/{bot}_case{N}_{before|after}.png`
14. **Report results** — visual evidence + summary of each case: messages sent, responses received, suggestions

**Standard script to interact via CDP (Node.js):**
```javascript
const CDP = require('./cdp-client'); // or inline ws + net
const cdp = new CDP(wsUrl);
await cdp.eval(\`document.getElementById('messageInput').value = 'message'\`);
cdp.eval(\`document.getElementById('chatForm').dispatchEvent(new Event('submit', {bubbles:true,cancelable:true}))\`);
const response = await cdp.eval(\`
  (() => {
    var msgs = document.getElementById('messages');
    if (!msgs) return '';
    for (var i = msgs.children.length - 1; i >= 0; i--) {
      var msg = msgs.children[i];
      if (msg.classList.contains('bot')) {
        var content = msg.querySelector('.bot-message');
        return content ? content.textContent.trim().substring(0, 800) : '';
      }
    }
    return '';
  })()
\`);
```

**Key commands:**
```bash
# Check health
curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health

# Get Drive credentials from Vault (ALWAYS do this first)
source <(grep -E '^VAULT_' ${WORKSPACE}/botserver/.env)
export VAULT_ADDR=$VAULT_ADDR
export VAULT_CACERT=${WORKSPACE}/botserver-stack/conf/system/certificates/ca/ca.crt
export VAULT_TOKEN=$(cat /tmp/vault-token-gb 2>/dev/null || echo $VAULT_TOKEN)
VAULT_BIN=${WORKSPACE}/botserver-stack/bin/vault/vault
DRIVE_ACCESSKEY=$($VAULT_BIN kv get -field=accesskey secret/gbo/drive)
DRIVE_SECRET=$($VAULT_BIN kv get -field=secret secret/gbo/drive)
DRIVE_PORT=$($VAULT_BIN kv get -field=port secret/gbo/drive)

# Configure mc with Vault credentials (NEVER hardcode)
/tmp/mc alias set local http://127.0.0.1:${DRIVE_PORT} ${DRIVE_ACCESSKEY} ${DRIVE_SECRET} --api s3v4

# Upload bot to MinIO
/tmp/mc mb local/{bot}.gbai
/tmp/mc cp --recursive botserver-stack/data/system/work/{bot}.gbai/ local/{bot}.gbai/

# Check botserver logs for errors
grep -E "ERROR|WARN|drive_monitor" botserver.log | tail -20
```

---

## ☁️ Cloud SaaS Product Architecture (CRM + Default Bot)

Products live in `botproducts` crate, NOT in CRM. Products and CRM share the same `org_id`/`bot_id`/`branch_id` scope but are separate domains with no FK between them.

**Products are scoped by `branch_id`** (not by `org_id` or `bot_id` directly). The `get_bot_context()` at `botproducts/src/lib.rs:33` resolves to `branch_id = Uuid::nil()` in the global SaaS admin mode, or to the organization's real branch when a user signs up.

### Feature Gates & Dependency Chain

| Feature | Enables | Effect |
|---------|---------|--------|
| `people` | CRM (contacts, tickets, leads) | `botcrm` crate |
| `billing` | Product CRUD routes | `botproducts` crate |
| `saas` | seeding + cloud API + subscriptions | includes `billing` + `botproducts` |

Chain: `saas` → `billing` → `botproducts`

### Product Seeding (idempotent)

**Two trigger points** via `botproducts::seed::seed_default_products(conn, branch_id)`:

1. **Server init** (`init.rs:269`): called once with `Uuid::nil()` (global catalog — visible to all orgs before signup)
2. **Org signup** (`botcloud/api.rs:376`): called with the new org's `branch_id` (creates a dedicated product scope for the org's branch)

Seeded products by category (all `stock_quantity: -1` / unlimited):

| Category | SKUs | product_type |
|----------|------|-------------|
| Plans | `free`($0), `shared`($3.99), `private-cloud`(custom) | `plan` |
| VMs | `vps-small`($9.99), `vps-medium`($19.99), `vps-large`($39.99) | `infrastructure` |
| GPU | `gpu-basic`($39.99), `gpu-advanced`($99.99) | `infrastructure` |
| Storage | `storage-50gb`($9.99), `storage-200gb`($29.99), `storage-1tb`($99.99) | `infrastructure` |
| Comms | `number-local`($5.99), `number-tollfree`($9.99), `domain-com`($21.99/yr), `domain-org`($19.99/yr) | `communication` |
| LLM Tokens | `llm-1m`($9.99), `llm-10m`($79.99) | `llm-tokens` |

### `get_default_bot` Resolver

Determines which bot's product scope to use. The default bot acts as the SaaS backend — the super admin logs into the default bot (port 3000) and sees CRM, products, clients, billing all integrated. The Store, Plans and other cloud pages (port 4000) read from the same scope.

- **SaaS active** (`get_default_bot=Some(|_c| (nil, "default"))`): queries use `branch_id = Uuid::nil()` — products seeded globally visible
- **SaaS inativo** (`get_default_bot=None`): **BUG conhecido** — handlers retornam `None` prematuramente, causando grids vazios

All crates must use `Some(...)` when the corresponding feature is active:

| Crate | Feature | get_default_bot | File/line |
|-------|---------|----------------|--------------|
| `botproducts` (products) | `billing` | `Some(\|_c\| (nil, "default"))` | `server.rs:379` |
| `bottickets` (tickets) | `tickets` | `Some(\|_c\| (nil, "default"))` | `server.rs:387` |
| `botpeople` (people) | `people` | `Some(\|_c\| (nil, "default"))` | `server.rs:397` |
| `botattendant` (attendant) | `attendant` | `Some(\|_c\| (nil, "default"))` | `server.rs:409` |
| `botworkspaces` (workspaces) | `workspaces` | `Some(\|_c\| (nil, "default"))` | `server.rs:371` |

**⚠️ IMPORTANT:** Do not confuse with `get_default_bot` used in `botcloud` (signup) — there the closure returns the first active bot from the database (`query_first_bot`) for per-organization scoping. In the suite/admin, we use `(nil, "default")` for the global default bot scope.

**Bug history:** ProductsState had `get_default_bot: None` for weeks after the module refactoring (jun/2026). All other crates (tickets, people, attendant, workspaces) correctly used `Some(|_c| (nil, "default"))`. The fix was to apply the same pattern in `products_routes` at `server.rs:379-380`.

Implementation at `botproducts/src/lib.rs:33-45` (`get_bot_context()`).

### Available Product Routes

**REST API** (`/api/products/*`): CRUD for items, services, categories, price-lists, stock, stats, low-stock

**HTMX fragments** (`/api/ui/products/*`): grids/tables for items, services, pricelists, stats, search

### Cloud Pages That Display Products

| Page | Path | Shows |
|------|------|-------|
| Store | `botui/ui/cloud/store.html` | Full catalog (hardcoded + API overlay) |
| Plans | `botui/ui/cloud/plans.html` | Plan grid from API |
| Offers | `botui/ui/cloud/offers.html` | Bundles |
| Dashboard | `botui/ui/cloud/dashboard.html` | Current plan + usage |
| Signup | `botui/ui/login/signup.html` (porta 5000) | Plan selector |

### ⚠️ Important LLM Rules

- **NEVER create a separate admin products HTML page.** Products are shown through existing Store/Plans/Dashboard pages.
- **Products are NOT CRM entities** — they share the `org_id`/`bot_id` scope but have separate tables and no FK relationship.
- **Seeding is idempotent** — checks `products::table.filter(branch_id)` before inserting.
- **Default bot with nil UUID** is a global fallback. In production SaaS, `query_first_bot` finds the real active bot.

### Cloud Domain Architecture

| Domain | Port | Serves | Notes |
|--------|------|--------|-------|
| `pragmatismo.com.br` | Caddy → 4000 | Landing page + Cloud store/Plans | Static landing + cloud UI via botui |
| `cloud.pragmatismo.com.br` | Caddy → 4000 | Cloud dashboard, store, plans | Same botui cloud server, different proxy route |
| `chat.pragmatismo.com.br` | Caddy → 8080 | Bot API, WebSocket | Proxied directly to botserver |
| `login.pragmatismo.com.br` | Caddy → 5000 | Login, signup | Botui login server |

**Store page** (`/store` or `/cloud/store`): served by botui on port 4000, file `botui/ui/cloud/store.html`. The page loads products via `GET /api/catalog/products` (public endpoint). If the API is offline, `calc.js` uses local estimates (fallback).

> **Important:** Products are seeded by `branch_id`. The global catalog (SAAS) uses `branch_id = nil`. When a user signs up, a new branch is created and products are seeded for it — each organization sees its own product set.

---

## 🌐 Domain Management — Custom DNS → Bot Mapping

**Managed in the cloud manager UI** (`/domains` on port 4000, admin-only via super admin check). Associates hostnames like `chat.generalbots.org` with specific bots.

### How It Works

When a user visits `chat.generalbots.org` (proxied to port 3000), botui:
1. Reads the `Host` header from the HTTP request
2. Calls `GET /api/domains/resolve?host=chat.generalbots.org` (public, no auth) on the botserver
3. Botserver looks up the domain in the `bot_domains` table
4. If found, returns `{ found: true, bot_name: "gbwebsite", bot_id: "...", org_id: "...", branch_id: "..." }`
5. botui injects the resolved `bot_name` into `window.__INITIAL_BOT_NAME__` (same as URL path-based resolution)
6. If not found, falls back to URL path extraction (current behavior)

### Database

**Table:** `bot_domains` (migration `6.5.18-bot-domains`)

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID PK | Unique identifier |
| `domain` | VARCHAR(255) UNIQUE | The hostname (e.g. `chat.generalbots.org`) |
| `bot_id` | UUID FK → bots | Which bot this domain routes to |
| `org_id` | UUID FK → organizations (optional) | Org scope for multi-tenant |
| `branch_id` | UUID FK → branches (optional) | Branch scope for multi-tenant |
| `created_at` | TIMESTAMPTZ | Record creation time |
| `updated_at` | TIMESTAMPTZ | Last update time |

### API Endpoints (all on port 8080)

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `GET` | `/api/cloud/domains` | JWT + super admin | List all domain mappings |
| `POST` | `/api/cloud/domains` | JWT + super admin | Create domain mapping |
| `PUT` | `/api/cloud/domains/{id}` | JWT + super admin | Update domain mapping |
| `DELETE` | `/api/cloud/domains/{id}` | JWT + super admin | Delete domain mapping |
| `GET` | `/api/domains/resolve?host=` | **Public** (no auth) | Resolve hostname to bot name |

### Files Changed

| File | Change |
|------|--------|
| `botserver/migrations/6.5.18-bot-domains/up.sql` | New migration for `bot_domains` table |
| `botserver/crates/botcloud/src/schema_ext.rs` | Added Diesel table definition for `bot_domains` |
| `botserver/crates/botcloud/src/domains.rs` | **New** — CRUD handlers + resolve endpoint |
| `botserver/crates/botcloud/src/lib.rs` | Added `pub mod domains` |
| `botserver/crates/botcloud/src/api.rs` | Added routes + public access in JWT middleware |
| `botui/src/ui_server/suite.rs` | Added `resolve_bot_from_host()` + Host header check in `index()` |
| `botui/ui/cloud/domains.html` | **New** — Cloud UI page for managing domain mappings |

### Cloud UI Page

The **Domain Manager** is at `/domains` on port 4000. Only visible to super admins (same gating as Vouchers page). Features:
- **Create Mapping:** Form to enter domain, bot ID, optional org/branch IDs
- **List Mappings:** Table showing all domain → bot associations
- **Delete:** Remove a domain mapping

This is the **existing "Domains" nav link** in the sidebar at `/store/apps` — it's now live (previously placeholder).

### Adding a Domain Mapping

```bash
# Via API (requires JWT token from cloud login)
curl -X POST http://localhost:8080/api/cloud/domains \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "domain": "chat.generalbots.org",
    "bot_id": "<bot-uuid>",
    "org_id": null,
    "branch_id": null
  }'
```

### Testing Domain Resolution

```bash
# Direct API test (no auth required)
curl -s "http://localhost:8080/api/domains/resolve?host=chat.generalbots.org"

# Expected response when mapped:
# {"found":true,"bot_id":"...","bot_name":"gbwebsite","org_id":null,"branch_id":null}

# Expected response when NOT mapped:
# {"found":false}
```

### Architecture Flow

```
User's Browser
  │  GET http://chat.generalbots.org/
  │  Host: chat.generalbots.org
  ▼
Caddy/Proxy (port 80/443 → 3000)
  │
  ▼
botui suite.rs index() handler:
  1. Extract Host header → "chat.generalbots.org"
  2. GET http://localhost:8080/api/domains/resolve?host=chat.generalbots.org
  3. Response: bot_name = "gbwebsite"
  4. Inject window.__INITIAL_BOT_NAME__ = "gbwebsite"
  5. Serve desktop.html → browser loads chat for gbwebsite bot
```

---

## ☁️ Cloud Management Testing

### Ports
| Service | Port | Description |
|---------|-------|-----------|
| Cloud UI (botui) | **4000** | Dashboard, store, offers, plans pages (**without** login/signup) |
| Cloud API (botserver) | **8080** | `/api/cloud/auth/signup`, `/api/cloud/auth/login`, etc. |
| Login UI (botui) | **5000** | Login and registration pages (`/login`, `/signup`) — **only** service with auth |

### Plan Testing Flow (Free, Shared, Private Cloud)

Use Playwright connected via CDP to the existing Chrome on port 9222. Signup is done exclusively on port 5000 (`login.pragmatismo.com.br`), which redirects the forms to the cloud API (port 8080):

```python
from playwright.async_api import async_playwright

async def test_cloud_plans():
    async with async_playwright() as p:
        browser = await p.chromium.connect_over_cdp("http://localhost:9222")
        ctx = browser.contexts[0]  # use the default Chrome context

        plans = [
            ('free', 'http://localhost:5000/signup'),
            ('shared', 'http://localhost:5000/signup?plan=shared'),
            ('private-cloud', 'http://localhost:5000/signup?plan=private-cloud'),
        ]

        for plan_id, url in plans:
            page = await ctx.new_page()
            await page.goto(url, wait_until='networkidle')
            await page.fill('#signup-name', 'Test User')
            await page.fill('#signup-botname', f'Bot{suffix}')
            await page.fill('#signup-email', f'{plan_id}-{suffix}@example.com')
            await page.fill('#signup-password', 'Test1234!')
            await page.click('#signup-btn')
            # DO NOT close the page — keep it open for inspection
```

### Expected behavior per plan

| Plan | Redirect | Created subscription |
|-------|-----------------|-------------------|
| **free** | `/cloud/dashboard` | `billing_recurring` status=`active`, amount=`0.0` |
| **shared** | `/cloud/dashboard` | `billing_recurring` status=`trialing`, trial=14 days |
| **private-cloud** | `/cloud/store` | None (Custom/upon request) |

### Known bug (fixed)

**Symptom:** Signup with `free` or `shared` plans returns error `"Insert ... subscription: insert or update on table 'billing_recurring' violates foreign key constraint 'billing_recurring_org_id_fkey'"`

**Cause:** The `9.16-branch-id-isolation` migration changed the FK from `billing_recurring.org_id` to reference `branches(id)` instead of `organizations(org_id)`, but the `handle_signup` handler in `botserver/crates/botcloud/src/api.rs` was passing `org_id` (from the organization) instead of `branch_id`.

**Fix:** Replace `org_id` with `branch_id` in calls to `create_free_subscription` and `create_trial_subscription` in `handle_signup`.

---

## 🔧 Dev Dependencies (Hot Testing)

### Test start.bas Content
```bash
# start.bas must use double quotes (BASIC syntax)
cat /opt/gbo/data/default.gbai/default.gbdialog/start.bas
# Should show: TALK "Hello from default bot"
```

### Verify Tool Registration
```bash
grep -E "Registered tool|Compiled.*start" botserver.log
```
Expected: `Registered tool 'start' in database`

### Clean Reset from Scratch
```bash
killall -9 botserver vault postgres valkey minio qdrant zitadel 2>/dev/null
sleep 2
rm -rf botserver-stack/ botserver/botserver-stack/ .env botserver/.env botserver.log 2>/dev/null
BOTMODELS_HOST="http://localhost:8085" BOTMODELS_API_KEY="starter" RUST_LOG=info \
  nohup ./target/debug/botserver --noconsole > botserver.log 2>&1 &
```

---

---

## ➕ Adding New Features Workflow

### Step 1: Plan the Feature

**Understand requirements:**
1. What problem does this solve?
2. Which module owns this functionality? (Check [Module Responsibility Matrix](../README.md#-module-responsibility-matrix))
3. What data structures are needed?
4. What are the security implications?

**Design checklist:**
- [ ] Does it fit existing architecture patterns?
- [ ] Will it require database migrations?
- [ ] Does it need new API endpoints?
- [ ] Will it affect existing features?
- [ ] What are the error cases?

### Step 2: Implement the Feature

**Follow the pattern:**
```rust
// 1. Add types to botlib if shared across crates
// botlib/src/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeature {
    pub id: Uuid,
    pub name: String,
}

// 2. Add database schema if needed
// botserver/migrations/YYYY-MM-DD-HHMMSS_feature_name/up.sql
CREATE TABLE new_features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

// 3. Add Diesel model
// botserver/src/core/shared/models/core.rs
#[derive(Queryable, Insertable)]
#[diesel(table_name = new_features)]
pub struct NewFeatureDb {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

// 4. Add business logic
// botserver/src/features/new_feature.rs
pub async fn create_feature(
    state: &AppState,
    name: String,
) -> Result<NewFeature, Error> {
    // Implementation
}

// 5. Add API endpoint
// botserver/src/api/routes.rs
async fn create_feature_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateFeatureRequest>,
) -> Result<Json<NewFeature>, (StatusCode, String)> {
    // Handler implementation
}
```

**Security checklist:**
- [ ] Input validation (use `sanitize_identifier` for SQL)
- [ ] Authentication required?
- [ ] Authorization checks?
- [ ] Rate limiting needed?
- [ ] Error messages sanitized? (use `log_and_sanitize`)
- [ ] No `unwrap()` or `expect()` in production code

### Step 3: Add BASIC Keywords (if applicable)

**For features accessible from .bas scripts:**
```rust
// botserver/src/basic/keywords/new_feature.rs
pub fn new_feature_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine,
) {
    let state_clone = state.clone();
    let session_clone = user_session.clone();

    engine
        .register_custom_syntax(
            ["NEW_FEATURE", "$expr$"],
            true,
            move |context, inputs| {
                let param = context.eval_expression_tree(&inputs[0])?.to_string();
                
                // Call async function from sync context using separate thread
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all().build().ok();
                    let result = if let Some(rt) = rt {
                        rt.block_on(async {
                            create_feature(&state_clone, param).await
                        })
                    } else {
                        Err("Failed to create runtime".into())
                    };
                    let _ = tx.send(result);
                });
                let result = rx.recv().unwrap_or(Err("Channel error".into()));
                
                match result {
                    Ok(feature) => Ok(Dynamic::from(feature.name)),
                    Err(e) => Err(format!("Failed: {}", e).into()),
                }
            },
        )
        .expect("valid syntax registration");
}
```

### Step 4: Test the Feature

**Local testing:**
```bash
# 1. Run migrations
diesel migration run

# 2. Build and restart
./restart.sh

# 3. Test via API
curl -X POST http://localhost:8080/api/features \
  -H "Content-Type: application/json" \
  -d '{"name": "test"}'

# 4. Test via BASIC script
# Create test.bas in /opt/gbo/data/testbot.gbai/testbot.gbdialog/
# NEW_FEATURE "test"

# 5. Check logs
tail -f botserver.log | grep -i "new_feature"
```

**Integration test:**
```rust
// bottest/tests/new_feature_test.rs
#[tokio::test]
async fn test_create_feature() {
    let state = setup_test_state().await;
    let result = create_feature(&state, "test".to_string()).await;
    assert!(result.is_ok());
}
```

### Step 5: Document the Feature

**Update documentation:**
- Add to `botbook/src/features/` if user-facing
- Add to module README.md if developer-facing
- Add inline code comments for complex logic
- Update API documentation

**Example documentation:**
```markdown
## NEW_FEATURE Keyword

Creates a new feature with the given name.

**Syntax:**
```basic
NEW_FEATURE "feature_name"
```

**Example:**
```basic
NEW_FEATURE "My Feature"
TALK "Feature created!"
```

**Returns:** Feature name as string
```

### Step 6: Commit & Deploy

**Commit pattern:**
```bash
git add .
git commit -m "feat: Add NEW_FEATURE keyword

- Adds new_features table with migrations
- Implements create_feature business logic
- Adds NEW_FEATURE BASIC keyword
- Includes API endpoint at POST /api/features
- Tests: Unit tests for business logic, integration test for API"

git push alm main
git push origin main
```

---

## 🧪 Testing Strategy

### Unit Tests
- **Location**: Each crate has `tests/` directory or inline `#[cfg(test)]` modules
- **Naming**: Test functions use `test_` prefix or describe what they test
- **Running**: `cargo test -p <crate_name>` or `cargo test` for all

### Integration Tests
- **Location**: `bottest/` crate contains integration tests
- **Scope**: Tests full workflows across multiple crates
- **Running**: `cargo test -p bottest`

### Coverage Goals
- **Critical paths**: 80%+ coverage required
- **Error handling**: ALL error paths must have tests
- **Security**: All security guards must have tests

### WhatsApp Integration Testing

#### Prerequisites
1. **Enable WhatsApp Feature**: Build botserver with whatsapp feature enabled:
   ```bash
   cargo check -p botserver --features whatsapp
   ```
2. **Bot Configuration**: Ensure the bot has WhatsApp credentials configured in `config.csv`:
   - `whatsapp-api-key` - API key from Meta Business Suite
   - `whatsapp-verify-token` - Custom token for webhook verification
   - `whatsapp-phone-number-id` - Phone Number ID from Meta
   - `whatsapp-business-account-id` - Business Account ID from Meta

#### Using Localtunnel (lt) as Reverse Proxy

Check database for message storage:
```bash
psql -h localhost -U postgres -d botserver -c "SELECT * FROM messages WHERE bot_id = '<bot_id>' ORDER BY created_at DESC LIMIT 5;"
```

## 🐛 Debugging Rules

### 🚨 CRITICAL ERROR HANDLING RULE

**STOP EVERYTHING WHEN ERRORS APPEAR**

When ANY error appears in logs during startup or operation:
1. **IMMEDIATELY STOP** - Do not continue with other tasks
2. **IDENTIFY THE ERROR** - Read the full error message and context
3. **FIX THE ERROR** - Address the root cause, not symptoms
4. **VERIFY THE FIX** - Ensure error is completely resolved
5. **ONLY THEN CONTINUE** - Never ignore or work around errors

**NEVER restart servers to "fix" errors - FIX THE ACTUAL PROBLEM**

### Log Locations

| Component | Log File | What's Logged |
|-----------|----------|---------------|
| **botserver** | `botserver.log` | API requests, errors, script execution, **client navigation events** |
| **botui** | `botui.log` | UI rendering, WebSocket connections |
| **drive_monitor** | In botserver logs with `[drive_monitor]` prefix | File sync, compilation |
| **client errors** | In botserver logs with `CLIENT:` prefix | JavaScript errors, navigation events |

---

## 🔧 Bug Fixing Workflow

### Step 1: Reproduce & Diagnose

**Identify the symptom:**
```bash
# Check recent errors
grep -E " E | W " botserver.log | tail -20

# Check specific component
grep "component_name" botserver.log | tail -50

# Monitor live
tail -f botserver.log | grep -E "ERROR|WARN"
```

**Trace the data flow:**
1. Find where the bug manifests (UI, API, database, cache)
2. Work backwards through the call chain
3. Check logs at each layer

**Example: "Suggestions not showing"**
```bash
# 1. Check if frontend is requesting suggestions
grep "GET /api/suggestions" botserver.log | tail -5

# 2. Check if suggestions exist in cache
/opt/gbo/bin/botserver-stack/bin/cache/bin/valkey-cli --scan --pattern "suggestions:*"

# 3. Check if suggestions are being generated
grep "ADD_SUGGESTION" botserver.log | tail -10

# 4. Verify the Redis key format
grep "Adding suggestion to Redis key" botserver.log | tail -5
```

### Step 2: Find the Code

**Use code search tools:**
```bash
# Find function/keyword implementation
cd botserver/src && grep -r "ADD_SUGGESTION_TOOL" --include="*.rs"

# Find where Redis keys are constructed
grep -r "suggestions:" --include="*.rs" | grep format

# Find struct definition
grep -r "pub struct UserSession" --include="*.rs"
```

**Check module responsibility:**
- Refer to [Module Responsibility Matrix](../README.md#-module-responsibility-matrix)
- Check `mod.rs` files for module structure
- Look for related functions in same file

### Step 3: Fix the Bug

**Identify root cause:**
- Wrong variable used? (e.g., `user_id` instead of `bot_id`)
- Missing validation?
- Race condition?
- Configuration issue?

**Make minimal changes:**
```rust
// ❌ BAD: Rewrite entire function
fn add_suggestion(...) {
    // 100 lines of new code
}

// ✅ GOOD: Fix only the bug
fn add_suggestion(...) {
    // Change line 318:
    - let key = format!("suggestions:{}:{}", user_session.user_id, session_id);
    + let key = format!("suggestions:{}:{}", user_session.bot_id, session_id);
}
```

**Search for similar bugs:**
```bash
# If you fixed user_id -> bot_id in one place, check all occurrences
grep -n "user_session.user_id" botserver/src/basic/keywords/add_suggestion.rs
```

### Step 4: Test Locally

**Verify the fix:**
```bash
# 1. Build
cargo check -p botserver

# 2. Restart
./restart.sh

# 3. Test the specific feature
# - Open browser to http://localhost:3000/<botname>
# - Trigger the bug scenario
# - Verify it's fixed

# 4. Check logs for errors
tail -20 botserver.log | grep -E "ERROR|WARN"
```

### Step 5: Commit & Deploy

**Commit with clear message:**
```bash
cd botserver
git add src/path/to/file.rs
git commit -m "Fix: Use bot_id instead of user_id in suggestion keys

- Root cause: Wrong field used in Redis key format
- Impact: Suggestions stored under wrong key, frontend couldn't retrieve
- Files: src/basic/keywords/add_suggestion.rs (5 occurrences)
- Testing: Verified suggestions now appear in UI"
```

**Push to remotes:**
```bash
# Push submodule
git push alm main
git push origin main

# Update root repository
cd ..
git add botserver
git commit -m "Update botserver: Fix suggestion key bug"
git push alm main
git push origin main
```

**Production deployment:**
- ALM push triggers CI/CD pipeline
- Wait ~10 minutes for build + deploy
- Service auto-restarts on binary update
- Test in production after deployment

### Step 6: Document

**Add to AGENTS-PROD.md if production-relevant:**
- Common symptom
- Diagnosis commands
- Fix procedure
- Prevention tips

**Update code comments if needed:**
```rust
// Redis key format: suggestions:bot_id:session_id
// Note: Must use bot_id (not user_id) to match frontend queries
let key = format!("suggestions:{}:{}", user_session.bot_id, session_id);
```

---

## 🎨 Frontend Standards

### HTMX-First Approach
- Use HTMX to minimize JavaScript
- Server returns HTML fragments, not JSON
- Use `hx-get`, `hx-post`, `hx-target`, `hx-swap`
- WebSocket via htmx-ws extension

### Local Assets Only - NO CDN
```html
<!-- ✅ CORRECT -->
<script src="js/vendor/htmx.min.js"></script>

<!-- ❌ WRONG -->
<script src="https://unpkg.com/htmx.org@1.9.10"></script>
```

---

## 🚀 Performance & Size Standards

### Binary Size Optimization
- **Release Profile**: Always maintain `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`, `panic = "abort"`.
- **Dependencies**: 
  - Run `cargo tree --duplicates` weekly
  - Run `cargo machete` to remove unused dependencies
  - Use `default-features = false` and explicitly opt-in to needed features

### Linting & Code Quality
- **Clippy**: Code MUST pass `cargo clippy --workspace` with **0 warnings**.
- **No Allow**: NEVER use `#[allow(clippy::...)]` in source code - FIX the code instead.

---

## 🔧 Technical Debt

### Critical Issues to Address
- Error handling debt: instances of `unwrap()`/`expect()` in production code
- Performance debt: excessive `clone()`/`to_string()` calls
- File size debt: files exceeding 450 lines

### Weekly Maintenance Tasks
```bash
cargo tree --duplicates   # Find duplicate dependencies
cargo machete            # Remove unused dependencies
ls -lh target/debug/botserver  # Check binary size
cargo audit              # Security audit
```

---

## 📋 Continuation Prompt

When starting a new session or continuing work:

```
Continue on gb/ workspace. Follow AGENTS.md strictly:

1. Launch background checks: nohup cargo check -p <crate> > /tmp/<crate>_check.log 2>&1 &
2. While compiling: read files, analyze code, plan fixes
3. Poll results when convenient: tail /tmp/<crate>_check.log
4. Fix ALL warnings and errors - NO #[allow()] attributes
5. Delete unused code, don't suppress warnings
6. Remove unused parameters, don't prefix with _
7. Replace ALL unwrap()/expect() with proper error handling
8. Re-launch checks in background after fixes
9. Loop until 0 warnings, 0 errors
10. Refactor files >450 lines
```

---

## 🔑 Memory & Main Directives

**LOOP AND COMPACT UNTIL 0 WARNINGS - MAXIMUM PRECISION**

- 0 warnings
- 0 errors
- Trust project diagnostics
- Respect all rules
- No `#[allow()]` in source code
- Real code fixes only

**Remember:**
- **OFFLINE FIRST** - Fix all errors from list before compiling
- **BATCH BY FILE** - Fix ALL errors in a file at once
- **WRITE ONCE** - Single edit per file with all fixes
- **VERIFY LAST** - Only compile/diagnostics after ALL fixes
- **NEVER BLOCK** - Launch checks with `nohup &`, continue working while they compile
- **ALWAYS PARALLEL** - Run multiple crate checks simultaneously, never one-at-a-time
- **DELETE DEAD CODE** - Don't keep unused code around
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## Deploy in Prod Workflow

### CI/CD Pipeline (Primary Method)

1. **Push to ALM** — triggers CI/CD automatically:
   ```bash
   cd botserver
   git push alm main
   git push origin main
   cd ..
   git add botserver
   git commit -m "Update botserver: <description>"
   git push alm main
   git push origin main
   ```

2. **Wait for CI programmatically** — poll Forgejo API until build completes:
   ```bash
   # ALM is at http://<ALM_HOST>:4747 (port 4747, NOT 3000)
   # The runner is in container alm-ci, registered with token from DB
   
   # Method 1: Poll API for latest workflow run status
   ALM_URL="http://<ALM_HOST>:4747"
   REPO="GeneralBots/BotServer"
   MAX_WAIT=600  # 10 minutes
   ELAPSED=0
   
   while [ $ELAPSED -lt $MAX_WAIT ]; do
     STATUS=$(curl -sf "$ALM_URL/api/v1/repos/$REPO/actions/runs?per_page=1" | python3 -c "import sys,json; runs=json.load(sys.stdin); print(runs[0]['status'] if runs else 'unknown')")
     if [ "$STATUS" = "completed" ] || [ "$STATUS" = "failure" ] || [ "$STATUS" = "cancelled" ]; then
       echo "CI finished with status: $STATUS"
       break
     fi
     echo "CI status: $STATUS (waiting ${ELAPSED}s...)"
     sleep 15
     ELAPSED=$((ELAPSED + 15))
   done
   
   # Method 2: Check runner logs directly
   ssh <PROD_HOST> "sudo incus exec alm-ci -- tail -20 /opt/gbo/logs/forgejo-runner.log"
   
   # Method 3: Check binary timestamp after CI completes
   sleep 240
   ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 <PROD_HOST> \
     "sudo incus exec system -- stat -c '%y' /opt/gbo/bin/botserver"
   ```

3. **Restart in prod** — after binary updates:
   ```bash
   ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 <PROD_HOST> \
     "sudo incus exec system -- pkill -f botserver || true"
   sleep 2
   ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 <PROD_HOST> \
     "sudo incus exec system -- bash -c 'cd /opt/gbo/bin && RUST_LOG=info nohup ./botserver --noconsole > /opt/gbo/logs/stdout.log 2>&1 &'"
   ```

4. **Verify deployment**:
   ```bash
   # Wait for bootstrap (~2 min)
   sleep 120
   # Check health
   ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 <PROD_HOST> \
     "sudo incus exec system -- curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health"
   # Check logs
   ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 <PROD_HOST> \
     "sudo incus exec system -- tail -30 /opt/gbo/logs/stdout.log"
   ```

### Production Container Architecture

| Container | Service | Port | Notes |
|-----------|---------|------|-------|
| system | BotServer | 8080 | Main API server |
| vault | Vault | 8200 | Secrets management (isolated) |
| tables | PostgreSQL | 5432 | Database |
| cache | Valkey | 6379 | Cache |
| drive | MinIO | 9100 | Object storage |
| directory | Zitadel | 9000 | Identity provider |
| meet | LiveKit | 7880 | Video conferencing |
| vectordb | Qdrant | 6333 | Vector database |
| llm | llama.cpp | 8081 | Local LLM |
| email | Stalwart | 25/587 | Mail server |
| alm | Forgejo | 4747 | Git server (NOT 3000!) |
| alm-ci | Forgejo Runner | - | CI runner |
| proxy | Caddy | 80/443 | Reverse proxy |

**Important:** ALM (Forgejo) listens on port **4747**, not 3000. The runner token is stored in the `action_runner_token` table in the `PROD-ALM` database.

### CI Runner Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Runner not connecting | Wrong ALM port (3000 vs 4747) | Use port 4747 in runner registration |
| `registration file not found` | `.runner` file missing or wrong format | Re-register: `forgejo-runner register --instance http://<ALM_HOST>:4747 --token <TOKEN> --name gbo --labels ubuntu-latest:docker://node:20-bookworm --no-interactive` |
| `unsupported protocol scheme` | `.runner` file has wrong JSON format | Delete `.runner` and re-register |
| `connection refused` to ALM | iptables blocking or ALM not running | Check `sudo incus exec alm -- ss -tlnp \| grep 4747` |
| CI not picking up jobs | Runner not registered or labels mismatch | Check runner labels match workflow `runs-on` field |

---

## 🖥️ Production Operations Guide

### ⚠️ CRITICAL SAFETY RULES
1. **NEVER modify iptables rules without explicit confirmation** — always confirm the exact rules, source IPs, ports, and destinations before applying
2. **NEVER touch the PROD project without asking first** — no changes to production services, configs, or containers without user approval
3. **ALWAYS backup files to `/tmp` before editing** — e.g. `cp /path/to/file /tmp/$(basename /path/to/file).bak-$(date +%Y%m%d%H%M%S)`

### Infrastructure Overview
- **Host OS:** Ubuntu LTS
- **Container engine:** Incus (LXC-based)
- **Base path:** `/opt/gbo/` (General Bots Operations)
- **Data path:** `/opt/gbo/data` — shared data, configs, bot definitions
- **Bin path:** `/opt/gbo/bin` — compiled binaries
- **Conf path:** `/opt/gbo/conf` — service configurations
- **Log path:** `/opt/gbo/logs` — application logs

### Container Architecture

| Role | Service | Typical Port | Notes |
|------|---------|-------------|-------|
| **dns** | CoreDNS | 53 | DNS resolution, zone files in `/opt/gbo/data` |
| **proxy** | Caddy | 80/443 | Reverse proxy, TLS termination |
| **tables** | PostgreSQL | 5432 | Primary database |
| **email** | Stalwart | 993/465/587 | Mail server (IMAPS, SMTPS, Submission) |
| **system** | BotServer + Valkey | 8080/6379 | Main API + cache |
| **webmail** | Roundcube | behind proxy | PHP-FPM webmail frontend |
| **alm** | Forgejo | 4747 | Git/ALM server (NOT 3000!) |
| **alm-ci** | Forgejo Runner | - | CI/CD runner |
| **drive** | MinIO | 9000/9100 | Object storage |
| **table-editor** | NocoDB | behind proxy | Database UI, connects to tables |
| **vault** | Vault | 8200 | Secrets management |
| **directory** | Zitadel | 9000 | Identity provider |
| **meet** | LiveKit | 7880 | Video conferencing |
| **vectordb** | Qdrant | 6333 | Vector database |
| **llm** | llama.cpp | 8081 | Local LLM inference |

### Container Management

```bash
# List all containers
sudo incus list

# Start/Stop/Restart
sudo incus start <container>
sudo incus stop <container>
sudo incus restart <container>

# Exec into container
sudo incus exec <container> -- bash

# View container logs
sudo incus log <container>
sudo incus log <container> --show-log

# File operations
sudo incus file pull <container>/path/to/file /local/dest
sudo incus file push /local/src <container>/path/to/dest

# Create snapshot before changes
sudo incus snapshot create <container> pre-change-$(date +%Y%m%d%H%M%S)
```

### Service Management (inside container)

```bash
# Check if process is running
sudo incus exec <container> -- pgrep -a <process-name>

# Restart service (systemd)
sudo incus exec <container> -- systemctl restart <service>

# Follow logs
sudo incus exec <container> -- journalctl -u <service> -f

# Check listening ports
sudo incus exec <container> -- ss -tlnp
```

### Quick Health Check

```bash
# Check all containers status
sudo incus list --format csv

# Quick service check across containers
for c in dns proxy tables system email webmail alm alm-ci drive table-editor; do
  echo -n "$c: "
  sudo incus exec $c -- pgrep -a $(case $c in
    dns) echo "coredns";;
    proxy) echo "caddy";;
    tables) echo "postgres";;
    system) echo "botserver";;
    email) echo "stalwart";;
    webmail) echo "php-fpm";;
    alm) echo "forgejo";;
    alm-ci) echo "runner";;
    drive) echo "minio";;
    table-editor) echo "nocodb";;
  esac) >/dev/null && echo OK || echo FAIL
done
```

### Network & NAT

#### Port Forwarding Pattern
External ports on the host are DNAT'd to container IPs via iptables. NAT rules live in `/etc/iptables.rules`.

**Critical rule pattern** — always use the external interface (`-i <iface>`) to avoid loopback issues:
```
-A PREROUTING -i <external-iface> -p tcp --dport <port> -j DNAT --to-destination <container-ip>:<port>
```

#### Typical Port Map

| External | Service | Notes |
|----------|---------|-------|
| 53 | DNS | Public DNS resolution |
| 80/443 | HTTP/HTTPS | Via Caddy proxy |
| 5432 | PostgreSQL | Restricted access only |
| 993 | IMAPS | Secure email retrieval |
| 465 | SMTPS | Secure email sending |
| 587 | SMTP Submission | STARTTLS |
| 25 | SMTP | Often blocked by ISPs |
| 4747 | Forgejo | Behind proxy |
| 9000 | MinIO API | Internal only |
| 8200 | Vault | Isolated |

#### Network Diagnostics

```bash
# Check NAT rules
sudo iptables -t nat -L -n | grep DNAT

# Test connectivity from container
sudo incus exec <container> -- ping -c 3 8.8.8.8

# Test DNS resolution
sudo incus exec <container> -- dig <domain>

# Test port connectivity
nc -zv <container-ip> <port>
```

### Key Service Operations

#### DNS (CoreDNS)
- **Config:** `/opt/gbo/conf/Corefile`
- **Zones:** `/opt/gbo/data/<domain>.zone`
- **Test:** `dig @<dns-container-ip> <domain>`

#### Database (PostgreSQL)
- **Data:** `/opt/gbo/data`
- **Backup:** `pg_dump -U postgres -F c -f /tmp/backup.dump <dbname>`
- **Restore:** `pg_restore -U postgres -d <dbname> /tmp/backup.dump`

#### Email (Stalwart)
- **Config:** `/opt/gbo/conf/config.toml`
- **DKIM:** Check TXT records for `selector._domainkey.<domain>`
- **Webmail:** Behind proxy
- **Admin:** Accessible via configured admin port

**Recovery from crash:**
```bash
# Check if service starts with config validation
sudo incus exec email -- /opt/gbo/bin/stalwart -c /opt/gbo/conf/config.toml --help

# Check error logs
sudo incus exec email -- cat /opt/gbo/logs/stderr.log

# Restore from snapshot if config corrupted
sudo incus snapshot list email
sudo incus copy email/<snapshot> email-temp
sudo incus start email-temp
sudo incus file pull email-temp/opt/gbo/conf/config.toml /tmp/config.toml
sudo incus file push /tmp/config.toml email/opt/gbo/conf/config.toml
```

#### Proxy (Caddy)
- **Config:** `/opt/gbo/conf/config`
- **Backup before edit:** `cp /opt/gbo/conf/config /opt/gbo/conf/config.bak-$(date +%Y%m%d)`
- **Validate:** `caddy validate --config /opt/gbo/conf/config`
- **Reload:** `caddy reload --config /opt/gbo/conf/config`

#### Storage (MinIO / Drive)
- **Console:** Behind proxy
- **Internal API:** http://127.0.0.1:9100 (dev stack)
- **Data:** `/opt/gbo/data`
- **Bucket format:** `{botname}.gbai` — each bot is a MinIO bucket
- **Credentials:** Stored in Vault at `secret/gbo/drive`

**Getting Drive credentials from Vault:**
```bash
export VAULT_ADDR=https://localhost:8200
export VAULT_CACERT=${WORKSPACE}/botserver-stack/conf/system/certificates/ca/ca.crt

# Read token from /tmp (preferred) or VAULT_* from .env
export VAULT_TOKEN=$(cat /tmp/vault-token-gb 2>/dev/null || grep VAULT_TOKEN ${WORKSPACE}/botserver/.env | cut -d= -f2)

# Retrieve drive credentials
VAULT_BIN=${WORKSPACE}/botserver-stack/bin/vault/vault
DRIVE_ACCESSKEY=$($VAULT_BIN kv get -field=accesskey secret/gbo/drive)
DRIVE_SECRET=$($VAULT_BIN kv get -field=secret secret/gbo/drive)
DRIVE_PORT=$($VAULT_BIN kv get -field=port secret/gbo/drive)
```

**Using mc (MinIO Client) with Drive:**
```bash
# Install mc if not present (to /tmp, never repo root)
curl -sL https://dl.min.io/client/mc/release/linux-amd64/mc -o /tmp/mc && chmod +x /tmp/mc

# Configure mc alias (use credentials from Vault above)
/tmp/mc alias set local http://127.0.0.1:${DRIVE_PORT} ${DRIVE_ACCESSKEY} ${DRIVE_SECRET} --api s3v4

# List bot buckets
/tmp/mc ls local/

# List files in a bot bucket
/tmp/mc ls local/{botname}.gbai/

# Upload file to bot KB
/tmp/mc cp /path/to/file.xlsx local/{botname}.gbai/{botname}.gbkb/docs/

# Create a new bot bucket and upload files
/tmp/mc mb local/testbot.gbai
/tmp/mc cp start.bas local/testbot.gbai/testbot.gbdialog/start.bas

# Download a file from a bot bucket
/tmp/mc cp local/{botname}.gbai/{botname}.gbkb/docs/file.xlsx /tmp/
```

**Vault secret paths for all services:**
| Path | Contents |
|------|----------|
| `secret/gbo/drive` | accesskey, secret, host, port |
| `secret/gbo/tables` | Database credentials |
| `secret/gbo/cache` | Valkey/Redis credentials |
| `secret/gbo/directory` | Zitadel/identity credentials |
| `secret/gbo/llm` | LLM service credentials |
| `secret/gbo/vectordb` | Qdrant credentials |
| `secret/gbo/alm` | Forgejo/ALM credentials |
| `secret/gbo/email` | Stalwart/mail credentials |
| `secret/gbo/meet` | LiveKit credentials |
| `secret/gbo/encryption` | Encryption keys |

#### Bot System (system)
- **Service:** BotServer + Valkey (Redis-compatible)
- **Binary:** `/opt/gbo/bin/botserver`
- **Valkey:** port 6379

#### Git/ALM (Forgejo)
- **Port:** 4747 (NOT 3000!)
- **Behind proxy:** Access via configured hostname
- **CI Runner:** Separate container, registered with token from DB

#### CI/CD (Forgejo Runner)
- **Config:** `/opt/gbo/bin/config.yaml`
- **Init:** `/etc/systemd/system/alm-ci-runner.service` (runs as `gbuser`, NOT root)
- **Logs:** `/opt/gbo/logs/out.log`, `/opt/gbo/logs/err.log`
- **Auto-start:** Via systemd (enabled)
- **Runner user:** `gbuser` (uid 1000) — all `/opt/gbo/` files owned by `gbuser:gbuser`
- **sccache:** Installed at `/usr/local/bin/sccache`, configured via `RUSTC_WRAPPER=sccache` in workflow
- **Workspace:** `/opt/gbo/data/` (NOT `/opt/gbo/ci/`)
- **Cargo cache:** `/home/gbuser/.cargo/` (registry + git db)
- **Rustup:** `/home/gbuser/.rustup/`
- **SSH keys:** `/home/gbuser/.ssh/id_ed25519` (for deploy to system container)
- **Deploy mechanism:** CI builds binary → tar+gzip via SSH → `/opt/gbo/bin/botserver` on system container

### Backup & Recovery

#### Snapshot Recovery
```bash
# List snapshots
sudo incus snapshot list <container>

# Restore from snapshot
sudo incus copy <container>/<snapshot> <container>-restored
sudo incus start <container>-restored

# Get files from snapshot without starting
sudo incus file pull <container>/<snapshot>/path/to/file .
```

#### Backup Scripts
- Host config backup: `/opt/gbo/bin/backup-local-host.sh`
- Remote backup to S3: `/opt/gbo/bin/backup-remote.sh`

### Troubleshooting

#### Container Won't Start
```bash
# Check status
sudo incus list
sudo incus info <container>

# Check logs
sudo incus log <container> --show-log

# Try starting with verbose
sudo incus start <container> -v
```

#### Service Not Running
```bash
# Find process
sudo incus exec <container> -- pgrep -a <process>

# Check listening ports
sudo incus exec <container> -- ss -tlnp | grep <port>

# Check application logs
sudo incus exec <container> -- tail -50 /opt/gbo/logs/stderr.log
```

#### Email Delivery Issues
```bash
# Check mail server is running
sudo incus exec email -- pgrep -a stalwart

# Check IMAP/SMTP ports
nc -zv <email-ip> 993
nc -zv <email-ip> 465
nc -zv <email-ip> 587

# Check DKIM DNS records
dig TXT <selector>._domainkey.<domain>

# Check mail logs
sudo incus exec email -- tail -100 /opt/gbo/logs/email.log
```

### Maintenance

#### Update Container
```bash
# Stop container
sudo incus stop <container>

# Create snapshot backup
sudo incus snapshot create <container> pre-update-$(date +%Y%m%d)

# Update packages
sudo incus exec <container> -- apt update && apt upgrade -y

# Restart
sudo incus start <container>
```

#### Disk Space Management
```bash
# Check host disk usage
df -h /

# Check btrfs pool (if applicable)
sudo btrfs filesystem df /var/lib/incus

# Clean old logs in container
sudo incus exec <container> -- find /opt/gbo/logs -name "*.log.*" -mtime +7 -delete
```

### Container Tricks & Optimizations

#### Resource Limits
```bash
# Set CPU limit
sudo incus config set <container> limits.cpu 2

# Set memory limit
sudo incus config set <container> limits.memory 4GiB

# Set disk limit
sudo incus config device set <container> root size 20GiB
```

#### Profile Management
```bash
# List profiles
sudo incus profile list

# Apply profile to container
sudo incus profile add <container> <profile>

# Clone container for testing
sudo incus copy <source> <target> --ephemeral
```

#### Network Optimization
```bash
# Add static DHCP-like assignment
sudo incus config device add <container> eth0 nic nictype=bridged parent=<bridge>

# Set custom DNS for container
sudo incus config set <container> raw.lxc "lxc.net.0.ipv4.address=<ip>"
```

#### Quick Container Cloning for Testing
```bash
# Snapshot and clone for safe testing
sudo incus snapshot create <container> test-base
sudo incus copy <container>/test-base <container>-test
sudo incus start <container>-test
# ... test safely ...
sudo incus stop <container>-test
sudo incus delete <container>-test
```

---

## AutoTask & BASIC Keywords Reference

### AutoTask System Overview

AutoTask is an AI-driven task execution system that:

1. **Analyzes user intent** - "Send email to all customers", "Create weekly report"
2. **Plans execution steps** - Break down into actionable tasks
3. **Generates BASIC scripts** - Using available keywords to accomplish the task
4. **Executes scripts** - Run immediately or schedule for later

### File Locations

```
.gbdrive/
├── reports/           # Generated reports
├── documents/         # Created documents
├── exports/           # Data exports
└── apps/{appname}/    # HTMX apps (synced to SITES_ROOT)

.gbdialog/
├── schedulers/        # Scheduled jobs (cron-based)
├── tools/             # Voice/chat triggered tools
└── handlers/          # Event handlers
```

### Complete BASIC Keywords Reference

#### Data Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `GET` | `GET FROM {table} WHERE {condition}` | Query database records |
| `SET` | `SET {variable} = {value}` | Set variable value |
| `SAVE` | `SAVE {data} TO {table}` | Insert/update database record |
| `FIND` | `FIND {value} IN {table}` | Search for specific value |
| `FIRST` | `FIRST({array})` | Get first element |
| `LAST` | `LAST({array})` | Get last element |
| `FORMAT` | `FORMAT "{template}", var1, var2` | Format string with variables |

#### Communication

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SEND MAIL` | `SEND MAIL TO "{email}" WITH subject, body` | Send email |
| `SEND TEMPLATE` | `SEND TEMPLATE "{name}" TO "{email}"` | Send email template |
| `SEND SMS` | `SEND SMS TO "{phone}" MESSAGE "{text}"` | Send SMS |
| `TALK` | `TALK "{message}"` | Respond to user |
| `HEAR` | `HEAR "{phrase}" AS {variable}` | Listen for user input |

#### File Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE FILE` | `CREATE FILE "{path}" WITH {content}` | Create file in .gbdrive |
| `READ FILE` | `READ FILE "{path}"` | Read file content |
| `WRITE FILE` | `WRITE FILE "{path}" WITH {content}` | Write to file |
| `DELETE FILE` | `DELETE FILE "{path}"` | Delete file |
| `COPY FILE` | `COPY FILE "{source}" TO "{dest}"` | Copy file |
| `MOVE FILE` | `MOVE FILE "{source}" TO "{dest}"` | Move/rename file |
| `LIST FILES` | `LIST FILES "{path}"` | List directory contents |
| `UPLOAD` | `UPLOAD {data} TO "{path}"` | Upload file |
| `DOWNLOAD` | `DOWNLOAD "{url}" TO "{path}"` | Download file |

#### HTTP Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `GET HTTP` | `GET HTTP "{url}"` | HTTP GET request |
| `POST HTTP` | `POST HTTP "{url}" WITH {data}` | HTTP POST request |
| `PUT HTTP` | `PUT HTTP "{url}" WITH {data}` | HTTP PUT request |
| `DELETE HTTP` | `DELETE HTTP "{url}"` | HTTP DELETE request |
| `WEBHOOK` | `WEBHOOK "{url}" WITH {data}` | Send webhook |

#### AI/LLM Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `LLM` | `LLM "{prompt}"` | Call LLM with prompt |
| `USE KB` | `USE KB "{knowledge_base}"` | Use knowledge base for context |
| `CLEAR KB` | `CLEAR KB` | Clear knowledge base context |
| `USE TOOL` | `USE TOOL "{tool_name}"` | Enable external tool |
| `CLEAR TOOLS` | `CLEAR TOOLS` | Disable all tools |
| `USE WEBSITE` | `USE WEBSITE "{url}"` | Scrape website for context |

#### Task & Scheduling

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE_TASK` | `CREATE_TASK "{title}", "{assignee}", "{due}", {project}` | Create task |
| `WAIT` | `WAIT {seconds}` | Pause execution |
| `ON` | `ON "{event}" DO {action}` | Event handler |
| `ON EMAIL` | `ON EMAIL FROM "{filter}" DO {action}` | Email trigger |
| `ON CHANGE` | `ON CHANGE {table} DO {action}` | Database change trigger |

#### Bot & Memory

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SET BOT MEMORY` | `SET BOT MEMORY "{key}" = {value}` | Store bot-level data |
| `GET BOT MEMORY` | `GET BOT MEMORY "{key}"` | Retrieve bot-level data |
| `REMEMBER` | `REMEMBER "{key}" = {value}` | Store session data |
| `SET CONTEXT` | `SET CONTEXT "{key}" = {value}` | Set conversation context |
| `ADD_SUGGESTION` | `ADD_SUGGESTION "{text}"` | Add response suggestion |
| `CLEAR SUGGESTIONS` | `CLEAR SUGGESTIONS` | Clear suggestions |

#### User & Session

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `SET USER` | `SET USER "{property}" = {value}` | Update user property |
| `TRANSFER TO HUMAN` | `TRANSFER TO HUMAN` | Escalate to human agent |
| `ADD_MEMBER` | `ADD_MEMBER "{group}", "{email}", "{role}"` | Add user to group |

#### Documents & Content

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `CREATE DRAFT` | `CREATE DRAFT "{title}" WITH {content}` | Create document draft |
| `CREATE SITE` | `CREATE SITE "{name}" WITH {config}` | Create website |
| `SAVE FROM UNSTRUCTURED` | `SAVE FROM UNSTRUCTURED {data} TO {table}` | Parse and save data |

#### Multi-Bot Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `ADD BOT` | `ADD BOT "{name}" WITH TRIGGER "{phrase}"` | Add sub-bot |
| `REMOVE BOT` | `REMOVE BOT "{name}"` | Remove sub-bot |
| `LIST BOTS` | `LIST BOTS` | List active bots |
| `DELEGATE TO` | `DELEGATE TO "{bot}"` | Delegate to another bot |
| `SEND TO BOT` | `SEND TO BOT "{name}" MESSAGE "{msg}"` | Inter-bot message |
| `BROADCAST MESSAGE` | `BROADCAST MESSAGE "{msg}"` | Broadcast to all bots |

#### Social Media

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `POST TO SOCIAL` | `POST TO SOCIAL "{platform}" MESSAGE "{text}"` | Social media post |
| `GET SOCIAL FEED` | `GET SOCIAL FEED "{platform}"` | Get social feed |

#### Control Flow

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `IF/THEN/ELSE/END IF` | `IF condition THEN ... ELSE ... END IF` | Conditional |
| `FOR EACH/NEXT` | `FOR EACH item IN collection ... NEXT` | Loop |
| `SWITCH/CASE/END SWITCH` | `SWITCH var CASE val ... END SWITCH` | Switch statement |
| `PRINT` | `PRINT {value}` | Debug output |

#### Built-in Variables

| Variable | Description |
|----------|-------------|
| `TODAY` | Current date |
| `NOW` | Current datetime |
| `USER` | Current user object |
| `SESSION` | Current session object |
| `BOT` | Current bot object |

---

## 🔧 Common Bug Fixes

### IF/THEN/ELSE Panic (`dag.rs`)
- **Symptom:** Panic `IF/THEN/ELSE syntax: ParseError(BadInput(ImproperSymbol("$stmt$")))` during Rhai engine registration.
- **Cause:** Rhai 1.25.x does not support the `$stmt$` marker in `register_custom_syntax`. It was replaced by `$block$`.
- **Fix in `botserver/crates/botbasic_core/src/keywords/dag.rs`:**
  - `$stmt$` → `$block$` (3 occurrences: IF/THEN/ELSE, PARALLEL/AND, ON ERROR)
  - `.expect("...")` → `if let Err(e) = ... { log::error!("...") }` (without panic in production)
  - Requires `rt-multi-thread` feature from tokio in `botbasic_core/Cargo.toml`

### Embedding URL Hardcoded to Empty
- **Symptom:** `Embedding server connection failed for : builder error` with empty URL
- **Cause:** `botqdrant/src/embedding.rs:14` hardcoded `let embedding_url = "".to_string()` instead of using `self.llm_endpoint`
- **Fix:** Replace with `let embedding_url = &self.llm_endpoint;`

### Bot Access Denied (WebSocket)
- **Symptom:** `WS access denied for bot <name>: Access denied` or WebSocket closes with code 1006
- **Cause:** `bots.is_public = false` in the database
- **Fix:** `UPDATE bots SET is_public = true WHERE name = '<bot>';`

---

## ✅ SaaS Product Listing Test — Results (2026-06-28)

### Ports After Fix

| Port | Service | Cloud Access? | Suite Access? |
|-------|---------|---------------|---------------|
| **3000** | Suite (botui) | ❌ 404 | ✅ Yes |
| **4000** | Cloud (botui) | ✅ Yes | ❌ N/A |
| **5000** | Login (botui) | ✅ Login | ✅ Login |
| **8080** | API (botserver) | ✅ API | ✅ API proxy |

**Fix:** `botui/src/ui_server/suite.rs` — added `/cloud/` blocking in the `index` handler (port 3000 fallback). Cloud pages now return 404 on port 3000.

### Populated Products (17 via automatic seed)

| Category | Items | SKUs |
|-----------|-------|------|
| `plan` | 3 | free ($0), shared ($3.99), private-cloud (custom) |
| `infrastructure` | 8 | vps-small/medium/large, gpu-basic/advanced, storage-50/200/1000 |
| `communication` | 4 | number-local, number-tollfree, domain-com, domain-org |
| `llm-tokens` | 2 | llm-tokens-1m ($9.99), llm-tokens-10m ($79.99) |

### API Endpoints Verificados

| Endpoint | Formato | Status |
|----------|---------|--------|
| `GET /api/products/items` | JSON (17 items) | ✅ 200 |
| `GET /api/catalog/prices.json` | JSON-LD Schema.org (17 items) | ✅ 200 |
| `GET /api/products/categories` | JSON (7 categories) | ✅ 200 |

### Cloud Pages Verified (port 4000)

| Page | URL | Screenshot |
|--------|-----|-----------|
| Plans | `/cloud/plans` | `/tmp/saas_admin_plans.png` |
| Store | `/cloud/store` | `/tmp/saas_admin_store.png` |
| Signup | `/cloud/signup` | `/tmp/saas_admin_signup.png` |
| Dashboard | `/cloud/dashboard` | `/tmp/saas_admin_dashboard.png` |
| Offers | `/cloud/offers` | `/tmp/saas_admin_offers.png` |

### Evidence Slideshow
```bash
# Serve slideshow (port 9090)
python3 -m http.server 9090 --directory /tmp
# Open in Chrome
http://localhost:9090/slideshow.html
```
