# Testing Architecture

## Overview

The General Bots testing framework is designed with a multi-layered, isolated approach to ensure comprehensive coverage from individual components to complete user workflows.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Test Execution Layer                     │
│  (GitHub Actions, CI/CD, Local Development)                │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
        ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌──────────┐
   │  Unit   │  │ Integr. │  │   E2E    │
   │ Tests   │  │ Tests   │  │  Tests   │
   └────┬────┘  └────┬────┘  └─────┬────┘
        │            │            │
        └────────────┼────────────┘
                     │
        ┌────────────▼────────────┐
        │   Test Harness Layer    │
        │ (Context, Utils, Mocks) │
        └────────────┬────────────┘
                     │
        ┌────────────┼────────────┐
        │            │            │
        ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌──────────┐
   │botserver│  │  Browser│  │ Services │
   │(Testing)│  │ (WebDrv)│  │(Mock/Iso)│
   └─────────┘  └─────────┘  └──────────┘
        │            │            │
        └────────────┼────────────┘
                     │
        ┌────────────▼────────────┐
        │  Temporary Stack Layer  │
        │ (Isolated Environments) │
        └────────────┬────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
   ┌─────────────┐          ┌──────────────┐
   │ PostgreSQL  │          │ Redis, MinIO │
   │ (Isolated)  │          │  (Isolated)  │
   └─────────────┘          └──────────────┘
```

## Test Layers

### 1. Unit Tests

**Purpose**: Test individual components in isolation

**Scope**:
- Single functions or methods
- Mocked external dependencies
- No database or external services

**Example**:
```rust
#[test]
fn test_message_formatting() {
    let msg = format_message("Hello");
    assert_eq!(msg, "Hello!");
}
```

**Location**: `bottest/tests/unit/`

### 2. Integration Tests

**Purpose**: Test multiple components working together

**Scope**:
- Multi-component interactions
- Real database connections
- Service integration
- Error handling across components

**Example**:
```rust
#[tokio::test]
async fn test_message_storage_and_retrieval() {
    let db = setup_test_db().await;
    let msg = Message::new("Hello");
    db.save(&msg).await.unwrap();
    let retrieved = db.get(msg.id).await.unwrap();
    assert_eq!(retrieved.text, "Hello");
}
```

**Location**: `bottest/tests/integration/`

### 3. End-to-End Tests

**Purpose**: Test complete user workflows

**Scope**:
- Complete user journeys
- Browser interactions
- Multi-phase workflows
- Real-world scenarios

**Phases**:
1. Platform Loading
2. botserver Initialization
3. User Authentication
4. Chat Interaction
5. Logout & Session Management

**Example**:
```rust
#[tokio::test]
async fn test_complete_platform_flow_login_chat_logout() {
    let ctx = E2ETestContext::setup_with_browser().await?;
    
    verify_platform_loading(&ctx).await?;
    verify_botserver_running(&ctx).await?;
    test_user_login(browser, &ctx).await?;
    test_chat_interaction(browser, &ctx).await?;
    test_user_logout(browser, &ctx).await?;
    
    ctx.close().await;
}
```

**Location**: `bottest/tests/e2e/`

## Test Harness

The test harness provides utilities for test setup and context management:

```
TestHarness
├── Setup utilities
│   ├── Create test database
│   ├── Start mock services
│   ├── Initialize configurations
│   └── Provision test data
├── Context management
│   ├── Resource tracking
│   ├── Cleanup coordination
│   └── Error handling
└── Helper functions
    ├── HTTP requests
    ├── Browser interactions
    └── Service mocking
```

### E2ETestContext

Provides complete environment for E2E testing:

```rust
pub struct E2ETestContext {
    pub ctx: TestContext,
    pub server: botserverInstance,
    pub browser: Option<Browser>,
}

impl E2ETestContext {
    pub async fn setup() -> Result<Self>
    pub async fn setup_with_browser() -> Result<Self>
    pub fn base_url(&self) -> &str
    pub fn has_browser(&self) -> bool
    pub async fn close(self)
}
```

## Temporary Stack Architecture

Isolated test environments for complete system integration:

```
/tmp/botserver-test-{timestamp}-{id}/
├── postgres/
│   ├── data/               ← PostgreSQL data files
│   ├── postgresql.log      ← Database logs
│   └── postgresql.conf     ← Configuration
├── redis/
│   ├── data/               ← Redis persistence
│   └── redis.log
├── minio/
│   ├── data/               ← S3-compatible storage
│   └── minio.log
├── botserver/
│   ├── config/
│   │   ├── config.toml     ← Application config
│   │   └── .env            ← Environment variables
│   ├── logs/
│   │   ├── botserver.log   ← Main application logs
│   │   ├── api.log         ← API logs
│   │   └── debug.log       ← Debug logs
│   ├── cache/              ← Local cache
│   └── state.json          ← Stack metadata
└── env.stack               ← Connection strings for tests
```

## Isolation Strategy

### Service Isolation

Each test gets dedicated service instances:

- **Database**: Separate PostgreSQL cluster on port 5433
- **Cache**: Separate Redis instance on port 6380
- **Storage**: Separate MinIO instance on port 9001
- **API**: Separate botserver on port 8000

### Network Isolation

- All services on localhost (127.0.0.1)
- Non-standard ports to avoid conflicts
- Docker containers for complete OS-level isolation

### Data Isolation

- Separate database schemas per test
- Temporary file systems for storage
- No shared configuration between tests
- Automatic cleanup on completion

## Test Execution Flow

```
1. Test Initialization
   ├─ Parse environment variables
   ├─ Check prerequisites (WebDriver, services)
   └─ Create test context

2. Stack Setup
   ├─ Create temporary directory
   ├─ Initialize databases
   ├─ Start services
   └─ Wait for readiness

3. Test Execution
   ├─ Setup phase
   ├─ Action phase
   ├─ Verification phase
   └─ Assertion phase

4. Cleanup
   ├─ Close browser connections
   ├─ Shutdown services gracefully
   ├─ Remove temporary directories
   └─ Report results
```

## Browser Automation

Uses WebDriver (Selenium) protocol for browser testing:

```
Test Code
    ↓
Reqwest HTTP Client
    ↓
WebDriver Protocol (JSON-RPC)
    ↓
chromedriver / Selenium Server
    ↓
Chrome/Chromium Browser
    ↓
Test Verification
```

### WebDriver Commands

- Navigate to URL
- Find elements by selector
- Click buttons and links
- Fill form inputs
- Wait for elements
- Execute JavaScript
- Take screenshots
- Get element text

## Error Handling

Comprehensive error handling at all levels:

```
Test Execution
    │
    ├─ Setup Error
    │  └─ Fail fast, preserve environment
    │
    ├─ Execution Error
    │  ├─ Log detailed context
    │  ├─ Capture screenshots
    │  └─ Optionally preserve stack
    │
    └─ Cleanup Error
       └─ Log warning, continue cleanup
```

## Performance Considerations

### Test Execution Times

- **Unit Tests**: ~0.1-1 second
- **Integration Tests**: ~1-10 seconds
- **E2E Tests**: ~30-60 seconds
- **Full Suite**: ~2-3 minutes

### Optimization Strategies

1. **Parallel Execution**: Run independent tests simultaneously
2. **Caching**: Reuse expensive resources
3. **Lazy Loading**: Initialize only needed components
4. **Release Mode**: Use `--release` for faster compilation
5. **Selective Testing**: Run only relevant tests during development

## CI/CD Integration

### GitHub Actions Workflow

```
Trigger (push/PR)
    ↓
Setup Environment
    ├─ Install Rust
    ├─ Start WebDriver
    └─ Setup test infrastructure
    ↓
Run Tests
    ├─ Unit tests
    ├─ Integration tests
    └─ E2E tests
    ↓
Collect Artifacts
    ├─ Test results
    ├─ Coverage reports
    ├─ Screenshots/logs
    └─ Performance metrics
    ↓
Report Results
    └─ Pass/fail status
```

## Best Practices

### 1. Test Organization

- Keep tests focused and single-purpose
- Use descriptive names
- Group related tests
- Organize by layer (unit/integration/e2e)

### 2. Test Design

- Make tests independent
- Use realistic data
- Test both happy and error paths
- Avoid test interdependencies

### 3. Test Maintenance

- Keep tests up to date with code
- Remove obsolete tests
- Refactor test helpers
- Monitor test execution time

### 4. Test Documentation

- Document complex test logic
- Explain test prerequisites
- Document setup/teardown
- Include troubleshooting tips

## Debugging

### Debug Helpers

- `RUST_LOG=debug` - Verbose logging
- `HEADED=1` - Show browser UI
- `--nocapture` - Print test output
- `--test-threads=1` - Run sequentially

### Debug Techniques

- Check server logs
- Review screenshots
- Inspect HTTP requests
- Step through code
- Use REPL for experimentation

## Future Enhancements

1. **Load Testing** - Concurrent user scenarios
2. **Visual Regression** - Screenshot comparison
3. **Accessibility Testing** - WCAG compliance
4. **Security Testing** - Vulnerability scanning
5. **Performance Profiling** - Memory and CPU analysis
6. **Multi-region** - Test across deployments
7. **Snapshot Testing** - Compare outputs over time

## References

- [End-to-End Testing Guide](./e2e-testing.md)
- [Test Harness API](./test-harness.md)
- [CI/CD Integration](./ci-cd.md)
- [Performance Benchmarking](./performance.md)