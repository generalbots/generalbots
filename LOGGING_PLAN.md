# BotServer Cinema Viewer Logging Plan

## 🎬 The Cinema Viewer Philosophy

Think of your logs as a movie with different viewing modes:

- **INFO** = **The Movie** - Watch the main story unfold (production-ready)
- **DEBUG** = **Director's Commentary** - Behind-the-scenes details (troubleshooting)
- **TRACE** = **Raw Footage** - Every single take and detail (deep debugging)
- **WARN** = **Plot Holes** - Things that shouldn't happen but are recoverable
- **ERROR** = **Scene Failures** - Critical issues that break the narrative

---

## 📊 Log Level Definitions

### ERROR - Critical Failures
**When to use:** System cannot proceed, requires immediate attention

```rust
// ✅ GOOD - Actionable, clear context
error!("Database connection failed - retrying in 5s: {}", e);
error!("Authentication failed for user {}: invalid credentials", user_id);
error!("Stage 2 BUILD failed: {}", e);

// ❌ BAD - Vague, no context
error!("Error!");
error!("Failed");
error!("Something went wrong: {}", e);
```

### WARN - Recoverable Issues
**When to use:** Unexpected state but system can continue

```rust
// ✅ GOOD - Explains the issue and impact
warn!("Failed to create data directory: {}. Using fallback.", e);
warn!("LLM server not ready - deferring embedding generation");
warn!("Rate limit approaching for API key {}", key_id);

// ❌ BAD - Not actionable
warn!("Warning");
warn!("Something happened");
```

### INFO - The Main Story
**When to use:** Key events, state changes, business milestones

```rust
// ✅ GOOD - Tells a story, shows progress
info!("Pipeline starting - task: {}, intent: {}", task_id, intent);
info!("Stage 1 PLAN complete - {} nodes planned", node_count);
info!("User {} logged in from {}", user_id, ip_address);
info!("Server started on port {}", port);

// ❌ BAD - Too verbose, implementation details
info!("Entering function process_request");
info!("Variable x = {}", x);
info!("Loop iteration {}", i);
```

### DEBUG - Behind the Scenes
**When to use:** Troubleshooting information, decision points, state inspections

```rust
// ✅ GOOD - Helps diagnose issues
debug!("Request payload: {:?}", payload);
debug!("Using cache key: {}", cache_key);
debug!("Retry attempt {} of {}", attempt, max_retries);
debug!("Selected LLM model: {} for task type: {}", model, task_type);

// ❌ BAD - Too trivial
debug!("Variable assigned");
debug!("Function called");
```

### TRACE - Raw Footage
**When to use:** Step-by-step execution, loop iterations, detailed flow

```rust
// ✅ GOOD - Detailed execution path
trace!("Starting monitoring loop");
trace!("Processing file: {:?}", path);
trace!("Checking bot directory: {}", dir);
trace!("WebSocket message received: {} bytes", len);

// ❌ BAD - Noise without value
trace!("Line 100");
trace!("Got here");
trace!("...");
```

---

## 🎭 Logging Patterns by Module Type

### 1. Orchestration & Pipeline Modules
**Examples:** `auto_task/orchestrator.rs`, `auto_task/agent_executor.rs`

**INFO Level:** Show the story arc
```rust
info!("Pipeline starting - task: {}, intent: {}", task_id, intent);
info!("Stage 1 PLAN starting - Agent #1 analyzing request");
info!("Stage 1 PLAN complete - {} nodes planned", node_count);
info!("Stage 2 BUILD complete - {} resources, url: {}", count, url);
info!("Pipeline complete - task: {}, nodes: {}, resources: {}", task_id, nodes, resources);
```

**DEBUG Level:** Show decision points
```rust
debug!("Classified intent as: {:?}", classification);
debug!("Selected app template: {}", template_name);
debug!("Skipping stage 3 - no resources to review");
```

**TRACE Level:** Show execution flow
```rust
trace!("Broadcasting thought to UI: {}", thought);
trace!("Updating agent {} status: {} -> {}", id, old, new);
trace!("Sub-task generated: {}", task_name);
```

### 2. File Monitoring & Compilation
**Examples:** `drive/local_file_monitor.rs`, `drive/drive_monitor/mod.rs`

**INFO Level:** Key file operations
```rust
info!("Local file monitor started - watching /opt/gbo/data/*.gbai");
info!("Compiling bot: {} ({} files)", bot_name, file_count);
info!("Bot {} compiled successfully - {} tools generated", bot_name, tool_count);
```

**DEBUG Level:** File processing details
```rust
debug!("Detected change in: {:?}", path);
debug!("Recompiling {} - modification detected", bot_name);
debug!("Skipping {} - no changes detected", bot_name);
```

**TRACE Level:** File system operations
```rust
trace!("Scanning directory: {:?}", dir);
trace!("File modified: {:?} at {:?}", path, time);
trace!("Watching directory: {:?}", path);
```

### 3. Security & Authentication
**Examples:** `security/jwt.rs`, `security/api_keys.rs`, `security/sql_guard.rs`

**INFO Level:** Security events (always log these!)
```rust
info!("User {} logged in from {}", user_id, ip);
info!("API key {} created for user {}", key_id, user_id);
info!("Failed login attempt for user {} from {}", username, ip);
info!("Rate limit exceeded for IP: {}", ip);
```

**DEBUG Level:** Security checks
```rust
debug!("Validating JWT for user {}", user_id);
debug!("Checking API key permissions: {:?}", permissions);
debug!("SQL query sanitized: {}", safe_query);
```

**TRACE Level:** Security internals
```rust
trace!("Token expiry check: {} seconds remaining", remaining);
trace!("Permission check: {} -> {}", resource, allowed);
trace!("Hashing password with cost factor: {}", cost);
```

### 4. API Handlers
**Examples:** HTTP endpoint handlers in `core/`, `drive/`, etc.

**INFO Level:** Request lifecycle
```rust
info!("Request started: {} {} from {}", method, path, ip);
info!("Request completed: {} {} -> {} ({}ms)", method, path, status, duration);
info!("User {} created resource: {}", user_id, resource_id);
```

**DEBUG Level:** Request details
```rust
debug!("Request headers: {:?}", headers);
debug!("Request body: {:?}", body);
debug!("Response payload: {} bytes", size);
```

**TRACE Level:** Request processing
```rust
trace!("Parsing JSON body");
trace!("Validating request parameters");
trace!("Serializing response");
```

### 5. Database Operations
**Examples:** `core/shared/models/`, Diesel queries

**INFO Level:** Database lifecycle
```rust
info!("Database connection pool initialized ({} connections)", pool_size);
info!("Migration completed - {} tables updated", count);
info!("Database backup created: {}", backup_path);
```

**DEBUG Level:** Query information
```rust
debug!("Executing query: {}", query);
debug!("Query returned {} rows in {}ms", count, duration);
debug!("Cache miss for key: {}", key);
```

**TRACE Level:** Query details
```rust
trace!("Preparing statement: {}", sql);
trace!("Binding parameter {}: {:?}", index, value);
trace!("Fetching next row");
```

### 6. LLM & AI Operations
**Examples:** `llm/`, `core/kb/`

**INFO Level:** LLM operations
```rust
info!("LLM request started - model: {}, tokens: {}", model, estimated_tokens);
info!("LLM response received - {} tokens, {}ms", tokens, duration);
info!("Embedding generated - {} dimensions", dimensions);
info!("Knowledge base indexed - {} documents", doc_count);
```

**DEBUG Level:** LLM details
```rust
debug!("LLM prompt: {}", prompt_preview);
debug!("Using temperature: {}, max_tokens: {}", temp, max);
debug!("Selected model variant: {}", variant);
```

**TRACE Level:** LLM internals
```rust
trace!("Sending request to LLM API: {}", url);
trace!("Streaming token: {}", token);
trace!("Parsing LLM response chunk");
```

### 7. Startup & Initialization
**Examples:** `main.rs`, `main_module/bootstrap.rs`

**INFO Level:** Startup milestones
```rust
info!("Server starting on port {}", port);
info!("Database initialized - PostgreSQL connected");
info!("Cache initialized - Valkey connected");
info!("Secrets loaded from Vault");
info!("BotServer ready - {} bots loaded", bot_count);
```

**DEBUG Level:** Configuration details
```rust
debug!("Using config: {:?}", config);
debug!("Environment: {}", env);
debug!("Feature flags: {:?}", features);
```

**TRACE Level:** Initialization steps
```rust
trace!("Loading .env file");
trace!("Setting up signal handlers");
trace!("Initializing thread registry");
```

---

## 🎯 The Cinema Viewer Experience

### Level 1: Watching the Movie (INFO)
```bash
RUST_LOG=botserver=info
```

**What you see:**
```
INFO botserver: Server starting on port 8080
INFO botserver: Database initialized - PostgreSQL connected
INFO botserver: User alice@example.com logged in from 192.168.1.100
INFO botserver::auto_task::orchestrator: Pipeline starting - task: abc123, intent: Create CRM
INFO botserver::auto_task::orchestrator: Stage 1 PLAN complete - 5 nodes planned
INFO botserver::auto_task::orchestrator: Stage 2 BUILD complete - 12 resources, url: /apps/crm
INFO botserver::auto_task::orchestrator: Pipeline complete - task: abc123, nodes: 5, resources: 12
INFO botserver: User alice@example.com logged out
```

**Perfect for:** Production monitoring, understanding system flow

### Level 2: Director's Commentary (DEBUG)
```bash
RUST_LOG=botserver=debug
```

**What you see:** Everything from INFO plus:
```
DEBUG botserver::auto_task::orchestrator: Classified intent as: AppGeneration
DEBUG botserver::auto_task::orchestrator: Selected app template: crm
DEBUG botserver::security::jwt: Validating JWT for user alice@example.com
DEBUG botserver::drive::local_file_monitor: Detected change in: /opt/gbo/data/crm.gbai
DEBUG botserver::llm: Using temperature: 0.7, max_tokens: 2000
```

**Perfect for:** Troubleshooting issues, understanding decisions

### Level 3: Raw Footage (TRACE)
```bash
RUST_LOG=botserver=trace
```

**What you see:** Everything from DEBUG plus:
```
TRACE botserver::drive::local_file_monitor: Scanning directory: /opt/gbo/data
TRACE botserver::auto_task::orchestrator: Broadcasting thought to UI: Analyzing...
TRACE botserver::llm: Streaming token: Create
TRACE botserver::llm: Streaming token: a
TRACE botserver::llm: Streaming token: CRM
TRACE botserver::core::db: Preparing statement: SELECT * FROM bots
```

**Perfect for:** Deep debugging, performance analysis, finding bugs

---

## ✨ Best Practices

### 1. Tell a Story
```rust
// ✅ GOOD - Shows the narrative
info!("Pipeline starting - task: {}", task_id);
info!("Stage 1 PLAN complete - {} nodes planned", nodes);
info!("Stage 2 BUILD complete - {} resources", resources);
info!("Pipeline complete - app deployed at {}", url);

// ❌ BAD - Just data points
info!("Task started");
info!("Nodes: {}", nodes);
info!("Resources: {}", resources);
info!("Done");
```

### 2. Use Structured Data
```rust
// ✅ GOOD - Easy to parse and filter
info!("User {} logged in from {}", user_id, ip);
info!("Request completed: {} {} -> {} ({}ms)", method, path, status, duration);

// ❌ BAD - Hard to parse
info!("User login happened");
info!("Request finished successfully");
```

### 3. Include Context
```rust
// ✅ GOOD - Provides context
error!("Database connection failed for bot {}: {}", bot_id, e);
warn!("Rate limit approaching for user {}: {}/{} requests", user_id, count, limit);

// ❌ BAD - No context
error!("Connection failed: {}", e);
warn!("Rate limit warning");
```

### 4. Use Appropriate Levels
```rust
// ✅ GOOD - Right level for right information
info!("Server started on port {}", port);          // Key event
debug!("Using config: {:?}", config);              // Troubleshooting
trace!("Listening on socket {:?}", socket);        // Deep detail

// ❌ BAD - Wrong levels
trace!("Server started");                           // Too important for trace
info!("Loop iteration {}", i);                      // Too verbose for info
error!("Variable is null");                         // Not an error
```

### 5. Avoid Noise
```rust
// ✅ GOOD - Meaningful information
debug!("Retry attempt {} of {} for API call", attempt, max);

// ❌ BAD - Just noise
debug!("Entering function");
debug!("Exiting function");
debug!("Variable assigned");
```

### 6. Log State Changes
```rust
// ✅ GOOD - Shows what changed
info!("User {} role changed: {} -> {}", user_id, old_role, new_role);
info!("Bot {} status: {} -> {}", bot_id, old_status, new_status);

// ❌ BAD - No before/after
info!("User role updated");
info!("Bot status changed");
```

### 7. Include Timings for Operations
```rust
// ✅ GOOD - Performance visibility
info!("Database migration completed in {}ms", duration);
info!("LLM response received - {} tokens, {}ms", tokens, duration);
debug!("Query executed in {}ms", duration);

// ❌ BAD - No performance data
info!("Migration completed");
info!("LLM response received");
```

---

## 🔧 Implementation Guide

### Step 1: Audit Current Logging
```bash
# Find all logging statements
find botserver/src -name "*.rs" -exec grep -n "info!\|debug!\|trace!\|warn!\|error!" {} +

# Count by level
grep -r "info!" botserver/src | wc -l
grep -r "debug!" botserver/src | wc -l
grep -r "trace!" botserver/src | wc -l
```

### Step 2: Categorize by Module
Create a spreadsheet or document listing:
- Module name
- Current log levels used
- Purpose of the module
- What story should it tell

### Step 3: Refactor Module by Module
Start with critical path modules:
1. **auto_task/orchestrator.rs** - Already done! ✅
2. **drive/local_file_monitor.rs** - File operations
3. **security/jwt.rs** - Authentication events
4. **main.rs** - Startup sequence
5. **core/bot/** - Bot lifecycle

### Step 4: Test Different Verbosity Levels
```bash
# Test INFO level (production)
RUST_LOG=botserver=info cargo run

# Test DEBUG level (troubleshooting)
RUST_LOG=botserver=debug cargo run

# Test TRACE level (development)
RUST_LOG=botserver=trace cargo run
```

### Step 5: Document Module-Specific Patterns
For each module, document:
- What story does it tell at INFO level?
- What troubleshooting info at DEBUG level?
- What raw details at TRACE level?

---

## 📋 Quick Reference Card

### Log Level Decision Tree

```
Is this a failure that stops execution?
  └─ YES → ERROR
  └─ NO → Is this unexpected but recoverable?
           └─ YES → WARN
           └─ NO → Is this a key business event?
                    └─ YES → INFO
                    └─ NO → Is this useful for troubleshooting?
                             └─ YES → DEBUG
                             └─ NO → Is this step-by-step execution detail?
                                      └─ YES → TRACE
                                      └─ NO → Don't log it!
```

### Module-Specific Cheat Sheet

| Module Type | INFO | DEBUG | TRACE |
|-------------|------|-------|-------|
| **Orchestration** | Stage start/complete, pipeline milestones | Decision points, classifications | UI broadcasts, state changes |
| **File Monitoring** | Monitor start, bot compiled | Changes detected, recompiles | File scans, timestamps |
| **Security** | Logins, key events, failures | Validations, permission checks | Token details, hash operations |
| **API Handlers** | Request start/end, resource changes | Headers, payloads | JSON parsing, serialization |
| **Database** | Connections, migrations | Queries, row counts | Statement prep, row fetching |
| **LLM** | Requests, responses, indexing | Prompts, parameters | Token streaming, chunking |
| **Startup** | Service ready, milestones | Config, environment | Init steps, signal handlers |

---

## 🎬 Example: Complete Pipeline Logging

Here's how a complete auto-task pipeline looks at different levels:

### INFO Level (The Movie)
```
INFO Pipeline starting - task: task-123, intent: Create a CRM system
INFO Stage 1 PLAN starting - Agent #1 analyzing request
INFO Stage 1 PLAN complete - 5 nodes planned
INFO Stage 2 BUILD starting - Agent #2 generating code
INFO Stage 2 BUILD complete - 12 resources, url: /apps/crm-system
INFO Stage 3 REVIEW starting - Agent #3 checking code quality
INFO Stage 3 REVIEW complete - all checks passed
INFO Stage 4 DEPLOY starting - Agent #4 deploying to /apps/crm-system
INFO Stage 4 DEPLOY complete - app live at /apps/crm-system
INFO Stage 5 MONITOR starting - Agent #1 setting up monitoring
INFO Stage 5 MONITOR complete - monitoring active
INFO Pipeline complete - task: task-123, nodes: 5, resources: 12, url: /apps/crm-system
```

### DEBUG Level (Director's Commentary)
```
INFO Pipeline starting - task: task-123, intent: Create a CRM system
DEBUG Classified intent as: AppGeneration
DEBUG Selected app template: crm_standard
INFO Stage 1 PLAN starting - Agent #1 analyzing request
DEBUG Generated 5 sub-tasks from intent
INFO Stage 1 PLAN complete - 5 nodes planned
INFO Stage 2 BUILD starting - Agent #2 generating code
DEBUG Using database schema: contacts, deals, activities
DEBUG Generated 3 tables, 8 pages, 1 tool
INFO Stage 2 BUILD complete - 12 resources, url: /apps/crm-system
...
```

### TRACE Level (Raw Footage)
```
INFO Pipeline starting - task: task-123, intent: Create a CRM system
DEBUG Classified intent as: AppGeneration
TRACE Extracting entities from: "Create a CRM system"
TRACE Found entity: CRM
TRACE Found entity: system
DEBUG Selected app template: crm_standard
INFO Stage 1 PLAN starting - Agent #1 analyzing request
TRACE Broadcasting thought to UI: Analyzing request...
TRACE Deriving plan sub-tasks
TRACE Sub-task 1: Create database schema
TRACE Sub-task 2: Generate list page
TRACE Sub-task 3: Generate form pages
TRACE Sub-task 4: Create BASIC tools
TRACE Sub-task 5: Setup navigation
DEBUG Generated 5 sub-tasks from intent
...
```

---

## 🎯 Goals & Metrics

### Success Criteria
1. **INFO logs tell a complete story** - Can understand system flow without DEBUG/TRACE
2. **DEBUG logs enable troubleshooting** - Can diagnose issues with context
3. **TRACE logs show execution details** - Can see step-by-step for deep debugging
4. **No log spam** - Production logs are concise and meaningful
5. **Consistent patterns** - Similar modules log similarly

### Metrics to Track
- Lines of logs per request at INFO level: < 20
- Lines of logs per request at DEBUG level: < 100
- Lines of logs per request at TRACE level: unlimited
- Error logs include context: 100%
- WARN logs explain impact: 100%

---

## 🚀 Next Steps

1. **Audit** current logging in all 341 files
2. **Prioritize** modules by criticality
3. **Refactor** module by module following this plan
4. **Test** at each log level
5. **Document** module-specific patterns
6. **Train** team on logging standards
7. **Monitor** log volume and usefulness
8. **Iterate** based on feedback

---

## 📚 References

- [Rust log crate documentation](https://docs.rs/log/)
- [env_logger documentation](https://docs.rs/env_logger/)
- [Structured logging best practices](https://www.honeycomb.io/blog/structured-logging/)
- [The Log: What every software engineer should know](https://blog.codinghorror.com/the-log-everything-manifesto/)

---

**Remember:** Good logging is like good cinematography - it should be invisible when done right, but tell a compelling story when you pay attention to it. 🎬