# Security Auditing

Regular security audits ensure your botserver installation remains protected against known vulnerabilities. This guide covers automated scanning, manual reviews, and best practices.

---

## Rust Dependency Auditing

### cargo-audit

botserver uses `cargo-audit` to scan Rust dependencies for known vulnerabilities.

**Install cargo-audit:**

```bash
cargo install cargo-audit
```

**Run audit:**

```bash
cd botserver
cargo audit
```

**Expected output (clean):**

```
    Fetching advisory database from `https://github.com/RustSec/advisory-db`
      Loaded 650 security advisories (from ~/.cargo/advisory-db)
    Scanning Cargo.lock for vulnerabilities (425 crate dependencies)
```

**Output with vulnerabilities:**

```
Crate:     openssl
Version:   0.10.38
Title:     `openssl` `X509NameRef::entries` is unsound
Date:      2023-11-23
ID:        RUSTSEC-2023-0072
URL:       https://rustsec.org/advisories/RUSTSEC-2023-0072
Severity:  medium
Solution:  Upgrade to >=0.10.60
```

### Automated CI/CD Auditing

Add to your CI pipeline (`.github/workflows/security.yml`):

```yaml
name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Strict Auditing

Fail on any warning:

```bash
cargo audit --deny warnings
```

Fail on unmaintained crates:

```bash
cargo audit --deny unmaintained
```

Generate JSON report:

```bash
cargo audit --json > audit-report.json
```

---

## Stack Component Vulnerabilities

### CVE Monitoring

Monitor security advisories for each component:

| Component | Security Feed |
|-----------|---------------|
| PostgreSQL | [postgresql.org/support/security](https://www.postgresql.org/support/security/) |
| Vault | [security.hashicorp.com](https://www.hashicorp.com/security) |
| MinIO | [github.com/minio/minio/security](https://github.com/minio/minio/security/advisories) |
| Zitadel | [github.com/zitadel/zitadel/security](https://github.com/zitadel/zitadel/security/advisories) |
| llama.cpp | [github.com/ggml-org/llama.cpp/security](https://github.com/ggml-org/llama.cpp/security/advisories) |
| Valkey | [github.com/valkey-io/valkey/security](https://github.com/valkey-io/valkey/security/advisories) |
| Caddy | [github.com/caddyserver/caddy/security](https://github.com/caddyserver/caddy/security/advisories) |
| Stalwart | [github.com/stalwartlabs/mail-server/security](https://github.com/stalwartlabs/mail-server/security/advisories) |

### Trivy Container Scanning

If using containers, scan with Trivy:

```bash
# Install Trivy
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

# Scan filesystem
trivy fs --security-checks vuln,config ./botserver-stack/

# Scan specific binary
trivy fs --security-checks vuln ./botserver-stack/bin/vault/
```

### Grype Binary Scanning

Scan binaries for vulnerabilities:

```bash
# Install Grype
curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin

# Scan directory
grype dir:./botserver-stack/bin/
```

---

## Network Security Audit

### Port Scanning

Verify only expected ports are open:

```bash
# Local port check
ss -tlnp | grep LISTEN

# Expected ports
# 8200  - Vault
# 5432  - PostgreSQL
# 8080  - Zitadel / API
# 9000  - MinIO API
# 9001  - MinIO Console
# 6379  - Valkey
# 8081  - LLM Server
# 8082  - Embedding Server
# 443   - HTTPS Proxy
# 53    - DNS
```

External port scan:

```bash
nmap -sT -p- localhost
```

### TLS Certificate Audit

Check certificate validity:

```bash
# Check expiration
openssl x509 -in botserver-stack/conf/system/certificates/api/server.crt -noout -dates

# Check certificate chain
openssl verify -CAfile botserver-stack/conf/system/certificates/ca/ca.crt \
    botserver-stack/conf/system/certificates/api/server.crt
```

### Firewall Rules

Ensure proper firewall configuration:

```bash
# UFW (Ubuntu)
sudo ufw status verbose

# iptables
sudo iptables -L -n -v
```

Recommended rules:

```bash
# Allow only necessary ports
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 8080/tcp  # API (if exposed)
```

---

## Secrets Audit

### Vault Health Check

```bash
# Check Vault seal status
curl -s http://localhost:8200/v1/sys/seal-status | jq

# List enabled auth methods
VAULT_ADDR=http://localhost:8200 vault auth list

# Audit enabled secrets engines
VAULT_ADDR=http://localhost:8200 vault secrets list
```

### Environment Variable Audit

Check for leaked secrets:

```bash
# Search for hardcoded secrets
grep -r "password" --include="*.toml" --include="*.json" --include="*.csv" .
grep -r "secret" --include="*.toml" --include="*.json" --include="*.csv" .
grep -r "api_key" --include="*.toml" --include="*.json" --include="*.csv" .

# Check .env file permissions
ls -la .env
# Should be: -rw------- (600)
```

### Rotate Secrets

Regular rotation schedule:

```bash
# Generate new database password
./botserver rotate-secret tables

# Generate new drive credentials
./botserver rotate-secret drive

# Rotate all secrets
./botserver rotate-secrets --all
```

---

## Code Security Analysis

### Static Analysis with Clippy

```bash
# Run Clippy with all lints
cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery

# Security-focused lints
cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used
```

### SAST with Semgrep

```bash
# Install Semgrep
pip install semgrep

# Run Rust security rules
semgrep --config p/rust .

# Run all security rules
semgrep --config p/security-audit .
```

### Dependency Review

Check for outdated dependencies:

```bash
# List outdated crates
cargo outdated

# Check for yanked crates
cargo audit --deny yanked
```

---

## Database Security

### PostgreSQL Audit

```bash
# Check authentication settings
cat botserver-stack/conf/tables/pg_hba.conf

# Verify SSL is enabled
psql $DATABASE_URL -c "SHOW ssl;"

# Check user permissions
psql $DATABASE_URL -c "SELECT * FROM pg_roles WHERE rolname NOT LIKE 'pg_%';"
```

### Connection Security

Ensure encrypted connections:

```sql
-- Check current connections
SELECT datname, usename, ssl, client_addr 
FROM pg_stat_ssl 
JOIN pg_stat_activity ON pg_stat_ssl.pid = pg_stat_activity.pid;
```

---

## Compliance Checks

### OWASP Top 10

| Risk | Mitigation | Status Check |
|------|------------|--------------|
| Injection | Parameterized queries | `grep -r "raw_sql" src/` |
| Broken Auth | Zitadel handles auth | Check Zitadel config |
| Sensitive Data | Vault encryption | `vault status` |
| XXE | No XML parsing | N/A |
| Broken Access | RBAC via Zitadel | Check permissions |
| Security Misconfig | Audit configs | Review `conf/` |
| XSS | Template escaping | Askama auto-escapes |
| Insecure Deserialization | Serde validation | Code review |
| Vulnerable Components | `cargo audit` | Automated |
| Logging | Structured logs | Check log config |

### SOC 2 Checklist

- [ ] Access controls documented
- [ ] Encryption at rest enabled
- [ ] Encryption in transit (TLS)
- [ ] Audit logging enabled
- [ ] Backup procedures documented
- [ ] Incident response plan
- [ ] Vulnerability management process

---

## Audit Schedule

| Audit Type | Frequency | Tool |
|------------|-----------|------|
| Dependency vulnerabilities | Daily (CI) | cargo-audit |
| Container scanning | Weekly | Trivy |
| Secret rotation | Monthly | Vault |
| Port scanning | Monthly | nmap |
| Full security review | Quarterly | Manual |
| Penetration testing | Annually | External |

---

## Automated Security Script

Create `security-audit.sh`:

```bash
#!/bin/bash
set -e

echo "=== botserver Security Audit ==="
echo "Date: $(date)"
echo

echo "--- Rust Dependency Audit ---"
cargo audit --deny warnings || echo "WARN: Vulnerabilities found"

echo
echo "--- Checking for Hardcoded Secrets ---"
if grep -r "password.*=" --include="*.rs" src/ 2>/dev/null | grep -v "fn\|let\|//"; then
    echo "WARN: Potential hardcoded passwords found"
fi

echo
echo "--- Port Scan ---"
ss -tlnp | grep LISTEN

echo
echo "--- Certificate Expiry ---"
for cert in botserver-stack/conf/system/certificates/*/server.crt; do
    if [ -f "$cert" ]; then
        expiry=$(openssl x509 -in "$cert" -noout -enddate 2>/dev/null | cut -d= -f2)
        echo "$cert: $expiry"
    fi
done

echo
echo "--- Vault Status ---"
curl -s http://localhost:8200/v1/sys/seal-status 2>/dev/null | jq -r '.sealed' || echo "Vault not running"

echo
echo "=== Audit Complete ==="
```

Run periodically:

```bash
chmod +x security-audit.sh
./security-audit.sh > audit-$(date +%Y%m%d).log
```

---

## Reporting Vulnerabilities

If you discover a security vulnerability in botserver:

1. **Do NOT** create a public GitHub issue
2. Email security@generalbots.ai with details
3. Include steps to reproduce
4. Allow 90 days for fix before disclosure

---

## See Also

- [Secrets Management](../10-configuration-deployment/secrets-management.md) - Vault configuration
- [Updating Components](./updating-components.md) - Applying security updates
- [Backup and Recovery](./backup-recovery.md) - Data protection