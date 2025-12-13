# botserver Development Prompt Guide

**Version:** 6.1.0  
**Purpose:** Consolidated LLM context for botserver development

---

## Version Management - CRITICAL

**Current version is 6.1.0 - DO NOT CHANGE without explicit approval!**

```bash
# Check current version
grep "^version" Cargo.toml
```

### Rules

1. **Version is 6.1.0 across ALL workspace crates**
2. **NEVER change version without explicit user approval**
3. **All migrations use 6.1.0_* prefix**
4. **Migration folder naming: `6.1.0_{feature_name}/`**

---

## Database Standards - CRITICAL

### TABLES AND INDEXES ONLY

**NEVER create in migrations:**
- ❌ Views (`CREATE VIEW`)
- ❌ Triggers (`CREATE TRIGGER`)
- ❌ Functions (`CREATE FUNCTION`)
- ❌ Stored Procedures

**ALWAYS use:**
- ✅ Tables (`CREATE TABLE IF NOT EXISTS`)
- ✅ Indexes (`CREATE INDEX IF NOT EXISTS`)
- ✅ Constraints (inline in table definitions)

### Why?
- Diesel ORM compatibility
- Simpler rollbacks
- Better portability
- Easier testing

### JSON Storage Pattern

Use TEXT columns with `_json` suffix instead of JSONB:
```sql
-- CORRECT
members_json TEXT DEFAULT '[]'

-- WRONG
members JSONB DEFAULT '[]'::jsonb
```

---

## Official Icons - MANDATORY

**NEVER generate icons with LLM. ALWAYS use official SVG icons from assets.**

Icons are stored in two locations (kept in sync):
- `botui/ui/suite/assets/icons/` - Runtime icons for UI
- `botbook/src/assets/icons/` - Documentation icons

### Available Icons

| Icon | File | Usage |
|------|------|-------|
| Logo | `gb-logo.svg` | Main GB branding |
| Bot | `gb-bot.svg` | Bot/assistant representation |
| Analytics | `gb-analytics.svg` | Charts, metrics, dashboards |
| Calendar | `gb-calendar.svg` | Scheduling, events |
| Chat | `gb-chat.svg` | Conversations, messaging |
| Compliance | `gb-compliance.svg` | Security, auditing |
| Designer | `gb-designer.svg` | Workflow automation |
| Drive | `gb-drive.svg` | File storage, documents |
| Mail | `gb-mail.svg` | Email functionality |
| Meet | `gb-meet.svg` | Video conferencing |
| Paper | `gb-paper.svg` | Document editing |
| Research | `gb-research.svg` | Search, investigation |
| Sources | `gb-sources.svg` | Knowledge bases |
| Tasks | `gb-tasks.svg` | Task management |

### Icon Guidelines

- All icons use `stroke="currentColor"` for CSS theming
- ViewBox: `0 0 24 24`
- Stroke width: `1.5`
- Rounded line caps and joins

**DO NOT:**
- Generate new icons with AI/LLM
- Use emoji or unicode symbols as icons
- Use external icon libraries
- Create inline SVG content

---

## Project Overview

botserver is the core backend for General Bots - an open-source conversational AI platform built in Rust. It provides:

- **Bootstrap System**: Auto-installs PostgreSQL, MinIO, Redis, LLM servers
- **Package Manager**: Manages bot deployments and service lifecycle
- **BASIC Interpreter**: Executes conversation scripts via Rhai
- **Multi-Channel Support**: Web, WhatsApp, Teams, Email
- **Knowledge Base**: Document ingestion with vector search

### Workspace Structure

```
botserver/     # Main server (this project)
botlib/        # Shared library - types, utilities, HTTP client
botui/         # Web/Desktop UI (Axum + Tauri)
botapp/        # Desktop app wrapper (Tauri)
botbook/       # Documentation (mdBook)
botmodels/     # Data models visualization
botplugin/     # Browser extension
```

---

## Database Migrations

### Creating New Migrations

```bash
# 1. Version is always 6.1.0
# 2. List existing migrations
ls -la migrations/

# 3. Create new migration folder
mkdir migrations/6.1.0_my_feature

# 4. Create up.sql and down.sql (TABLES AND INDEXES ONLY)
```

### Migration Structure

```
migrations/
├── 6.0.0_initial_schema/
├── 6.0.1_bot_memories/
├── ...
├── 6.1.0_enterprise_features/
│   ├── up.sql
│   └── down.sql
└── 6.1.0_next_feature/        # YOUR NEW MIGRATION
    ├── up.sql
    └── down.sql
```

### Migration Best Practices

- Use `IF NOT EXISTS` for all CREATE TABLE statements
- Use `IF EXISTS` for all DROP statements in down.sql
- Always create indexes for foreign keys
- **NO triggers** - handle updated_at in application code
- **NO views** - use queries in application code
- **NO functions** - use application logic
- Use TEXT with `_json` suffix for JSON data (not JSONB)

---

## LLM Workflow Strategy

### Two Types of LLM Work

1. **Execution Mode (Fazer)**
   - Pre-annotate phrases and send for execution
   - Focus on automation freedom
   - Less concerned with code details
   - Primary concern: Is the LLM destroying something?
   - Trust but verify output doesn't break existing functionality

2. **Review Mode (Conferir)**
   - Read generated code with full attention
   - Line-by-line verification
   - Check for correctness, security, performance
   - Validate against requirements

### Development Process

1. **One requirement at a time** with sequential commits
2. **Start with docs** - explain user behavior before coding
3. **Design first** - spend time on architecture
4. **On unresolved error** - stop and consult with web search enabled

### LLM Fallback Strategy (After 3 attempts / 10 minutes)

1. DeepSeek-V3-0324 (good architect, reliable)
2. gpt-5-chat (slower but thorough)
3. gpt-oss-120b (final validation)
4. Claude Web (for complex debugging, unit tests, UI)

---

## Code Generation Rules

### CRITICAL REQUIREMENTS

```
- KISS, NO TALK, SECURED ENTERPRISE GRADE THREAD SAFE CODE ONLY
- Use rustc 1.90.0 (1159e78c4 2025-09-14)
- No placeholders, never comment/uncomment code, no explanations
- All code must be complete, professional, production-ready
- REMOVE ALL COMMENTS FROM GENERATED CODE
- Always include full updated code files - never partial
- Only return files that have actual changes
- DO NOT WRITE ERROR HANDLING CODE - LET IT CRASH
- Return 0 warnings - review unused imports!
- NO DEAD CODE - implement real functionality, never use _ for unused
```

### Documentation Rules

```
- Rust code examples ONLY in docs/reference/architecture.md (gbapp chapter)
- All other docs: BASIC, bash, JSON, SQL, YAML only
- Scan for ALL_CAPS.md files created at wrong places - delete or integrate to docs/
- Keep only README.md and PROMPT.md at project root level
```

### Frontend Rules

```
- Use HTMX to minimize JavaScript - delegate logic to Rust server
- NO external CDN - all JS/CSS must be local in vendor/ folders
- Server-side rendering with Askama templates returning HTML
- Endpoints return HTML fragments, not JSON (for HTMX)
```

### Rust Patterns

```rust
// Use rand::rng() instead of rand::thread_rng()
let mut rng = rand::rng();

// Use diesel for database (NOT sqlx)
use diesel::prelude::*;

// All config from AppConfig - no hardcoded values
let url = config.drive.endpoint.clone();

// Logging (all-in-one-line, unique messages)
info!("Processing request id={} user={}", req_id, user_id);
```

### Dependency Management

```
- Use diesel - remove any sqlx references
- After adding to Cargo.toml: cargo audit must show 0 warnings
- If audit fails, find alternative library
- Minimize redundancy - check existing libs before adding new ones
- Review src/ to identify reusable patterns and libraries
```

### botserver Specifics

```
- Sessions MUST be retrieved by id when session_id is present
- Never suggest installing software - bootstrap/package_manager handles it
- Configuration stored in .gbot/config and database bot_configuration table
- Pay attention to shared::utils and shared::models for reuse
```

---

## Documentation Validation

### Chapter Validation Process

For each documentation chapter:

1. Read the chapter instructions step by step
2. Check if source code accomplishes each instruction
3. Verify paths exist and are correct
4. Ensure 100% alignment between docs and implementation
5. Fix either docs or code to match

### Documentation Structure

```
docs/
├── api/                    # API documentation (no Rust code)
├── guides/                 # How-to guides (no Rust code)
└── reference/
    ├── architecture.md     # ONLY place for Rust code examples
    ├── basic-language.md   # BASIC only
    └── configuration.md    # Config examples only
```

---

## Adding New Features

### Adding a Rhai Keyword

```rust
pub fn my_keyword(state: &AppState, engine: &mut Engine) {
    let db = state.db_custom.clone();
    
    engine.register_custom_syntax(
        ["MY", "KEYWORD", "$expr$"],
        true,
        {
            let db = db.clone();
            move |context, inputs| {
                let value = context.eval_expression_tree(&inputs[0])?;
                let binding = db.as_ref().unwrap();
                let fut = execute_my_keyword(binding, value);
                
                let result = tokio::task::block_in_place(||
                    tokio::runtime::Handle::current().block_on(fut))
                    .map_err(|e| format!("DB error: {}", e))?;
                    
                Ok(Dynamic::from(result))
            }
        }
    ).unwrap();
}

pub async fn execute_my_keyword(
    pool: &PgPool,
    value: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    info!("Executing my_keyword value={}", value);
    
    use diesel::prelude::*;
    let result = diesel::insert_into(my_table::table)
        .values(&NewRecord { value })
        .execute(pool)?;
        
    Ok(json!({ "rows_affected": result }))
}
```

### Adding a Data Model

```rust
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub status: i16,
    pub email: String,
    pub age: Option<i16>,
    pub metadata: Vec<u8>,
    pub created_at: DateTime<Utc>,
}
```

### Adding a Service/Endpoint (HTMX Pattern)

```rust
use axum::{routing::get, Router, extract::State, response::Html};
use askama::Template;

#[derive(Template)]
#[template(path = "partials/items.html")]
struct ItemsTemplate {
    items: Vec<Item>,
}

pub fn configure() -> Router<AppState> {
    Router::new()
        .route("/api/items", get(list_handler))
}

async fn list_handler(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let conn = state.conn.get().unwrap();
    let items = items::table.load::<Item>(&conn).unwrap();
    let template = ItemsTemplate { items };
    Html(template.render().unwrap())
}
```

---

## Final Steps Before Commit

```bash
# Check for warnings
cargo check 2>&1 | grep warning

# Audit dependencies (must be 0 warnings)
cargo audit

# Build release
cargo build --release

# Run tests
cargo test

# Verify no dead code with _ prefixes
grep -r "let _" src/ --include="*.rs"

# Verify version is 6.1.0
grep "^version" Cargo.toml | grep "6.1.0"

# Verify no views/triggers/functions in migrations
grep -r "CREATE VIEW\|CREATE TRIGGER\|CREATE FUNCTION" migrations/
```

### Pre-Commit Checklist

1. Version is 6.1.0 in all workspace Cargo.toml files
2. No views, triggers, or functions in migrations
3. All JSON columns use TEXT with `_json` suffix

---

## Output Format

### Shell Script Format

```sh
#!/bin/bash

cat > src/module/file.rs << 'EOF'
use std::io;

pub fn my_function() -> Result<(), io::Error> {
    Ok(())
}
EOF
```

### Rules

- Only return MODIFIED files
- Never return unchanged files
- Use `cat > path << 'EOF'` format
- Include complete file content
- No partial snippets

---

## Key Files Reference

```
src/main.rs              # Entry point, bootstrap, Axum server
src/lib.rs               # Module exports, feature gates
src/core/
  bootstrap/mod.rs       # Auto-install services
  session/mod.rs         # Session management
  bot/mod.rs             # Bot orchestration
  config/mod.rs          # Configuration management
  package_manager/       # Service lifecycle
src/basic/               # BASIC/Rhai interpreter
src/shared/
  state.rs               # AppState definition
  utils.rs               # Utility functions
  models.rs              # Database models
```

---

## Dependencies (Key Libraries)

| Library | Version | Purpose |
|---------|---------|---------|
| axum | 0.7.5 | Web framework |
| diesel | 2.1 | PostgreSQL ORM (NOT sqlx) |
| tokio | 1.41 | Async runtime |
| rhai | git | BASIC scripting |
| reqwest | 0.12 | HTTP client |
| serde | 1.0 | Serialization |
| askama | 0.12 | HTML Templates |

---

## Remember

- **Two LLM modes**: Execution (fazer) vs Review (conferir)
- **Rust code**: Only in architecture.md documentation
- **HTMX**: Minimize JS, delegate to server
- **Local assets**: No CDN, all vendor files local
- **Dead code**: Never use _ prefix, implement real code
- **cargo audit**: Must pass with 0 warnings
- **diesel**: No sqlx references
- **Sessions**: Always retrieve by ID when present
- **Config**: Never hardcode values, use AppConfig
- **Bootstrap**: Never suggest manual installation
- **Warnings**: Target zero warnings before commit
- **Version**: Always 6.1.0 - do not change without approval
- **Migrations**: TABLES AND INDEXES ONLY - no views, triggers, functions
- **Stalwart**: Use Stalwart IMAP/JMAP API for email features (sieve, filters, etc.)
- **JSON**: Use TEXT columns with `_json` suffix, not JSONB