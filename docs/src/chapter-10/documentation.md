# Documentation

Good documentation is essential for maintaining and growing BotServer. This guide covers documentation standards and practices for contributors.

## Overview

BotServer documentation includes:
- Code documentation (inline comments and doc comments)
- API documentation
- User guides
- BASIC language reference
- Architecture documentation
- README files

## Documentation Structure

### Repository Documentation

```
botserver/
├── README.md              # Project overview
├── CHANGELOG.md          # Version history
├── docs/                 # mdBook documentation
│   └── src/             # Documentation source
└── templates/*/README.md # Template documentation
```

### mdBook Documentation

The main documentation in `docs/src/`:
- User guides
- Developer guides
- API references
- Architecture documentation
- BASIC language reference

## Code Documentation

### Rust Doc Comments

Use triple slashes for public items:

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

Document modules with `//!` at the top:

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

Use inline comments for complex logic:

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

Document REST endpoints clearly:

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

Document WebSocket protocols:

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

Document BASIC keywords with examples:

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

Provide complete working examples:

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

Use clear hierarchy:
```markdown
# Main Title
Brief introduction paragraph.

## Section
Section content.

### Subsection
Detailed information.

#### Detail Point
Specific details.
```

### Code Blocks

Always specify language:
````markdown
```rust
let x = 42;
```

```bash
cargo build --release
```

```basic
TALK "Hello"
```
````

### Tables

Use tables for structured data:
```markdown
| Parameter | Type | Description |
|-----------|------|-------------|
| user_id | UUID | User identifier |
| bot_id | UUID | Bot identifier |
```

### Links

Use relative links for internal docs:
```markdown
See [Authentication](../chapter-11/README.md) for details.
```

## Writing Style

### Be Clear and Concise

Good:
> "BotServer uses PostgreSQL for structured data storage."

Bad:
> "The system employs a sophisticated relational database management system, specifically PostgreSQL, for the purpose of persisting structured information."

### Use Active Voice

Good:
> "The function returns an error if validation fails."

Bad:
> "An error is returned by the function if validation is failed."

### Provide Context

Good:
> "Sessions expire after 24 hours to balance security with user convenience."

Bad:
> "Sessions expire after 24 hours."

## Documentation Process

### When to Document

- **Before coding**: Document the design and API
- **While coding**: Add inline comments for complex logic
- **After coding**: Update with learnings and examples
- **During review**: Ensure documentation is complete

### Documentation Checklist

Before submitting PR:
- [ ] All public functions have doc comments
- [ ] Complex logic has inline comments
- [ ] README updated if needed
- [ ] Examples provided for new features
- [ ] API documentation updated
- [ ] Breaking changes noted
- [ ] CHANGELOG updated

## Tools

### Documentation Generation

Generate Rust docs:
```bash
cargo doc --open
```

### Documentation Serving

Serve mdBook locally:
```bash
cd docs
mdbook serve
```

### Spell Checking

Use spell checker:
```bash
# Install
cargo install cargo-spellcheck

# Run
cargo spellcheck check
```

## Common Mistakes

### Missing Context

Bad:
```rust
// Increment counter
counter += 1;
```

Good:
```rust
// Increment retry counter to track failed attempts
// This is used for exponential backoff calculation
counter += 1;
```

### Outdated Documentation

Always update docs when code changes:
- Parameter changes
- Behavior modifications
- New error conditions
- Deprecated features

### Unclear Examples

Bad:
```basic
let x = GET "file"
let y = LLM x
TALK y
```

Good:
```basic
# Load company policy document
let policy = GET "policies/vacation.pdf"

# Generate summary using LLM
let summary = LLM "Summarize the key points: " + policy

# Present summary to user
TALK summary
```

## Contributing Documentation

### Where to Contribute

- Fix typos and errors anywhere
- Add examples to existing docs
- Clarify unclear sections
- Document undocumented features
- Translate documentation

### Documentation PRs

Documentation-only PRs are welcome:
- Can be merged quickly
- Don't require extensive testing
- Help new users
- Improve project quality

## Summary

Good documentation makes BotServer accessible to users and maintainable for developers. Always consider documentation as part of the development process, not an afterthought. Clear, accurate, and up-to-date documentation is as valuable as the code itself.