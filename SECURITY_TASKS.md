# Security Audit Tasks - botserver

**Priority:** CRITICAL
**Auditor Focus:** Rust Security Best Practices

---

## 🔴 CRITICAL - Fix Immediately

### 1. Remove All `.unwrap()` Calls (403 occurrences)

```bash
grep -rn "unwrap()" src --include="*.rs" | wc -l
# Result: 403
```

**Action:** Replace every `.unwrap()` with:
- `?` operator for propagating errors
- `.unwrap_or_default()` for safe defaults
- `.ok_or_else(|| Error::...)?` for custom errors

**Files with highest count:**
```bash
grep -rn "unwrap()" src --include="*.rs" -c | sort -t: -k2 -rn | head -20
```

---

### 2. Remove All `.expect()` Calls (76 occurrences)

```bash
grep -rn "\.expect(" src --include="*.rs" | wc -l
# Result: 76
```

**Action:** Same as unwrap - use `?` or proper error handling.

---

### 3. SQL Injection Vectors - Dynamic Query Building

**Location:** Multiple files build SQL with `format!`

```
src/basic/keywords/db_api.rs:168 - format!("SELECT COUNT(*) as count FROM {}", table_name)
src/basic/keywords/db_api.rs:603 - format!("DELETE FROM {} WHERE id = $1", table_name)
src/basic/keywords/db_api.rs:665 - format!("SELECT COUNT(*) as count FROM {}", table_name)
```

**Action:**
- Validate `table_name` against whitelist of allowed tables
- Use parameterized queries exclusively
- Add schema validation before query execution

---

### 4. Command Injection Risk - External Process Execution

**Locations:**
```
src/security/antivirus.rs - Command::new("powershell")
src/core/kb/document_processor.rs - Command::new("pdftotext"), Command::new("pandoc")
src/core/bot/manager.rs - Command::new("mc")
src/nvidia/mod.rs - Command::new("nvidia-smi")
```

**Action:**
- Never pass user input to command arguments
- Use absolute paths for executables
- Validate/sanitize all inputs before shell execution
- Consider sandboxing or containerization

---

## 🟠 HIGH - Fix This Sprint

### 5. Secrets in Memory

**Concern:** API keys, passwords, tokens may persist in memory

**Action:**
- Use `secrecy` crate for sensitive data (`SecretString`, `SecretVec`)
- Implement `Zeroize` trait for structs holding secrets
- Clear secrets from memory after use

---

### 6. Missing Input Validation on API Endpoints

**Action:** Add validation for all handler inputs:
- Length limits on strings
- Range checks on numbers
- Format validation (emails, URLs, UUIDs)
- Use `validator` crate with derive macros

---

### 7. Rate Limiting Missing

**Action:**
- Add rate limiting middleware to all public endpoints
- Implement per-IP and per-user limits
- Use `tower-governor` or similar

---

### 8. Missing Authentication Checks

**Action:** Audit all handlers for:
- Session validation
- Permission checks (RBAC)
- Bot ownership verification

---

### 9. CORS Configuration Review

**Action:**
- Restrict allowed origins (no wildcard `*` in production)
- Validate Origin header
- Set appropriate headers

---

### 10. File Path Traversal

**Locations:** File serving, upload handlers

**Action:**
- Canonicalize paths before use
- Validate paths are within allowed directories
- Use `sanitize_path_component` consistently

---

## 🟡 MEDIUM - Fix Next Sprint

### 11. Logging Sensitive Data

**Action:**
- Audit all `log::*` calls for sensitive data
- Never log passwords, tokens, API keys
- Redact PII in logs

---

### 12. Error Message Information Disclosure

**Action:**
- Return generic errors to clients
- Log detailed errors server-side only
- Never expose stack traces to users

---

### 13. Cryptographic Review

**Action:**
- Verify TLS 1.3 minimum
- Check certificate validation
- Review encryption algorithms used
- Ensure secure random number generation (`rand::rngs::OsRng`)

---

### 14. Dependency Audit

```bash
cargo audit
cargo deny check
```

**Action:**
- Fix all reported vulnerabilities
- Remove unused dependencies
- Pin versions in Cargo.lock

---

### 15. TODO/FIXME Comments (Security-Related)

```
src/auto_task/autotask_api.rs:1829 - TODO: Fetch from database
src/auto_task/autotask_api.rs:1849 - TODO: Implement recommendation
```

**Action:** Complete or remove all TODO comments.

---

## 🟢 LOW - Backlog

### 16. Add Security Headers

- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy`
- `Strict-Transport-Security`

### 17. Implement Request ID Tracking

- Add unique ID to each request
- Include in logs for tracing

### 18. Database Connection Pool Hardening

- Set max connections
- Implement connection timeouts
- Add health checks

### 19. Add Panic Handler

- Catch panics at boundaries
- Log and return 500, don't crash

### 20. Memory Limits

- Set max request body size
- Limit file upload sizes
- Implement streaming for large files

---

## Verification Commands

```bash
# Check for unwrap
grep -rn "unwrap()" src --include="*.rs" | wc -l

# Check for expect
grep -rn "\.expect(" src --include="*.rs" | wc -l

# Check for panic
grep -rn "panic!" src --include="*.rs" | wc -l

# Check for unsafe
grep -rn "unsafe" src --include="*.rs"

# Check for SQL injection vectors
grep -rn "format!.*SELECT\|format!.*INSERT\|format!.*UPDATE\|format!.*DELETE" src --include="*.rs"

# Check for command execution
grep -rn "Command::new\|std::process::Command" src --include="*.rs"

# Run security audit
cargo audit

# Check dependencies
cargo deny check
```

---

## Acceptance Criteria

- [ ] 0 `.unwrap()` calls in production code (tests excluded)
- [ ] 0 `.expect()` calls in production code
- [ ] 0 `panic!` macros
- [ ] 0 `unsafe` blocks (or documented justification)
- [ ] All SQL uses parameterized queries
- [ ] All external commands validated
- [ ] `cargo audit` shows 0 vulnerabilities
- [ ] Rate limiting on all public endpoints
- [ ] Input validation on all handlers
- [ ] Secrets use `secrecy` crate