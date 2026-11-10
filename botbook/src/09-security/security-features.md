# General Bots Security Features Guide 🟡 BETA

## Overview

This document provides a comprehensive overview of all security features and configurations available in General Bots, designed for security experts and enterprise deployments. Understanding these features enables organizations to deploy General Bots with confidence in regulated environments.


## Feature Flags

### Core Security Features

Security features are configured through Cargo.toml or via build flags at compile time. A basic build with desktop UI uses `cargo build --features desktop`. A full security-enabled build uses `cargo build --features "desktop,vectordb,email"`. A server-only build without desktop UI uses `cargo build --no-default-features --features "vectordb,email"`.

### Available Features

The desktop feature provides the Tauri desktop UI with a sandboxed runtime and controlled system access, and is enabled by default. The vectordb feature enables Qdrant integration for AI-powered threat detection and semantic search, and must be explicitly enabled. The email feature provides IMAP and SMTP support, which requires secure credential storage, and must also be explicitly enabled.

### Enterprise Security Features

Enterprise-ready security features include built-in encryption for data at rest via the aes-gcm library, comprehensive audit logging capabilities, role-based access control implemented through the Directory Service, multi-factor authentication available via the Directory Service, and SAML/OIDC single sign-on support also through the Directory Service.


## Authentication and Authorization

### Directory Service Integration

General Bots uses the Directory Service as the primary identity provider. Currently this is Zitadel, though it can be migrated to Keycloak or other OIDC providers. The integration provides OAuth2 and OIDC authentication, JWT token validation, user and group management, permission management, and session handling.

### Password Security

Password hashing uses the Argon2id algorithm, which is memory-hard and GPU-resistant. The configuration uses 19456 KB of memory, 2 iterations, parallelism of 1, and a random 32-byte salt. This configuration provides strong protection against both online and offline attacks while maintaining reasonable authentication performance.

### Token Management

Server-minted access tokens are JWT format with **HS256** signing: the suite
`JwtConfig::default()` selects HS256, and cloud login
(`botcloud::api::handle_login`) mints HS256 tokens with the persisted
`saas_jwt_secret`. Exactly one module turns either token into claims —
`botsecurity_core::verified_jwt` — which pins the algorithm to HS256, compares
the HMAC in constant time, requires an `exp` and honours `nbf` with a 60-second
leeway. Zitadel `id_token`s are RS256 and are a different credential: the
server's request authenticator does not verify them through a JWKS yet (see the
allowlist in [Multi-Tenant Data Isolation](./multi-tenant-isolation.md#one-verifier-for-the-whole-tree)).
Refresh tokens consist of secure random 256-bit values for session renewal.
Session tokens use UUID v4 format with cache storage for fast validation. Token
rotation happens automatically when tokens approach expiry, ensuring continuous
secure access without user interruption.


### Multi-Factor Authentication

Zitadel provides TOTP and WebAuthn for the directory login. Separately, the
server carries an in-process MFA module
(`botserver/crates/botsecurity-auth/src/mfa/`) that fixes **one** TOTP wire
format so the enrollment QR code and the verifier cannot disagree: an unpadded
uppercase RFC 4648 base32 secret, `SHA1`/6 digits/30 s written into the
`otpauth://` URI, a ±1 period verification window, and a legacy 40-character hex
row converted on read. The format, the migration and the vectors are in
[User Authentication](./user-auth.md#totp-secret-format-botsecurity-auth).

## Encryption and Cryptography

### Cryptographic Libraries

The platform uses well-vetted cryptographic libraries for all security operations. The aes-gcm library version 0.10 provides authenticated encryption using AES-256-GCM. The argon2 library version 0.5 handles password hashing with Argon2id. The sha2 library version 0.10.9 provides cryptographic hashing with SHA-256. The hmac library version 0.12.1 enables message authentication using HMAC-SHA256. The rand library version 0.9.2 provides cryptographic random number generation using ChaCha20.

### Data Encryption

Encryption at rest protects stored data throughout the system. Database encryption applies column-level encryption to sensitive fields. File storage encryption uses AES-256-GCM for all uploaded files. Configuration encryption protects secrets using a master key.

Encryption in transit protects data during transmission. All external communications use TLS 1.3 for strong protection. Service-to-service communication uses mutual TLS (mTLS) for bidirectional authentication. Certificate pinning applies to critical services to prevent man-in-the-middle attacks.


## Network Security

### API Security

Rate limiting through Caddy protects against abuse. Per-IP limits default to 100 requests per minute. Per-user limits default to 1000 requests per hour. These limits are configured in the Caddyfile and can be adjusted for specific deployment requirements.

CORS is fail-closed (#2129). The API reads its allowlist from the `cors-allowed-origins` configuration value (comma-separated exact origins) and every builder in `botsecurity-core` shares one rule: with no origin configured the response carries **no** `Access-Control-Allow-Origin`, so browsers treat the API as same-origin only. A configured origin is echoed exactly, and `Access-Control-Allow-Credentials: true` is announced only for an origin on that list — never for a caller-supplied origin that merely looks like one. A `*` entry in the configuration is ignored (it can never widen a credentialed allowlist); deployments must list their origins. The only escape hatch is `CORS_ALLOW_ANY_ORIGIN=1`, a development switch that is off unless explicitly set, accepts only format-valid `http(s)` origins and always refuses credentials. HTTP methods are explicitly allowed rather than using wildcards.

Input validation protects against injection attacks. Schema validation applies to all inputs before processing. SQL injection prevention uses PostgreSQL prepared statements exclusively. XSS protection applies output encoding to all user-generated content. Path traversal prevention validates all file paths against allowed directories.

### WebSocket Security

WebSocket connections require authentication before establishment. Message size limits default to 10MB to prevent resource exhaustion. Heartbeat and ping-pong mechanisms validate connection health. Suspicious activity triggers automatic disconnection to protect the system.

### HTTP Security Headers (IMP-09)

Every response from every server in the workspace carries six headers: `Content-Security-Policy`, `Strict-Transport-Security`, `X-Frame-Options`, `X-Content-Type-Options: nosniff`, `Referrer-Policy` and `Permissions-Policy`. One implementation emits them — `botsecurity_core::headers::with_security_headers` — and it is applied to all four surfaces: the API (`botserver`, port 8080), the suite (`botui`, port 3000), the cloud UI (port 4000) and the login UI (port 5000). The deployment gateway, the sandbox listener and the ChatGPT OAuth loopback listener are separate `axum::serve` calls and each applies the same layer.

The layer is applied *outermost*, wrapping the finished router. Axum only layers routes that already exist, so it must be the last step; doing so is what puts the headers on fallbacks, nested static files (`ServeDir`) and error responses — 401, 403, 404, 429 and 500 — and not just on successful responses. Handler-scoped policies win where they exist: the CSP and `X-Frame-Options` entries use insert-if-absent, so a sandboxed preview or a same-origin file preview keeps its own stricter value.

`Permissions-Policy` delegates `microphone` and `camera` to `self` in the default profile, because the chat composer records voice notes and takes camera shots from the same origin; setting them to `()` makes those buttons silently fail. API-only responses use the `api()` profile, which disables every sensor but still emits all six headers — a profile may widen a policy, never drop a header.

**Plain-HTTP local development.** HSTS is emitted by default. A browser that receives it on an `http://localhost` origin pins that host to HTTPS for the whole `max-age`, and a dev server that does not speak TLS then refuses to load until the browser's transport-security state is cleared. Local dev therefore sets `GB_SECURITY_HSTS=off` (accepted values: `0`, `off`, `false`, `no`). Production never sets it. The flag removes HSTS only; the other five headers stay on.

The `scripts/check_security_headers.py` gate (wired into `ci.yml`) fails when an `axum::serve` is mounted without the layer, when a `configure_*_router` factory stops applying it, when the API layer stack is no longer layered outermost, or when one of the six headers disappears from the shared applier.


## Data Protection

### Database Security

PostgreSQL security features provide comprehensive database protection. Row-level security (RLS) restricts access to specific rows based on user context. Column encryption protects personally identifiable information. Audit logging records all database access. Connection pooling limits resource consumption. Prepared statements prevent SQL injection. SSL/TLS connections are enforced for all database communication.

### File Storage Security

Drive configuration provides secure object storage. Bucket encryption uses AES-256 for all stored objects. Policy-based access control restricts file access. Versioning enables recovery from accidental changes. Immutable objects support prevents tampering. TLS encryption protects data in transit.

Local storage follows security best practices. Directory permissions are set to 700 for restricted access. File permissions are set to 600 for owner-only access. Temporary files undergo secure deletion to prevent data leakage.

### Memory Protection

Memory protection measures prevent sensitive data exposure. Zeroization clears sensitive data from memory after use. Logging configurations exclude secrets from log output. Secure random generation uses cryptographic sources. Protected memory pages safeguard cryptographic keys during operation.


## Audit and Compliance

### Log Security

Structured logging configuration ensures comprehensive audit trails. Log level uses INFO in production and DEBUG in development. Format uses JSON for machine parsing and analysis. Rotation occurs daily with 30-day retention by default. Sensitive data is automatically redacted from log output.

### Audit Events

The system automatically logs security-relevant events including authentication attempts both successful and failed, authorization failures when users attempt unauthorized actions, data access operations for both reads and writes, configuration changes by administrators, administrative actions across the system, API calls with relevant parameters, and security violations when detected.

### Compliance Support

GDPR compliance features include data deletion capabilities and data export for portability. SOC2 compliance is supported through comprehensive audit trails and access controls. HIPAA compliance can be achieved with encryption and access logging configuration. PCI DSS requirements are addressed through no credit card storage and tokenization support for payment processing.


## Security Configuration

### Environment Variables

Required security settings include BOTSERVER_JWT_SECRET as a 256-bit hex string for token signing, BOTSERVER_ENCRYPTION_KEY as a 256-bit hex string for data encryption, and DATABASE_ENCRYPTION_KEY as a 256-bit hex string for database field encryption.

Directory service configuration requires ZITADEL_DOMAIN pointing to your Zitadel instance, ZITADEL_CLIENT_ID with your application client ID, and ZITADEL_CLIENT_SECRET with your application secret.

Drive configuration requires MINIO_ENDPOINT for the storage server address, MINIO_ACCESS_KEY and MINIO_SECRET_KEY for authentication, and MINIO_USE_SSL set to true for encrypted connections.

Cache configuration requires CACHE_URL pointing to the Redis-compatible server and CACHE_PASSWORD for authentication.

Optional security enhancements include BOTSERVER_ENABLE_AUDIT to enable comprehensive audit logging, BOTSERVER_REQUIRE_MFA to enforce multi-factor authentication, BOTSERVER_SESSION_TIMEOUT to set session duration in seconds, BOTSERVER_MAX_LOGIN_ATTEMPTS to limit failed login attempts, and BOTSERVER_LOCKOUT_DURATION to set account lockout time in seconds.

Network security settings include BOTSERVER_ALLOWED_ORIGINS for CORS whitelist, BOTSERVER_RATE_LIMIT_PER_IP for per-IP request limits, BOTSERVER_RATE_LIMIT_PER_USER for per-user request limits, and BOTSERVER_MAX_UPLOAD_SIZE for maximum file upload size in bytes.

Local development over plain HTTP sets GB_SECURITY_HSTS to off, false, 0 or no to suppress Strict-Transport-Security for that process only (see HTTP Security Headers above). Production leaves it unset so HSTS is emitted.

### Database Configuration

PostgreSQL security settings should be added to postgresql.conf to enable SSL with ssl set to on, specify certificate files with ssl_cert_file and ssl_key_file, configure strong ciphers with ssl_ciphers, enable server cipher preference with ssl_prefer_server_ciphers, and set the ECDH curve with ssl_ecdh_curve. The database connection string should include sslmode=require to enforce encrypted connections.

### Caddy Configuration

Caddy provides secure reverse proxy functionality with automatic HTTPS. Global options should disable the admin interface and enable automatic HTTPS. TLS configuration should enforce TLS 1.3 only with strong cipher suites. Security headers should include Strict-Transport-Security, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection, Referrer-Policy, and Content-Security-Policy. Rate limiting should be configured per remote host. The reverse proxy should forward appropriate headers including X-Real-IP, X-Forwarded-For, and X-Forwarded-Proto. Access logging should output to files in JSON format for analysis.


## Best Practices

### Development Practices

Dependency management requires regular security updates. Run cargo audit to check for known vulnerabilities. Run cargo update to apply security patches. Use cargo audit --deny warnings in CI to prevent vulnerable dependencies.

Code quality is enforced through Cargo.toml lints. Unsafe code is prohibited in application code. Unwrap calls are forbidden in production code paths. Panic macros are not allowed. Complete error handling is required for all fallible operations.

Security testing validates protection mechanisms. Run the security test suite with cargo test --features security_tests. Fuzzing for input validation uses cargo fuzz run api_fuzzer to find edge cases.

### Mechanical gates in CI

Structural rules that a reviewer would otherwise have to remember are enforced
by scripts. Each one fails the build with `file:line`, and each allowlist is
**shrink-only**: an entry that no longer suppresses a violation is itself
reported.

| Gate | Command | Fails on |
|---|---|---|
| JWT hardening (#2125) | `python3 scripts/check_jwt_hardening.py --check` | `jsonwebtoken::dangerous` / `insecure_decode`; a file that base64url-decodes a JWT payload and reads `sub`/`user_id`/`email` without calling the shared verifier |
| Container runtime (#2126) | `python3 scripts/check_container_runtime.py --check` | spawning `lxc`/`lxd`/`lxc-execute` (even through `SafeCommand`), or `incus` through a raw `Command::new` instead of `SafeCommand` |
| Security directives (IMP-07/IMP-08) | `python3 scripts/check_security_directives.py` | a state-changing handler reachable without the CSRF layer and without an entry in `scripts/security_directives_exemptions.toml`; a missing general/auth/API rate limiter or the WebSocket message limiter |
| Security headers (IMP-09) | `python3 scripts/check_security_headers.py --check` | an `axum::serve` without the header layer, a `configure_*_router` factory that stopped applying it, an API stack no longer layered outermost, a missing header, or a profile that drops a required header (rules R1–R5) |

Each gate carries a `--self-test` that proves its rules fire, so a regex that
silently stops matching fails instead of passing everything. The ones wired into
`.github/workflows/ci.yml` run in the `guard` job, and every script also runs
standalone with the command above. Every allowlist and exemption list is
shrink-only: `scripts/jwt_hardening_allowlist.txt`,
`scripts/container_runtime_allowlist.txt`,
`scripts/security_directives_exemptions.toml`,
`scripts/security_headers_allowlist.txt`.

`scripts/prod_readiness.py` is the aggregator: it runs the checks registered in
`scripts/prod_readiness_gates.py` and prints one row per check with the issue
that owns it. It distinguishes three states, and `SKIP` is never a pass:

| Status | Meaning |
|---|---|
| `PASS` | the check ran and passed |
| `FAIL` | the check ran and failed — a regression; the gate exits 1 |
| `SKIP` | the check (or its baseline) has not landed yet; the row names the owning issue |

```bash
python3 scripts/prod_readiness.py                  # table, exit 1 on FAIL
python3 scripts/prod_readiness.py --json           # machine output
python3 scripts/prod_readiness.py --check-markdown # fail when the scoreboard drifted
```

The scoreboard is generated into
[Production Readiness Gate](../10-configuration-deployment/production-readiness.md);
regenerate it with `--markdown` instead of editing it.

### Deployment Practices

Container security for Incus deployments requires disabling privileged mode with security.privileged set to false, enabling isolated ID mapping with security.idmap.isolated set to true, and disabling nesting with security.nesting set to false. Applications should run as non-root users within containers.

Container security profiles should specify resource limits including CPU and memory caps. Root device configuration should use appropriate storage pools. Security settings should prevent privilege escalation.

Network policies should restrict traffic appropriately. Ingress should only be allowed from the Caddy proxy. Egress should be limited to PostgreSQL, Drive, Qdrant, and Cache. All other traffic should be blocked. Internal communication between components should use isolated networks.

### Monitoring Practices

Security metrics to track include failed authentication rate, unusual API access patterns, resource usage anomalies, and geographic access patterns for detecting account compromise.

Alerting thresholds should trigger warnings at 5 or more failed logins, lock accounts at 10 or more failed logins, alert on unusual geographic access patterns, and issue critical alerts for any privilege escalation attempts.

Incident response capabilities include automatic session termination when threats are detected, account lockout for repeated failures, and comprehensive logging for forensic analysis.


## Security Checklist

Before deploying General Bots in production, verify that all environment variables are set with strong random values, TLS is properly configured with valid certificates, database connections use SSL, file storage uses encryption, audit logging is enabled, rate limiting is configured appropriately, security headers are set in the reverse proxy, monitoring and alerting are configured, backup and recovery procedures are tested, and incident response procedures are documented.


## See Also

The Security Policy chapter provides organizational security policies and procedures. The Password Security chapter details password requirements and implementation. The User Authentication chapter covers authentication flows and configuration. The Compliance Requirements chapter addresses regulatory compliance in detail.