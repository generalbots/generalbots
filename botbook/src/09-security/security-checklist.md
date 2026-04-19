# Security Review Checklist for SaaS Deployment

This checklist covers critical security considerations before deploying General Bots as a multi-tenant SaaS platform.

## Pre-Deployment Security Audit

### 1. Authentication & Authorization

- [ ] **Password Security**
  - [ ] Argon2id hashing with secure parameters
  - [ ] Minimum password length enforced (12+ characters)
  - [ ] Password complexity requirements enabled
  - [ ] Breached password checking enabled

- [ ] **Session Management**
  - [ ] Cryptographically secure session tokens (256-bit)
  - [ ] Session timeout configured (default: 1 hour idle)
  - [ ] Session revocation on password change
  - [ ] Concurrent session limits per user

- [ ] **Multi-Factor Authentication**
  - [ ] TOTP support enabled for admin accounts
  - [ ] MFA enforcement for privileged operations
  - [ ] Recovery codes securely generated and stored

- [ ] **OAuth2/OIDC**
  - [ ] State parameter validation
  - [ ] PKCE enforcement for public clients
  - [ ] Token rotation enabled
  - [ ] Redirect URI validation (exact match)

### 2. Rate Limiting & Resource Protection

- [ ] **API Rate Limits** (from `botlib::limits`)
  - [ ] Per-user limits: 1,000 requests/minute
  - [ ] Per-user limits: 10,000 requests/hour
  - [ ] Global limits prevent platform exhaustion
  - [ ] HTTP 429 responses with `Retry-After` header

- [ ] **Script Execution Limits**
  - [ ] Loop iteration limit: 100,000
  - [ ] Script timeout: 300 seconds
  - [ ] Recursion depth limit: 100
  - [ ] String length limit: 10 MB

- [ ] **File & Upload Limits**
  - [ ] Max file size: 100 MB
  - [ ] Max upload size: 50 MB
  - [ ] Max request body: 10 MB
  - [ ] File type validation enabled

- [ ] **Connection Limits**
  - [ ] Max concurrent requests per user: 100
  - [ ] Max WebSocket connections per user: 10
  - [ ] Database connection pooling configured

### 3. Input Validation & Injection Prevention

- [ ] **SQL Injection**
  - [ ] All queries use parameterized statements (Diesel ORM)
  - [ ] Dynamic table names sanitized via `sanitize_identifier()`
  - [ ] No raw SQL string concatenation

- [ ] **Cross-Site Scripting (XSS)**
  - [ ] HTML output properly escaped
  - [ ] Content-Security-Policy headers configured
  - [ ] X-Content-Type-Options: nosniff

- [ ] **Path Traversal**
  - [ ] File paths sanitized (no `..` allowed)
  - [ ] Operations restricted to tenant's `.gbdrive` scope
  - [ ] Symbolic links not followed

- [ ] **Command Injection**
  - [ ] No shell command execution from user input
  - [ ] BASIC scripts sandboxed in Rhai runtime

### 4. Data Protection

- [ ] **Encryption at Rest**
  - [ ] Database encryption enabled
  - [ ] Object storage (MinIO) encryption enabled
  - [ ] Secrets encrypted with AES-GCM

- [ ] **Encryption in Transit**
  - [ ] TLS 1.2+ required for all connections
  - [ ] HTTPS enforced (no HTTP fallback)
  - [ ] Internal service communication encrypted

- [ ] **Secrets Management**
  - [ ] API keys stored in environment variables
  - [ ] No hardcoded credentials in code
  - [ ] Secrets rotated regularly
  - [ ] `.env` files excluded from version control

- [ ] **Data Isolation**
  - [ ] Multi-tenant data separation verified
  - [ ] User cannot access other tenants' data
  - [ ] Bot-level isolation enforced

### 5. API Security

- [ ] **URL Constants** (from `ApiUrls`)
  - [ ] All routes use constants from `core/urls.rs`
  - [ ] No hardcoded `/api/...` strings in route definitions
  - [ ] URL parameters properly validated

- [ ] **Request Validation**
  - [ ] Content-Type validation
  - [ ] Request size limits enforced
  - [ ] Malformed JSON rejected

- [ ] **Response Security**
  - [ ] No sensitive data in error messages
  - [ ] Stack traces disabled in production
  - [ ] Consistent error response format

### 6. Infrastructure Security

- [ ] **Network Security**
  - [ ] Firewall rules configured
  - [ ] Internal services not exposed
  - [ ] Database not publicly accessible

- [ ] **Container Security**
  - [ ] Non-root container users
  - [ ] Read-only filesystem where possible
  - [ ] Resource limits (CPU, memory) configured

- [ ] **Logging & Monitoring**
  - [ ] Authentication events logged
  - [ ] Rate limit violations logged
  - [ ] Error rates monitored
  - [ ] Logs do not contain sensitive data (passwords, tokens)

### 7. LLM & AI Security

- [ ] **Prompt Injection Prevention**
  - [ ] System prompts protected
  - [ ] User input properly delimited
  - [ ] Output validation enabled

- [ ] **Token Limits**
  - [ ] Max tokens per request: 128,000
  - [ ] LLM requests rate limited: 60/minute
  - [ ] Cost monitoring enabled

- [ ] **Data Privacy**
  - [ ] No PII sent to external LLM APIs (if applicable)
  - [ ] Conversation data retention policy defined
  - [ ] User consent obtained

### 8. Compliance

- [ ] **GDPR** (EU)
  - [ ] Data processing agreements in place
  - [ ] Right to deletion implemented
  - [ ] Data export capability available
  - [ ] Privacy policy published

- [ ] **LGPD** (Brazil)
  - [ ] Legal basis for processing documented
  - [ ] Data protection officer designated
  - [ ] Breach notification process defined

- [ ] **SOC 2** (Enterprise)
  - [ ] Access controls documented
  - [ ] Change management process
  - [ ] Incident response plan

## Deployment Verification

### Pre-Production Testing

```bash
# Run security-focused tests
cargo test --all

# Check for memory issues
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Verify rate limiting
curl -X POST http://localhost:9000/api/test \
  -H "Content-Type: application/json" \
  --data '{}' \
  --parallel --parallel-max 1000

# Expected: HTTP 429 after limit exceeded
```

### Production Hardening

```bash
# Verify TLS configuration
openssl s_client -connect your-domain.com:443 -tls1_2

# Check security headers
curl -I https://your-domain.com

# Expected headers:
# Strict-Transport-Security: max-age=31536000
# X-Content-Type-Options: nosniff
# X-Frame-Options: DENY
# Content-Security-Policy: default-src 'self'
```

## Incident Response

### In Case of Security Incident

1. **Contain**: Disable affected accounts/services
2. **Investigate**: Review logs, identify scope
3. **Notify**: Inform affected users within 72 hours (GDPR)
4. **Remediate**: Fix vulnerability, rotate credentials
5. **Document**: Create incident report

### Emergency Contacts

- Security Team: security@your-domain.com
- Infrastructure: ops@your-domain.com
- Legal/Compliance: legal@your-domain.com

## Regular Security Tasks

| Frequency | Task |
|-----------|------|
| Daily | Review authentication failure logs |
| Weekly | Check rate limit violations |
| Monthly | Rotate API keys and secrets |
| Quarterly | Dependency vulnerability scan |
| Annually | Full security audit |

## See Also

- [System Limits](./system-limits.md) - Resource constraints
- [Security Features](./security-features.md) - Implementation details
- [Compliance Requirements](./compliance-requirements.md) - Regulatory requirements
- [Security Policy](./security-policy.md) - Organizational policies