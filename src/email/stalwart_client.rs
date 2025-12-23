



















use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{debug, error, info, warn};




const DEFAULT_TIMEOUT_SECS: u64 = 30;


pub const DEFAULT_QUEUE_POLL_INTERVAL_SECS: u64 = 30;


pub const DEFAULT_METRICS_POLL_INTERVAL_SECS: u64 = 60;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {

    pub is_running: bool,

    pub total_queued: u64,

    pub messages: Vec<QueuedMessage>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {

    pub id: String,

    pub from: String,

    pub to: Vec<String>,

    #[serde(default)]
    pub subject: Option<String>,

    pub status: DeliveryStatus,

    #[serde(default)]
    pub attempts: u32,

    #[serde(default)]
    pub next_retry: Option<DateTime<Utc>>,

    #[serde(default)]
    pub last_error: Option<String>,

    #[serde(default)]
    pub size: u64,

    #[serde(default)]
    pub queued_at: Option<DateTime<Utc>>,
}


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


#[derive(Debug, Clone, Deserialize)]
struct QueueListResponse {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    items: Vec<QueuedMessage>,
}




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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {

    pub id: Option<u64>,

    #[serde(rename = "type")]
    pub principal_type: PrincipalType,

    pub name: String,

    #[serde(default)]
    pub emails: Vec<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub quota: u64,

    #[serde(default)]
    pub roles: Vec<String>,

    #[serde(default)]
    pub members: Vec<String>,

    #[serde(default)]
    pub disabled: bool,
}


#[derive(Debug, Clone, Serialize)]
pub struct AccountUpdate {

    pub action: String,

    pub field: String,

    pub value: Value,
}

impl AccountUpdate {

    pub fn set(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "set".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }


    pub fn add_item(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "addItem".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }


    pub fn remove_item(field: &str, value: impl Into<Value>) -> Self {
        Self {
            action: "removeItem".to_string(),
            field: field.to_string(),
            value: value.into(),
        }
    }


    pub fn clear(field: &str) -> Self {
        Self {
            action: "clear".to_string(),
            field: field.to_string(),
            value: Value::Null,
        }
    }
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResponderConfig {

    pub enabled: bool,

    pub subject: String,

    pub body_plain: String,

    #[serde(default)]
    pub body_html: Option<String>,

    #[serde(default)]
    pub start_date: Option<NaiveDate>,

    #[serde(default)]
    pub end_date: Option<NaiveDate>,

    #[serde(default)]
    pub only_contacts: bool,

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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRule {

    pub id: String,

    pub name: String,

    #[serde(default)]
    pub priority: i32,

    pub enabled: bool,

    pub conditions: Vec<RuleCondition>,

    pub actions: Vec<RuleAction>,

    #[serde(default = "default_stop_processing")]
    pub stop_processing: bool,
}

fn default_stop_processing() -> bool {
    true
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {

    pub field: String,

    pub operator: String,

    pub value: String,

    #[serde(default)]
    pub header_name: Option<String>,

    #[serde(default)]
    pub case_sensitive: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {

    pub action_type: String,

    #[serde(default)]
    pub value: String,
}




#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metrics {

    #[serde(default)]
    pub messages_received: u64,

    #[serde(default)]
    pub messages_delivered: u64,

    #[serde(default)]
    pub messages_rejected: u64,

    #[serde(default)]
    pub queue_size: u64,

    #[serde(default)]
    pub smtp_connections: u64,

    #[serde(default)]
    pub imap_connections: u64,

    #[serde(default)]
    pub uptime_seconds: u64,

    #[serde(default)]
    pub memory_used: u64,

    #[serde(default)]
    pub cpu_usage: f64,

    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {

    pub timestamp: DateTime<Utc>,

    pub level: String,

    #[serde(default)]
    pub component: Option<String>,

    pub message: String,

    #[serde(default)]
    pub context: Option<Value>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct LogList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<LogEntry>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {

    pub timestamp: DateTime<Utc>,

    pub event_type: String,

    #[serde(default)]
    pub message_id: Option<String>,

    #[serde(default)]
    pub from: Option<String>,

    #[serde(default)]
    pub to: Vec<String>,

    #[serde(default)]
    pub remote_host: Option<String>,

    #[serde(default)]
    pub result: Option<String>,

    #[serde(default)]
    pub error: Option<String>,

    #[serde(default)]
    pub duration_ms: Option<u64>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct TraceList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<TraceEvent>,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {

    pub id: String,

    pub report_type: String,

    pub domain: String,

    #[serde(default)]
    pub reporter: Option<String>,

    #[serde(default)]
    pub date_start: Option<DateTime<Utc>>,

    #[serde(default)]
    pub date_end: Option<DateTime<Utc>>,

    pub data: Value,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ReportList {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub items: Vec<Report>,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamClassifyRequest {

    pub from: String,

    pub to: Vec<String>,

    #[serde(default)]
    pub remote_ip: Option<String>,

    #[serde(default)]
    pub ehlo_host: Option<String>,

    #[serde(default)]
    pub headers: Option<String>,

    #[serde(default)]
    pub body: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamClassifyResult {

    pub score: f64,

    pub classification: String,

    #[serde(default)]
    pub tests: Vec<SpamTest>,

    #[serde(default)]
    pub action: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamTest {

    pub name: String,

    pub score: f64,

    #[serde(default)]
    pub description: Option<String>,
}




#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success { data: T },
    SuccessDirect(T),
    Error { error: String },
}




#[derive(Debug, Clone)]
pub struct StalwartClient {
    base_url: String,
    auth_token: String,
    http_client: Client,
}

impl StalwartClient {












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


        if text.is_empty() || text == "null" {

            return serde_json::from_str("null")
                .or_else(|_| serde_json::from_str("{}"))
                .or_else(|_| serde_json::from_str("true"))
                .context("Empty response from Stalwart API");
        }

        serde_json::from_str(&text).context("Failed to parse Stalwart API response")
    }


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


    pub async fn get_queued_message(&self, message_id: &str) -> Result<QueuedMessage> {
        self.request(
            Method::GET,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }


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


    pub async fn retry_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(
            Method::PATCH,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }


    pub async fn cancel_delivery(&self, message_id: &str) -> Result<bool> {
        self.request(
            Method::DELETE,
            &format!("/api/queue/messages/{}", message_id),
            None,
        )
        .await
    }


    pub async fn stop_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/stop", None).await
    }


    pub async fn start_queue(&self) -> Result<bool> {
        self.request(Method::PATCH, "/api/queue/status/start", None).await
    }


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


    pub async fn create_account_full(&self, principal: &Principal, password: &str) -> Result<u64> {
        let mut body = serde_json::to_value(principal)?;
        if let Some(obj) = body.as_object_mut() {
            obj.insert("secrets".to_string(), json!([password]));
        }
        self.request(Method::POST, "/api/principal", Some(body)).await
    }


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


    pub async fn get_account(&self, account_id: &str) -> Result<Principal> {
        self.request(
            Method::GET,
            &format!("/api/principal/{}", account_id),
            None,
        )
        .await
    }


    pub async fn get_account_by_email(&self, email: &str) -> Result<Principal> {
        self.request(
            Method::GET,
            &format!("/api/principal?filter=emails:{}", email),
            None,
        )
        .await
    }


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


    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        self.request::<Value>(
            Method::DELETE,
            &format!("/api/principal/{}", account_id),
            None,
        )
        .await?;
        Ok(())
    }


    pub async fn list_principals(&self, principal_type: Option<PrincipalType>) -> Result<Vec<Principal>> {
        let path = match principal_type {
            Some(t) => format!("/api/principal?type={:?}", t).to_lowercase(),
            None => "/api/principal".to_string(),
        };
        self.request(Method::GET, &path, None).await
    }


    pub async fn add_members(&self, account_id: &str, members: Vec<String>) -> Result<()> {
        let updates: Vec<AccountUpdate> = members
            .into_iter()
            .map(|m| AccountUpdate::add_item("members", m))
            .collect();
        self.update_account(account_id, updates).await
    }


    pub async fn remove_members(&self, account_id: &str, members: Vec<String>) -> Result<()> {
        let updates: Vec<AccountUpdate> = members
            .into_iter()
            .map(|m| AccountUpdate::remove_item("members", m))
            .collect();
        self.update_account(account_id, updates).await
    }






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


    pub fn generate_vacation_sieve(&self, config: &AutoResponderConfig) -> String {
        let mut script = String::from("require [\"vacation\", \"variables\", \"date\", \"relational\"];\n\n");


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


        let subject = config.subject.replace('"', "\\\"").replace('\n', " ");
        let body = config.body_plain.replace('"', "\\\"").replace('\n', "\\n");

        script.push_str(&format!(
            "vacation :days {} :subject \"{}\" \"{}\";\n",
            config.vacation_days, subject, body
        ));

        script
    }


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


    pub fn generate_filter_sieve(&self, rule: &EmailRule) -> String {
        let mut script =
            String::from("require [\"fileinto\", \"reject\", \"vacation\", \"imap4flags\", \"copy\"];\n\n");

        script.push_str(&format!("# Rule: {}\n", rule.name));

        if !rule.enabled {
            script.push_str("# DISABLED\n");
            return script;
        }


        let mut conditions = Vec::new();
        for condition in &rule.conditions {
            let cond_str = self.generate_condition_sieve(condition);
            if !cond_str.is_empty() {
                conditions.push(cond_str);
            }
        }

        if conditions.is_empty() {

            script.push_str("# Always applies\n");
        } else {

            script.push_str(&format!("if allof ({}) {{\n", conditions.join(", ")));
        }


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


        if rule.stop_processing {
            if conditions.is_empty() {
                script.push_str("stop;\n");
            } else {
                script.push_str("    stop;\n");
            }
        }


        if !conditions.is_empty() {
            script.push_str("}\n");
        }

        script
    }


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






    pub async fn get_metrics(&self) -> Result<Metrics> {
        self.request(Method::GET, "/api/telemetry/metrics", None).await
    }


    pub async fn get_logs(&self, page: u32, limit: u32) -> Result<LogList> {
        self.request(
            Method::GET,
            &format!("/api/logs?page={}&limit={}", page, limit),
            None,
        )
        .await
    }


    pub async fn get_logs_by_level(&self, level: &str, page: u32, limit: u32) -> Result<LogList> {
        self.request(
            Method::GET,
            &format!("/api/logs?level={}&page={}&limit={}", level, page, limit),
            None,
        )
        .await
    }


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


    pub async fn get_recent_traces(&self, limit: u32) -> Result<TraceList> {
        self.request(
            Method::GET,
            &format!("/api/telemetry/traces?limit={}", limit),
            None,
        )
        .await
    }


    pub async fn get_trace(&self, trace_id: &str) -> Result<Vec<TraceEvent>> {
        self.request(
            Method::GET,
            &format!("/api/telemetry/trace/{}", trace_id),
            None,
        )
        .await
    }


    pub async fn get_dmarc_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/dmarc?page={}&limit=50", page),
            None,
        )
        .await
    }


    pub async fn get_tls_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/tls?page={}&limit=50", page),
            None,
        )
        .await
    }


    pub async fn get_arf_reports(&self, page: u32) -> Result<ReportList> {
        self.request(
            Method::GET,
            &format!("/api/reports/arf?page={}&limit=50", page),
            None,
        )
        .await
    }


    pub async fn get_live_metrics_token(&self) -> Result<String> {
        self.request(Method::GET, "/api/telemetry/live/metrics-token", None)
            .await
    }


    pub async fn get_live_tracing_token(&self) -> Result<String> {
        self.request(Method::GET, "/api/telemetry/live/tracing-token", None)
            .await
    }






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


    pub async fn classify_message(&self, message: &SpamClassifyRequest) -> Result<SpamClassifyResult> {
        self.request(
            Method::POST,
            "/api/spam-filter/classify",
            Some(serde_json::to_value(message)?),
        )
        .await
    }






    pub async fn troubleshoot_delivery(&self, recipient: &str) -> Result<Value> {
        self.request(
            Method::GET,
            &format!("/api/troubleshoot/delivery/{}", urlencoding::encode(recipient)),
            None,
        )
        .await
    }


    pub async fn check_dmarc(&self, domain: &str, from_email: &str) -> Result<Value> {
        let body = json!({
            "domain": domain,
            "from": from_email
        });
        self.request(Method::POST, "/api/troubleshoot/dmarc", Some(body))
            .await
    }


    pub async fn get_dns_records(&self, domain: &str) -> Result<Value> {
        self.request(
            Method::GET,
            &format!("/api/dns/records/{}", domain),
            None,
        )
        .await
    }


    pub async fn undelete_messages(&self, account_id: &str) -> Result<Value> {
        self.request(
            Method::POST,
            &format!("/api/store/undelete/{}", account_id),
            None,
        )
        .await
    }


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


