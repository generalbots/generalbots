# Code Standards

BotServer follows Rust best practices and conventions to maintain a consistent, readable, and maintainable codebase.

## Rust Style Guide

### Formatting

Use `rustfmt` for automatic formatting:
```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

Configuration in `.rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
```

### Linting

Use `clippy` for code quality:
```bash
# Run clippy
cargo clippy -- -D warnings

# Fix clippy suggestions
cargo clippy --fix
```

## Naming Conventions

### General Rules

- **snake_case**: Functions, variables, modules
- **PascalCase**: Types, traits, enums
- **SCREAMING_SNAKE_CASE**: Constants
- **'lifetime**: Lifetime parameters

### Examples

```rust
// Good naming
const MAX_RETRIES: u32 = 3;

struct UserSession {
    session_id: Uuid,
}

fn process_message(content: &str) -> Result<String> {
    // ...
}

trait MessageHandler {
    fn handle(&self, msg: &Message);
}

// Bad naming
const maxRetries: u32 = 3;  // Should be SCREAMING_SNAKE_CASE
struct user_session {}       // Should be PascalCase
fn ProcessMessage() {}        // Should be snake_case
```

## Code Organization

### Module Structure

```rust
// mod.rs or lib.rs
pub mod user;
pub mod session;
pub mod auth;

// Re-exports
pub use user::User;
pub use session::Session;
```

### Import Ordering

```rust
// 1. Standard library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use tokio::sync::Mutex;
use uuid::Uuid;

// 3. Local crates
use crate::config::Config;
use crate::models::User;

// 4. Super/self
use super::utils;
use self::helper::*;
```

## Error Handling

### Use Result Types

```rust
// Good: Explicit error handling
fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

// Bad: Panic on error
fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}
```

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Network error")]
    Network(#[from] reqwest::Error),
}
```

## Documentation

### Module Documentation

```rust
//! # User Management Module
//! 
//! This module handles user authentication and profile management.
//! 
//! ## Example
//! ```
//! use crate::user::User;
//! let user = User::new("john", "john@example.com");
//! ```

pub mod user {
    // ...
}
```

### Function Documentation

```rust
/// Creates a new user session.
///
/// # Arguments
/// * `user_id` - The unique identifier of the user
/// * `bot_id` - The bot instance identifier
///
/// # Returns
/// * `Result<Session>` - The created session or an error
///
/// # Example
/// ```
/// let session = create_session(user_id, bot_id)?;
/// ```
pub fn create_session(user_id: Uuid, bot_id: Uuid) -> Result<Session> {
    // ...
}
```

## Async/Await Best Practices

### Async Functions

```rust
// Good: Necessary async
async fn fetch_data(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}

// Bad: Unnecessary async
async fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b  // No await needed
}
```

### Spawning Tasks

```rust
// Good: Structured concurrency
let handle = tokio::spawn(async move {
    process_task().await
});

let result = handle.await??;

// Handle errors properly
match handle.await {
    Ok(Ok(value)) => value,
    Ok(Err(e)) => return Err(e),
    Err(e) => return Err(e.into()),
}
```

## Memory Management

### Use Arc for Shared State

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl AppState {
    pub async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().await;
        data.get(key).cloned()
    }
}
```

### Avoid Unnecessary Cloning

```rust
// Good: Borrow when possible
fn process(data: &str) -> String {
    data.to_uppercase()
}

// Bad: Unnecessary clone
fn process(data: String) -> String {
    data.clone().to_uppercase()  // Clone not needed
}
```

## Database Patterns

### Use Diesel Properly

```rust
// Good: Type-safe queries
use diesel::prelude::*;

let user = users::table
    .filter(users::id.eq(user_id))
    .first::<User>(&mut conn)?;

// Bad: Raw SQL without type safety
let user = sql_query("SELECT * FROM users WHERE id = ?")
    .bind::<Text, _>(user_id)
    .load(&mut conn)?;
```

## Testing Standards

### Test Naming

```rust
#[test]
fn test_user_creation_succeeds_with_valid_data() {
    // Clear test name indicating what is being tested
}

#[test]
fn test_user_creation_fails_with_invalid_email() {
    // Indicates failure condition
}
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod user_creation {
        #[test]
        fn with_valid_data() {}
        
        #[test]
        fn with_invalid_email() {}
    }
    
    mod user_authentication {
        #[test]
        fn with_correct_password() {}
        
        #[test]
        fn with_wrong_password() {}
    }
}
```

## Security Standards

### Never Hardcode Secrets

```rust
// Good: Environment variables
let api_key = std::env::var("API_KEY")?;

// Bad: Hardcoded secret
let api_key = "sk-1234567890abcdef";
```

### Validate Input

```rust
// Good: Validate and sanitize
fn process_input(input: &str) -> Result<String> {
    if input.len() > 1000 {
        return Err("Input too long".into());
    }
    
    if !input.chars().all(|c| c.is_alphanumeric()) {
        return Err("Invalid characters".into());
    }
    
    Ok(input.to_string())
}
```

## Performance Guidelines

### Use Iterators

```rust
// Good: Iterator chains
let sum: i32 = numbers
    .iter()
    .filter(|n| **n > 0)
    .map(|n| n * 2)
    .sum();

// Less efficient: Collecting intermediate
let filtered: Vec<_> = numbers.iter().filter(|n| **n > 0).collect();
let doubled: Vec<_> = filtered.iter().map(|n| *n * 2).collect();
let sum: i32 = doubled.iter().sum();
```

## Code Review Checklist

Before submitting code:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted with rustfmt
- [ ] Clippy passes without warnings
- [ ] Documentation is updated
- [ ] No hardcoded secrets
- [ ] Error handling is proper
- [ ] Performance implications considered
- [ ] Security implications reviewed

## Summary

Following these standards ensures BotServer code remains consistent, maintainable, and high-quality. Always prioritize clarity and correctness over cleverness.