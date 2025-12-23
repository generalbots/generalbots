//! Human-in-the-Loop Approvals
//!
//! This module provides human approval workflows that pause script execution
//! until a human approves, rejects, or modifies a pending action. It enables:
//!
//! - Multi-channel approval requests (email, SMS, Teams, mobile push)
//! - Timeout handling with default actions
//! - Approval chains for multi-level authorization
//! - Audit logging of all approval decisions
//!
//! ## BASIC Keywords
//!
//! ```basic
//! ' Request approval via email
//! HEAR approval ON EMAIL "manager@company.com"
//!
//! ' Request approval via mobile push
//! HEAR approval ON MOBILE "+1-555-0100"
//!
//! ' Request approval via Teams channel
//! HEAR approval ON TEAMS "approvals-channel"
//!
//! ' With timeout and default action
//! HEAR approval ON EMAIL "manager@company.com" TIMEOUT 3600 DEFAULT "auto-approve"
//!
//! ' Multi-level approval chain
//! APPROVAL CHAIN "expense-approval"
//!     LEVEL 1 ON EMAIL "supervisor@company.com" TIMEOUT 3600
//!     LEVEL 2 ON EMAIL "director@company.com" TIMEOUT 7200 IF amount > 10000
//!     LEVEL 3 ON EMAIL "vp@company.com" TIMEOUT 14400 IF amount > 50000
//! END APPROVAL CHAIN
//!
//! ' Request approval with context
//! SET APPROVAL CONTEXT "action", "expense_report"
//! SET APPROVAL CONTEXT "amount", 15000
//! SET APPROVAL CONTEXT "description", "Q4 Marketing Campaign"
//! result = REQUEST APPROVAL "expense-approval"
//!
//! ' Check approval status
//! status = GET APPROVAL STATUS requestId
//! ```
//!
//! ## Config.csv Properties
//!
//! ```csv
//! name,value
//! approval-enabled,true
//! approval-default-timeout,3600
//! approval-reminder-interval,1800
//! approval-max-reminders,3
//! approval-audit-enabled,true
//! approval-webhook-url,https://webhook.example.com/approvals
//! ```

use chrono::{DateTime, Duration, Utc};
use rhai::{Array, Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

/// Approval request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request identifier
    pub id: Uuid,
    /// Bot ID that initiated the request
    pub bot_id: Uuid,
    /// Session ID where the request originated
    pub session_id: Uuid,
    /// User ID who triggered the approval
    pub initiated_by: Uuid,
    /// Type of approval being requested
    pub approval_type: String,
    /// Current status
    pub status: ApprovalStatus,
    /// Channel for sending the request
    pub channel: ApprovalChannel,
    /// Recipient identifier (email, phone, channel name)
    pub recipient: String,
    /// Context data for the approval
    pub context: serde_json::Value,
    /// Message shown to approver
    pub message: String,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Default action if timeout
    pub default_action: Option<ApprovalDecision>,
    /// Current level in approval chain (1-indexed)
    pub current_level: u32,
    /// Total levels in approval chain
    pub total_levels: u32,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// When the request expires
    pub expires_at: DateTime<Utc>,
    /// When reminders were sent
    pub reminders_sent: Vec<DateTime<Utc>>,
    /// The final decision
    pub decision: Option<ApprovalDecision>,
    /// Who made the decision
    pub decided_by: Option<String>,
    /// When the decision was made
    pub decided_at: Option<DateTime<Utc>>,
    /// Comments from the approver
    pub comments: Option<String>,
}

/// Approval status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    /// Waiting for approver response
    Pending,
    /// Approved by approver
    Approved,
    /// Rejected by approver
    Rejected,
    /// Timed out, using default action
    TimedOut,
    /// Cancelled by requester
    Cancelled,
    /// Escalated to next level
    Escalated,
    /// Error occurred
    Error,
}

impl Default for ApprovalStatus {
    fn default() -> Self {
        ApprovalStatus::Pending
    }
}

/// Approval decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    Approve,
    Reject,
    Escalate,
    Defer,
    RequestInfo,
}

/// Channel for sending approval requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalChannel {
    Email,
    Sms,
    Mobile,
    Teams,
    Slack,
    Webhook,
    InApp,
}

impl Default for ApprovalChannel {
    fn default() -> Self {
        ApprovalChannel::Email
    }
}

impl std::fmt::Display for ApprovalChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalChannel::Email => write!(f, "email"),
            ApprovalChannel::Sms => write!(f, "sms"),
            ApprovalChannel::Mobile => write!(f, "mobile"),
            ApprovalChannel::Teams => write!(f, "teams"),
            ApprovalChannel::Slack => write!(f, "slack"),
            ApprovalChannel::Webhook => write!(f, "webhook"),
            ApprovalChannel::InApp => write!(f, "in_app"),
        }
    }
}

/// Approval chain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalChain {
    /// Chain name/identifier
    pub name: String,
    /// Bot ID this chain belongs to
    pub bot_id: Uuid,
    /// Levels in the chain
    pub levels: Vec<ApprovalLevel>,
    /// Whether to stop on first rejection
    pub stop_on_reject: bool,
    /// Whether all levels must approve
    pub require_all: bool,
    /// Description of the chain
    pub description: Option<String>,
    /// When the chain was created
    pub created_at: DateTime<Utc>,
}

/// Single level in an approval chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    /// Level number (1-indexed)
    pub level: u32,
    /// Channel to use
    pub channel: ApprovalChannel,
    /// Recipient identifier
    pub recipient: String,
    /// Timeout for this level
    pub timeout_seconds: u64,
    /// Condition for this level (evaluated at runtime)
    pub condition: Option<String>,
    /// Whether this level can be skipped
    pub skippable: bool,
    /// Approvers at this level (for group approvals)
    pub approvers: Vec<String>,
    /// Required number of approvals (for group)
    pub required_approvals: u32,
}

/// Approval audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAuditEntry {
    /// Audit entry ID
    pub id: Uuid,
    /// Related approval request ID
    pub request_id: Uuid,
    /// Action that occurred
    pub action: AuditAction,
    /// Who performed the action
    pub actor: String,
    /// Details of the action
    pub details: serde_json::Value,
    /// When the action occurred
    pub timestamp: DateTime<Utc>,
    /// IP address if available
    pub ip_address: Option<String>,
    /// User agent if available
    pub user_agent: Option<String>,
}

/// Actions tracked in audit log
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    RequestCreated,
    NotificationSent,
    ReminderSent,
    Approved,
    Rejected,
    Escalated,
    TimedOut,
    Cancelled,
    CommentAdded,
    ContextUpdated,
}

/// Configuration for approval system
#[derive(Debug, Clone)]
pub struct ApprovalConfig {
    /// Whether approvals are enabled
    pub enabled: bool,
    /// Default timeout in seconds
    pub default_timeout: u64,
    /// Interval between reminders
    pub reminder_interval: u64,
    /// Maximum reminders to send
    pub max_reminders: u32,
    /// Whether to audit all actions
    pub audit_enabled: bool,
    /// Webhook URL for external notifications
    pub webhook_url: Option<String>,
    /// Email template for approval requests
    pub email_template: Option<String>,
    /// Base URL for approval links
    pub approval_base_url: Option<String>,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        ApprovalConfig {
            enabled: true,
            default_timeout: 3600,
            reminder_interval: 1800,
            max_reminders: 3,
            audit_enabled: true,
            webhook_url: None,
            email_template: None,
            approval_base_url: None,
        }
    }
}

/// Approval Manager
#[derive(Debug)]
pub struct ApprovalManager {
    config: ApprovalConfig,
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new(config: ApprovalConfig) -> Self {
        ApprovalManager { config }
    }

    /// Create from config map
    pub fn from_config(config_map: &HashMap<String, String>) -> Self {
        let config = ApprovalConfig {
            enabled: config_map
                .get("approval-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            default_timeout: config_map
                .get("approval-default-timeout")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            reminder_interval: config_map
                .get("approval-reminder-interval")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1800),
            max_reminders: config_map
                .get("approval-max-reminders")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            audit_enabled: config_map
                .get("approval-audit-enabled")
                .map(|v| v == "true")
                .unwrap_or(true),
            webhook_url: config_map.get("approval-webhook-url").cloned(),
            email_template: config_map.get("approval-email-template").cloned(),
            approval_base_url: config_map.get("approval-base-url").cloned(),
        };
        ApprovalManager::new(config)
    }

    /// Create a new approval request
    pub fn create_request(
        &self,
        bot_id: Uuid,
        session_id: Uuid,
        initiated_by: Uuid,
        approval_type: &str,
        channel: ApprovalChannel,
        recipient: &str,
        context: serde_json::Value,
        message: &str,
        timeout_seconds: Option<u64>,
        default_action: Option<ApprovalDecision>,
    ) -> ApprovalRequest {
        let timeout = timeout_seconds.unwrap_or(self.config.default_timeout);
        let now = Utc::now();

        ApprovalRequest {
            id: Uuid::new_v4(),
            bot_id,
            session_id,
            initiated_by,
            approval_type: approval_type.to_string(),
            status: ApprovalStatus::Pending,
            channel,
            recipient: recipient.to_string(),
            context,
            message: message.to_string(),
            timeout_seconds: timeout,
            default_action,
            current_level: 1,
            total_levels: 1,
            created_at: now,
            expires_at: now + Duration::seconds(timeout as i64),
            reminders_sent: Vec::new(),
            decision: None,
            decided_by: None,
            decided_at: None,
            comments: None,
        }
    }

    /// Check if a request has expired
    pub fn is_expired(&self, request: &ApprovalRequest) -> bool {
        Utc::now() > request.expires_at
    }

    /// Check if a reminder should be sent
    pub fn should_send_reminder(&self, request: &ApprovalRequest) -> bool {
        if request.status != ApprovalStatus::Pending {
            return false;
        }

        if request.reminders_sent.len() >= self.config.max_reminders as usize {
            return false;
        }

        let last_notification = request
            .reminders_sent
            .last()
            .copied()
            .unwrap_or(request.created_at);

        let since_last = Utc::now() - last_notification;
        since_last.num_seconds() >= self.config.reminder_interval as i64
    }

    /// Generate approval URL
    pub fn generate_approval_url(&self, request_id: Uuid, action: &str, token: &str) -> String {
        let base_url = self
            .config
            .approval_base_url
            .as_deref()
            .unwrap_or("https://bot.example.com/approve");

        format!(
            "{}/{}?action={}&token={}",
            base_url, request_id, action, token
        )
    }

    /// Generate email content for approval request
    pub fn generate_email_content(&self, request: &ApprovalRequest, token: &str) -> EmailContent {
        let approve_url = self.generate_approval_url(request.id, "approve", token);
        let reject_url = self.generate_approval_url(request.id, "reject", token);

        let subject = format!(
            "Approval Required: {} ({})",
            request.approval_type, request.id
        );

        let body = format!(
            r#"
An approval is requested for:

Type: {}
Message: {}

Context:
{}

This request will expire at: {}

To approve, click: {}
To reject, click: {}

If you have questions, reply to this email.
"#,
            request.approval_type,
            request.message,
            serde_json::to_string_pretty(&request.context).unwrap_or_default(),
            request.expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
            approve_url,
            reject_url
        );

        EmailContent {
            subject,
            body,
            html_body: None,
        }
    }

    /// Process a decision
    pub fn process_decision(
        &self,
        request: &mut ApprovalRequest,
        decision: ApprovalDecision,
        decided_by: &str,
        comments: Option<String>,
    ) {
        request.decision = Some(decision.clone());
        request.decided_by = Some(decided_by.to_string());
        request.decided_at = Some(Utc::now());
        request.comments = comments;

        request.status = match decision {
            ApprovalDecision::Approve => ApprovalStatus::Approved,
            ApprovalDecision::Reject => ApprovalStatus::Rejected,
            ApprovalDecision::Escalate => ApprovalStatus::Escalated,
            ApprovalDecision::Defer | ApprovalDecision::RequestInfo => ApprovalStatus::Pending,
        };
    }

    /// Handle timeout
    pub fn handle_timeout(&self, request: &mut ApprovalRequest) {
        if let Some(default_action) = &request.default_action {
            request.decision = Some(default_action.clone());
            request.decided_by = Some("system:timeout".to_string());
            request.decided_at = Some(Utc::now());
            request.status = match default_action {
                ApprovalDecision::Approve => ApprovalStatus::Approved,
                ApprovalDecision::Reject => ApprovalStatus::Rejected,
                _ => ApprovalStatus::TimedOut,
            };
        } else {
            request.status = ApprovalStatus::TimedOut;
        }
    }

    /// Evaluate approval chain condition
    pub fn evaluate_condition(
        &self,
        condition: &str,
        context: &serde_json::Value,
    ) -> Result<bool, String> {
        // Simple condition evaluation
        // Format: "field operator value" e.g., "amount > 10000"
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!("Invalid condition format: {}", condition));
        }

        let field = parts[0];
        let operator = parts[1];
        let value_str = parts[2];

        let field_value = context
            .get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("Field not found or not numeric: {}", field))?;

        let compare_value: f64 = value_str
            .parse()
            .map_err(|_| format!("Invalid comparison value: {}", value_str))?;

        let result = match operator {
            ">" => field_value > compare_value,
            ">=" => field_value >= compare_value,
            "<" => field_value < compare_value,
            "<=" => field_value <= compare_value,
            "==" | "=" => (field_value - compare_value).abs() < f64::EPSILON,
            "!=" => (field_value - compare_value).abs() >= f64::EPSILON,
            _ => return Err(format!("Unknown operator: {}", operator)),
        };

        Ok(result)
    }
}

/// Email content structure
#[derive(Debug, Clone)]
pub struct EmailContent {
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
}

/// Convert ApprovalRequest to Rhai Dynamic
impl ApprovalRequest {
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = Map::new();

        map.insert("id".into(), self.id.to_string().into());
        map.insert("bot_id".into(), self.bot_id.to_string().into());
        map.insert("session_id".into(), self.session_id.to_string().into());
        map.insert("initiated_by".into(), self.initiated_by.to_string().into());
        map.insert("approval_type".into(), self.approval_type.clone().into());
        map.insert(
            "status".into(),
            format!("{:?}", self.status).to_lowercase().into(),
        );
        map.insert("channel".into(), self.channel.to_string().into());
        map.insert("recipient".into(), self.recipient.clone().into());
        map.insert("context".into(), json_to_dynamic(&self.context));
        map.insert("message".into(), self.message.clone().into());
        map.insert(
            "timeout_seconds".into(),
            (self.timeout_seconds as i64).into(),
        );
        map.insert("current_level".into(), (self.current_level as i64).into());
        map.insert("total_levels".into(), (self.total_levels as i64).into());
        map.insert("created_at".into(), self.created_at.to_rfc3339().into());
        map.insert("expires_at".into(), self.expires_at.to_rfc3339().into());

        if let Some(ref decision) = self.decision {
            map.insert(
                "decision".into(),
                format!("{:?}", decision).to_lowercase().into(),
            );
        }

        if let Some(ref decided_by) = self.decided_by {
            map.insert("decided_by".into(), decided_by.clone().into());
        }

        if let Some(ref decided_at) = self.decided_at {
            map.insert("decided_at".into(), decided_at.to_rfc3339().into());
        }

        if let Some(ref comments) = self.comments {
            map.insert("comments".into(), comments.clone().into());
        }

        Dynamic::from(map)
    }
}

/// Convert JSON value to Rhai Dynamic
fn json_to_dynamic(value: &serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let array: Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(array)
        }
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

/// Register approval keywords with Rhai engine
pub fn register_approval_keywords(engine: &mut Engine) {
    // Helper functions for working with approvals in scripts

    engine.register_fn("approval_is_approved", |request: Map| -> bool {
        request
            .get("status")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| s == "approved")
            .unwrap_or(false)
    });

    engine.register_fn("approval_is_rejected", |request: Map| -> bool {
        request
            .get("status")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| s == "rejected")
            .unwrap_or(false)
    });

    engine.register_fn("approval_is_pending", |request: Map| -> bool {
        request
            .get("status")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| s == "pending")
            .unwrap_or(false)
    });

    engine.register_fn("approval_is_timed_out", |request: Map| -> bool {
        request
            .get("status")
            .and_then(|v| v.clone().try_cast::<String>())
            .map(|s| s == "timedout")
            .unwrap_or(false)
    });

    engine.register_fn("approval_decision", |request: Map| -> String {
        request
            .get("decision")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_else(|| "pending".to_string())
    });

    engine.register_fn("approval_decided_by", |request: Map| -> String {
        request
            .get("decided_by")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    engine.register_fn("approval_comments", |request: Map| -> String {
        request
            .get("comments")
            .and_then(|v| v.clone().try_cast::<String>())
            .unwrap_or_default()
    });

    info!("Approval keywords registered");
}

/// SQL for creating approval tables
pub const APPROVAL_SCHEMA: &str = r#"
-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    approval_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    channel VARCHAR(50) NOT NULL,
    recipient VARCHAR(500) NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    message TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    default_action VARCHAR(50),
    current_level INTEGER NOT NULL DEFAULT 1,
    total_levels INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reminders_sent JSONB NOT NULL DEFAULT '[]',
    decision VARCHAR(50),
    decided_by VARCHAR(500),
    decided_at TIMESTAMP WITH TIME ZONE,
    comments TEXT
);

-- Approval chains
CREATE TABLE IF NOT EXISTS approval_chains (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    bot_id UUID NOT NULL,
    levels JSONB NOT NULL DEFAULT '[]',
    stop_on_reject BOOLEAN NOT NULL DEFAULT true,
    require_all BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Approval audit log
CREATE TABLE IF NOT EXISTS approval_audit_log (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    actor VARCHAR(500) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(50),
    user_agent TEXT
);

-- Approval tokens (for secure links)
CREATE TABLE IF NOT EXISTS approval_tokens (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL UNIQUE,
    action VARCHAR(50) NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_approval_requests_bot_id ON approval_requests(bot_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_session_id ON approval_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_status ON approval_requests(status);
CREATE INDEX IF NOT EXISTS idx_approval_requests_expires_at ON approval_requests(expires_at);
CREATE INDEX IF NOT EXISTS idx_approval_requests_pending ON approval_requests(status, expires_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_approval_audit_request_id ON approval_audit_log(request_id);
CREATE INDEX IF NOT EXISTS idx_approval_audit_timestamp ON approval_audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_approval_tokens_token ON approval_tokens(token);
CREATE INDEX IF NOT EXISTS idx_approval_tokens_request_id ON approval_tokens(request_id);
"#;

/// SQL for approval operations
pub mod sql {
    pub const INSERT_REQUEST: &str = r#"
        INSERT INTO approval_requests (
            id, bot_id, session_id, initiated_by, approval_type, status,
            channel, recipient, context, message, timeout_seconds,
            default_action, current_level, total_levels, created_at,
            expires_at, reminders_sent
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
        )
    "#;

    pub const UPDATE_REQUEST: &str = r#"
        UPDATE approval_requests
        SET status = $2,
            decision = $3,
            decided_by = $4,
            decided_at = $5,
            comments = $6
        WHERE id = $1
    "#;

    pub const GET_REQUEST: &str = r#"
        SELECT * FROM approval_requests WHERE id = $1
    "#;

    pub const GET_PENDING_REQUESTS: &str = r#"
        SELECT * FROM approval_requests
        WHERE status = 'pending'
        AND expires_at > NOW()
        ORDER BY created_at ASC
    "#;

    pub const GET_EXPIRED_REQUESTS: &str = r#"
        SELECT * FROM approval_requests
        WHERE status = 'pending'
        AND expires_at <= NOW()
    "#;

    pub const GET_REQUESTS_BY_SESSION: &str = r#"
        SELECT * FROM approval_requests
        WHERE session_id = $1
        ORDER BY created_at DESC
    "#;

    pub const UPDATE_REMINDERS: &str = r#"
        UPDATE approval_requests
        SET reminders_sent = reminders_sent || $2::jsonb
        WHERE id = $1
    "#;

    pub const INSERT_AUDIT: &str = r#"
        INSERT INTO approval_audit_log (
            id, request_id, action, actor, details, timestamp, ip_address, user_agent
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        )
    "#;

    pub const GET_AUDIT_LOG: &str = r#"
        SELECT * FROM approval_audit_log
        WHERE request_id = $1
        ORDER BY timestamp ASC
    "#;

    pub const INSERT_TOKEN: &str = r#"
        INSERT INTO approval_tokens (
            id, request_id, token, action, expires_at, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6
        )
    "#;

    pub const GET_TOKEN: &str = r#"
        SELECT * FROM approval_tokens
        WHERE token = $1 AND used = false AND expires_at > NOW()
    "#;

    pub const USE_TOKEN: &str = r#"
        UPDATE approval_tokens
        SET used = true, used_at = NOW()
        WHERE token = $1
    "#;

    pub const INSERT_CHAIN: &str = r#"
        INSERT INTO approval_chains (
            id, name, bot_id, levels, stop_on_reject, require_all, description, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        )
        ON CONFLICT (bot_id, name)
        DO UPDATE SET
            levels = $4,
            stop_on_reject = $5,
            require_all = $6,
            description = $7
    "#;

    pub const GET_CHAIN: &str = r#"
        SELECT * FROM approval_chains
        WHERE bot_id = $1 AND name = $2
    "#;
}
