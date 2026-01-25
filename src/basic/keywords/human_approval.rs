use chrono::{DateTime, Duration, Utc};
use rhai::{Array, Dynamic, Engine, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,

    pub bot_id: Uuid,

    pub session_id: Uuid,

    pub initiated_by: Uuid,

    pub approval_type: String,

    pub status: ApprovalStatus,

    pub channel: ApprovalChannel,

    pub recipient: String,

    pub context: serde_json::Value,

    pub message: String,

    pub timeout_seconds: u64,

    pub default_action: Option<ApprovalDecision>,

    pub current_level: u32,

    pub total_levels: u32,

    pub created_at: DateTime<Utc>,

    pub expires_at: DateTime<Utc>,

    pub reminders_sent: Vec<DateTime<Utc>>,

    pub decision: Option<ApprovalDecision>,

    pub decided_by: Option<String>,

    pub decided_at: Option<DateTime<Utc>>,

    pub comments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ApprovalStatus {
    #[default]
    Pending,

    Approved,

    Rejected,

    TimedOut,

    Cancelled,

    Escalated,

    Error,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    Approve,
    Reject,
    Escalate,
    Defer,
    RequestInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ApprovalChannel {
    #[default]
    Email,
    Sms,
    Mobile,
    Teams,
    Slack,
    Webhook,
    InApp,
}


impl std::fmt::Display for ApprovalChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email => write!(f, "email"),
            Self::Sms => write!(f, "sms"),
            Self::Mobile => write!(f, "mobile"),
            Self::Teams => write!(f, "teams"),
            Self::Slack => write!(f, "slack"),
            Self::Webhook => write!(f, "webhook"),
            Self::InApp => write!(f, "in_app"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalChain {
    pub name: String,

    pub bot_id: Uuid,

    pub levels: Vec<ApprovalLevel>,

    pub stop_on_reject: bool,

    pub require_all: bool,

    pub description: Option<String>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLevel {
    pub level: u32,

    pub channel: ApprovalChannel,

    pub recipient: String,

    pub timeout_seconds: u64,

    pub condition: Option<String>,

    pub skippable: bool,

    pub approvers: Vec<String>,

    pub required_approvals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAuditEntry {
    pub id: Uuid,

    pub request_id: Uuid,

    pub action: AuditAction,

    pub actor: String,

    pub details: serde_json::Value,

    pub timestamp: DateTime<Utc>,

    pub ip_address: Option<String>,

    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct ApprovalConfig {
    pub enabled: bool,

    pub default_timeout: u64,

    pub reminder_interval: u64,

    pub max_reminders: u32,

    pub audit_enabled: bool,

    pub webhook_url: Option<String>,

    pub email_template: Option<String>,

    pub approval_base_url: Option<String>,
}

pub struct CreateApprovalRequestParams<'a> {
    pub bot_id: Uuid,
    pub session_id: Uuid,
    pub initiated_by: Uuid,
    pub approval_type: &'a str,
    pub channel: ApprovalChannel,
    pub recipient: &'a str,
    pub context: serde_json::Value,
    pub message: &'a str,
    pub timeout_seconds: Option<u64>,
    pub default_action: Option<ApprovalDecision>,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
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

#[derive(Debug)]
pub struct ApprovalManager {
    config: ApprovalConfig,
}

impl ApprovalManager {
    pub fn new(config: ApprovalConfig) -> Self {
        Self { config }
    }

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
        Self::new(config)
    }

    pub fn create_request(
        &self,
        params: CreateApprovalRequestParams<'_>,
    ) -> ApprovalRequest {
        let timeout = params.timeout_seconds.unwrap_or(self.config.default_timeout);
        let now = Utc::now();

        ApprovalRequest {
            id: Uuid::new_v4(),
            bot_id: params.bot_id,
            session_id: params.session_id,
            initiated_by: params.initiated_by,
            approval_type: params.approval_type.to_string(),
            status: ApprovalStatus::Pending,
            channel: params.channel,
            recipient: params.recipient.to_string(),
            context: params.context,
            message: params.message.to_string(),
            timeout_seconds: timeout,
            default_action: params.default_action,
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

    pub fn is_expired(&self, request: &ApprovalRequest) -> bool {
        Utc::now() > request.expires_at
    }

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

    pub fn generate_email_content(&self, request: &ApprovalRequest, token: &str) -> EmailContent {
        let approve_url = self.generate_approval_url(request.id, "approve", token);
        let reject_url = self.generate_approval_url(request.id, "reject", token);

        let subject = format!(
            "Approval Required: {} ({})",
            request.approval_type, request.id
        );

        let body = format!(
            r"
An approval is requested for:

Type: {}
Message: {}

Context:
{}

This request will expire at: {}

To approve, click: {}
To reject, click: {}

If you have questions, reply to this email.
",
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

    pub fn evaluate_condition(
        &self,
        condition: &str,
        context: &serde_json::Value,
    ) -> Result<bool, String> {
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

#[derive(Debug, Clone)]
pub struct EmailContent {
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
}

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
            i64::from(self.timeout_seconds as u32).into(),
        );
        map.insert("current_level".into(), i64::from(self.current_level).into());
        map.insert("total_levels".into(), i64::from(self.total_levels).into());
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

pub fn register_approval_keywords(engine: &mut Engine) {
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

pub const APPROVAL_SCHEMA: &str = r"
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
";

pub mod sql {
    pub const INSERT_REQUEST: &str = r"
        INSERT INTO approval_requests (
            id, bot_id, session_id, initiated_by, approval_type, status,
            channel, recipient, context, message, timeout_seconds,
            default_action, current_level, total_levels, created_at,
            expires_at, reminders_sent
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
        )
    ";

    pub const UPDATE_REQUEST: &str = r"
        UPDATE approval_requests
        SET status = $2,
            decision = $3,
            decided_by = $4,
            decided_at = $5,
            comments = $6
        WHERE id = $1
    ";

    pub const GET_REQUEST: &str = r"
        SELECT * FROM approval_requests WHERE id = $1
    ";

    pub const GET_PENDING_REQUESTS: &str = r"
        SELECT * FROM approval_requests
        WHERE status = 'pending'
        AND expires_at > NOW()
        ORDER BY created_at ASC
    ";

    pub const GET_EXPIRED_REQUESTS: &str = r"
        SELECT * FROM approval_requests
        WHERE status = 'pending'
        AND expires_at <= NOW()
    ";

    pub const GET_REQUESTS_BY_SESSION: &str = r"
        SELECT * FROM approval_requests
        WHERE session_id = $1
        ORDER BY created_at DESC
    ";

    pub const UPDATE_REMINDERS: &str = r"
        UPDATE approval_requests
        SET reminders_sent = reminders_sent || $2::jsonb
        WHERE id = $1
    ";

    pub const INSERT_AUDIT: &str = r"
        INSERT INTO approval_audit_log (
            id, request_id, action, actor, details, timestamp, ip_address, user_agent
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
        )
    ";

    pub const GET_AUDIT_LOG: &str = r"
        SELECT * FROM approval_audit_log
        WHERE request_id = $1
        ORDER BY timestamp ASC
    ";

    pub const INSERT_TOKEN: &str = r"
        INSERT INTO approval_tokens (
            id, request_id, token, action, expires_at, created_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6
        )
    ";

    pub const GET_TOKEN: &str = r"
        SELECT * FROM approval_tokens
        WHERE token = $1 AND used = false AND expires_at > NOW()
    ";

    pub const USE_TOKEN: &str = r"
        UPDATE approval_tokens
        SET used = true, used_at = NOW()
        WHERE token = $1
    ";

    pub const INSERT_CHAIN: &str = r"
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
    ";

    pub const GET_CHAIN: &str = r"
        SELECT * FROM approval_chains
        WHERE bot_id = $1 AND name = $2
    ";
}
