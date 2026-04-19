# End-to-End Testing

End-to-end (E2E) testing validates complete user workflows from platform loading through authentication, interaction, and logout.

## Overview

E2E tests simulate real user interactions:

1. **Platform Loading** - UI and API infrastructure operational
2. **botserver Initialization** - Backend service running and ready
3. **User Authentication** - Login workflow functional
4. **Chat Interaction** - Message sending and receiving
5. **Logout** - Session management and access control

## Complete Platform Flow Test

The main E2E test validates the entire user journey:

```rust
#[tokio::test]
async fn test_complete_platform_flow_login_chat_logout() {
    // Setup
    let ctx = E2ETestContext::setup_with_browser().await?;
    let browser = ctx.browser.as_ref().unwrap();

    // Phase 1: Platform Loading
    verify_platform_loading(&ctx).await?;

    // Phase 2: botserver Running
    verify_botserver_running(&ctx).await?;

    // Phase 3: User Login
    test_user_login(browser, &ctx).await?;

    // Phase 4: Chat Interaction
    test_chat_interaction(browser, &ctx).await?;

    // Phase 5: Logout
    test_user_logout(browser, &ctx).await?;

    ctx.close().await;
}
```

## Test Phases

### Phase 1: Platform Loading

Verifies UI and API infrastructure:

```rust
verify_platform_loading(&ctx).await?
```

Checks:
- Health endpoint responds with 2xx status
- API endpoints are accessible
- Database migrations completed
- Services are initialized

### Phase 2: botserver Initialization

Verifies the backend service is operational:

```rust
verify_botserver_running(&ctx).await?
```

Checks:
- Process is alive and responding
- Configuration properly loaded
- Dependencies connected (DB, cache, storage)
- Health checks pass

### Phase 3: User Authentication

Tests the login workflow:

```rust
test_user_login(browser, &ctx).await?
```

Tests:
- Navigate to login page
- Form elements present and functional
- Accept valid test credentials (test@example.com / TestPassword123!)
- Create session and authentication token
- Redirect to dashboard/chat interface

### Phase 4: Chat Interaction

Tests messaging functionality:

```rust
test_chat_interaction(browser, &ctx).await?
```

Tests:
- Chat interface loads correctly
- User can type and send messages
- Bot responds with valid output
- Message history persists
- Multiple exchanges work correctly

### Phase 5: Logout & Session Management

Tests secure session handling:

```rust
test_user_logout(browser, &ctx).await?
```

Tests:
- Logout button/action works
- Session is invalidated
- User redirected to login page
- Protected routes block unauthenticated access
- Cannot access chat after logout

## Running E2E Tests

### HTTP-Only Tests (No Browser Required)

These tests verify API and infrastructure without browser automation:

```bash
cd gb/bottest

# Platform loading verification
cargo test --test e2e test_platform_loading_http_only -- --nocapture

# botserver startup verification
cargo test --test e2e test_botserver_startup -- --nocapture
```

Execution time: ~2-5 seconds

### Complete Flow Tests (Requires WebDriver)

Full browser-based tests with user interactions:

```bash
# Start WebDriver first
chromedriver --port=4444 &

# Run complete platform flow
cargo test --test e2e test_complete_platform_flow_login_chat_logout -- --nocapture

# Run simplified flow
cargo test --test e2e test_login_and_chat_flow -- --nocapture
```

Execution time: ~30-60 seconds

## WebDriver Setup

### Option 1: Local Installation

```bash
# Download chromedriver from https://chromedriver.chromium.org/
# Place in PATH, then start:
chromedriver --port=4444
```

### Option 2: Docker

```bash
docker run -d -p 4444:4444 selenium/standalone-chrome
```

### Option 3: Docker Compose

```bash
docker-compose up -d webdriver
```

## Environment Variables

Control test behavior:

| Variable | Default | Purpose |
|----------|---------|---------|
| `HEADED` | unset | Show browser window instead of headless |
| `WEBDRIVER_URL` | `http://localhost:4444` | WebDriver server endpoint |
| `SKIP_E2E_TESTS` | unset | Skip E2E tests if set |
| `RUST_LOG` | info | Logging level: debug, info, warn, error |
| `KEEP_TEMP_STACK_ON_ERROR` | unset | Preserve temp directory on failure |

### Examples

```bash
# Show browser UI for debugging
HEADED=1 cargo test --test e2e -- --nocapture

# Use custom WebDriver
WEBDRIVER_URL=http://localhost:4445 cargo test --test e2e -- --nocapture

# Verbose logging
RUST_LOG=debug cargo test --test e2e -- --nocapture

# Run single-threaded with output
cargo test --test e2e -- --nocapture --test-threads=1
```

## Test Helpers

Reusable helper functions for custom tests:

```rust
// Verify platform is operational
verify_platform_loading(&ctx) -> Result<()>

// Verify botserver is running
verify_botserver_running(&ctx) -> Result<()>

// Perform login with credentials
test_user_login(browser, &ctx) -> Result<()>

// Send message and wait for response
test_chat_interaction(browser, &ctx) -> Result<()>

// Logout and verify session invalidation
test_user_logout(browser, &ctx) -> Result<()>
```

## Test Context

Setup a test context for E2E testing:

```rust
use bottest::prelude::*;
use bottest::web::{Browser, BrowserConfig};

// HTTP-only context
let ctx = E2ETestContext::setup().await?;

// With browser automation
let ctx = E2ETestContext::setup_with_browser().await?;
let browser = ctx.browser.as_ref().unwrap();

// Access base URL
let url = ctx.base_url();

// Access running server
let is_running = ctx.server.is_running();

// Cleanup
ctx.close().await;
```

## Common Issues

### WebDriver Not Available

**Problem**: Test fails with "WebDriver not available"

**Solution**:
```bash
# Start WebDriver
chromedriver --port=4444
# or
docker run -d -p 4444:4444 selenium/standalone-chrome
```

### Port Already in Use

**Problem**: Services fail to start due to port conflicts

**Solution**:
```bash
# Kill existing services
pkill -f chromedriver
pkill -f botserver
pkill -f postgres
pkill -f redis-server
```

### Test Hangs or Timeout

**Problem**: Test appears to hang or timeout

**Solution**:
```bash
# Run with timeout and verbose output
timeout 120s RUST_LOG=debug cargo test --test e2e test_name -- --nocapture --test-threads=1
```

### Browser Connection Issues

**Problem**: Browser fails to connect to WebDriver

**Solution**:
```bash
# Use different WebDriver port
WEBDRIVER_URL=http://localhost:4445 cargo test --test e2e -- --nocapture
```

## Debugging

### View Test Output
```bash
# Show all output
cargo test --test e2e test_name -- --nocapture

# With timestamps
RUST_LOG=debug cargo test --test e2e test_name -- --nocapture

# Save to file
cargo test --test e2e test_name -- --nocapture 2>&1 | tee test.log
```

### Watch Browser in Action
```bash
# Run with visible browser
HEADED=1 cargo test --test e2e test_name -- --nocapture --test-threads=1
```

### Check Server Logs
```bash
# Monitor logs while tests run
tail -f /tmp/bottest-*/botserver.log

# In another terminal:
cargo test --test e2e test_name -- --nocapture
```

## Performance

Typical execution times:

| Test | Time | Resources |
|------|------|-----------|
| Platform loading (HTTP-only) | ~2s | Minimal |
| botserver startup (HTTP-only) | ~5s | Minimal |
| Login and chat flow | ~20s | Browser + Memory |
| Complete flow with all phases | ~45s | Browser + Memory |
| Full E2E test suite | ~2-3 min | High |

Use release mode for faster execution:
```bash
cargo test --test e2e --release -- --nocapture
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests
on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    services:
      chromedriver:
        image: selenium/standalone-chrome
        options: --shm-size=2gb
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd gb/bottest && cargo test --test e2e -- --nocapture
```

## Temporary Stack Architecture (Future)

When botserver implements `--temp-stack`, E2E tests will run in isolated environments:

```bash
botserver --temp-stack
# Creates: /tmp/botserver-test-{timestamp}-{random}/
# With isolated: PostgreSQL, Redis, MinIO, Mock LLM
# Auto-cleanup after test completion
```

Benefits:
- ✓ Isolation - Each test in separate environment
- ✓ Reproducibility - Consistent setup every time
- ✓ Automation - No manual configuration
- ✓ Safety - Won't interfere with development
- ✓ Cleanup - Automatic resource management
- ✓ Parallel - Multiple tests simultaneously
- ✓ CI/CD Ready - Perfect for automated pipelines

## Writing Custom E2E Tests

Create new test files in `gb/bottest/tests/e2e/`:

```rust
#[tokio::test]
async fn test_my_feature() {
    // Setup context
    let ctx = E2ETestContext::setup_with_browser().await?;
    let browser = ctx.browser.as_ref().unwrap();

    // Navigate to feature
    browser.navigate(&format!("{}/my-feature", ctx.base_url())).await?;

    // Interact with UI
    browser.click("button.action").await?;
    browser.wait_for_element(".result", Duration::from_secs(10)).await?;

    // Verify results
    let text = browser.get_text(".result").await?;
    assert_eq!(text, "Expected result");

    // Cleanup
    ctx.close().await;
}
```

Register in `tests/e2e/mod.rs`:

```rust
mod my_feature;
```

## Best Practices

1. **Keep tests focused** - Test one user workflow per test
2. **Use meaningful names** - `test_complete_platform_flow` not `test_1`
3. **Explicit waits** - Use `wait_for_element` instead of `sleep`
4. **Test realistic flows** - Use actual test credentials
5. **Verify results explicitly** - Check status codes, UI elements, and state
6. **Clean up properly** - Always call `ctx.close().await`
7. **Handle errors gracefully** - Use `?` operator for error propagation
8. **Make tests independent** - Don't rely on test execution order

## Test Success Criteria

✓ Platform fully loads without errors
✓ botserver starts and becomes ready
✓ User can login with credentials
✓ Chat messages are sent and responses received
✓ User can logout and session is invalidated
✓ Protected routes block unauthenticated access
✓ Tests run consistently multiple times
✓ Tests complete within acceptable time (~60 seconds)

## See Also

- [Testing Overview](./README.md) - Testing strategy and structure
- [Performance Testing](./performance.md) - Benchmarks and load tests
- [Test Architecture](./architecture.md) - Design patterns and best practices
- [Integration Testing](./integration.md) - Multi-component testing