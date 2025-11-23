# Adding Dependencies

BotServer is a single-crate Rust application, so all dependencies are managed through the root `Cargo.toml` file. This guide covers how to add, update, and manage dependencies.

## Adding a Dependency

### Basic Dependency

To add a new crate, edit `Cargo.toml` and add it to the `[dependencies]` section:

```toml
[dependencies]
serde = "1.0"
```

Then update your dependencies:

```bash
cargo build
```

### Dependency with Features

Many crates offer optional features. Enable them like this:

```toml
[dependencies]
tokio = { version = "1.41", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Version-Specific Dependencies

Use specific version constraints:

```toml
[dependencies]
# Exact version
diesel = "=2.1.0"

# Minimum version
anyhow = ">=1.0.0"

# Compatible version (caret)
regex = "^1.11"

# Wildcard
uuid = "1.*"
```

### Git Dependencies

Add dependencies directly from Git repositories:

```toml
[dependencies]
rhai = { git = "https://github.com/therealprof/rhai.git", branch = "features/use-web-time" }
```

Or use a specific commit:

```toml
[dependencies]
my-crate = { git = "https://github.com/user/repo", rev = "abc123" }
```

Or a tag:

```toml
[dependencies]
my-crate = { git = "https://github.com/user/repo", tag = "v1.0.0" }
```

### Optional Dependencies

For features that aren't always needed:

```toml
[dependencies]
qdrant-client = { version = "1.12", optional = true }
imap = { version = "3.0.0-alpha.15", optional = true }
```

Then define features that enable them:

```toml
[features]
vectordb = ["qdrant-client"]
email = ["imap"]
```

### Platform-Specific Dependencies

Add dependencies only for specific platforms:

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
```

## Existing Dependencies

BotServer currently uses these major dependencies:

### Web Framework
- `axum` - HTTP web framework
- `tower` - Middleware and service abstraction
- `tower-http` - HTTP-specific middleware (CORS, static files, tracing)
- `hyper` - Low-level HTTP implementation

### Async Runtime
- `tokio` - Async runtime with full feature set
- `tokio-stream` - Stream utilities
- `async-trait` - Async traits
- `async-stream` - Async stream macros
- `async-lock` - Async locking primitives

### Database
- `diesel` - ORM for PostgreSQL
- `diesel_migrations` - Database migration management
- `r2d2` - Connection pooling
- `redis` - Cache client (Valkey/Redis-compatible)

### Storage
- `aws-config` - AWS SDK configuration
- `aws-sdk-s3` - S3-compatible storage (drive component)
- `qdrant-client` - Vector database (optional)

### Security
- `argon2` - Password hashing
- `aes-gcm` - Encryption
- `hmac` - HMAC authentication
- `sha2` - SHA-256 hashing

### Scripting
- `rhai` - BASIC interpreter engine

### Data Formats
- `serde` - Serialization/deserialization
- `serde_json` - JSON support
- `csv` - CSV parsing
- `base64` - Base64 encoding

### Document Processing
- `pdf-extract` - PDF text extraction
- `mailparse` - Email parsing
- `zip` - ZIP archive handling

### Communication
- `reqwest` - HTTP client
- `lettre` - SMTP email sending
- `imap` - IMAP email reading (optional)
- `livekit` - Video conferencing

### Desktop (Optional)
- `tauri` - Desktop application framework
- `tauri-plugin-dialog` - Native file dialogs
- `tauri-plugin-opener` - Open files/URLs

### Utilities
- `anyhow` - Error handling
- `log` - Logging facade
- `env_logger` - Environment-based logging
- `tracing` - Structured logging
- `chrono` - Date/time handling
- `uuid` - UUID generation
- `regex` - Regular expressions
- `rand` - Random number generation

### Testing
- `mockito` - HTTP mocking
- `tempfile` - Temporary file handling

## Adding a New Dependency: Example

Let's walk through adding a new dependency for JSON Web Tokens (JWT):

### 1. Choose a Crate

Search on [crates.io](https://crates.io):

```bash
cargo search jsonwebtoken
```

### 2. Add to Cargo.toml

```toml
[dependencies]
jsonwebtoken = "9.2"
```

### 3. Update Dependencies

```bash
cargo build
```

### 4. Import in Code

In your Rust file (e.g., `src/auth/mod.rs`):

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
```

### 5. Use the Dependency

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}
```

## Managing Dependencies

### Update All Dependencies

```bash
cargo update
```

### Update Specific Dependency

```bash
cargo update -p serde
```

### Check for Outdated Dependencies

Install and use `cargo-outdated`:

```bash
cargo install cargo-outdated
cargo outdated
```

### Upgrade to Latest Compatible Versions

Install and use `cargo-edit`:

```bash
cargo install cargo-edit
cargo upgrade
```

### Audit for Security Vulnerabilities

```bash
cargo install cargo-audit
cargo audit
```

### Check Dependency Tree

```bash
cargo tree
```

View dependencies for specific package:

```bash
cargo tree -p diesel
```

### Find Duplicate Dependencies

```bash
cargo tree --duplicates
```

## Feature Management

BotServer uses feature flags to enable optional functionality.

### Current Features

```toml
[features]
default = ["desktop"]
vectordb = ["qdrant-client"]
email = ["imap"]
desktop = ["dep:tauri", "dep:tauri-plugin-dialog", "dep:tauri-plugin-opener"]
```

### Adding a New Feature

1. Add optional dependency:

```toml
[dependencies]
elasticsearch = { version = "8.5", optional = true }
```

2. Create feature:

```toml
[features]
search = ["elasticsearch"]
```

3. Use conditional compilation:

```rust
#[cfg(feature = "search")]
pub mod search {
    use elasticsearch::Elasticsearch;
    
    pub fn create_client() -> Elasticsearch {
        // Implementation
    }
}
```

4. Build with feature:

```bash
cargo build --features search
```

## Build Dependencies

For build-time dependencies (used in `build.rs`):

```toml
[build-dependencies]
tauri-build = { version = "2", features = [] }
```

## Development Dependencies

For dependencies only needed during testing:

```toml
[dev-dependencies]
mockito = "1.7.0"
tempfile = "3"
```

These are not included in release builds.

## Dependency Best Practices

### 1. Version Pinning

Use specific versions for production:

```toml
# Good - specific version
serde = "1.0.193"

# Risky - any 1.x version
serde = "1"
```

### 2. Minimize Dependencies

Only add dependencies you actually need. Each dependency:
- Increases build time
- Increases binary size
- Adds maintenance burden
- Introduces security risk

### 3. Check License Compatibility

Ensure dependency licenses are compatible with AGPL-3.0:

```bash
cargo install cargo-license
cargo license
```

### 4. Prefer Maintained Crates

Check crate activity:
- Recent releases
- Active GitHub repository
- Responsive maintainers
- Good documentation

### 5. Review Security Advisories

Regularly audit dependencies:

```bash
cargo audit
```

### 6. Use Features to Reduce Size

Don't enable unnecessary features:

```toml
# Bad - includes everything
tokio = "1.41"

# Good - only what you need
tokio = { version = "1.41", features = ["rt-multi-thread", "net", "sync"] }
```

## Common Issues

### Conflicting Versions

If multiple crates need different versions of the same dependency:

```
error: failed to select a version for `serde`
```

Solution: Update dependencies or use `cargo tree` to identify conflicts.

### Missing System Libraries

If a dependency requires system libraries:

```
error: linking with `cc` failed
```

Solution: Install required system packages (see [Building from Source](./building.md)).

### Feature Not Found

If you reference a non-existent feature:

```
error: feature `invalid-feature` is not found
```

Solution: Check feature names in `Cargo.toml`.

## Removing Dependencies

### 1. Remove from Cargo.toml

Delete the dependency line.

### 2. Remove Imports

Find and remove all `use` statements:

```bash
rg "use dependency_name" src/
```

### 3. Clean Build

```bash
cargo clean
cargo build
```

### 4. Verify

Check the dependency is gone:

```bash
cargo tree | grep dependency_name
```

## Alternative Registries

### Using a Custom Registry

```toml
[dependencies]
my-crate = { version = "1.0", registry = "my-registry" }

[registries.my-registry]
index = "https://my-registry.example.com/index"
```

### Private Crates

For private company crates, use Git dependencies or a private registry like Artifactory or CloudSmith.

## Dependency Documentation

After adding a dependency, document its usage:

1. Add comment in `Cargo.toml`:

```toml
[dependencies]
# JWT token generation and validation
jsonwebtoken = "9.2"
```

2. Document in code:

```rust
/// Creates a JWT token for user authentication.
/// 
/// Uses the `jsonwebtoken` crate to encode user claims
/// with an expiration time.
pub fn create_jwt(user_id: &str) -> Result<String, Error> {
    // Implementation
}
```

## Next Steps

- Review [Module Structure](./crates.md) to understand where to use new dependencies
- Check [Service Layer](./services.md) to see how dependencies integrate
- Read [Creating Custom Keywords](./custom-keywords.md) to extend BASIC with new functionality