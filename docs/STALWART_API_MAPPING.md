# Stalwart API Mapping for General Bots

**Version:** 6.1.0  
**Purpose:** Map Stalwart native features vs General Bots custom tables

---

## Overview

Stalwart Mail Server provides a comprehensive REST Management API. Many email features that we might implement in our database are already available natively in Stalwart. This document maps what to use from Stalwart vs what we manage ourselves.

---

## API Base URL

```
https://{stalwart_host}:{port}/api
```

Default ports:
- HTTP: 8080
- HTTPS: 443

---

## Feature Mapping

### ✅ USE STALWART API (Do NOT create tables)

| Feature | Stalwart Endpoint | Notes |
|---------|------------------|-------|
| **User/Account Management** | `GET/POST/PATCH/DELETE /principal/{id}` | Create, update, delete email accounts |
| **Email Queue** | `GET /queue/messages` | List queued messages for delivery |
| **Queue Status** | `GET /queue/status` | Check if queue is running |
| **Queue Control** | `PATCH /queue/status/start` `PATCH /queue/status/stop` | Start/stop queue processing |
| **Reschedule Delivery** | `PATCH /queue/messages/{id}` | Retry failed deliveries |
| **Cancel Delivery** | `DELETE /queue/messages/{id}` | Cancel queued message |
| **Distribution Lists** | `POST /principal` with `type: "list"` | Mailing lists are "principals" |
| **DKIM Signatures** | `POST /dkim` | Create DKIM keys per domain |
| **DNS Records** | `GET /dns/records/{domain}` | Get required DNS records |
| **Spam Training** | `POST /spam-filter/train/spam` `POST /spam-filter/train/ham` | Train spam filter |
| **Spam Classification** | `POST /spam-filter/classify` | Test spam score |
| **Telemetry/Metrics** | `GET /telemetry/metrics` | Server metrics for monitoring |
| **Live Metrics** | `GET /telemetry/metrics/live` | Real-time metrics (WebSocket) |
| **Logs** | `GET /logs` | Query server logs |
| **Traces** | `GET /telemetry/traces` | Delivery traces |
| **Live Tracing** | `GET /telemetry/traces/live` | Real-time tracing |
| **DMARC Reports** | `GET /reports/dmarc` | Incoming DMARC reports |
| **TLS Reports** | `GET /reports/tls` | TLS-RPT reports |
| **ARF Reports** | `GET /reports/arf` | Abuse feedback reports |
| **Troubleshooting** | `GET /troubleshoot/delivery/{recipient}` | Debug delivery issues |
| **DMARC Check** | `POST /troubleshoot/dmarc` | Test DMARC/SPF/DKIM |
| **Settings** | `GET/POST /settings` | Server configuration |
| **Undelete** | `GET/POST /store/undelete/{account_id}` | Recover deleted messages |
| **Account Purge** | `GET /store/purge/account/{id}` | Purge account data |
| **Encryption Settings** | `GET/POST /account/crypto` | Encryption-at-rest |
| **2FA/App Passwords** | `GET/POST /account/auth` | Authentication settings |

### ⚠️ USE BOTH (Stalwart + Our Tables)

| Feature | Stalwart | Our Table | Why Both? |
|---------|----------|-----------|-----------|
| **Auto-Responders** | Sieve scripts via settings | `email_auto_responders` | We store UI config, sync to Stalwart Sieve |
| **Email Rules/Filters** | Sieve scripts | `email_rules` | We store UI-friendly rules, compile to Sieve |
| **Shared Mailboxes** | Principal with shared access | `shared_mailboxes` | We track permissions, Stalwart handles access |

### ✅ USE OUR TABLES (Stalwart doesn't provide)

| Feature | Our Table | Why? |
|---------|-----------|------|
| **Global Email Signature** | `global_email_signatures` | Bot-level branding, not in Stalwart |
| **User Email Signature** | `email_signatures` | User preferences, append before send |
| **Scheduled Send** | `scheduled_emails` | We queue and release at scheduled time |
| **Email Templates** | `email_templates` | Business templates with variables |
| **Email Labels** | `email_labels`, `email_label_assignments` | UI organization, not IMAP folders |
| **Email Tracking** | `sent_email_tracking` (existing) | Open/click tracking pixels |

---

## Stalwart API Integration Code

### Client Setup

```rust
// src/email/stalwart_client.rs
pub struct StalwartClient {
    base_url: String,
    auth_token: String,
    http_client: reqwest::Client,
}

impl StalwartClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            auth_token: token.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    async fn request<T: DeserializeOwned>(&self, method: Method, path: &str, body: Option<Value>) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http_client.request(method, &url)
            .header("Authorization", format!("Bearer {}", self.auth_token));
        
        if let Some(b) = body {
            req = req.json(&b);
        }
        
        let resp = req.send().await?;
        let data: ApiResponse<T> = resp.json().await?;
        Ok(data.data)
    }
}
```

### Queue Monitoring (for Analytics Dashboard)

```rust
impl StalwartClient {
    /// Get email queue status for monitoring dashboard
    pub async fn get_queue_status(&self) -> Result<QueueStatus> {
        let status: bool = self.request(Method::GET, "/api/queue/status", None).await?;
        let messages: QueueList = self.request(Method::GET, "/api/queue/messages?limit=100", None).await?;
        
        Ok(QueueStatus {
            is_running: status,
            total_queued: messages.total,
            messages: messages.items,
        })
    }

    /// Get queued message details
    pub async fn get_queued_message(&self, message_id: &str) -> Result<QueuedMessage> {
        self.request(Method::GET, &format!("/api/queue/messages/{}", message_id), None).await
    }

    /// Retry failed delivery
    pub async fn retry_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(Method::PATCH, &format!("/api/queue/messages/{}", message_id), None).await
    }

    /// Cancel queued message
    pub async fn cancel_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(Method::DELETE, &format!("/api/queue/messages/{}", message_id), None).await
    }

    /// Stop all queue processing
    pub async fn stop_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/stop", None).await
    }

    /// Resume queue processing
    pub async fn start_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/start", None).await
    }
}
```

### Account/Principal Management

```rust
impl StalwartClient {
    /// Create email account
    pub async fn create_account(&self, email: &str, password: &str, display_name: &str) -> Result<u64> {
        let body = json!({
            "type": "individual",
            "name": email.split('@').next().unwrap_or(email),
            "emails": [email],
            "secrets": [password],
            "description": display_name,
            "quota": 0,
            "roles": ["user"]
        });
        
        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Create distribution list
    pub async fn create_distribution_list(&self, name: &str, email: &str, members: Vec<String>) -> Result<u64> {
        let body = json!({
            "type": "list",
            "name": name,
            "emails": [email],
            "members": members,
            "description": format!("Distribution list: {}", name)
        });
        
        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Get account details
    pub async fn get_account(&self, account_id: &str) -> Result<Principal> {
        self.request(Method::GET, &format!("/api/principal/{}", account_id), None).await
    }

    /// Update account
    pub async fn update_account(&self, account_id: &str, updates: Vec<AccountUpdate>) -> Result<()> {
        let body: Vec<Value> = updates.iter().map(|u| json!({
            "action": u.action,
            "field": u.field,
            "value": u.value
        })).collect();
        
        self.request(Method::PATCH, &format!("/api/principal/{}", account_id), Some(json!(body))).await
    }

    /// Delete account
    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        self.request(Method::DELETE, &format!("/api/principal/{}", account_id), None).await
    }
}
```

### Sieve Rules (Auto-Responders & Filters)

```rust
impl StalwartClient {
    /// Set vacation/out-of-office auto-responder via Sieve
    pub async fn set_auto_responder(&self, account_id: &str, config: &AutoResponderConfig) -> Result<()> {
        let sieve_script = self.generate_vacation_sieve(config);
        
        let updates = vec![json!({
            "type": "set",
            "prefix": format!("sieve.scripts.{}.vacation", account_id),
            "value": sieve_script
        })];
        
        self.request(Method::POST, "/api/settings", Some(json!(updates))).await
    }

    fn generate_vacation_sieve(&self, config: &AutoResponderConfig) -> String {
        let mut script = String::from("require [\"vacation\", \"variables\"];\n\n");
        
        if let Some(start) = &config.start_date {
            script.push_str(&format!("# Active from: {}\n", start));
        }
        if let Some(end) = &config.end_date {
            script.push_str(&format!("# Active until: {}\n", end));
        }
        
        script.push_str(&format!(
            r#"vacation :days 1 :subject "{}" "{}";"#,
            config.subject.replace('"', "\\\""),
            config.body_plain.replace('"', "\\\"")
        ));
        
        script
    }

    /// Set email filter rule via Sieve
    pub async fn set_filter_rule(&self, account_id: &str, rule: &EmailRule) -> Result<()> {
        let sieve_script = self.generate_filter_sieve(rule);
        
        let updates = vec![json!({
            "type": "set",
            "prefix": format!("sieve.scripts.{}.filter_{}", account_id, rule.id),
            "value": sieve_script
        })];
        
        self.request(Method::POST, "/api/settings", Some(json!(updates))).await
    }

    fn generate_filter_sieve(&self, rule: &EmailRule) -> String {
        let mut script = String::from("require [\"fileinto\", \"reject\", \"vacation\"];\n\n");
        
        // Generate conditions
        for condition in &rule.conditions {
            match condition.field.as_str() {
                "from" => script.push_str(&format!(
                    "if header :contains \"From\" \"{}\" {{\n",
                    condition.value
                )),
                "subject" => script.push_str(&format!(
                    "if header :contains \"Subject\" \"{}\" {{\n",
                    condition.value
                )),
                _ => {}
            }
        }
        
        // Generate actions
        for action in &rule.actions {
            match action.action_type.as_str() {
                "move" => script.push_str(&format!("  fileinto \"{}\";\n", action.value)),
                "delete" => script.push_str("  discard;\n"),
                "mark_read" => script.push_str("  setflag \"\\\\Seen\";\n"),
                _ => {}
            }
        }
        
        if rule.stop_processing {
            script.push_str("  stop;\n");
        }
        
        script.push_str("}\n");
        script
    }
}
```

### Telemetry & Monitoring

```rust
impl StalwartClient {
    /// Get server metrics for dashboard
    pub async fn get_metrics(&self) -> Result<Metrics> {
        self.request(Method::GET, "/api/telemetry/metrics", None).await
    }

    /// Get server logs
    pub async fn get_logs(&self, page: u32, limit: u32) -> Result<LogList> {
        self.request(
            Method::GET, 
            &format!("/api/logs?page={}&limit={}", page, limit), 
            None
        ).await
    }

    /// Get delivery traces
    pub async fn get_traces(&self, trace_type: &str, page: u32) -> Result<TraceList> {
        self.request(
            Method::GET,
            &format!("/api/telemetry/traces?type={}&page={}&limit=50", trace_type, page),
            None
        ).await
    }

    /// Get specific trace details
    pub async fn get_trace(&self, trace_id: &str) -> Result<Vec<TraceEvent>> {
        self.request(Method::GET, &format!("/api/telemetry/trace/{}", trace_id), None).await
    }

    /// Get DMARC reports
    pub async fn get_dmarc_reports(&self, page: u32) -> Result<ReportList> {
        self.request(Method::GET, &format!("/api/reports/dmarc?page={}&limit=50", page), None).await
    }

    /// Get TLS reports
    pub async fn get_tls_reports(&self, page: u32) -> Result<ReportList> {
        self.request(Method::GET, &format!("/api/reports/tls?page={}&limit=50", page), None).await
    }
}
```

### Spam Filter

```rust
impl StalwartClient {
    /// Train message as spam
    pub async fn train_spam(&self, raw_message: &str) -> Result<()> {
        self.http_client
            .post(&format!("{}/api/spam-filter/train/spam", self.base_url))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "message/rfc822")
            .body(raw_message.to_string())
            .send()
            .await?;
        Ok(())
    }

    /// Train message as ham (not spam)
    pub async fn train_ham(&self, raw_message: &str) -> Result<()> {
        self.http_client
            .post(&format!("{}/api/spam-filter/train/ham", self.base_url))
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "message/rfc822")
            .body(raw_message.to_string())
            .send()
            .await?;
        Ok(())
    }

    /// Classify message (get spam score)
    pub async fn classify_message(&self, message: &SpamClassifyRequest) -> Result<SpamClassifyResult> {
        self.request(Method::POST, "/api/spam-filter/classify", Some(json!(message))).await
    }
}
```

---

## Monitoring Dashboard Integration

### Endpoints to Poll

| Metric | Endpoint | Poll Interval |
|--------|----------|---------------|
| Queue Size | `GET /queue/messages` | 30s |
| Queue Status | `GET /queue/status` | 30s |
| Server Metrics | `GET /telemetry/metrics` | 60s |
| Recent Logs | `GET /logs?limit=100` | 60s |
| Delivery Traces | `GET /telemetry/traces?type=delivery.attempt-start` | 60s |
| Failed Deliveries | `GET /queue/messages?filter=status:failed` | 60s |

### WebSocket Endpoints (Real-time)

| Feature | Endpoint | Token Endpoint |
|---------|----------|----------------|
| Live Metrics | `ws://.../telemetry/metrics/live` | `GET /telemetry/live/metrics-token` |
| Live Traces | `ws://.../telemetry/traces/live` | `GET /telemetry/live/tracing-token` |

---

## Tables to REMOVE from Migration

Based on this mapping, these tables are **REDUNDANT** and should be removed:

```sql
-- REMOVE: Stalwart handles distribution lists via principals
-- DROP TABLE IF EXISTS distribution_lists;

-- KEEP: We need this for UI config, but sync to Stalwart Sieve
-- email_auto_responders (KEEP but add stalwart_sieve_id column)

-- KEEP: We need this for UI config, but sync to Stalwart Sieve  
-- email_rules (KEEP but add stalwart_sieve_id column)
```

---

## Migration Updates Needed

The current `6.1.0_enterprise_suite` migration already correctly:

1. ✅ Keeps `global_email_signatures` - Stalwart doesn't have this
2. ✅ Keeps `email_signatures` - User preference, not in Stalwart
3. ✅ Keeps `scheduled_emails` - We manage scheduling
4. ✅ Keeps `email_templates` - Business feature
5. ✅ Keeps `email_labels` - UI organization
6. ✅ Has `stalwart_sieve_id` in `email_auto_responders` - For sync
7. ✅ Has `stalwart_sieve_id` in `email_rules` - For sync
8. ✅ Has `stalwart_account_id` in `shared_mailboxes` - For sync

The `distribution_lists` table could potentially be removed since Stalwart handles lists as principals, BUT we may want to keep it for:
- Caching/faster lookups
- UI metadata not stored in Stalwart
- Offline resilience

**Recommendation**: Keep `distribution_lists` but sync with Stalwart principals.

---

## Sync Strategy

### On Create (Our DB → Stalwart)

```rust
async fn create_distribution_list(db: &Pool, stalwart: &StalwartClient, list: NewDistributionList) -> Result<Uuid> {
    // 1. Create in Stalwart first
    let stalwart_id = stalwart.create_distribution_list(
        &list.name,
        &list.email_alias,
        list.members.clone()
    ).await?;
    
    // 2. Store in our DB with stalwart reference
    let id = db.insert_distribution_list(DistributionList {
        name: list.name,
        email_alias: list.email_alias,
        members_json: serde_json::to_string(&list.members)?,
        stalwart_principal_id: Some(stalwart_id.to_string()),
        ..Default::default()
    }).await?;
    
    Ok(id)
}
```

### On Update (Sync both)

```rust
async fn update_distribution_list(db: &Pool, stalwart: &StalwartClient, id: Uuid, updates: ListUpdates) -> Result<()> {
    // 1. Get current record
    let list = db.get_distribution_list(&id).await?;
    
    // 2. Update Stalwart if we have a reference
    if let Some(stalwart_id) = &list.stalwart_principal_id {
        stalwart.update_principal(stalwart_id, updates.to_stalwart_updates()).await?;
    }
    
    // 3. Update our DB
    db.update_distribution_list(&id, updates).await?;
    
    Ok(())
}
```

### On Delete (Both)

```rust
async fn delete_distribution_list(db: &Pool, stalwart: &StalwartClient, id: Uuid) -> Result<()> {
    let list = db.get_distribution_list(&id).await?;
    
    // 1. Delete from Stalwart
    if let Some(stalwart_id) = &list.stalwart_principal_id {
        stalwart.delete_principal(stalwart_id).await?;
    }
    
    // 2. Delete from our DB
    db.delete_distribution_list(&id).await?;
    
    Ok(())
}
```

---

## Summary

| Category | Use Stalwart | Use Our Tables | Use Both |
|----------|-------------|----------------|----------|
| Account Management | ✅ | | |
| Email Queue | ✅ | | |
| Queue Monitoring | ✅ | | |
| Distribution Lists | | | ✅ |
| Auto-Responders | | | ✅ |
| Email Rules/Filters | | | ✅ |
| Shared Mailboxes | | | ✅ |
| Email Signatures | | ✅ | |
| Scheduled Send | | ✅ | |
| Email Templates | | ✅ | |
| Email Labels | | ✅ | |
| Email Tracking | | ✅ | |
| Spam Training | ✅ | | |
| Telemetry/Logs | ✅ | | |
| DMARC/TLS Reports | ✅ | | |