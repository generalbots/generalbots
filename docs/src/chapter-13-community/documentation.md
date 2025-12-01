# Documentation

Good documentation is essential for maintaining and growing BotServer. This guide covers documentation standards and practices for contributors.

## Overview

BotServer documentation includes code documentation through inline comments and doc comments, API documentation, user guides, the BASIC language reference, architecture documentation, and README files throughout the repository.

## Documentation Structure

### Repository Documentation

The repository follows a structured documentation layout. The root contains `README.md` for the project overview and `CHANGELOG.md` for version history. The `docs/` directory contains mdBook documentation with source files in `docs/src/`. Each template directory also includes its own README file explaining that specific template.

### mdBook Documentation

The main documentation lives in `docs/src/` and covers user guides, developer guides, API references, architecture documentation, and the BASIC language reference.

## Code Documentation

### Rust Doc Comments

Use triple slashes for public items to generate documentation that integrates with Rust's documentation system:

```rust
/// Creates a new user session for the specified bot.
///
/// # Arguments
/// * `user_id` - The unique identifier of the user
/// * `bot_id` - The bot instance to connect to
///
/// # Returns
/// * `Result<Session>` - The created session or an error
///
/// # Example
/// ```
/// let session = create_session(user_id, bot_id)?;
/// println!("Session created: {}", session.id);
/// ```
pub fn create_session(user_id: Uuid, bot_id: Uuid) -> Result<Session> {
    // Implementation
}
```

### Module Documentation

Document modules with `//!` at the top of the file to provide context for the entire module:

```rust
//! # Session Management Module
//! 
//! This module handles user sessions and bot interactions.
//! 
//! ## Features
//! - Session creation and validation
//! - Token management
//! - Session persistence
//! 
//! ## Usage
//! ```
//! use crate::session::{Session, create_session};
//! ```

// Module code follows
```

### Inline Comments

Use inline comments for complex logic where the code's purpose isn't immediately obvious:

```rust
// Calculate the exponential backoff delay
// Using the formula: delay = base * 2^attempt
let delay = Duration::from_millis(100 * 2_u64.pow(attempt));

// Check if we've exceeded max retries
// This prevents infinite loops in case of permanent failures
if attempt > MAX_RETRIES {
    return Err("Max retries exceeded");
}
```

## API Documentation

### Endpoint Documentation

Document REST endpoints clearly with the HTTP method, path, purpose, request format, response format, and possible error codes:

```markdown
## Create User

**POST** `/api/users`

Creates a new user account.

### Request
```json
{
  "username": "john_doe",
  "email": "john@example.com"
}
```

### Response
```json
{
  "user_id": "user-123",
  "created_at": "2024-01-20T10:00:00Z"
}
```

### Errors
- `400` - Invalid input data
- `409` - Username already exists
```

### WebSocket Documentation

Document WebSocket protocols with connection details, message formats for both directions, and any special handling requirements:

```markdown
## WebSocket Protocol

### Connection
```
ws://localhost:8080/ws
```

### Message Format
Client → Server:
```json
{
  "type": "message",
  "content": "Hello",
  "session_id": "session-123"
}
```

Server → Client:
```json
{
  "type": "response",
  "content": "Bot response",
  "is_complete": true
}
```
```

## BASIC Script Documentation

### Keyword Documentation

Document BASIC keywords with syntax, parameters, and working examples:

```markdown
## TALK Keyword

Sends a message to the user.

### Syntax
```basic
TALK message
```

### Parameters
- `message` - The text to send to the user

### Examples
```basic
TALK "Hello, how can I help?"

let greeting = "Welcome!"
TALK greeting
```
```

### Script Examples

Provide complete working examples that demonstrate real-world usage patterns:

```basic
# greeting.bas
# A simple greeting bot that personalizes messages

# Get user's name
TALK "What's your name?"
let name = HEAR

# Create personalized greeting
let greeting = "Hello, " + name + "!"
TALK greeting

# Store for future use
SET_BOT_MEMORY "user_name", name
```

## Markdown Best Practices

### Structure

Use clear hierarchy with headings that progress logically from broad concepts to specific details. Start with a main title using a single hash, then use second-level headings for major sections, third-level for subsections, and so on.

### Code Blocks

Always specify the language for syntax highlighting in code blocks. Use `rust` for Rust code, `bash` for shell commands, `basic` for BASIC scripts, `json` for JSON data, and `toml` for configuration files.

### Tables

Use tables for structured data where comparison or quick reference is useful, such as parameter lists, feature comparisons, or API endpoints.

### Links

Use relative links for internal documentation to ensure links work regardless of where the documentation is hosted. For example, link to authentication documentation as `../chapter-11/README.md` rather than using absolute URLs.

## Writing Style

### Be Clear and Concise

Write directly and avoid unnecessary words. Instead of "The system employs a sophisticated relational database management system, specifically PostgreSQL, for the purpose of persisting structured information," simply write "BotServer uses PostgreSQL for structured data storage."

### Use Active Voice

Prefer active voice over passive voice for clarity. Write "The function returns an error if validation fails" rather than "An error is returned by the function if validation is failed."

### Provide Context

Explain not just what something does, but why it matters. Instead of only stating "Sessions expire after 24 hours," add the reasoning: "Sessions expire after 24 hours to balance security with user convenience."

## Documentation Process

### When to Document

Document before coding to clarify design and API structure. Add inline comments while coding to explain complex logic. After coding, update documentation with any learnings and add examples. During code review, ensure documentation is complete and accurate.

### Documentation Checklist

Before submitting a pull request, verify that all public functions have doc comments, complex logic has inline comments explaining the reasoning, README files are updated if the PR affects them, examples are provided for new features, API documentation reflects any changes, breaking changes are noted prominently, and the CHANGELOG is updated.

## Tools

### Documentation Generation

Generate Rust documentation with `cargo doc --open`, which builds and opens the documentation in your browser.

### Documentation Serving

Serve mdBook documentation locally during development:

```bash
cd docs
mdbook serve
```

### Spell Checking

Install and use cargo-spellcheck to catch spelling errors:

```bash
cargo install cargo-spellcheck
cargo spellcheck check
```

## Common Mistakes

### Missing Context

Avoid comments that merely restate the code. Instead of commenting "Increment counter" above `counter += 1`, explain why: "Increment retry counter to track failed attempts. This is used for exponential backoff calculation."

### Outdated Documentation

Always update documentation when code changes. This includes parameter changes, behavior modifications, new error conditions, and deprecated features. Outdated documentation is often worse than no documentation.

### Unclear Examples

Examples should be complete and demonstrate realistic usage. Instead of terse, unclear examples with generic variable names, provide full examples with meaningful names, comments explaining each step, and realistic use cases.

## Contributing Documentation

### Where to Contribute

Documentation contributions are welcome in many forms. Fix typos and errors anywhere you find them. Add examples to existing documentation. Clarify unclear sections. Document undocumented features. Translate documentation to other languages.

### Documentation PRs

Documentation-only pull requests are welcome and valuable. They can be merged quickly, don't require extensive testing, help new users get started, and improve overall project quality.

## Summary

Good documentation makes BotServer accessible to users and maintainable for developers. Always consider documentation as part of the development process, not an afterthought. Clear, accurate, and up-to-date documentation is as valuable as the code itself.