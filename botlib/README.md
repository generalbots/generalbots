# BotLib - General Bots Shared Library

**Version:** 6.2.0  
**Purpose:** Shared library for General Bots workspace

---

## Overview

BotLib is the foundational shared library for the General Bots workspace, providing common types, error handling, HTTP client functionality, and utilities used across all projects. It serves as the core dependency for botserver, botui, botapp, and other workspace members, ensuring consistency and reducing code duplication.

For comprehensive documentation, see **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** or the **[BotBook](../botbook)** for detailed guides and API references.

---

## 🏗️ Module Structure

```
src/
├── lib.rs           # Public exports, feature gates
├── error.rs         # Error types (thiserror)
├── models.rs        # Shared data models
├── message_types.rs # Message type definitions
├── http_client.rs   # HTTP client wrapper (feature-gated)
├── branding.rs      # Version, branding constants
└── version.rs       # Version information
```

---

## ✅ ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

### Absolute Prohibitions

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use _ prefix for unused variables - DELETE or USE them
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
```

---

## 📦 Key Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| anyhow | 1.0 | Error handling |
| thiserror | 2.0 | Error derive |
| chrono | 0.4 | Date/time |
| serde | 1.0 | Serialization |
| uuid | 1.11 | UUIDs |
| diesel | 2.1 | Database ORM |
| reqwest | 0.12 | HTTP client |

---

## 🔧 Features

### Feature Gates

BotLib uses Cargo features to enable optional functionality:

```toml
[features]
default = []
http-client = ["reqwest"]  # Enable HTTP client
# Add more features as needed
```

### Using Features

```toml
# In dependent crate's Cargo.toml
[dependencies.botlib]
workspace = true
features = ["http-client"]  # Enable HTTP client
```

---

## ✅ Mandatory Code Patterns

### Error Handling

```rust
// ❌ WRONG
let value = something.unwrap();

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
```

### Self Usage

```rust
impl MyStruct {
    fn new() -> Self { Self { } }  // ✅ Not MyStruct
}
```

### Format Strings

```rust
format!("Hello {name}")  // ✅ Not format!("{}", name)
```

### Display vs ToString

```rust
// ❌ WRONG
impl ToString for MyType { }

// ✅ CORRECT
impl std::fmt::Display for MyType { }
```

### Derive Eq with PartialEq

```rust
#[derive(PartialEq, Eq)]  // ✅ Always both
struct MyStruct { }
```

---

## 📚 Documentation

For complete documentation, guides, and API references:

- **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** - Full online documentation
- **[BotBook](../botbook)** - Local comprehensive guide with tutorials and examples
- **[General Bots Repository](https://github.com/GeneralBots/BotServer)** - Main project repository

---

## 🔗 Related Projects

- **[botserver](https://github.com/GeneralBots/botserver)** - Main API server
- **[botui](https://github.com/GeneralBots/botui)** - Web UI interface
- **[botapp](https://github.com/GeneralBots/botapp)** - Desktop application
- **[botbook](https://github.com/GeneralBots/botbook)** - Documentation

---

## 🔑 Remember

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **INLINE FORMAT ARGS** - `format!("{name}")` not `format!("{}", name)`
- **USE SELF** - In impl blocks, use Self not the type name
- **DERIVE EQ** - Always derive Eq with PartialEq
- **DISPLAY NOT TOSTRING** - Implement Display, not ToString
- **Version 6.2.0** - Do not change without approval
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.