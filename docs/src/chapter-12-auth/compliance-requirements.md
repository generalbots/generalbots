# Compliance Requirements Checklist

## Overview

This document provides a comprehensive checklist for security and compliance requirements across multiple frameworks (GDPR, SOC 2, ISO 27001, HIPAA, LGPD) using the actual components deployed in General Bots.

## Component Stack

| Component | Purpose | License |
|-----------|---------|---------|
| **Caddy** | Reverse proxy, TLS termination, web server | Apache 2.0 |
| **PostgreSQL** | Relational database | PostgreSQL License |
| **General Bots Directory** | Identity and access management (Zitadel/Keycloak) | Apache 2.0 |
| **Drive** | S3-compatible object storage | AGPLv3 |
| **Stalwart** | Mail server (SMTP/IMAP) | AGPLv3 |
| **Qdrant** | Vector database | Apache 2.0 |
| **Cache (Valkey)** | In-memory cache (Redis-compatible) | BSD 3-Clause |
| **LiveKit** | Video conferencing | Apache 2.0 |
| **Ubuntu** | Operating system | Various |

---

## Compliance Requirements Matrix

### Legend
- ✅ = Implemented and configured
- ⚠️ = Partially implemented, needs configuration
- ⬜ = Not yet implemented
- 🔄 = Automated process
- 📝 = Manual process required

---

## Network & Web Server (Caddy)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | TLS 1.3 Configuration | Caddy | All | Automatic TLS 1.3 with modern ciphers |
| ✅ | Access Logging | Caddy | All | JSON format logs to `/var/log/caddy/access.log` |
| ✅ | Rate Limiting | Caddy | ISO 27001 | Per-IP rate limiting in Caddyfile |
| ⚠️ | WAF Rules | Caddy | HIPAA | Consider Caddy security plugins or external WAF |
| ✅ | Security Headers | Caddy | All | HSTS, CSP, X-Frame-Options, X-Content-Type-Options |
| ✅ | Reverse Proxy Security | Caddy | All | Secure forwarding with real IP preservation |
| ✅ | Certificate Management | Caddy | All | Automatic Let's Encrypt with auto-renewal |
| 🔄 | HTTPS Redirect | Caddy | All | Automatic HTTP to HTTPS redirect |

**Configuration File**: `/etc/caddy/Caddyfile`

```
app.example.com {
    tls {
        protocols tls1.3
        ciphers TLS_AES_256_GCM_SHA384
    }
    header {
        Strict-Transport-Security "max-age=31536000"
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        Content-Security-Policy "default-src 'self'"
    }
    rate_limit {
        zone static {
            key {remote_host}
            events 100
            window 1m
        }
    }
    reverse_proxy localhost:3000
}
```

---

## Identity & Access Management (General Bots Directory)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | MFA Implementation | Directory | All | TOTP/SMS/Hardware token support |
| ✅ | RBAC Configuration | Directory | All | Role-based access control with custom roles |
| ✅ | Password Policy | Directory | All | Min 12 chars, complexity requirements, history |
| ✅ | OAuth2/OIDC Setup | Directory | ISO 27001 | OAuth 2.0 and OpenID Connect flows |
| ✅ | Audit Logging | Directory | All | Comprehensive user activity logs |
| ✅ | Session Management | Directory | All | Configurable timeouts and invalidation |
| ✅ | SSO Support | Directory | Enterprise | SAML and OIDC SSO integration |
| ⚠️ | Password Rotation | Directory | HIPAA | Configure 90-day rotation policy |
| 📝 | Access Reviews | Directory | All | Quarterly manual review of user permissions |

**Configuration**: Directory Admin Console (`http://localhost:8080`)

**Key Settings**:
- Password min length: 12 characters
- MFA: Required for admins
- Session timeout: 8 hours
- Idle timeout: 30 minutes

---

## Database (PostgreSQL)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | Encryption at Rest | PostgreSQL | All | File-system level encryption (LUKS) |
| ✅ | Encryption in Transit | PostgreSQL | All | TLS/SSL connections enforced |
| ✅ | Access Control | PostgreSQL | All | Role-based database permissions |
| ✅ | Audit Logging | PostgreSQL | All | pgAudit extension for detailed logging |
| ✅ | Connection Pooling | PostgreSQL | All | Built-in connection management |
| ⚠️ | Row-Level Security | PostgreSQL | HIPAA | Configure RLS policies for sensitive tables |
| ⚠️ | Column Encryption | PostgreSQL | GDPR | Encrypt PII columns with pgcrypto |
| 🔄 | Automated Backups | PostgreSQL | All | Daily backups via pg_dump/pg_basebackup |
| ✅ | Point-in-Time Recovery | PostgreSQL | HIPAA | WAL archiving enabled |

**Configuration**: Installed and configured automatically via installer.rs

```sql
-- Enable SSL
ssl = on
ssl_cert_file = '/path/to/server.crt'
ssl_key_file = '/path/to/server.key'
ssl_ciphers = 'HIGH:MEDIUM:+3DES:!aNULL'

-- Enable audit logging
shared_preload_libraries = 'pgaudit'
pgaudit.log = 'write, ddl'
pgaudit.log_catalog = off

-- Connection settings
max_connections = 100
password_encryption = scram-sha-256

-- Logging
log_connections = on
log_disconnections = on
log_duration = on
log_statement = 'all'
```

---

## Object Storage (Drive)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | Encryption at Rest | Drive | All | Server-side encryption (SSE-S3) |
| ✅ | Encryption in Transit | Drive | All | TLS for all connections |
| ✅ | Bucket Policies | Drive | All | Fine-grained access control policies |
| ✅ | Object Versioning | Drive | HIPAA | Version control for data recovery |
| ✅ | Access Logging | Drive | All | Detailed audit logs for all operations |
| ⚠️ | Lifecycle Rules | Drive | LGPD | Configure data retention and auto-deletion |
| ✅ | Immutable Objects | Drive | Compliance | WORM (Write-Once-Read-Many) support |
| 🔄 | Replication | Drive | HIPAA | Multi-site replication for DR |
| ✅ | IAM Integration | Drive | All | Integration with Directory Service via OIDC |

**Configuration**: `/conf/drive/config.env`

**Bucket Policy Example**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {"AWS": ["arn:aws:iam::*:user/app-user"]},
      "Action": ["s3:GetObject"],
      "Resource": ["arn:aws:s3:::bucket-name/*"]
    }
  ]
}
```

---

## Email Server (Stalwart)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | DKIM Signing | Stalwart | All | Domain key authentication |
| ✅ | SPF Records | Stalwart | All | Sender policy framework |
| ✅ | DMARC Policy | Stalwart | All | Domain-based message authentication |
| ✅ | Mail Encryption | Stalwart | All | TLS for SMTP/IMAP (STARTTLS + implicit) |
| ✅ | Content Filtering | Stalwart | All | Spam and malware filtering |
| ⚠️ | Mail Archiving | Stalwart | HIPAA | Configure long-term email archiving |
| ✅ | Sieve Filtering | Stalwart | All | Server-side mail filtering |
| ✅ | Authentication | Stalwart | All | OIDC integration with Directory Service |
| 📝 | Retention Policy | Stalwart | GDPR/LGPD | Define and implement email retention |

**Configuration**: `/conf/mail/config.toml`

```toml
[server.listener."smtp"]
bind = ["0.0.0.0:25"]
protocol = "smtp"

[server.listener."smtp-submission"]
bind = ["0.0.0.0:587"]
protocol = "smtp"
tls.implicit = false

[server.listener."smtp-submissions"]
bind = ["0.0.0.0:465"]
protocol = "smtp"
tls.implicit = true

[authentication]
mechanisms = ["plain", "login"]
directory = "oidc"

[directory."oidc"]
type = "oidc"
issuer = "http://localhost:8080"
```

**DNS Records**:
```
; SPF Record
example.com. IN TXT "v=spf1 ip4:203.0.113.0/24 -all"

; DKIM Record
default._domainkey.example.com. IN TXT "v=DKIM1; k=rsa; p=MIGfMA0GCS..."

; DMARC Record
_dmarc.example.com. IN TXT "v=DMARC1; p=quarantine; rua=mailto:dmarc@example.com"
```

---

## Cache (Valkey)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | Authentication | Valkey | All | Password-protected access |
| ✅ | TLS Support | Valkey | All | Encrypted connections |
| ✅ | Access Control | Valkey | All | ACL-based permissions |
| ⚠️ | Persistence | Valkey | Data Recovery | RDB/AOF for data persistence |
| ✅ | Memory Limits | Valkey | All | Maxmemory policies configured |
| 📝 | Data Expiration | Valkey | GDPR | Set TTL for cached personal data |

**Configuration**: `/etc/valkey/valkey.conf`

```
# Authentication
requirepass SecurePassword123!

# TLS
tls-port 6380
tls-cert-file /path/to/cert.pem
tls-key-file /path/to/key.pem
tls-protocols "TLSv1.3"

# ACL
aclfile /etc/valkey/users.acl

# Memory management
maxmemory 2gb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
```

---

## Vector Database (Qdrant)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ✅ | API Authentication | Qdrant | All | API key authentication |
| ✅ | TLS Support | Qdrant | All | HTTPS enabled |
| ✅ | Access Control | Qdrant | All | Collection-level permissions |
| ⚠️ | Data Encryption | Qdrant | HIPAA | File-system level encryption |
| 🔄 | Backup Support | Qdrant | All | Snapshot-based backups |
| 📝 | Data Retention | Qdrant | GDPR | Implement collection cleanup policies |

**Configuration**: `/etc/qdrant/config.yaml`

```yaml
service:
  host: 0.0.0.0
  http_port: 6333
  grpc_port: 6334

security:
  api_key: "your-secure-api-key"
  read_only_api_key: "read-only-key"

storage:
  storage_path: /var/lib/qdrant/storage
  snapshots_path: /var/lib/qdrant/snapshots

telemetry:
  enabled: false
```

---

## Operating System (Ubuntu)

| Status | Requirement | Component | Standard | Implementation |
|--------|-------------|-----------|----------|----------------|
| ⚠️ | System Hardening | Ubuntu | All | Apply CIS Ubuntu Linux benchmarks |
| ✅ | Automatic Updates | Ubuntu | All | Unattended-upgrades for security patches |
| ⚠️ | Audit Daemon | Ubuntu | All | Configure auditd for system events |
| ✅ | Firewall Rules | Ubuntu | All | UFW configured with restrictive rules |
| ⚠️ | Disk Encryption | Ubuntu | All | LUKS full-disk encryption |
| ⚠️ | AppArmor | Ubuntu | All | Enable mandatory access control |
| 📝 | User Management | Ubuntu | All | Disable root login, use sudo |
| 📝 | SSH Hardening | Ubuntu | All | Key-based auth only, disable password auth |

**Firewall Configuration**:
```bash
# UFW firewall rules
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw allow 25/tcp    # SMTP
ufw allow 587/tcp   # SMTP submission
ufw allow 993/tcp   # IMAPS
ufw enable
```

**Automatic Updates**:
```bash
# /etc/apt/apt.conf.d/50unattended-upgrades
Unattended-Upgrade::Allowed-Origins {
    "${distro_id}:${distro_codename}-security";
};
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-Time "03:00";
```

**Audit Rules**: `/etc/audit/rules.d/audit.rules`
```
# Monitor authentication
-w /var/log/auth.log -p wa -k auth_log
-w /etc/passwd -p wa -k user_modification
-w /etc/group -p wa -k group_modification

# Monitor network
-a always,exit -F arch=b64 -S connect -k network_connect

# Monitor file access
-w /etc/shadow -p wa -k shadow_modification
```

---

## Cross-Component Requirements

### Monitoring & Logging

| Status | Requirement | Implementation | Standard |
|--------|-------------|----------------|----------|
| ✅ | Centralized Logging | All logs to `/var/log/` with rotation | All |
| ⚠️ | Log Aggregation | ELK Stack or similar SIEM | ISO 27001 |
| ✅ | Health Monitoring | Prometheus + Grafana | All |
| 📝 | Alert Configuration | Set up alerts for security events | All |
| ✅ | Metrics Collection | Component-level metrics | All |

### Backup & Recovery

| Status | Requirement | Implementation | Standard |
|--------|-------------|----------------|----------|
| 🔄 | Automated Backups | Daily automated backups | All |
| ✅ | Backup Encryption | AES-256 encrypted backups | All |
| ✅ | Off-site Storage | Drive replication to secondary site | HIPAA |
| 📝 | Backup Testing | Quarterly restore tests | All |
| ✅ | Retention Policy | 90 days for full, 30 for incremental | All |

**Backup Script**: `/usr/local/bin/backup-system.sh`
```bash
#!/bin/bash
BACKUP_DATE=$(date +%Y%m%d_%H%M%S)

# PostgreSQL backup
pg_dump -h localhost -U postgres generalbots | \
  gzip | \
  openssl enc -aes-256-cbc -salt -out /backup/pg_${BACKUP_DATE}.sql.gz.enc

# Drive backup
mc mirror drive/generalbots /backup/drive_${BACKUP_DATE}/

# Qdrant snapshot
curl -X POST "http://localhost:6333/collections/botserver/snapshots"
```

### Network Security

| Status | Requirement | Implementation | Standard |
|--------|-------------|----------------|----------|
| ✅ | Network Segmentation | Component isolation via firewall | All |
| ✅ | Internal TLS | TLS between all components | ISO 27001 |
| ⚠️ | VPN Access | WireGuard VPN for admin access | All |
| ✅ | Rate Limiting | Caddy rate limiting | All |
| 📝 | DDoS Protection | CloudFlare or similar | Production |

---

## Compliance-Specific Requirements

### GDPR

| Status | Requirement | Implementation |
|--------|-------------|----------------|
| ✅ | Data Encryption | AES-256 at rest, TLS 1.3 in transit |
| ✅ | Right to Access | API endpoints for data export |
| ✅ | Right to Deletion | Data deletion workflows implemented |
| ✅ | Right to Portability | JSON export functionality |
| ✅ | Consent Management | Zitadel consent flows |
| 📝 | Data Processing Records | Document all data processing activities |
| ✅ | Breach Notification | Incident response plan includes 72h notification |

### SOC 2

| Status | Requirement | Implementation |
|--------|-------------|----------------|
| ✅ | Access Controls | RBAC via Zitadel |
| ✅ | Audit Logging | Comprehensive logging across all components |
| ✅ | Change Management | Version control and deployment procedures |
| ✅ | Monitoring | Real-time monitoring with Prometheus |
| 📝 | Risk Assessment | Annual risk assessment required |
| ✅ | Encryption | Data encrypted at rest and in transit |

### ISO 27001

| Status | Requirement | Implementation |
|--------|-------------|----------------|
| ✅ | Asset Inventory | Documented component list |
| ✅ | Access Control | Zitadel RBAC |
| ✅ | Cryptography | Modern encryption standards |
| 📝 | Physical Security | Data center security documentation |
| ✅ | Operations Security | Automated patching and monitoring |
| 📝 | Incident Management | Documented incident response procedures |
| 📝 | Business Continuity | DR plan and testing |

### HIPAA

| Status | Requirement | Implementation |
|--------|-------------|----------------|
| ✅ | Encryption | PHI encrypted at rest and in transit |
| ✅ | Access Controls | Role-based access with MFA |
| ✅ | Audit Controls | Comprehensive audit logging |
| ⚠️ | Integrity Controls | Checksums and versioning |
| ✅ | Transmission Security | TLS 1.3 for all communications |
| 📝 | Business Associate Agreements | Required for third-party vendors |
| ⚠️ | Email Archiving | Stalwart archiving configuration needed |

### LGPD (Brazilian GDPR)

| Status | Requirement | Implementation |
|--------|-------------|----------------|
| ✅ | Data Encryption | Same as GDPR |
| ✅ | User Rights | Same as GDPR |
| ✅ | Consent | Zitadel consent management |
| 📝 | Data Protection Officer | Designate DPO |
| ⚠️ | Data Retention | Configure lifecycle policies in Drive |
| ✅ | Breach Notification | Same incident response as GDPR |

---

## Implementation Priority

### High Priority (Critical for Production)
1. ✅ TLS 1.3 everywhere (Caddy, PostgreSQL, Drive, Stalwart)
2. ✅ MFA for all admin accounts (Zitadel)
3. ✅ Firewall configuration (UFW)
4. ✅ Automated security updates (unattended-upgrades)
5. 🔄 Automated encrypted backups

### Medium Priority (Required for Compliance)
6. ⚠️ Disk encryption (LUKS)
7. ⚠️ Audit daemon (auditd)
8. ⚠️ WAF rules (Caddy plugins or external)
9. 📝 Access reviews (quarterly)
10. ⚠️ Email archiving (Stalwart)

### Lower Priority (Enhanced Security)
11. ⚠️ VPN access (WireGuard)
12. ⚠️ Log aggregation (ELK Stack)
13. ⚠️ AppArmor/SELinux
14. 📝 CIS hardening
15. 📝 Penetration testing

---

## Verification Checklist

### Weekly Tasks
- [ ] Review security logs (Caddy, PostgreSQL, Zitadel)
- [ ] Check backup completion status
- [ ] Review failed authentication attempts
- [ ] Update security patches

### Monthly Tasks
- [ ] Access review for privileged accounts
- [ ] Review audit logs for anomalies
- [ ] Test backup restoration
- [ ] Update vulnerability database

### Quarterly Tasks
- [ ] Full access review for all users
- [ ] Compliance check (run automated checks)
- [ ] Security configuration audit
- [ ] Disaster recovery drill

### Annual Tasks
- [ ] Penetration testing
- [ ] Full compliance audit
- [ ] Risk assessment update
- [ ] Security policy review
- [ ] Business continuity test

---

## Quick Start Implementation

```bash
# 1. Enable firewall
sudo ufw enable
sudo ufw allow 22,80,443,25,587,993/tcp

# 2. Configure automatic updates
sudo apt install unattended-upgrades
sudo dpkg-reconfigure --priority=low unattended-upgrades

# 3. Enable PostgreSQL SSL
sudo -u postgres psql -c "ALTER SYSTEM SET ssl = 'on';"
sudo systemctl restart postgresql

# 4. Set Drive encryption
mc admin config set drive/ server-side-encryption-s3 on

# 5. Configure Zitadel MFA
# Via web console: Settings > Security > MFA > Require for admins

# 6. Enable Caddy security headers
# Add to Caddyfile (see Network & Web Server section)

# 7. Set up daily backups
sudo crontab -e
# Add: 0 2 * * * /usr/local/bin/backup-system.sh
```

---

## Support & Resources

- **Internal Security Team**: security@pragmatismo.com.br
- **Compliance Officer**: compliance@pragmatismo.com.br
- **Documentation**: https://docs.pragmatismo.com.br
- **Component Documentation**: See "Component Security Documentation" in security-features.md

---

## Document Control

- **Version**: 1.0
- **Last Updated**: 2024-01-15
- **Next Review**: 2024-07-15
- **Owner**: Security Team
- **Approved By**: CTO
