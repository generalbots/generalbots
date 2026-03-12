# Security Package - Comprehensive Security Features

## Purpose
Implements comprehensive security features for the botserver. Provides authentication, authorization, encryption, and security monitoring capabilities.

## Key Files
- **auth.rs**: Authentication and authorization
- **auth_api/**: API endpoints for authentication
- **auth_provider.rs**: Authentication provider implementations
- **ca.rs**: Certificate authority management
- **cert_pinning.rs**: Certificate pinning functionality
- **command_guard.rs**: Safe command execution
- **cors.rs**: CORS (Cross-Origin Resource Sharing)
- **csrf.rs**: CSRF (Cross-Site Request Forgery) protection
- **dlp.rs**: DLP (Data Loss Prevention)
- **encryption.rs**: Encryption utilities
- **error_sanitizer.rs**: Error sanitization for responses
- **file_validation.rs**: File validation and security checks
- **headers.rs**: Security headers configuration
- **jwt.rs**: JWT (JSON Web Token) handling
- **log_sanitizer.rs**: Log sanitization
- **mfa.rs**: MFA (Multi-Factor Authentication)
- **passkey.rs**: Passkey authentication
- **password.rs**: Password hashing and validation
- **prompt_security.rs**: Prompt security validation
- **protection/**: Advanced security protection
- **rate_limiter.rs**: Rate limiting implementation
- **rbac_middleware.rs**: RBAC (Role-Based Access Control) middleware
- **security_monitoring.rs**: Security monitoring
- **sql_guard.rs**: SQL injection protection
- **tls.rs**: TLS (Transport Layer Security)
- **validation.rs**: Input validation
- **zitadel_auth.rs**: Zitadel authentication integration

## Security Directives - MANDATORY

### 1. Error Handling - NO PANICS IN PRODUCTION
```rust
// ❌ FORBIDDEN
value.unwrap()
value.expect("message")
panic!("error")
todo!()
unimplemented!()

// ✅ REQUIRED
value?
value.ok_or_else(|| Error::NotFound)?
value.unwrap_or_default()
value.unwrap_or_else(|e| { log::error!("{}", e); default })
if let Some(v) = value { ... }
match value { Ok(v) => v, Err(e) => return Err(e.into()) }
```

### 2. Command Execution - USE SafeCommand
```rust
// ❌ FORBIDDEN
Command::new("some_command").arg(user_input).output()

// ✅ REQUIRED
use crate::security::command_guard::SafeCommand;
SafeCommand::new("allowed_command")?
    .arg("safe_arg")?
    .execute()
```

### 3. Error Responses - USE ErrorSanitizer
```rust
// ❌ FORBIDDEN
Json(json!({ "error": e.to_string() }))
format!("Database error: {}", e)

// ✅ REQUIRED
use crate::security::error_sanitizer::log_and_sanitize;
let sanitized = log_and_sanitize(&e, "context", None);
(StatusCode::INTERNAL_SERVER_ERROR, sanitized)
```

### 4. SQL - USE sql_guard
```rust
// ❌ FORBIDDEN
format!("SELECT * FROM {}", user_table)

// ✅ REQUIRED
use crate::security::sql_guard::{sanitize_identifier, validate_table_name};
let safe_table = sanitize_identifier(&user_table);
validate_table_name(&safe_table)?;
```

## Features

### Authentication Methods
- **Password-based**: Secure password hashing (bcrypt)
- **Passkey**: WebAuthn passkey authentication
- **MFA**: Multi-factor authentication support
- **JWT**: JSON Web Token authentication
- **Zitadel**: External identity provider integration

### Authorization
- **RBAC**: Role-Based Access Control
- **Permissions**: Fine-grained permission system
- **Middleware**: Request-level authorization checks

### Security Monitoring
- **Security logs**: Detailed security event logging
- **Anomaly detection**: Suspicious activity detection
- **Audit trails**: Complete audit history

### Encryption
- **Data at rest**: Encryption for stored data
- **Data in transit**: TLS 1.3 encryption
- **Secrets management**: Secure secrets storage

## Usage Patterns

### Authentication Flow
```rust
use crate::security::auth::AuthService;

async fn login(email: String, password: String) -> Result<AuthResult, AuthError> {
    let auth_service = AuthService::new();
    auth_service.authenticate(email, password).await
}

async fn verify_token(token: String) -> Result<Claims, TokenError> {
    let auth_service = AuthService::new();
    auth_service.verify_token(token).await
}
```

### Permissions Check
```rust
use crate::security::rbac::has_permission;

async fn check_permission(user_id: Uuid, permission: &str) -> Result<bool, RbacError> {
    has_permission(user_id, permission).await
}
```

## Configuration
Security settings are configured in:
- `botserver/.env` - Environment variables
- Configuration files in `config/` directory
- Database for dynamic settings

## Security Headers
All responses include mandatory security headers:
- `Content-Security-Policy`: Default to 'self'
- `Strict-Transport-Security`: 2 years duration
- `X-Frame-Options`: DENY
- `X-Content-Type-Options`: nosniff
- `Referrer-Policy`: strict-origin-when-cross-origin
- `Permissions-Policy`: Geolocation, microphone, camera disabled

## Testing
Security package has comprehensive tests:
- Unit tests for each security guard
- Integration tests for authentication flows
- Performance tests for rate limiting
- Security regression tests