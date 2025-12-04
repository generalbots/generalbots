# BotServer Development Prompt Guide

**Version:** 6.1.0  
**Purpose:** Consolidated LLM context for BotServer development

---

## Project Overview

BotServer is the core backend for General Bots - an open-source conversational AI platform built in Rust. It provides:

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
```

### Rust Patterns

```rust
// Use rand::rng() instead of rand::thread_rng()
let mut rng = rand::rng();

// Use diesel for database (NOT sqlx)
use diesel::prelude::*;

// All config from AppConfig - no hardcoded values
let url = config.drive.endpoint.clone();  // NOT "api.openai.com"

// Logging (all-in-one-line, unique messages)
info!("Processing request id={} user={}", req_id, user_id);
debug!("Cache hit for key={}", key);
trace!("Raw response bytes={}", bytes.len());
```

### BotServer Specifics

```
- Sessions MUST be retrieved by id when session_id is present
- Never suggest installing software - bootstrap/package_manager handles it
- Configuration stored in .gbot/config and database bot_configuration table
- Pay attention to shared::utils and shared::models for reuse
```

---

## Adding New Features

### Adding a Rhai Keyword

```rust
// 1. Define enum (Rust-only, NOT in database)
#[repr(i32)]
pub enum TriggerKind {
    Scheduled = 0,
    TableUpdate = 1,
    TableInsert = 2,
}

// 2. Register keyword with engine
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

// 3. Async execution with diesel
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
    pub status: i16,           // Use i16 for enum storage
    pub email: String,
    pub age: Option<i16>,      // Nullable fields
    pub metadata: Vec<u8>,     // Binary data
    pub created_at: DateTime<Utc>,
}
```

### Adding a Service/Endpoint

```rust
use axum::{routing::{get, post}, Router, Json, extract::State};

pub fn configure() -> Router<AppState> {
    Router::new()
        .route("/api/resource", get(list_handler))
        .route("/api/resource", post(create_handler))
}

async fn list_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Resource>> {
    let conn = state.conn.get().unwrap();
    let items = resources::table.load::<Resource>(&conn).unwrap();
    Json(items)
}

async fn create_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRequest>,
) -> Json<Resource> {
    let conn = state.conn.get().unwrap();
    let item = diesel::insert_into(resources::table)
        .values(&payload)
        .get_result(&conn)
        .unwrap();
    Json(item)
}
```

---

## LLM Workflow Strategy

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

### Final Steps Before Commit

```bash
# Remove warnings
cargo check 2>&1 | grep warning

# If many warnings, add #[allow(dead_code)] temporarily
# Then fix properly in dedicated pass

# Final validation
cargo build --release
cargo test
```

---

## Output Format

### Shell Script Format

When returning code changes, use this exact format:

```sh
#!/bin/bash

cat > src/module/file.rs << 'EOF'
use std::io;

pub fn my_function() -> Result<(), io::Error> {
    Ok(())
}
EOF

cat > src/another_file.rs << 'EOF'
pub fn another() {
    println!("Hello");
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

## Error Fixing Guide

When fixing Rust compiler errors:

1. **Respect Cargo.toml** - check dependencies, editions, features
2. **Type safety** - ensure all types match, trait bounds satisfied
3. **Ownership rules** - fix borrowing, ownership, lifetime issues
4. **Only return input files** - other files already exist

Common errors to check:
- Borrow of moved value
- Unused variable
- Use of moved value
- Missing trait implementations

---

## Documentation Style

When writing documentation:

- Be pragmatic and concise with examples
- Create both guide-like and API-like sections
- Use clear and consistent terminology
- Ensure consistency in formatting
- Follow logical flow
- Relate to BASIC keyword list where applicable

---

## SVG Diagram Guidelines

For technical diagrams:

```
- Transparent background
- Width: 1040-1400px, Height: appropriate for content
- Simple colored borders (no fill, stroke-width="2.6")
- Font: Arial, sans-serif
- Dual-theme support with CSS classes
- Colors: Blue #4A90E2, Orange #F5A623, Purple #BD10E0, Green #7ED321
- Rounded rectangles (rx="6.5")
- Font sizes: 29-32px titles, 22-24px labels, 18-21px descriptions
```

---

## IDE Integration Rules

```
- Return identifiers/characters in English language only
- Do not emit any comments, remove existing ones
- Compact code emission where possible
- Ensure cargo check cycle removes warnings
- Never use defaults or magic values
- Check borrow, clone, types - return 0 warning code!
```

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
| diesel | 2.1 | PostgreSQL ORM |
| tokio | 1.41 | Async runtime |
| rhai | git | BASIC scripting |
| reqwest | 0.12 | HTTP client |
| serde | 1.0 | Serialization |
| askama | 0.12 | Templates |

---

## Testing Commands

```bash
# Build
cargo build

# Check warnings
cargo check

# Run tests
cargo test

# Run with features
cargo run --features "console,llm,drive"

# Audit dependencies
cargo audit
```

---

## Remember

- **Sessions**: Always retrieve by ID when present
- **Config**: Never hardcode values, use AppConfig
- **Bootstrap**: Never suggest manual installation
- **Database**: Use diesel, not sqlx
- **Logging**: Unique messages, appropriate levels
- **Warnings**: Target zero warnings before commit