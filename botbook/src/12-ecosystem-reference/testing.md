# Testing

botserver follows comprehensive testing practices to ensure reliability, performance, and maintainability of the codebase.

## Overview

Testing in botserver covers:
- Unit tests for individual functions
- Integration tests for components
- End-to-end tests for workflows
- Performance benchmarks
- BASIC script testing

## Test Organization

### Directory Structure

```
src/
├── module/
│   ├── mod.rs         # Module code
│   └── mod.test.rs    # Module tests
├── basic/keywords/
│   ├── keyword.rs     # Keyword implementation
│   └── keyword.test.rs # Keyword tests
tests/
├── integration/       # Integration tests
└── e2e/              # End-to-end tests
```

### Test Files

Tests are colocated with source code:
- `module.rs` - Implementation
- `module.test.rs` - Tests
- Or inline `#[cfg(test)]` modules

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in module
cargo test module_name::
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View coverage
open tarpaulin-report.html
```

## Unit Testing

### Basic Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_function_failure() {
        function_that_panics();
    }
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Integration Testing

### Database Tests

```rust
#[test]
fn test_database_operation() {
    // Use test database
    let conn = establish_test_connection();
    
    // Run migrations
    run_pending_migrations(&conn).unwrap();
    
    // Test operation
    let result = create_user(&conn, "test_user");
    assert!(result.is_ok());
    
    // Cleanup
    rollback_transaction(&conn);
}
```

### API Tests

```rust
#[tokio::test]
async fn test_api_endpoint() {
    // Create test app
    let app = create_test_app().await;
    
    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert response
    assert_eq!(response.status(), StatusCode::OK);
}
```

## BASIC Script Testing

### Testing Keywords

```rust
#[test]
fn test_custom_keyword() {
    let mut engine = Engine::new();
    let state = create_test_state();
    
    // Register keyword
    register_keyword(&state, &mut engine);
    
    // Execute script
    let script = r#"
        let result = MY_KEYWORD("input");
        result
    "#;
    
    let result: String = engine.eval(script).unwrap();
    assert_eq!(result, "expected output");
}
```

### Testing Script Compilation

```rust
#[test]
fn test_script_compilation() {
    let compiler = BasicCompiler::new(test_state(), test_bot_id());
    
    let script_path = "test.bas";
    let result = compiler.compile_file(script_path, "work_dir");
    
    assert!(result.is_ok());
    assert!(result.unwrap().mcp_tool.is_some());
}
```

## Test Utilities

### Test Fixtures

```rust
// test_utils.rs
pub fn create_test_state() -> Arc<AppState> {
    Arc::new(AppState {
        conn: create_test_pool(),
        config: test_config(),
        // ... other fields
    })
}

pub fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        // ...
    }
}
```

### Mock Objects

```rust
use mockall::*;

#[automock]
trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockEmailService::new();
    mock.expect_send_email()
        .times(1)
        .returning(|_, _, _| Ok(()));
    
    // Use mock in test
}
```

## Performance Testing

### Benchmarks

```rust
#![feature(test)]
extern crate test;

#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_function(b: &mut Bencher) {
        b.iter(|| {
            function_to_benchmark()
        });
    }
}
```

### Load Testing

```bash
# Using cargo-stress
cargo install cargo-stress
cargo stress --test load_test

# Custom load test
#[test]
#[ignore] // Run with --ignored flag
fn test_high_load() {
    let handles: Vec<_> = (0..100)
        .map(|_| {
            thread::spawn(|| {
                // Simulate load
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Test Best Practices

### Test Naming

```rust
// Good: Descriptive names
#[test]
fn test_user_creation_with_valid_email_succeeds() {}

#[test]
fn test_user_creation_with_invalid_email_fails() {}

// Bad: Generic names
#[test]
fn test1() {}
```

### Test Independence

```rust
// Each test should be independent
#[test]
fn test_independent_1() {
    let state = create_fresh_state();
    // Test logic
}

#[test]
fn test_independent_2() {
    let state = create_fresh_state(); // Fresh state
    // Test logic
}
```

### Test Data

```rust
// Use builders for test data
struct UserBuilder {
    username: String,
    email: String,
}

impl UserBuilder {
    fn new() -> Self {
        Self {
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
        }
    }
    
    fn with_username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }
    
    fn build(self) -> User {
        User {
            username: self.username,
            email: self.email,
            // ...
        }
    }
}
```

## Continuous Integration

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## Test Documentation

### Document Test Purpose

```rust
/// Tests that user creation fails when email is invalid.
/// 
/// This test ensures that the email validation logic
/// properly rejects malformed email addresses.
#[test]
fn test_invalid_email_rejection() {
    // Test implementation
}
```

## Common Testing Patterns

### Arrange-Act-Assert

```rust
#[test]
fn test_pattern() {
    // Arrange
    let input = prepare_test_data();
    let expected = "expected result";
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

### Given-When-Then

```rust
#[test]
fn test_user_story() {
    // Given: A user with valid credentials
    let user = create_valid_user();
    
    // When: The user attempts to login
    let result = login(user.username, user.password);
    
    // Then: The login should succeed
    assert!(result.is_ok());
}
```

## Summary

Comprehensive testing ensures botserver's reliability and makes refactoring safe. Focus on writing clear, independent tests that cover both success and failure cases, and maintain good test coverage across the codebase.