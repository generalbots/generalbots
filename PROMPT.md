# botserver Development Prompt Guide

**Version:** 6.1.0  
**Purpose:** Consolidated LLM context for botserver development

---

## ZERO TOLERANCE POLICY

**This project has the strictest code quality requirements possible:**

```toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "warn"
```

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code to silence warnings
❌ NEVER use _ prefix for unused variables - DELETE the variable or USE it
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!() - handle all cases
❌ NEVER use todo!() or unimplemented!() - write real code
❌ NEVER leave unused imports - DELETE them
❌ NEVER leave dead code - DELETE it or IMPLEMENT it
❌ NEVER use approximate constants (3.14159) - use std::f64::consts::PI
❌ NEVER silence clippy in code - FIX THE CODE or configure in Cargo.toml
❌ NEVER add comments explaining what code does - code must be self-documenting
❌ NEVER use CDN links - all assets must be local
```

---

## CARGO.TOML LINT EXCEPTIONS

When a clippy lint has **technical false positives** that cannot be fixed in code,
disable it in `Cargo.toml` with a comment explaining why:

```toml
[lints.clippy]
# Disabled: has false positives for functions with mut self, heap types (Vec, String)
missing_const_for_fn = "allow"
# Disabled: Tauri commands require owned types (Window) that cannot be passed by reference
needless_pass_by_value = "allow"
# Disabled: transitive dependencies we cannot control
multiple_crate_versions = "allow"
```

**Approved exceptions:**
- `missing_const_for_fn` - false positives for `mut self`, heap types
- `needless_pass_by_value` - Tauri/framework requirements
- `multiple_crate_versions` - transitive dependencies
- `future_not_send` - when async traits require non-Send futures

---

## MANDATORY CODE PATTERNS

### Error Handling - Use `?` Operator

```rust
// ❌ WRONG
let value = something.unwrap();
let value = something.expect("msg");

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
```

### Option Handling - Use Combinators

```rust
// ❌ WRONG
if let Some(x) = opt {
    x
} else {
    default
}

// ✅ CORRECT
opt.unwrap_or(default)
opt.unwrap_or_else(|| compute_default())
opt.map_or(default, |x| transform(x))
```

### Match Arms - Must Be Different

```rust
// ❌ WRONG - identical arms
match x {
    A => do_thing(),
    B => do_thing(),
    C => other(),
}

// ✅ CORRECT - combine identical arms
match x {
    A | B => do_thing(),
    C => other(),
}
```

### Self Usage in Impl Blocks

```rust
// ❌ WRONG
impl MyStruct {
    fn new() -> MyStruct { MyStruct { } }
}

// ✅ CORRECT
impl MyStruct {
    fn new() -> Self { Self { } }
}
```

### Format Strings - Inline Variables

```rust
// ❌ WRONG
format!("Hello {}", name)
log::info!("Processing {}", id);

// ✅ CORRECT
format!("Hello {name}")
log::info!("Processing {id}");
```

### Display vs ToString

```rust
// ❌ WRONG
impl ToString for MyType {
    fn to_string(&self) -> String { }
}

// ✅ CORRECT
impl std::fmt::Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { }
}
```

### Derive Eq with PartialEq

```rust
// ❌ WRONG
#[derive(PartialEq)]
struct MyStruct { }

// ✅ CORRECT
#[derive(PartialEq, Eq)]
struct MyStruct { }
```

### Must Use Attributes

```rust
// ❌ WRONG - pure function without #[must_use]
pub fn calculate() -> i32 { }

// ✅ CORRECT
#[must_use]
pub fn calculate() -> i32 { }
```

### Const Functions

```rust
// ❌ WRONG - could be const but isn't
pub fn default_value() -> i32 { 42 }

// ✅ CORRECT
pub const fn default_value() -> i32 { 42 }
```

### Pass by Reference

```rust
// ❌ WRONG - needless pass by value
fn process(data: String) { println!("{data}"); }

// ✅ CORRECT
fn process(data: &str) { println!("{data}"); }
```

### Clone Only When Needed

```rust
// ❌ WRONG - redundant clone
let x = value.clone();
use_value(&x);

// ✅ CORRECT
use_value(&value);
```

### Mathematical Constants

```rust
// ❌ WRONG
let pi = 3.14159;
let e = 2.71828;

// ✅ CORRECT
use std::f64::consts::{PI, E};
let pi = PI;
let e = E;
```

### Async Functions

```rust
// ❌ WRONG - async without await
async fn process() { sync_operation(); }

// ✅ CORRECT - remove async if no await needed
fn process() { sync_operation(); }
```

---

## Build Rules

```bash
# Development - ALWAYS debug build
cargo build
cargo check

# NEVER use release unless deploying
# cargo build --release  # NO!
```

---

## Version Management

**Version is 6.1.0 - NEVER CHANGE without explicit approval**

---

## Database Standards

**TABLES AND INDEXES ONLY:**

```
✅ CREATE TABLE IF NOT EXISTS
✅ CREATE INDEX IF NOT EXISTS
✅ Inline constraints

❌ CREATE VIEW
❌ CREATE TRIGGER  
❌ CREATE FUNCTION
❌ Stored Procedures
```

**JSON Columns:** Use TEXT with `_json` suffix, not JSONB

---

## Code Generation Rules

```
- KISS, NO TALK, SECURED ENTERPRISE GRADE THREAD SAFE CODE ONLY
- Use rustc 1.90.0+
- No placeholders, no explanations, no comments
- All code must be complete, professional, production-ready
- REMOVE ALL COMMENTS FROM GENERATED CODE
- Always include full updated code files - never partial
- Only return files that have actual changes
- Return 0 warnings - FIX ALL CLIPPY WARNINGS
- NO DEAD CODE - implement real functionality
```

---

## Documentation Rules

```
- Rust code examples ONLY in docs/reference/architecture.md
- All other docs: BASIC, bash, JSON, SQL, YAML only
- Keep only README.md and PROMPT.md at project root level
```

---

## Frontend Rules

```
- Use HTMX to minimize JavaScript - delegate logic to Rust server
- NO external CDN - all JS/CSS must be local in vendor/ folders
- Server-side rendering with Askama templates returning HTML
- Endpoints return HTML fragments, not JSON (for HTMX)
```

---

## Rust Patterns

```rust
// Random number generation
let mut rng = rand::rng();

// Database - ONLY diesel, never sqlx
use diesel::prelude::*;

// Config from AppConfig - no hardcoded values
let url = config.drive.endpoint.clone();

// Logging - all-in-one-line, unique messages, inline vars
info!("Processing request id={id} user={user_id}");
```

---

## Dependencies

```
- Use diesel - remove any sqlx references
- After adding to Cargo.toml: cargo audit must show 0 vulnerabilities
- If audit fails, find alternative library
- Minimize redundancy - check existing libs before adding
```

---

## Key Files

```
src/main.rs              # Entry point
src/lib.rs               # Module exports
src/basic/               # BASIC language keywords
src/core/                # Core functionality
src/shared/state.rs      # AppState definition
src/shared/utils.rs      # Utility functions
src/shared/models.rs     # Database models
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
| askama | 0.12 | HTML Templates |

---

## Remember

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **CARGO.TOML EXCEPTIONS OK** - Disable lints with false positives in Cargo.toml with comment
- **NO DEAD CODE** - Delete unused code, never prefix with _
- **NO UNWRAP/EXPECT** - Use ? operator or proper error handling
- **NO APPROXIMATE CONSTANTS** - Use std::f64::consts
- **INLINE FORMAT ARGS** - format!("{name}") not format!("{}", name)
- **USE SELF** - In impl blocks, use Self not the type name
- **DERIVE EQ** - Always derive Eq with PartialEq
- **DISPLAY NOT TOSTRING** - Implement Display, not ToString
- **USE DIAGNOSTICS** - Use IDE diagnostics tool, never call cargo clippy directly
- **PASS BY REF** - Don't clone unnecessarily
- **CONST FN** - Make functions const when possible
- **MUST USE** - Add #[must_use] to pure functions
- **diesel**: No sqlx references
- **Sessions**: Always retrieve by ID when present
- **Config**: Never hardcode values, use AppConfig
- **Bootstrap**: Never suggest manual installation
- **Version**: Always 6.1.0 - do not change
- **Migrations**: TABLES AND INDEXES ONLY
- **JSON**: Use TEXT columns with `_json` suffix
- **Session Continuation**: When running out of context, create detailed summary: (1) what was done, (2) what remains, (3) specific files and line numbers, (4) exact next steps.

---

## Monitor Keywords (ON EMAIL, ON CHANGE)

### ON EMAIL

```basic
ON EMAIL "support@company.com"
    email = GET LAST "email_received_events"
    TALK "New email from " + email.from_address
END ON
```

### ON CHANGE

```basic
ON CHANGE "gdrive://myaccount/folder"
    files = GET LAST "folder_change_events"
    FOR EACH file IN files
        TALK "File changed: " + file.name
    NEXT
END ON
```

**TriggerKind Enum:**
- Scheduled = 0
- TableUpdate = 1
- TableInsert = 2
- TableDelete = 3
- Webhook = 4
- EmailReceived = 5
- FolderChange = 6