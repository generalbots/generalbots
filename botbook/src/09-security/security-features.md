# General Bots Security Features Guide

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

Access tokens use JWT format with RS256 signing for verifiable authentication. Refresh tokens consist of secure random 256-bit values for session renewal. Session tokens use UUID v4 format with cache storage for fast validation. Token rotation happens automatically when tokens approach expiry, ensuring continuous secure access without user interruption.


## Encryption and Cryptography

### Cryptographic Libraries

The platform uses well-vetted cryptographic libraries for all security operations. The aes-gcm library version 0.10 provides authenticated encryption using AES-256-GCM. The argon2 library version 0.5 handles password hashing with Argon2id. The sha2 library version 0.10.9 provides cryptographic hashing with SHA-256. The hmac library version 0.12.1 enables message authentication using HMAC-SHA256. The rand library version 0.9.2 provides cryptographic random number generation using ChaCha20.

### Data Encryption

Encryption at rest protects stored data throughout the system. Database encryption applies column-level encryption to sensitive fields. File storage encryption uses AES-256-GCM for all uploaded files. Configuration encryption protects secrets using a master key.

Encryption in transit protects data during transmission. All external communications use TLS 1.3 for strong protection. Service-to-service communication uses mutual TLS (mTLS) for bidirectional authentication. Certificate pinning applies to critical services to prevent man-in-the-middle attacks.


## Network Security

### API Security

Rate limiting through Caddy protects against abuse. Per-IP limits default to 100 requests per minute. Per-user limits default to 1000 requests per hour. These limits are configured in the Caddyfile and can be adjusted for specific deployment requirements.

CORS configuration through Caddy controls cross-origin requests. Origins use a strict whitelist approach. Credentials are enabled for authenticated requests. HTTP methods are explicitly allowed rather than using wildcards.

Input validation protects against injection attacks. Schema validation applies to all inputs before processing. SQL injection prevention uses PostgreSQL prepared statements exclusively. XSS protection applies output encoding to all user-generated content. Path traversal prevention validates all file paths against allowed directories.

### WebSocket Security

WebSocket connections require authentication before establishment. Message size limits default to 10MB to prevent resource exhaustion. Heartbeat and ping-pong mechanisms validate connection health. Suspicious activity triggers automatic disconnection to protect the system.


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

### Database Configuration

PostgreSQL security settings should be added to postgresql.conf to enable SSL with ssl set to on, specify certificate files with ssl_cert_file and ssl_key_file, configure strong ciphers with ssl_ciphers, enable server cipher preference with ssl_prefer_server_ciphers, and set the ECDH curve with ssl_ecdh_curve. The database connection string should include sslmode=require to enforce encrypted connections.

### Caddy Configuration

Caddy provides secure reverse proxy functionality with automatic HTTPS. Global options should disable the admin interface and enable automatic HTTPS. TLS configuration should enforce TLS 1.3 only with strong cipher suites. Security headers should include Strict-Transport-Security, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection, Referrer-Policy, and Content-Security-Policy. Rate limiting should be configured per remote host. The reverse proxy should forward appropriate headers including X-Real-IP, X-Forwarded-For, and X-Forwarded-Proto. Access logging should output to files in JSON format for analysis.


## Best Practices

### Development Practices

Dependency management requires regular security updates. Run cargo audit to check for known vulnerabilities. Run cargo update to apply security patches. Use cargo audit --deny warnings in CI to prevent vulnerable dependencies.

Code quality is enforced through Cargo.toml lints. Unsafe code is prohibited in application code. Unwrap calls are forbidden in production code paths. Panic macros are not allowed. Complete error handling is required for all fallible operations.

Security testing validates protection mechanisms. Run the security test suite with cargo test --features security_tests. Fuzzing for input validation uses cargo fuzz run api_fuzzer to find edge cases.

### Deployment Practices

Container security for LXC deployments requires disabling privileged mode with security.privileged set to false, enabling isolated ID mapping with security.idmap.isolated set to true, and disabling nesting with security.nesting set to false. Applications should run as non-root users within containers.

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