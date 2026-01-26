# botserver Development Guide

**Version:** 6.2.0  
**Purpose:** Main API server for General Bots (Axum + Diesel + Rhai BASIC + HTMX in botui)

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ❌ ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER use approximate constants - use std::f64::consts
❌ NEVER use CDN links - all assets must be local
❌ NEVER add comments - code must be self-documenting
❌ NEVER build SQL queries with format! - use parameterized queries
❌ NEVER pass user input to Command::new() without validation
❌ NEVER log passwords, tokens, API keys, or PII
```

---

## 🔐 SECURITY REQUIREMENTS

### Error Handling

```rust
// ❌ WRONG
let value = something.unwrap();
let value = something.expect("msg");

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
let value = something.unwrap_or_default();
```

### Rhai Syntax Registration

```rust
// ❌ WRONG
engine.register_custom_syntax([...], false, |...| {...}).unwrap();

// ✅ CORRECT
if let Err(e) = engine.register_custom_syntax([...], false, |...| {...}) {
    log::warn!("Failed to register syntax: {e}");
}
```

### Regex Patterns

```rust
// ❌ WRONG
let re = Regex::new(r"pattern").unwrap();

// ✅ CORRECT
static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pattern").expect("invalid regex")
});
```

### Tokio Runtime

```rust
// ❌ WRONG
let rt = tokio::runtime::Runtime::new().unwrap();

// ✅ CORRECT
let Ok(rt) = tokio::runtime::Runtime::new() else {
    return Err("Failed to create runtime".into());
};
```

### SQL Injection Prevention

```rust
// ❌ WRONG
let query = format!("SELECT * FROM {}", table_name);

// ✅ CORRECT - whitelist validation
const ALLOWED_TABLES: &[&str] = &["users", "sessions"];
if !ALLOWED_TABLES.contains(&table_name) {
    return Err(Error::InvalidTable);
}
```

### Command Injection Prevention

```rust
// ❌ WRONG
Command::new("tool").arg(user_input).output()?;

// ✅ CORRECT
fn validate_input(s: &str) -> Result<&str, Error> {
    if s.chars().all(|c| c.is_alphanumeric() || c == '.') {
        Ok(s)
    } else {
        Err(Error::InvalidInput)
    }
}
let safe = validate_input(user_input)?;
Command::new("/usr/bin/tool").arg(safe).output()?;
```

---

## ✅ CODE PATTERNS

### Format Strings - Inline Variables

```rust
// ❌ WRONG
format!("Hello {}", name)

// ✅ CORRECT
format!("Hello {name}")
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

### Derive Eq with PartialEq

```rust
// ❌ WRONG
#[derive(PartialEq)]
struct MyStruct { }

// ✅ CORRECT
#[derive(PartialEq, Eq)]
struct MyStruct { }
```

### Option Handling

```rust
// ✅ CORRECT
opt.unwrap_or(default)
opt.unwrap_or_else(|| compute_default())
opt.map_or(default, |x| transform(x))
```

### Chrono DateTime

```rust
// ❌ WRONG
date.with_hour(9).unwrap().with_minute(0).unwrap()

// ✅ CORRECT
date.with_hour(9).and_then(|d| d.with_minute(0)).unwrap_or(date)
```

---

## 📁 KEY DIRECTORIES

```
src/
├── core/           # Bootstrap, config, routes
├── basic/          # Rhai BASIC interpreter
│   └── keywords/   # BASIC keyword implementations
├── security/       # Security modules
├── shared/         # Shared types, models
├── tasks/          # AutoTask system
└── auto_task/      # App generator
```

---

## 🗄️ DATABASE STANDARDS

- **TABLES AND INDEXES ONLY** (no views, triggers, functions)
- **JSON columns:** use TEXT with `_json` suffix
- **ORM:** Use diesel - no sqlx
- **Migrations:** Located in `botserver/migrations/`

---

## 🎨 FRONTEND RULES

- **Use HTMX** - minimize JavaScript
- **NO external CDN** - all assets local
- **Server-side rendering** with Askama templates

---

## 📦 KEY DEPENDENCIES

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

## 🚀 CI/CD WORKFLOW

When configuring CI/CD pipelines (e.g., Forgejo Actions):

- **Minimal Checkout**: Clone only the root `gb` and the `botlib` submodule. Do NOT recursively clone everything.
- **BotServer Context**: Replace the empty `botserver` directory with the current set of files being tested.

**Example Step:**
```yaml
- name: Setup Workspace
  run: |
    # 1. Clone only the root workspace configuration
    git clone --depth 1 <your-git-repo-url> workspace
    
    # 2. Setup only the necessary dependencies (botlib)
    cd workspace
    git submodule update --init --depth 1 botlib
    cd ..

    # 3. Inject current BotServer code
    rm -rf workspace/botserver
    mv botserver workspace/botserver
```

---

## 🔑 REMEMBER

- **ZERO WARNINGS** - fix every clippy warning
- **ZERO COMMENTS** - no comments, no doc comments
- **NO ALLOW IN CODE** - configure exceptions in Cargo.toml only
- **NO DEAD CODE** - delete unused code
- **NO UNWRAP/EXPECT** - use ? or combinators
- **PARAMETERIZED SQL** - never format! for queries
- **VALIDATE COMMANDS** - never pass raw user input
- **INLINE FORMAT ARGS** - `format!("{name}")` not `format!("{}", name)`
- **USE SELF** - in impl blocks, use Self not type name
- **Version 6.2.0** - do not change without approval