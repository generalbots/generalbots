# General Bots AI Agent Guidelines
- stop saving .png on root! Use /tmp. never allow new files on root.
- never push to alm without asking first - pbecause it is production!
- **❌ NEVER deploy to production manually — ALWAYS use CI/CD pipeline**
- **❌ NEVER include sensitive data (IPs, tokens, passwords, keys) in AGENTS.md or any documentation**
- **❌ NEVER use `scp`, direct SSH binary copy, or manual deployment to system container**
- **✅ ALWAYS push to ALM → CI builds on alm-ci → CI deploys to system container automatically**
8080 is server 3000 is client ui 
if you are in trouble with some tool, please go to the ofiical website to get proper install or instructions
To test web is http://localhost:3000 (botui!)
Use apenas a lingua culta ao falar .
test login here http://localhost:3000/suite/auth/login.html
> **⚠️ CRITICAL SECURITY WARNING**
I AM IN DEV ENV, but sometimes, pasting from PROD, do not treat my env as prod! Just fix, to me and push to CI. So I can test in PROD, for a while.
>Use Playwrigth MCP to start localhost:3000/<bot> now.
> **NEVER CREATE FILES WITH SECRETS IN THE REPOSITORY ROOT**
> - ❌ **NEVER** write internal IPs to logs or output
> - When debugging network issues, mask IPs (e.g., "10.x.x.x" instead of "10.16.164.222")
> - Use hostnames instead of IPs in configs and documentation
See botserver/src/drive/local_file_monitor.rs to see how to load from /opt/gbo/data the list of development bots.
- ❌ **NEVER** use `cargo clean` - causes 30min rebuilds, use `./reset.sh` for database issues

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

| Crate | Purpose | Port | Tech Stack |
|-------|---------|------|------------|
| **botserver** | Main API server, business logic | 8080 | Axum, Diesel, Rhai BASIC |
| **botui** | Web UI server (dev) + proxy | 3000 | Axum, HTML/HTMX/CSS |
| **botapp** | Desktop app wrapper | - | Tauri 2 |
| **botlib** | Shared library | - | Core types, errors |
| **botbook** | Documentation | - | mdBook |
| **bottest** | Integration tests | - | tokio-test |
| **botdevice** | IoT/Device support | - | Rust |
| **botplugin** | Browser extension | - | JS |

### Key Paths
- **Binary:** `target/debug/botserver`
- **Run from:** `botserver/` directory
- **Env file:** `botserver/.env`
- **UI Files:** `botui/ui/suite/`

---

## 🏗️ System Architecture Overview

### Chat Flow Architecture

```
User Message (WebSocket)
│
▼
┌─────────────────────────────────┐
│  1. WebSocket Connection        │  botserver/src/websocket.rs
│     - Session established       │  UserSession created
│     - Redis connection          │  session_id generated
└──────────────┬──────────────────┘
│
▼
┌─────────────────────────────────┐
│  2. start.bas Execution         │  /opt/gbo/data/{bot}.gbai/...
│     - Runs ONCE per session     │  {bot}.gbdialog/start.bas
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
ADD SUGGESTION "Check inventory"
ADD SUGGESTION "Create report"
ADD SUGGESTION "Send email"

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
ADD SUGGESTION "Check status"
ADD SUGGESTION "Create ticket"
ADD SUGGESTION "Contact support"

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
- ✅ BotUI listening on port 3000
- ✅ No errors in botserver.log

---

## 🔐 Security Directives - MANDATORY

### 1. Error Handling - NO PANICS IN PRODUCTION

```rust
// ❌ FORBIDDEN
value.unwrap()
value.expect("message")
panic!("error")
todo!()
unimplemented!()

// ✅ REQUIRED
value?
value.ok_or_else(|| Error::NotFound)?
value.unwrap_or_default()
value.unwrap_or_else(|e| { log::error!("{}", e); default })
if let Some(v) = value { ... }
match value { Ok(v) => v, Err(e) => return Err(e.into()) }
```

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
- ❌ **NEVER** compile directly for production - ALWAYS use push + CI/CD pipeline
- ❌ **NEVER** use `scp` or manual transfer to deploy - ONLY CI/CD ensures correct deployment
- ❌ **NEVER** manually copy binaries to production system container - ALWAYS push to ALM and let CI/CD build and deploy
- ❌ **NEVER** SSH into system container to deploy binaries - CI workflow handles build, transfer, and restart via alm-ci SSH
- ✅ **ALWAYS** push code to ALM → CI builds on alm-ci → CI deploys to system container via SSH from alm-ci
- ✅ **CI deploy path**: alm-ci builds at `/opt/gbo/data/botserver/target/debug/botserver` → tar+gzip via SSH → `/opt/gbo/bin/botserver` on system container → restart
- ❌ **NEVER** manually copy binaries to production system container - ALWAYS push to ALM and let CI/CD build and deploy
- ❌ **NEVER** SSH into system container to deploy binaries - CI workflow handles build, transfer, and restart via alm-ci SSH
- ✅ **ALWAYS** push code to ALM → CI builds on alm-ci → CI deploys to system container via SSH from alm-ci
- ✅ **CI deploy path**: alm-ci builds at `/opt/gbo/data/botserver/target/debug/botserver` → tar+gzip via SSH → `/opt/gbo/bin/botserver` on system container → restart

**Current Status:** ✅ **0 clippy warnings** (down from 61 - PERFECT SCORE in YOLO mode)
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
2. **Create subdirectory module** - e.g., `handlers/`
3. **Split by responsibility:**
   - `types.rs` - Structs, enums, type definitions
   - `handlers.rs` - HTTP handlers and routes
   - `operations.rs` - Core business logic
   - `utils.rs` - Helper functions
   - `mod.rs` - Re-exports and configuration
4. **Keep files focused** - Single responsibility
5. **Update mod.rs** - Re-export all public items

**NEVER let a single file exceed 450 lines - split proactively at 350 lines**

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

---

## 🧠 Memory Management

When compilation fails due to memory issues (process "Killed"):

```bash
pkill -9 cargo; pkill -9 rustc; pkill -9 botserver
CARGO_BUILD_JOBS=1 cargo check -p botserver 2>&1 | tail -200
```

---

---

## 🧪 Testing Staging Environment (STAGE-GBO)

To test `chat.stage.pragmatismo.com.br` or other services in the STAGE-GBO environment:
- Use the `10.0.3.x` subnet for container IPs (e.g., `10.0.3.10` for the system container).
- Route testing via the host gateway at `10.0.0.1` or directly hit container IPs inside the staging host.
- Do NOT confuse staging IP ranges (`10.0.3.x`) with production ranges.

---

## 🎭 Playwright Browser Testing - YOLO Mode

**When user requests to start YOLO mode with Playwright:**

1. **Start the browser** - Use `mcp__playwright__browser_navigate` to open http://localhost:3000/{botname}
2. **Take snapshot** - Use `mcp__playwright__browser_snapshot` to see current page state
3. **Test user flows** - Use click, type, fill_form, etc.
4. **Verify results** - Check for expected content, errors in console, network requests
5. **Validate backend** - Check database and services to confirm process completion
6. **Report findings** - Always include screenshot evidence with `browser_take_screenshot`

**⚠️ IMPORTANT - Desktop UI Navigation:**
- The desktop may have a maximized chat window covering other apps
- To access CRM/sidebar icons, click the **middle button** (restore/down arrow) in the chat window header to minimize it
- Or navigate directly via URL: http://localhost:3000/suite/crm (after login)

**Bot-Specific Testing URL Pattern:**
`http://localhost:3000/<botname>`

**Backend Validation Checks:**
After UI interactions, validate backend state via `psql` or `tail` logs.

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
curl -X POST http://localhost:9000/api/features \
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
   cargo build -p botserver --bin botserver --features whatsapp
   ```
2. **Bot Configuration**: Ensure the bot has WhatsApp credentials configured in `config.csv`:
   - `whatsapp-api-key` - API key from Meta Business Suite
   - `whatsapp-verify-token` - Custom token for webhook verification
   - `whatsapp-phone-number-id` - Phone Number ID from Meta
   - `whatsapp-business-account-id` - Business Account ID from Meta

#### Using Localtunnel (lt) as Reverse Proxy

# Check database for message storage
psql -h localhost -U postgres -d botserver -c "SELECT * FROM messages WHERE bot_id = '<bot_id>' ORDER BY created_at DESC LIMIT 5;"
---

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
cargo build --release && ls -lh target/release/botserver  # Check binary size
cargo audit              # Security audit
```

---

## 📋 Continuation Prompt

When starting a new session or continuing work:

```
Continue on gb/ workspace. Follow AGENTS.md strictly:

1. Check current state with build/diagnostics
2. Fix ALL warnings and errors - NO #[allow()] attributes
3. Delete unused code, don't suppress warnings
4. Remove unused parameters, don't prefix with _
5. Replace ALL unwrap()/expect() with proper error handling
6. Verify after each fix batch
7. Loop until 0 warnings, 0 errors
8. Refactor files >450 lines
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

#### Storage (MinIO)
- **Console:** Behind proxy
- **Internal API:** http://<drive-ip>:9000
- **Data:** `/opt/gbo/data`

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
| `ADD SUGGESTION` | `ADD SUGGESTION "{text}"` | Add response suggestion |
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
