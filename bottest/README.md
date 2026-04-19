# Bottest - General Bots Test Infrastructure

**Version:** 6.2.0  
**Purpose:** Test infrastructure for General Bots ecosystem

---

## Overview

Bottest provides the comprehensive testing infrastructure for the General Bots ecosystem, including unit tests, integration tests, and end-to-end (E2E) tests. It ensures code quality, reliability, and correct behavior across all components of the platform.

The test harness handles service orchestration, mock servers, fixtures, and browser automation, making it easy to write comprehensive tests that cover the entire system from database operations to full user flows.

For comprehensive documentation, see **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** or the **[BotBook](../botbook/src/17-testing)** for detailed guides and testing best practices.

---

## 🏗️ Testing Architecture

E2E tests use `USE_BOTSERVER_BOOTSTRAP=1` mode. The botserver handles all service installation during bootstrap.

```
TestHarness::full() / E2E Tests
    │
    ├── Allocate unique ports (15000+)
    ├── Create ./tmp/bottest-{uuid}/
    │
    ├── Start mock servers only
    │   ├── MockZitadel (wiremock)
    │   └── MockLLM (wiremock)
    │
    ├── Start botserver with --stack-path
    │   └── Botserver auto-installs:
    │       ├── PostgreSQL (tables)
    │       ├── MinIO (drive)
    │       └── Redis (cache)
    │
    └── Return TestContext
```

---

## 🧪 Test Categories

### Unit Tests (no services)

```rust
#[test]
fn test_pure_logic() {
    // No TestHarness needed
    assert_eq!(add(2, 3), 5);
}
```

### Integration Tests (with services)

```rust
#[tokio::test]
async fn test_with_database() {
    let ctx = TestHarness::quick().await?;
    let pool = ctx.db_pool().await?;
    
    // Use real database
    let user = fixtures::admin_user();
    ctx.insert(&user).await;
    
    // Test database operations
}
```

### E2E Tests (with browser)

```rust
#[tokio::test]
async fn test_user_flow() {
    let ctx = TestHarness::full().await?;
    let server = ctx.start_botserver().await?;
    let browser = Browser::new().await?;
    
    // Automate browser
    browser.goto(server.url()).await?;
    browser.click("#login-button").await?;
    
    // Verify user flow
    assert!(browser.is_visible("#dashboard").await?);
}
```

---

## 🎭 Mock Server Patterns

### Expect specific calls

```rust
ctx.mock_llm().expect_completion("hello", "Hi there!");
```

### Verify calls were made

```rust
ctx.mock_llm().assert_called_times(2);
```

### Simulate errors

```rust
ctx.mock_llm().next_call_fails(500, "Internal error");
```

### Mock authentication

```rust
ctx.mock_zitadel().expect_login_success("user@example.com", "password");
```

---

## 🏭 Fixture Patterns

### Factory functions

```rust
let user = fixtures::admin_user();
let bot = fixtures::bot_with_kb();
let session = fixtures::active_session(&user, &bot);
```

### Insert into database

```rust
ctx.insert(&user).await;
ctx.insert(&bot).await;
ctx.insert(&session).await;
```

### Custom fixtures

```rust
fn custom_bot() -> Bot {
    Bot {
        name: "Test Bot".to_string(),
        enabled: true,
        ..fixtures::base_bot()
    }
}
```

---

## ⚡ Parallel Safety

- Each test gets unique ports via PortAllocator
- Each test gets unique temp directory
- No shared state between tests
- Safe to run with `cargo test -j 8`

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

### Code Patterns

```rust
// ❌ WRONG
let value = something.unwrap();

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;

// Use Self in Impl Blocks
impl TestStruct {
    fn new() -> Self { Self { } }  // ✅ Not TestStruct
}

// Derive Eq with PartialEq
#[derive(PartialEq, Eq)]  // ✅ Always both
struct TestStruct { }

// Inline Format Args
format!("Hello {name}")  // ✅ Not format!("{}", name)
```

---

## 🚀 Running Tests

### Run all tests

```bash
cargo test -p bottest
```

### Run specific test category

```bash
# Unit tests only
cargo test -p bottest --lib

# Integration tests
cargo test -p bottest --test '*'

# E2E tests only
cargo test -p bottest --test '*' -- --ignored
```

### Run tests with output

```bash
cargo test -p bottest -- --nocapture
```

### Run tests in parallel

```bash
cargo test -p bottest -j 8
```

---

## 📁 Project Structure

```
bottest/
├── src/
│   ├── lib.rs              # Test harness exports
│   ├── harness.rs          # TestHarness implementation
│   ├── context.rs          # TestContext for resource access
│   ├── mocks/              # Mock server implementations
│   │   ├── zitadel.rs
│   │   └── llm.rs
│   ├── fixtures.rs         # Factory functions
│   └── utils.rs            # Testing utilities
├── tests/                  # Integration and E2E tests
│   ├── integration/
│   │   ├── database_tests.rs
│   │   └── api_tests.rs
│   └── e2e/
│       └── user_flows.rs
└── Cargo.toml
```

---

## 📚 Documentation

### Testing Documentation

All testing documentation is located in `botbook/src/17-testing/`:

- **README.md** - Testing overview and philosophy
- **e2e-testing.md** - E2E test guide with examples
- **architecture.md** - Testing architecture and design
- **best-practices.md** - Best practices and patterns
- **mock-servers.md** - Mock server configuration
- **fixtures.md** - Fixture usage and creation

### Additional Resources

- **[docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)** - Full online documentation
- **[BotBook](../botbook)** - Local comprehensive guide
- **[Testing Best Practices](../botbook/src/17-testing/best-practices.md)** - Detailed testing guidelines

---

## 🔗 Related Projects

| Project | Description |
|---------|-------------|
| [botserver](https://github.com/GeneralBots/botserver) | Main API server (tested) |
| [botui](https://github.com/GeneralBots/botui) | Web UI (E2E tested) |
| [botlib](https://github.com/GeneralBots/botlib) | Shared library |
| [botbook](https://github.com/GeneralBots/botbook) | Documentation |

---

## 🔑 Remember

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW ATTRIBUTES** - Never silence warnings
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **INLINE FORMAT ARGS** - `format!("{name}")` not `format!("{}", name)`
- **USE SELF** - In impl blocks, use Self not type name
- **Reuse bootstrap** - Don't duplicate botserver installation logic
- **Parallel safe** - Each test gets unique ports and directories
- **Version 6.2.0** - Do not change without approval
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)

---

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.