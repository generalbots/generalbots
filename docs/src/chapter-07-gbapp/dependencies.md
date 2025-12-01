# Adding Dependencies

BotServer is a single-crate Rust application, so all dependencies are managed through the root `Cargo.toml` file. This guide covers how to add, update, and manage dependencies effectively.

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

Many crates offer optional features that you can enable selectively. The syntax uses curly braces to specify both the version and the features array:

```toml
[dependencies]
tokio = { version = "1.41", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Version-Specific Dependencies

Cargo supports several version constraint formats to control which versions are acceptable. An exact version uses the equals sign prefix, while minimum versions use the greater-than-or-equal operator. The caret symbol indicates compatible versions according to semantic versioning, and wildcards allow any version within a major release:

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

You can add dependencies directly from Git repositories when you need unreleased features or custom forks. Specify the repository URL along with an optional branch name:

```toml
[dependencies]
rhai = { git = "https://github.com/therealprof/rhai.git", branch = "features/use-web-time" }
```

For reproducible builds, pin to a specific commit using the `rev` field:

```toml
[dependencies]
my-crate = { git = "https://github.com/user/repo", rev = "abc123" }
```

You can also reference a tagged release:

```toml
[dependencies]
my-crate = { git = "https://github.com/user/repo", tag = "v1.0.0" }
```

### Optional Dependencies

Some dependencies aren't always needed and can be marked as optional. These won't be compiled unless explicitly enabled through feature flags:

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

Certain dependencies are only needed on specific operating systems. Cargo's target configuration syntax lets you conditionally include dependencies based on the compilation target:

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
```

## Existing Dependencies

BotServer relies on a comprehensive set of dependencies organized by functionality.

### Web Framework

The HTTP layer is built on `axum` as the primary web framework, with `tower` providing middleware and service abstractions. The `tower-http` crate adds HTTP-specific middleware for CORS, static file serving, and tracing. At the lowest level, `hyper` handles the HTTP protocol implementation.

### Async Runtime

Asynchronous execution is powered by `tokio` with its full feature set enabled. Supporting crates include `tokio-stream` for stream utilities, `async-trait` for async trait definitions, `async-stream` for async stream macros, and `async-lock` for asynchronous locking primitives.

### Database

Database operations use `diesel` as the ORM for PostgreSQL, with `diesel_migrations` handling schema migrations. Connection pooling is managed by `r2d2`, and the `redis` crate provides cache client functionality compatible with both Valkey and Redis.

### Storage

Cloud storage integration relies on the AWS SDK, with `aws-config` for configuration and `aws-sdk-s3` for S3-compatible storage operations through the drive component. The optional `qdrant-client` crate enables vector database functionality.

### Security

Cryptographic operations use several specialized crates. Password hashing is handled by `argon2`, encryption by `aes-gcm`, HMAC authentication by `hmac`, and SHA-256 hashing by `sha2`.

### Scripting

The BASIC interpreter is powered by `rhai`, which provides a safe and fast embedded scripting engine.

### Data Formats

Serialization and deserialization use `serde` as the core framework, with `serde_json` for JSON support. Additional format support comes from `csv` for CSV parsing and `base64` for Base64 encoding.

### Document Processing

Document handling includes `pdf-extract` for PDF text extraction, `mailparse` for email parsing, and `zip` for ZIP archive handling.

### Communication

Network communication uses `reqwest` as the HTTP client. Email functionality is split between `lettre` for SMTP sending and the optional `imap` crate for reading emails. Video conferencing is provided by the `livekit` crate.

### Desktop (Optional)

Desktop application builds use `tauri` as the framework, along with `tauri-plugin-dialog` for native file dialogs and `tauri-plugin-opener` for opening files and URLs.

### Utilities

Common utilities include `anyhow` for error handling, `log` and `env_logger` for logging, `tracing` for structured logging, `chrono` for date and time handling, `uuid` for UUID generation, `regex` for regular expressions, and `rand` for random number generation.

### Testing

Test support comes from `mockito` for HTTP mocking and `tempfile` for temporary file handling.

## Adding a New Dependency: Example

This walkthrough demonstrates adding JSON Web Token (JWT) support to the project.

### 1. Choose a Crate

Search on [crates.io](https://crates.io) to find suitable crates:

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

### Updating Dependencies

To update all dependencies to their latest compatible versions, run `cargo update`. For updating a specific dependency, use `cargo update -p serde` with the package name.

### Checking for Outdated Dependencies

The `cargo-outdated` tool helps identify dependencies that have newer versions available:

```bash
cargo install cargo-outdated
cargo outdated
```

### Upgrading to Latest Compatible Versions

The `cargo-edit` tool provides convenient commands for managing dependencies:

```bash
cargo install cargo-edit
cargo upgrade
```

### Auditing for Security Vulnerabilities

Regular security audits are essential for production applications:

```bash
cargo install cargo-audit
cargo audit
```

### Viewing the Dependency Tree

Understanding your dependency graph helps identify bloat and conflicts:

```bash
cargo tree
```

To view dependencies for a specific package:

```bash
cargo tree -p diesel
```

### Finding Duplicate Dependencies

Different versions of the same crate increase binary size and compile time:

```bash
cargo tree --duplicates
```

## Feature Management

BotServer uses feature flags to enable optional functionality, allowing users to compile only what they need.

### Current Features

```toml
[features]
default = ["desktop"]
vectordb = ["qdrant-client"]
email = ["imap"]
desktop = ["dep:tauri", "dep:tauri-plugin-dialog", "dep:tauri-plugin-opener"]
```

### Adding a New Feature

Start by adding the dependency as optional:

```toml
[dependencies]
elasticsearch = { version = "8.5", optional = true }
```

Then create a feature that enables it:

```toml
[features]
search = ["elasticsearch"]
```

Use conditional compilation in your code to only include the functionality when the feature is enabled:

```rust
#[cfg(feature = "search")]
pub mod search {
    use elasticsearch::Elasticsearch;
    
    pub fn create_client() -> Elasticsearch {
        // Implementation
    }
}
```

Build with the feature enabled:

```bash
cargo build --features search
```

## Build Dependencies

Dependencies needed only at build time (used in `build.rs`) go in a separate section:

```toml
[build-dependencies]
tauri-build = { version = "2", features = [] }
```

## Development Dependencies

Dependencies needed only during testing should be placed in the dev-dependencies section. These are not included in release builds:

```toml
[dev-dependencies]
mockito = "1.7.0"
tempfile = "3"
```

## Dependency Best Practices

### Version Pinning

For production builds, prefer specific versions over ranges to ensure reproducible builds. While `serde = "1.0.193"` guarantees a specific version, `serde = "1"` could pull in any 1.x release, potentially introducing unexpected changes.

### Minimize Dependencies

Every dependency you add increases build time, binary size, and maintenance burden while introducing potential security risks. Only add dependencies that provide significant value and aren't easily implemented inline.

### Check License Compatibility

All dependencies must have licenses compatible with AGPL-3.0. The `cargo-license` tool helps audit your dependency licenses:

```bash
cargo install cargo-license
cargo license
```

### Prefer Maintained Crates

When choosing between crates that provide similar functionality, evaluate them based on recent release activity, GitHub repository engagement, maintainer responsiveness, and documentation quality.

### Review Security Advisories

Make dependency auditing part of your regular development workflow. Running `cargo audit` regularly helps catch known vulnerabilities before they become problems.

### Use Features to Reduce Size

Many crates include features you don't need. Instead of enabling everything with `tokio = "1.41"`, specify only the features you actually use:

```toml
tokio = { version = "1.41", features = ["rt-multi-thread", "net", "sync"] }
```

## Common Issues

### Conflicting Versions

When multiple crates require different versions of the same dependency, Cargo will fail to resolve the dependency graph. Use `cargo tree` to identify which crates are causing the conflict, then update dependencies or look for alternative crates.

### Missing System Libraries

Some crates require system libraries to be installed. If you see linker errors mentioning `cc`, check the crate's documentation for required system packages and refer to the Building from Source guide.

### Feature Not Found

Referencing a non-existent feature will cause a build error. Double-check feature names in the crate's `Cargo.toml` on crates.io or in its repository.

## Removing Dependencies

To remove a dependency, first delete it from `Cargo.toml`. Then find and remove all import statements using grep or ripgrep:

```bash
rg "use dependency_name" src/
```

After removing the imports, clean and rebuild:

```bash
cargo clean
cargo build
```

Verify the dependency is completely removed:

```bash
cargo tree | grep dependency_name
```

## Alternative Registries

### Using a Custom Registry

For private crates or custom registries, configure the registry in your `Cargo.toml`:

```toml
[dependencies]
my-crate = { version = "1.0", registry = "my-registry" }

[registries.my-registry]
index = "https://my-registry.example.com/index"
```

For private company crates, consider Git dependencies or a private registry like Artifactory or CloudSmith.

## Dependency Documentation

Good documentation makes dependencies easier to maintain. Add comments in `Cargo.toml` explaining why each dependency exists:

```toml
[dependencies]
# JWT token generation and validation
jsonwebtoken = "9.2"
```

Document usage in your code with doc comments that explain the dependency's role:

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

Review the [Module Structure](./crates.md) documentation to understand where to use new dependencies within the codebase. The [Service Layer](./services.md) guide shows how dependencies integrate into the application architecture. For extending BASIC with new functionality that leverages dependencies, see [Creating Custom Keywords](./custom-keywords.md).