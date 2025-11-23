# 🔒 BotServer Security Features Guide

## Overview

This document provides a comprehensive overview of all security features and configurations available in BotServer, designed for security experts and enterprise deployments.

## 📋 Table of Contents

- [Feature Flags](#feature-flags)
- [Authentication & Authorization](#authentication--authorization)
- [Encryption & Cryptography](#encryption--cryptography)
- [Network Security](#network-security)
- [Data Protection](#data-protection)
- [Audit & Compliance](#audit--compliance)
- [Security Configuration](#security-configuration)
- [Best Practices](#best-practices)

## Feature Flags

### Core Security Features

Configure in `Cargo.toml` or via build flags:

```bash
# Basic build with desktop UI
cargo build --features desktop

# Full security-enabled build
cargo build --features "desktop,vectordb,email"

# Server-only build (no desktop UI)
cargo build --no-default-features --features "vectordb,email"
```

### Available Features

| Feature | Purpose | Security Impact | Default |
|---------|---------|-----------------|---------|
| `desktop` | Tauri desktop UI | Sandboxed runtime, controlled system access | ✅ |
| `vectordb` | Qdrant integration | AI-powered threat detection, semantic search | ❌ |
| `email` | IMAP/SMTP support | Requires secure credential storage | ❌ |

### Planned Security Features

Features to be implemented for enterprise deployments:

| Feature | Description | Implementation Status |
|---------|-------------|----------------------|
| `encryption` | Enhanced encryption for data at rest | Built-in via aes-gcm |
| `audit` | Comprehensive audit logging | Planned |
| `rbac` | Role-based access control | In Progress (Zitadel) |
| `mfa` | Multi-factor authentication | Planned |
| `sso` | SAML/OIDC SSO support | Planned |

## Authentication & Authorization

### Zitadel Integration

BotServer uses Zitadel as the primary identity provider:

```rust
// Location: src/auth/zitadel.rs
// Features:
- OAuth2/OIDC authentication
- JWT token validation
- User/group management
- Permission management
- Session handling
```

### Password Security

- **Algorithm**: Argon2id (memory-hard, GPU-resistant)
- **Configuration**: 
  - Memory: 19456 KB
  - Iterations: 2
  - Parallelism: 1
  - Salt: Random 32-byte

### Token Management

- **Access Tokens**: JWT with RS256 signing
- **Refresh Tokens**: Secure random 256-bit
- **Session Tokens**: UUID v4 with cache storage
- **Token Rotation**: Automatic refresh on expiry

## Encryption & Cryptography

### Dependencies

| Library | Version | Purpose | Algorithm |
|---------|---------|---------|-----------|
| `aes-gcm` | 0.10 | Authenticated encryption | AES-256-GCM |
| `argon2` | 0.5 | Password hashing | Argon2id |
| `sha2` | 0.10.9 | Cryptographic hashing | SHA-256 |
| `hmac` | 0.12.1 | Message authentication | HMAC-SHA256 |
| `rand` | 0.9.2 | Cryptographic RNG | ChaCha20 |

### Data Encryption

```rust
// Encryption at rest
- Database: Column-level encryption for sensitive fields
- File storage: AES-256-GCM for uploaded files
- Configuration: Encrypted secrets with master key

// Encryption in transit
- TLS 1.3 for all external communications
- mTLS for service-to-service communication
- Certificate pinning for critical services
```

## Network Security

### API Security

1. **Rate Limiting** (via Caddy)
   - Per-IP: 100 requests/minute
   - Per-user: 1000 requests/hour
   - Configured in Caddyfile

2. **CORS Configuration** (via Caddy)
   ```
   # Strict CORS policy in Caddyfile
   - Origins: Whitelist only
   - Credentials: true for authenticated requests
   - Methods: Explicitly allowed
   ```

3. **Input Validation**
   - Schema validation for all inputs
   - SQL injection prevention via PostgreSQL prepared statements
   - XSS protection with output encoding
   - Path traversal prevention

### WebSocket Security

- Authentication required for connection
- Message size limits (default: 10MB)
- Heartbeat/ping-pong for connection validation
- Automatic disconnection on suspicious activity

## Data Protection

### Database Security

```sql
-- PostgreSQL security features used:
- Row-level security (RLS)
- Column encryption for PII
- Audit logging
- Connection pooling
- Prepared statements only
- SSL/TLS connections enforced
```

### File Storage Security (Drive)

- **Drive Configuration**:
  - Bucket encryption: AES-256
  - Access: Policy-based access control
  - Versioning: Enabled
  - Immutable objects support
  - TLS encryption in transit

- **Local Storage**:
  - Directory permissions: 700
  - File permissions: 600
  - Temporary files: Secure deletion

### Memory Security

```rust
// Memory protection measures
- Zeroization of sensitive data
- No logging of secrets
- Secure random generation
- Protected memory pages for crypto keys
```

## Audit & Compliance

### Logging Configuration

```rust
// Structured logging with tracing
- Level: INFO (production), DEBUG (development)
- Format: JSON for machine parsing
- Rotation: Daily with 30-day retention
- Sensitive data: Redacted
```

### Audit Events

Events automatically logged:

- Authentication attempts
- Authorization failures
- Data access (read/write)
- Configuration changes
- Admin actions
- API calls
- Security violations

### Compliance Support

- **GDPR**: Data deletion, export capabilities
- **SOC2**: Audit trails, access controls
- **HIPAA**: Encryption, access logging (with configuration)
- **PCI DSS**: No credit card storage, tokenization support

## Security Configuration

### Environment Variables

```bash
# Required security settings
BOTSERVER_JWT_SECRET="[256-bit hex string]"
BOTSERVER_ENCRYPTION_KEY="[256-bit hex string]"
DATABASE_ENCRYPTION_KEY="[256-bit hex string]"

# Zitadel (Directory) configuration
ZITADEL_DOMAIN="https://your-instance.zitadel.cloud"
ZITADEL_CLIENT_ID="your-client-id"
ZITADEL_CLIENT_SECRET="your-client-secret"

# Drive configuration
MINIO_ENDPOINT="http://localhost:9000"
MINIO_ACCESS_KEY="minioadmin"
MINIO_SECRET_KEY="minioadmin"
MINIO_USE_SSL=true

# Vector Database configuration
# Configure in your .env file


# Cache configuration
CACHE_URL="redis://localhost:6379"
CACHE_PASSWORD="your-password"

# Optional security enhancements
BOTSERVER_ENABLE_AUDIT=true
BOTSERVER_REQUIRE_MFA=false
BOTSERVER_SESSION_TIMEOUT=3600
BOTSERVER_MAX_LOGIN_ATTEMPTS=5
BOTSERVER_LOCKOUT_DURATION=900

# Network security (Caddy handles TLS automatically)
BOTSERVER_ALLOWED_ORIGINS="https://app.example.com"
BOTSERVER_RATE_LIMIT_PER_IP=100
BOTSERVER_RATE_LIMIT_PER_USER=1000
BOTSERVER_MAX_UPLOAD_SIZE=104857600  # 100MB
```

### Database Configuration

```sql
-- PostgreSQL security settings
-- Add to postgresql.conf:
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
ssl_ciphers = 'HIGH:MEDIUM:+3DES:!aNULL'
ssl_prefer_server_ciphers = on
ssl_ecdh_curve = 'prime256v1'

-- Connection string:
DATABASE_URL="postgres://user:pass@localhost/db?sslmode=require"
```

### Caddy Configuration

```
# Caddyfile for secure reverse proxy
{
    # Global options
    admin off
    auto_https on
}

app.example.com {
    # TLS 1.3 only
    tls {
        protocols tls1.3
        ciphers TLS_AES_256_GCM_SHA384 TLS_CHACHA20_POLY1305_SHA256
    }
    
    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
        Referrer-Policy "strict-origin-when-cross-origin"
        Content-Security-Policy "default-src 'self'"
    }
    
    # Rate limiting
    rate_limit {
        zone static {
            key {remote_host}
            events 100
            window 1m
        }
    }
    
    # Reverse proxy to BotServer
    reverse_proxy localhost:3000 {
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
    
    # Access logging
    log {
        output file /var/log/caddy/access.log
        format json
    }
}
```

## Best Practices

### Development

1. **Dependency Management**
   ```bash
   # Regular security updates
   cargo audit
   cargo update
   
   # Check for known vulnerabilities
   cargo audit --deny warnings
   ```

2. **Code Quality**
   ```rust
   // Enforced via Cargo.toml lints:
   - No unsafe code
   - No unwrap() in production
   - No panic!() macros
   - Complete error handling
   ```

3. **Testing**
   ```bash
   # Security testing suite
   cargo test --features security_tests
   
   # Fuzzing for input validation
   cargo fuzz run api_fuzzer
   ```

### Deployment

1. **Container Security**
   ```bash
   # LXC security configuration
   lxc config set botserver-prod security.privileged=false
   lxc config set botserver-prod security.idmap.isolated=true
   lxc config set botserver-prod security.nesting=false
   
   # Run as non-root user
   lxc exec botserver-prod -- useradd -m botuser
   lxc exec botserver-prod -- su - botuser
   ```

2. **LXD/LXC Container Security**
   ```yaml
   # Container security profile
   config:
     security.nesting: "false"
     security.privileged: "false"
     limits.cpu: "4"
     limits.memory: "8GB"
   devices:
     root:
       path: /
       pool: default
       type: disk
   ```

3. **Network Policies**
   ```
   # Firewall rules (UFW/iptables)
   - Ingress: Only from Caddy proxy
   - Egress: PostgreSQL, Drive, Qdrant, Cache
   - Block: All other traffic
   - Internal: Component isolation
   ```

### Monitoring

1. **Security Metrics**
   - Failed authentication rate
   - Unusual API patterns
   - Resource usage anomalies
   - Geographic access patterns

2. **Alerting Thresholds**
   - 5+ failed logins: Warning
   - 10+ failed logins: Lock account
   - Unusual geographic access: Alert
   - Privilege escalation: Critical alert

3. **Incident Response**
   - Automatic session termination
   - Account lockout procedures
   - Audit log preservation
   - Forensic data collection

## Security Checklist

### Pre-Production

- [ ] All secrets in environment variables
- [ ] Database encryption enabled (PostgreSQL)
- [ ] Drive encryption enabled
- [ ] Caddy TLS configured (automatic with Let's Encrypt)
- [ ] Rate limiting enabled (Caddy)
- [ ] CORS properly configured (Caddy)
- [ ] Audit logging enabled
- [ ] Backup encryption verified
- [ ] Security headers configured (Caddy)
- [ ] Input validation complete
- [ ] Error messages sanitized
- [ ] Zitadel MFA configured
- [ ] Qdrant authentication enabled
- [ ] Valkey password protection enabled

### Production

- [ ] MFA enabled for all admin accounts (Zitadel)
- [ ] Regular security updates scheduled (all components)
- [ ] Monitoring alerts configured
- [ ] Incident response plan documented
- [ ] Regular security audits scheduled
- [ ] Penetration testing completed
- [ ] Compliance requirements met
- [ ] Disaster recovery tested (PostgreSQL, Drive backups)
- [ ] Access reviews scheduled (Zitadel)
- [ ] Security training completed
- [ ] Stalwart email security configured (DKIM, SPF, DMARC)
- [ ] LiveKit secure signaling enabled

## Contact

For security issues or questions:
- Security Email: security@pragmatismo.com.br
- Bug Bounty: See SECURITY.md
- Emergency: Use PGP-encrypted email

## Component Security Documentation

### Core Components
- [Caddy Security](https://caddyserver.com/docs/security) - Reverse proxy and TLS
- [PostgreSQL Security](https://www.postgresql.org/docs/current/security.html) - Database
- [Zitadel Security](https://zitadel.com/docs/guides/manage/security) - Identity and access
- [Drive Security](https://min.io/docs/minio/linux/operations/security.html) - S3-compatible object storage
- [Qdrant Security](https://qdrant.tech/documentation/guides/security/) - Vector database
- [Valkey Security](https://valkey.io/topics/security/) - Cache

### Communication Components
- [Stalwart Security](https://stalw.art/docs/security/) - Email server
- [LiveKit Security](https://docs.livekit.io/realtime/server/security/) - Video conferencing

## References

- [OWASP Top 10](https://owasp.org/Top10/)
- [CIS Controls](https://www.cisecurity.org/controls/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)