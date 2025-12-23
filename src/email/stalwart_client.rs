//! Stalwart Mail Server API Client
//!
//! This module provides a comprehensive client for interacting with the Stalwart Mail Server
//! Management API. It handles queue monitoring, account/principal management, Sieve script
//! generation for auto-responders and filters, telemetry/monitoring, and spam filter training.
//!
//! # Version: 6.1.0
//!
//! # Usage
//!
//! ```rust,ignore
//! let client = StalwartClient::new("https://mail.example.com", "api-token");
//!
//! // Get queue status
//! let status = client.get_queue_status().await?;
//!
//! // Create an account
//! let account_id = client.create_account("user@example.com", "password", "John Doe").await?;
//! ```

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Configuration

/// Default timeout for API requests in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default poll interval for queue monitoring in seconds
pub const DEFAULT_QUEUE_POLL_INTERVAL_SECS: u64 = 30;

/// Default poll interval for metrics in seconds
pub const DEFAULT_METRICS_POLL_INTERVAL_SECS: u64 = 60;

// Data Types - Queue Monitoring

/// Represents the overall queue status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// Whether the queue processor is running
    pub is_running: bool,
    /// Total number of messages in the queue
    pub total_queued: u64,
    /// List of queued messages (up to limit)
    pub messages: Vec<QueuedMessage>,
}

/// A message in the delivery queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    /// Unique message identifier
    pub id: String,
    /// Sender email address
    pub from: String,
    /// Recipient email addresses
    pub to: Vec<String>,
    /// Message subject (if available)
    #[serde(default)]
    pub subject: Option<String>,
    /// Current delivery status
    pub status: DeliveryStatus,
    /// Number of delivery attempts
    #[serde(default)]
    pub attempts: u32,
    /// Next scheduled delivery attempt
    #[serde(default)]
    pub next_retry: Option<DateTime<Utc>>,
    /// Last error message (if any)
    #[serde(default)]
    pub last_error: Option<String>,
    /// Message size in bytes
    #[serde(default)]
    pub size: u64,
    /// When the message was queued
    #[serde(default)]
    pub queued_at: Option<DateTime<Utc>>,
}

/// Delivery status for a queued message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Scheduled,
    InProgress,
    Failed,
    Deferred,
    #[serde(other)]
    Unknown,
}

/// Response from queue list endpoint
#[derive(Debug, Clone, Deserialize)]
struct QueueListResponse {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    items: Vec<QueuedMessage>,
}

// Data Types - Principal/Account Management

/// Types of principals in Stalwart
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalType {
    Individual,
    Group,
    List,
    Resource,
    Location,
    Superuser,
    #[serde(other)]
    Other,
}

/// A principal (user, group, list, etc.) in Stalwart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Principal ID
    pub id: Option<u64>,
    /// Principal type
    #[serde(rename = "type")]
    pub principal_type: PrincipalType,
    /// Username/identifier
    pub name: String,
    /// Email addresses associated with this principal
    #[serde(default)]
    pub emails: Vec<String>,
    /// Display name/description
    #[serde(default)]
    pub description: Option<String>,
    /// Quota in bytes (0 = unlimited)
    #[serde(default)]
    pub quota: u64,
    /// Roles assigned to this principal
    #[serde(default)]
    pub roles: Vec<String>,
    /// Member principals (for groups/lists)
    #[serde(default)]
    pub members: Vec<String>,
    /// Whether the account is disabled
    #[serde(default)]
    pub disabled: bool,
}

/// Update action for principal modifications
#[derive(Debug, Clone, Serialize)]
pub struct AccountUpdate {
    /// Action type: "set", "addItem", "removeItem", "clear"
    pub action: String,
    /// Field to update
    pub field: String,
    /// New value
    pub value: Value,
}

impl AccountUpdate {
    /// Create a "set" update
    pub fn set(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "set".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }

    /// Create an "addItem" update for array fields
    pub fn add_item(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "addItem".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }

    /// Create a "removeItem" update for array fields
    pub fn remove_item(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "removeItem".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }

    /// Create a "clear" update
    pub fn clear(field: &str) -> Self {
        Self {
            action: "clear".to_string(),
            field: field.to_string(),
            value: Value::Null,
        }
    }
}

// Data Types - Auto-Responder & Email Rules

/// Configuration for an auto-responder (out of office)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResponderConfig {
    /// Whether the auto-responder is enabled
    pub enabled: bool,
    /// Subject line for the auto-response
    pub subject: String,
    /// Plain text body
    pub body_plain: String,
    /// HTML body (optional)
    #[serde(default)]
    pub body_html: Option<String>,
    /// Start date (optional)
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    /// End date (optional)
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    /// Only respond to addresses in this list (empty = all)
    #[serde(default)]
    pub only_contacts: bool,
    /// Days between responses to the same sender
    #[serde(default = "default_vacation_days")]
    pub vacation_days: u32,
}

fn default_vacation_days() -> u32 {
    1
}

impl Default for AutoResponderConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            subject: "Out of Office".to_string(),
            body_plain: "I am currently out of the office and will respond upon my return.".to_string(),
            body_html: None,
            start_date: None,
            end_date: None,
            only_contacts: false,
            vacation_days: 1,
        }
    }
}

/// An email filtering rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable rule name
    pub name: String,
    /// Rule priority (lower = higher priority)
    #[serde(default)]
    pub priority: i32,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Conditions that must match
    pub conditions: Vec<RuleCondition>,
    /// Actions to perform when conditions match
    pub actions: Vec<RuleAction>,
    /// Whether to stop processing further rules after this one
    #[serde(default = "default_stop_processing")]
    pub stop_processing: bool,
}

fn default_stop_processing() -> bool {
    true
}

/// A condition for an email rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Field to match: "from", "to", "cc", "subject", "body", "header"
    pub field: String,
    /// Match operator: "contains", "equals", "startsWith", "endsWith", "regex", "notContains"
    pub operator: String,
    /// Value to match against
    pub value: String,
    /// Header name (only used when field = "header")
    #[serde(default)]
    pub header_name: Option<String>,
    /// Whether the match is case-sensitive
    #[serde(default)]
    pub case_sensitive: bool,
}

/// An action for an email rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    /// Action type: "move", "copy", "delete", "mark_read", "mark_flagged", "forward", "reply", "reject"
    pub action_type: String,
    /// Action value (folder name for move/copy, email for forward, etc.)
    #[serde(default)]
    pub value: String,
}

// Data Types - Telemetry & Monitoring

/// Server metrics from Stalwart
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metrics {
    /// Total messages received
    #[serde(default)]
    pub messages_received: u64,
    /// Total messages delivered
    #[serde(default)]
    pub messages_delivered: u64,
    /// Total messages rejected
    #[serde(default)]
    pub messages_rejected: u64,
    /// Current queue size
    #[serde(default)]
    pub queue_size: u64,
    /// Active SMTP connections
    #[serde(default)]
    pub smtp_connections: u64,
    /// Active IMAP connections
    #[serde(default)]
    pub imap_connections: u64,
    /// Server uptime in seconds
    #[serde(default)]
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    #[serde(default)]
    pub memory_used: u64,
    /// CPU usage percentage
    #[serde(default)]
    pub cpu_usage: f64,
    /// Additional metrics as key-value pairs
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

/// A log entry from Stalwart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Log level: "trace", "debug", "info", "warn", "error"
    pub level: String,
    /// Log component/module
    #[serde(default)]
    pub component: Option<String>,
    /// Log message
    pub message: String,
    /// Additional context
    #[serde(default)]
    pub context: Option<Value>,
}

/// List of log entries with pagination info
#[derive(Debug, Clone, Deserialize)]
pub struct LogList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<LogEntry>,
}

/// A delivery trace event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: String,
    /// Related message ID
    #[serde(default)]
    pub message_id: Option<String>,
    /// Sender
    #[serde(default)]
    pub from: Option<String>,
    /// Recipients
    #[serde(default)]
    pub to: Vec<String>,
    /// Remote host
    #[serde(default)]
    pub remote_host: Option<String>,
    /// Result/status
    #[serde(default)]
    pub result: Option<String>,
    /// Error message (if any)
    #[serde(default)]
    pub error: Option<String>,
    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

/// List of trace events
#[derive(Debug, Clone, Deserialize)]
pub struct TraceList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<TraceEvent>,
}

// Data Types - Reports

/// A DMARC/TLS/ARF report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report ID
    pub id: String,
    /// Report type: "dmarc", "tls", "arf"
    pub report_type: String,
    /// Domain the report is about
    pub domain: String,
    /// Reporter organization
    #[serde(default)]
    pub reporter: Option<String>,
    /// Report date range start
    #[serde(default)]
    pub date_start: Option<DateTime<Utc>>,
    /// Report date range end
    #[serde(default)]
    pub date_end: Option<DateTime<Utc>>,
    /// Report data (structure varies by type)
    pub data: Value,
}

/// List of reports
#[derive(Debug, Clone, Deserialize)]
pub struct ReportList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<Report>,
}

// Data Types - Spam Filter

/// Request to classify a message for spam
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamClassifyRequest {
    /// Sender email address
    pub from: String,
    /// Recipient email addresses
    pub to: Vec<String>,
    /// Sender's IP address (optional)
    #[serde(default)]
    pub remote_ip: Option<String>,
    /// EHLO/HELO hostname (optional)
    #[serde(default)]
    pub ehlo_host: Option<String>,
    /// Raw message headers (optional)
    #[serde(default)]
    pub headers: Option<String>,
    /// Message body (optional)
    #[serde(default)]
    pub body: Option<String>,
}

/// Result of spam classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamClassifyResult {
    /// Spam score (higher = more likely spam)
    pub score: f64,
    /// Classification: "spam", "ham", "unknown"
    pub classification: String,
    /// Individual test results
    #[serde(default)]
    pub tests: Vec<SpamTest>,
    /// Recommended action: "accept", "reject", "quarantine"
    #[serde(default)]
    pub action: Option<String>,
}

/// Individual spam test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamTest {
    /// Test name
    pub name: String,
    /// Test score contribution
    pub score: f64,
    /// Test description
    #[serde(default)]
    pub description: Option<String>,
}

// API Response Wrapper

/// Generic API response wrapper
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success { data: T },
    SuccessDirect(T),
    Error { error: String },
}

// Stalwart Client Implementation

/// Client for interacting with Stalwart Mail Server's Management API
#[derive(Debug, Clone)]
pub struct StalwartClient {
    base_url: String,
    auth_token: String,
    http_client: Client,
}

impl StalwartClient {
    /// Create a new Stalwart client
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the Stalwart server (e.g., "https://mail.example.com")
    /// * `token` - API authentication token
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = StalwartClient::new("https://mail.example.com", "api-token");
    /// ```
    pub fn new(base_url: &str, token: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: token.to_string(),
            http_client,
        }
    }

    /// Create a new Stalwart client with custom timeout
    pub fn with_timeout(base_url: &str, token: &str, timeout_secs: u64) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: token.to_string(),
            http_client,
        }
    }

    /// Make an authenticated request to the Stalwart API
    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        debug!("Stalwart API request: {} {}", method, url);

        let mut req = self
            .http_client
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Accept", "application/json");

        if let Some(b) = &body {
            req = req.header("Content-Type", "application/json").json(b);
        }

        let resp = req.send().await.context("Failed to send request to Stalwart")?;
        let status = resp.status();

        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Stalwart API error: {} - {}", status, error_text);
            return Err(anyhow!("Stalwart API error ({}): {}", status, error_text));
        }

        let text = resp.text().await.context("Failed to read response body")?;

        // Handle empty responses
        if text.is_empty() || text == "null" {
            // Try to return a default value for types that support it
            return serde_json::from_str("null")
                .or_else(|_| serde_json::from_str("{}"))
                .or_else(|_| serde_json::from_str("true"))
                .context("Empty response from Stalwart API");
        }

        serde_json::from_str(&text).context("Failed to parse Stalwart API response")
    }

    /// Make a request that returns raw bytes (for spam training)
    async fn request_raw(&self, method: Method, path: &str, body: &str, content_type: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        debug!("Stalwart API raw request: {} {}", method, url);

        let resp = self
            .http_client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", content_type)
            .body(body.to_string())
            .send()
            .await
            .context("Failed to send request to Stalwart")?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Stalwart API error ({}): {}", status, error_text));
        }

        Ok(())
    }

    // ========================================================================
    // Queue Monitoring
    // ========================================================================

    /// Get comprehensive queue status including all queued messages
    pub async fn get_queue_status(&self) -> Result<QueueStatus> {
        let status: bool = self
            .request(Method::GET, "/api/queue/status", None)
            .await
            .unwrap_or(false);

        let messages_resp: QueueListResponse = self
            .request(Method::GET, "/api/queue/messages?limit=100", None)
            .await
            .unwrap_or(QueueListResponse {
                total: 0,
                items: vec![],
            });

        Ok(QueueStatus {
            is_running: status,
            total_queued: messages_resp.total,
            messages: messages_resp.items,
        })
    }

    /// Get details of a specific queued message
    pub async fn get_queued_message(&self, message_id: &str) -> Result<QueuedMessage> {
        self.request(
            Method::GET,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }

    /// List queued messages with filters
    pub async fn list_queued_messages(
        &self,
        limit: u32,
        offset: u32,
        status_filter: Option<&str>,
    ) -> Result<QueueListResponse> {
        let mut path = format!("/api/queue/messages?limit={}&offset={}", limit, offset);
        if let Some(status) = status_filter {
            path.push_str(&format!("&filter=status:{}", status));
        }
        self.request(Method::GET, &path, None).await
    }

    /// Retry delivery of a failed message
    pub async fn retry_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(
            Method::PATCH,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }

    /// Cancel a queued message
    pub async fn cancel_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(
            Method::DELETE,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }

    /// Stop queue processing
    pub async fn stop_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/stop", None).await
    }

    /// Start/resume queue processing
    pub async fn start_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/start", None).await
    }

    /// Get count of failed deliveries
    pub async fn get_failed_delivery_count(&self) -> Result<u64> {
        let resp: QueueListResponse = self
            .request(
                Method::GET,
                "/api/queue/messages?filter=status:failed&limit=1",
                None,
            )
            .await?;
        Ok(resp.total)
    }

    // ========================================================================
    // Account/Principal Management
    // ========================================================================

    /// Create a new email account
    pub async fn create_account(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<u64> {
        let username = email.split('@').next().unwrap_or(email);

        let body = json!({
            "type": "individual",
            "name": username,
            "emails": [email],
            "secrets": [password],
            "description": display_name,
            "quota": 0,
            "roles": ["user"]
        });

        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Create a new email account with custom settings
    pub async fn create_account_full(&self, principal: &Principal, password: &str) -> Result<u64> {
        let mut body = serde_json::to_value(principal)?;
        if let Some(obj) = body.as_object_mut() {
            obj.insert("secrets".to_string(), json!([password]));
        }
        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Create a distribution list (mailing list)
    pub async fn create_distribution_list(
        &self,
        name: &str,
        email: &str,
        members: Vec<String>,
    ) -> Result<u64> {
        let body = json!({
            "type": "list",
            "name": name,
            "emails": [email],
            "members": members,
            "description": format!("Distribution list: {}", name)
        });

        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Create a shared mailbox
    pub async fn create_shared_mailbox(
        &self,
        name: &str,
        email: &str,
        members: Vec<String>,
    ) -> Result<u64> {
        let body = json!({
            "type": "group",
            "name": name,
            "emails": [email],
            "members": members,
            "description": format!("Shared mailbox: {}", name)
        });

        self.request(Method::POST, "/api/principal", Some(body)).await
    }

    /// Get account/principal details
    pub async fn get_account(&self, account_id: &str) -> Result<Principal> {
        self.request(
            Method::GET,
            &format!("/api/principal/{}", account_id),
            None,
        )
        .await
    }

    /// Get account by email address
    pub async fn get_account_by_email(&self, email: &str) -> Result<Principal> {
        self.request(
            Method::GET,
            &format!("/api/principal?filter=emails:{}", email),
            None,
        )
        .await
    }

    /// Update account properties
    pub async fn update_account(&self, account_id: &str, updates: Vec<AccountUpdate>) -> Result<()> {
        let body: Vec<Value> = updates
            .iter()
            .map(|u| {
                json!({
                    "action": u.action,
                    "field": u.field,
                    "value": u.value
                })
            })
            .collect();

        self.request::<Value>(
            Method::PATCH,
            &format!("/api/principal/{}", account_id),
            Some(json!(body)),
        )
        .await?;
        Ok(())
    }

    /// Delete an account/principal
    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        self.request::<Value>(
            Method::DELETE,
            &format!("/api/principal/{}", account_id),
            None,
        )
        .await?;
        Ok(())
    }

    /// List all principals of a specific type
    pub async fn list_principals(&self, principal_type: Option<PrincipalType>) -> Result<Vec<Principal>> {
        let path = match principal_type {
            Some(t) => format!("/api/principal?type={:?}", t).to_lowercase(),
            None => "/api/principal".to_string(),
        };
        self.request(Method::GET, &path, None).await
    }

    /// Add members to a distribution list or group
    pub async fn add_members(&self, account_id: &str, members: Vec<String>) -> Result<()> {
        let updates: Vec<AccountUpdate> = members
            .into_iter()
            .map(|m| AccountUpdate::add_item("members", m))
            .collect();
        self.update_account(account_id, updates).await
    }

    /// Remove members from a distribution list or group
    pub async fn remove_members(&self, account_id: &str, members: Vec<String>) -> Result<()> {
        let updates: Vec<AccountUpdate> = members
            .into_iter()
            .map(|m| AccountUpdate::remove_item("members", m))
            .collect();
        self.update_account(account_id, updates).await
    }

    // ========================================================================
    // Sieve Rules (Auto-Responders & Filters)
    // ========================================================================

    /// Set vacation/out-of-office auto-responder via Sieve script
    pub async fn set_auto_responder(
        &self,
        account_id: &str,
        config: &AutoResponderConfig,
    ) -> Result<String> {
        let sieve_script = self.generate_vacation_sieve(config);
        let script_id = format!("{}_vacation", account_id);

        let updates = vec![json!({
            "type": "set",
            "prefix": format!("sieve.scripts.{}", script_id),
            "value": sieve_script
        })];

        self.request::<Value>(Method::POST, "/api/settings", Some(json!(updates)))
            .await?;

        info!("Set auto-responder for account {}", account_id);
        Ok(script_id)
    }

    /// Disable auto-responder for an account
    pub async fn disable_auto_responder(&self, account_id: &str) -> Result<()> {
        let script_id = format!("{}_vacation", account_id);

        let updates = vec![json!({
            "type": "clear",
            "prefix": format!("sieve.scripts.{}", script_id)
        })];

        self.request::<Value>(Method::POST, "/api/settings", Some(json!(updates)))
            .await?;

        info!("Disabled auto-responder for account {}", account_id);
        Ok(())
    }

    /// Generate Sieve script for vacation auto-responder
    pub fn generate_vacation_sieve(&self, config: &AutoResponderConfig) -> String {
        let mut script = String::from("require [\"vacation\", \"variables\", \"date\", \"relational\"];\n\n");

        // Add date checks if start/end dates are specified
        if config.start_date.is_some() || config.end_date.is_some() {
            script.push_str("# Date-based activation\n");

            if let Some(start) = &config.start_date {
                script.push_str(&format!(
                    "if currentdate :value \"lt\" \"date\" \"{}\" {{ stop; }}\n",
                    start.format("%Y-%m-%d")
                ));
            }

            if let Some(end) = &config.end_date {
                script.push_str(&format!(
                    "if currentdate :value \"gt\" \"date\" \"{}\" {{ stop; }}\n",
                    end.format("%Y-%m-%d")
                ));
            }

            script.push('\n');
        }

        // Main vacation action
        let subject = config.subject.replace('"', "\\\"").replace('\n', " ");
        let body = config.body_plain.replace('"', "\\\"").replace('\n', "\\n");

        script.push_str(&format!(
            "vacation :days {} :subject \"{}\" \"{}\";\n",
            config.vacation_days, subject, body
        ));

        script
    }

    /// Set email filter rule via Sieve script
    pub async fn set_filter_rule(&self, account_id: &str, rule: &EmailRule) -> Result<String> {
        let sieve_script = self.generate_filter_sieve(rule);
        let script_id = format!("{}_filter_{}", account_id, rule.id);

        let updates = vec![json!({
            "type": "set",
            "prefix": format!("sieve.scripts.{}", script_id),
            "value": sieve_script
        })];

        self.request::<Value>(Method::POST, "/api/settings", Some(json!(updates)))
            .await?;

        info!("Set filter rule '{}' for account {}", rule.name, account_id);
        Ok(script_id)
    }

    /// Delete a filter rule
    pub async fn delete_filter_rule(&self, account_id: &str, rule_id: &str) -> Result<()> {
        let script_id = format!("{}_filter_{}", account_id, rule_id);

        let updates = vec![json!({
            "type": "clear",
            "prefix": format!("sieve.scripts.{}", script_id)
        })];

        self.request::<Value>(Method::POST, "/api/settings", Some(json!(updates)))
            .await?;

        info!("Deleted filter rule {} for account {}", rule_id, account_id);
        Ok(())
    }

    /// Generate Sieve script for an email filter rule
    pub fn generate_filter_sieve(&self, rule: &EmailRule) -> String {
        let mut script =
            String::from("require [\"fileinto\", \"reject\", \"vacation\", \"imap4flags\", \"copy\"];\n\n");

        script.push_str(&format!("# Rule: {}\n", rule.name));

        if !rule.enabled {
            script.push_str("# DISABLED\n");
            return script;
        }

        // Generate conditions
        let mut conditions = Vec::new();
        for condition in &rule.conditions {
            let cond_str = self.generate_condition_sieve(condition);
            if !cond_str.is_empty() {
                conditions.push(cond_str);
            }
        }

        if conditions.is_empty() {
            // No conditions means always match
            script.push_str("# Always applies\n");
        } else {
            // Combine conditions with allof (AND)
            script.push_str(&format!("if allof ({}) {{\n", conditions.join(", ")));
        }

        // Generate actions
        for action in &rule.actions {
            let action_str = self.generate_action_sieve(action);
            if !action_str.is_empty() {
                if conditions.is_empty() {
                    script.push_str(&format!("{}\n", action_str));
                } else {
                    script.push_str(&format!("    {}\n", action_str));
                }
            }
        }

        // Stop processing if configured
        if rule.stop_processing {
            if conditions.is_empty() {
                script.push_str("stop;\n");
            } else {
                script.push_str("    stop;\n");
            }
        }

        // Close the if block
        if !conditions.is_empty() {
            script.push_str("}\n");
        }

        script
    }

    /// Generate Sieve condition string
    pub fn generate_condition_sieve(&self, condition: &RuleCondition) -> String {
        let field_header = match condition.field.as_str() {
            "from" => "From",
            "to" => "To",
            "cc" => "Cc",
            "subject" => "Subject",
            "header" => condition.header_name.as_deref().unwrap_or("X-Custom"),
            _ => return String::new(),
        };

        let comparator = if condition.case_sensitive {
            ""
        } else {
            " :comparator \"i;ascii-casemap\""
        };

        let value = condition.value.replace('"', "\\\"");

        match condition.operator.as_str() {
            "contains" => format!("header :contains{} \"{}\" \"{}\"", comparator, field_header, value),
            "equals" => format!("header :is{} \"{}\" \"{}\"", comparator, field_header, value),
            "startsWith" => format!("header :matches{} \"{}\" \"{}*\"", comparator, field_header, value),
            "endsWith" => format!("header :matches{} \"{}\" \"*{}\"", comparator, field_header, value),
            "regex" => format!("header :regex{} \"{}\" \"{}\"", comparator, field_header, value),
            "notContains" => format!("not header :contains{} \"{}\" \"{}\"", comparator, field_header, value),
            _ => String::new(),
        }
    }

    /// Generate Sieve action string
    pub fn generate_action_sieve(&self, action: &RuleAction) -> String {
        match action.action_type.as_str() {
            "move" => format!("fileinto \"{}\";", action.value.replace('"', "\\\"")),
            "copy" => format!("fileinto :copy \"{}\";", action.value.replace('"', "\\\"")),
            "delete" => "discard;".to_string(),
            "mark_read" => "setflag \"\\\\Seen\";".to_string(),
            "mark_flagged" => "setflag \"\\\\Flagged\";".to_string(),
            "forward" => format!("redirect \"{}\";", action.value.replace('"', "\\\"")),
            "reject" => format!("reject \"{}\";", action.value.replace('"', "\\\"")),
            _ => String::new(),
        }
    }

    // ========================================================================
    // Telemetry & Monitoring
    // ========================================================================

    /// Get server metrics
    pub async fn get_metrics(&self) -> Result<Metrics> {
        self.request(Method::GET, "/api/telemetry/metrics", None).await
    }

    /// Get server logs with pagination
    pub async fn get_logs(&self, page: u32, limit: u32) -> Result<LogList> {
        self.request(
            Method::GET,
            &format!("/api/logs?page={}&limit={}", page, limit),
            None,
        )
        .await
    }

    /// Get server logs with level filter
    pub async fn get_logs_by_level(&self, level: &str, page: u32, limit: u32) -> Result<LogList> {
        self.request(
            Method::GET,
            &format!("/api/logs?level={}&page={}&limit={}", level, page, limit),
            None,
        )
        .await
    }

    /// Get delivery traces
    pub async fn get_traces(&self, trace_type: &str, page: u32) -> Result<TraceList> {
        self.request(
            Method::GET,
            &format!(
                "/api/telemetry/traces?type={}&page={}&limit=50",
                trace_type, page
            ),
            None,
        )
        .await
    }

    /// Get all recent traces
    pub async fn get_recent_traces(&self, limit: u32) -> Result<TraceList> {
        self.request(
            Method::GET,
            &format!("/api/telemetry/traces?limit={}", limit),
            None,
        )
        .await
    }

    /// Get specific trace details
    pub async fn get_trace(&self, trace_id: &str) -> Result<Vec<TraceEvent>> {
        self.request(
            Method::GET,
            &format!("/api/telemetry/trace/{}", trace_id),
            None,
        )
        .await
    }

    /// Get DMARC reports
    pub async fn get_dmarc_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/dmarc?page={}&limit=50", page),
            None,
        )
        .await
    }

    /// Get TLS reports
    pub async fn get_tls_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/tls?page={}&limit=50", page),
            None,
        )
        .await
    }

    /// Get ARF (Abuse Reporting Format) reports
    pub async fn get_arf_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/arf?page={}&limit=50", page),
            None,
        )
        .await
    }

    /// Get a token for WebSocket live metrics connection
    pub async fn get_live_metrics_token(&self) -> Result<String> {
        self.request(Method::GET, "/api/telemetry/live/metrics-token", None)
            .await
    }

    /// Get a token for WebSocket live tracing connection
    pub async fn get_live_tracing_token(&self) -> Result<String> {
        self.request(Method::GET, "/api/telemetry/live/tracing-token", None)
            .await
    }

    // ========================================================================
    // Spam Filter
    // ========================================================================

    /// Train message as spam
    pub async fn train_spam(&self, raw_message: &str) -> Result<()> {
        self.request_raw(
            Method::POST,
            "/api/spam-filter/train/spam",
            raw_message,
            "message/rfc822",
        )
        .await?;
        info!("Trained message as spam");
        Ok(())
    }

    /// Train message as ham (not spam)
    pub async fn train_ham(&self, raw_message: &str) -> Result<()> {
        self.request_raw(
            Method::POST,
            "/api/spam-filter/train/ham",
            raw_message,
            "message/rfc822",
        )
        .await?;
        info!("Trained message as ham");
        Ok(())
    }

    /// Classify a message (check spam score)
    pub async fn classify_message(&self, message: &SpamClassifyRequest) -> Result<SpamClassifyResult> {
        self.request(
            Method::POST,
            "/api/spam-filter/classify",
            Some(serde_json::to_value(message)?),
        )
        .await
    }

    // ========================================================================
    // Troubleshooting & Utilities
    // ========================================================================

    /// Troubleshoot delivery issues for a recipient
    pub async fn troubleshoot_delivery(&self, recipient: &str) -> Result<Value> {
        self.request(
            Method::GET,
            &format!("/api/troubleshoot/delivery/{}", urlencoding::encode(recipient)),
            None,
        )
        .await
    }

    /// Test DMARC/SPF/DKIM for a domain
    pub async fn check_dmarc(&self, domain: &str, from_email: &str) -> Result<Value> {
        let body = json!({
            "domain": domain,
            "from": from_email
        });
        self.request(Method::POST, "/api/troubleshoot/dmarc", Some(body))
            .await
    }

    /// Get required DNS records for a domain
    pub async fn get_dns_records(&self, domain: &str) -> Result<Value> {
        self.request(
            Method::GET,
            &format!("/api/dns/records/{}", domain),
            None,
        )
        .await
    }

    /// Recover deleted messages for an account
    pub async fn undelete_messages(&self, account_id: &str) -> Result<Value> {
        self.request(
            Method::POST,
            &format!("/api/store/undelete/{}", account_id),
            None,
        )
        .await
    }

    /// Purge all data for an account
    pub async fn purge_account(&self, account_id: &str) -> Result<()> {
        self.request::<Value>(
            Method::GET,
            &format!("/api/store/purge/account/{}", account_id),
            None,
        )
        .await?;
        warn!("Purged all data for account {}", account_id);
        Ok(())
    }

    /// Test connection to Stalwart server
    pub async fn health_check(&self) -> Result<bool> {
        match self.request::<Value>(Method::GET, "/api/queue/status", None).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Stalwart health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

// Tests
