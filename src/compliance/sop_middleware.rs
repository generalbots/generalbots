use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SopCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    DataDeletion,
    ConfigurationChange,
    SystemAccess,
    ApiAccess,
    FileAccess,
    UserManagement,
    RoleManagement,
    BillingOperation,
    ComplianceAction,
    SecurityEvent,
    AuditAction,
    BackupOperation,
    IncidentResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SopSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SopOutcome {
    Success,
    Failure,
    Denied,
    Error,
    Timeout,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: SopCategory,
    pub severity: SopSeverity,
    pub outcome: SopOutcome,
    pub operation: String,
    pub description: String,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub organization_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: Option<String>,
    pub request_path: Option<String>,
    pub request_id: Option<String>,
    pub response_status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub compliance_controls: Vec<String>,
    pub evidence_reference: Option<String>,
}

impl SopLogEntry {
    pub fn new(category: SopCategory, operation: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            severity: SopSeverity::Info,
            outcome: SopOutcome::Pending,
            operation: operation.to_string(),
            description: description.to_string(),
            user_id: None,
            user_email: None,
            organization_id: None,
            resource_type: None,
            resource_id: None,
            ip_address: None,
            user_agent: None,
            request_method: None,
            request_path: None,
            request_id: None,
            response_status: None,
            duration_ms: None,
            metadata: HashMap::new(),
            compliance_controls: Vec::new(),
            evidence_reference: None,
        }
    }

    pub fn with_severity(mut self, severity: SopSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_outcome(mut self, outcome: SopOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn with_user(mut self, user_id: Uuid, email: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.user_email = email;
        self
    }

    pub fn with_organization(mut self, org_id: Uuid) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn with_resource(mut self, resource_type: &str, resource_id: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = Some(resource_id.to_string());
        self
    }

    pub fn with_request_info(
        mut self,
        method: &str,
        path: &str,
        request_id: Option<String>,
    ) -> Self {
        self.request_method = Some(method.to_string());
        self.request_path = Some(path.to_string());
        self.request_id = request_id;
        self
    }

    pub fn with_client_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.ip_address = ip;
        self.user_agent = user_agent;
        self
    }

    pub fn with_response(mut self, status: u16, duration_ms: u64) -> Self {
        self.response_status = Some(status);
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    pub fn with_compliance_controls(mut self, controls: Vec<&str>) -> Self {
        self.compliance_controls = controls.into_iter().map(String::from).collect();
        self
    }

    pub fn with_evidence_reference(mut self, reference: &str) -> Self {
        self.evidence_reference = Some(reference.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopSearchQuery {
    pub category: Option<SopCategory>,
    pub severity: Option<SopSeverity>,
    pub outcome: Option<SopOutcome>,
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub operation_contains: Option<String>,
    pub compliance_control: Option<String>,
    pub page: u32,
    pub per_page: u32,
}

impl Default for SopSearchQuery {
    fn default() -> Self {
        Self {
            category: None,
            severity: None,
            outcome: None,
            user_id: None,
            organization_id: None,
            resource_type: None,
            from_date: None,
            to_date: None,
            operation_contains: None,
            compliance_control: None,
            page: 1,
            per_page: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopSearchResult {
    pub entries: Vec<SopLogEntry>,
    pub total_count: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopStatistics {
    pub total_entries: u64,
    pub entries_by_category: HashMap<String, u64>,
    pub entries_by_severity: HashMap<String, u64>,
    pub entries_by_outcome: HashMap<String, u64>,
    pub unique_users: u64,
    pub unique_organizations: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

pub struct SopLogger {
    entries: Arc<RwLock<Vec<SopLogEntry>>>,
    max_entries: usize,
    retention_days: i64,
    webhook_url: Option<String>,
    siem_endpoint: Option<String>,
}

impl Default for SopLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl SopLogger {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            max_entries: 100_000,
            retention_days: 365,
            webhook_url: None,
            siem_endpoint: None,
        }
    }

    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    pub fn with_retention_days(mut self, days: i64) -> Self {
        self.retention_days = days;
        self
    }

    pub fn with_webhook(mut self, url: String) -> Self {
        self.webhook_url = Some(url);
        self
    }

    pub fn with_siem_endpoint(mut self, endpoint: String) -> Self {
        self.siem_endpoint = Some(endpoint);
        self
    }

    pub async fn log(&self, entry: SopLogEntry) {
        let entry_clone = entry.clone();

        {
            let mut entries = self.entries.write().await;

            if entries.len() >= self.max_entries {
                let remove_count = self.max_entries / 10;
                entries.drain(0..remove_count);
            }

            entries.push(entry);
        }

        self.log_to_persistent_storage(&entry_clone).await;

        if self.should_alert(&entry_clone) {
            self.send_alert(&entry_clone).await;
        }

        if let Some(ref webhook_url) = self.webhook_url {
            self.send_to_webhook(webhook_url, &entry_clone).await;
        }

        if let Some(ref siem_endpoint) = self.siem_endpoint {
            self.send_to_siem(siem_endpoint, &entry_clone).await;
        }
    }

    pub async fn log_authentication(
        &self,
        user_id: Option<Uuid>,
        email: Option<String>,
        outcome: SopOutcome,
        method: &str,
        ip: Option<String>,
    ) {
        let severity = match outcome {
            SopOutcome::Success => SopSeverity::Info,
            SopOutcome::Failure | SopOutcome::Denied => SopSeverity::Medium,
            _ => SopSeverity::Low,
        };

        let mut entry = SopLogEntry::new(
            SopCategory::Authentication,
            "user_authentication",
            &format!("User authentication attempt via {method}"),
        )
        .with_severity(severity)
        .with_outcome(outcome)
        .with_client_info(ip, None)
        .with_compliance_controls(vec!["CC6.1", "CC6.6"]);

        if let Some(uid) = user_id {
            entry = entry.with_user(uid, email);
        }

        self.log(entry).await;
    }

    pub async fn log_authorization(
        &self,
        user_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        action: &str,
        outcome: SopOutcome,
    ) {
        let severity = match outcome {
            SopOutcome::Denied => SopSeverity::Medium,
            _ => SopSeverity::Info,
        };

        let entry = SopLogEntry::new(
            SopCategory::Authorization,
            "authorization_check",
            &format!("Authorization check for {action} on {resource_type}"),
        )
        .with_severity(severity)
        .with_outcome(outcome)
        .with_user(user_id, None)
        .with_resource(resource_type, resource_id)
        .with_metadata("action", serde_json::json!(action))
        .with_compliance_controls(vec!["CC6.1", "CC6.3"]);

        self.log(entry).await;
    }

    pub async fn log_data_access(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        access_type: &str,
    ) {
        let entry = SopLogEntry::new(
            SopCategory::DataAccess,
            "data_access",
            &format!("{access_type} access to {resource_type}"),
        )
        .with_severity(SopSeverity::Info)
        .with_outcome(SopOutcome::Success)
        .with_user(user_id, None)
        .with_organization(organization_id)
        .with_resource(resource_type, resource_id)
        .with_metadata("access_type", serde_json::json!(access_type))
        .with_compliance_controls(vec!["CC6.1", "CC6.5", "PI1.1"]);

        self.log(entry).await;
    }

    pub async fn log_data_modification(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        modification_type: &str,
        changes: Option<serde_json::Value>,
    ) {
        let mut entry = SopLogEntry::new(
            SopCategory::DataModification,
            "data_modification",
            &format!("{modification_type} on {resource_type}"),
        )
        .with_severity(SopSeverity::Medium)
        .with_outcome(SopOutcome::Success)
        .with_user(user_id, None)
        .with_organization(organization_id)
        .with_resource(resource_type, resource_id)
        .with_metadata("modification_type", serde_json::json!(modification_type))
        .with_compliance_controls(vec!["CC6.1", "CC7.2", "PI1.4"]);

        if let Some(c) = changes {
            entry = entry.with_metadata("changes", c);
        }

        self.log(entry).await;
    }

    pub async fn log_configuration_change(
        &self,
        user_id: Uuid,
        config_type: &str,
        config_key: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
    ) {
        let mut entry = SopLogEntry::new(
            SopCategory::ConfigurationChange,
            "configuration_change",
            &format!("Configuration change: {config_type}/{config_key}"),
        )
        .with_severity(SopSeverity::High)
        .with_outcome(SopOutcome::Success)
        .with_user(user_id, None)
        .with_resource(config_type, config_key)
        .with_compliance_controls(vec!["CC6.1", "CC7.1", "CC8.1"]);

        if let Some(old) = old_value {
            entry = entry.with_metadata("old_value", serde_json::json!(old));
        }
        if let Some(new) = new_value {
            entry = entry.with_metadata("new_value", serde_json::json!(new));
        }

        self.log(entry).await;
    }

    pub async fn log_security_event(
        &self,
        event_type: &str,
        description: &str,
        severity: SopSeverity,
        metadata: HashMap<String, serde_json::Value>,
    ) {
        let mut entry = SopLogEntry::new(SopCategory::SecurityEvent, event_type, description)
            .with_severity(severity)
            .with_outcome(SopOutcome::Success)
            .with_compliance_controls(vec!["CC7.2", "CC7.3", "CC7.4"]);

        for (key, value) in metadata {
            entry = entry.with_metadata(&key, value);
        }

        self.log(entry).await;
    }

    pub async fn log_incident_response(
        &self,
        incident_id: &str,
        action: &str,
        responder_id: Uuid,
        details: &str,
    ) {
        let entry = SopLogEntry::new(
            SopCategory::IncidentResponse,
            action,
            &format!("Incident response action: {details}"),
        )
        .with_severity(SopSeverity::High)
        .with_outcome(SopOutcome::Success)
        .with_user(responder_id, None)
        .with_resource("incident", incident_id)
        .with_compliance_controls(vec!["CC7.3", "CC7.4", "CC7.5"]);

        self.log(entry).await;
    }

    pub async fn log_backup_operation(
        &self,
        operation: &str,
        backup_id: &str,
        outcome: SopOutcome,
        size_bytes: Option<u64>,
    ) {
        let mut entry = SopLogEntry::new(
            SopCategory::BackupOperation,
            operation,
            &format!("Backup operation: {operation}"),
        )
        .with_severity(SopSeverity::Medium)
        .with_outcome(outcome)
        .with_resource("backup", backup_id)
        .with_compliance_controls(vec!["A1.2", "A1.3"]);

        if let Some(size) = size_bytes {
            entry = entry.with_metadata("size_bytes", serde_json::json!(size));
        }

        self.log(entry).await;
    }

    pub async fn search(&self, query: SopSearchQuery) -> SopSearchResult {
        let entries = self.entries.read().await;

        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| {
                if let Some(ref cat) = query.category {
                    if &e.category != cat {
                        return false;
                    }
                }
                if let Some(ref sev) = query.severity {
                    if &e.severity != sev {
                        return false;
                    }
                }
                if let Some(ref out) = query.outcome {
                    if &e.outcome != out {
                        return false;
                    }
                }
                if let Some(uid) = query.user_id {
                    if e.user_id != Some(uid) {
                        return false;
                    }
                }
                if let Some(oid) = query.organization_id {
                    if e.organization_id != Some(oid) {
                        return false;
                    }
                }
                if let Some(ref rt) = query.resource_type {
                    if e.resource_type.as_ref() != Some(rt) {
                        return false;
                    }
                }
                if let Some(from) = query.from_date {
                    if e.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = query.to_date {
                    if e.timestamp > to {
                        return false;
                    }
                }
                if let Some(ref op) = query.operation_contains {
                    if !e.operation.to_lowercase().contains(&op.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref ctrl) = query.compliance_control {
                    if !e.compliance_controls.contains(ctrl) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total_count = filtered.len() as u64;
        let total_pages = ((total_count as f64) / (query.per_page as f64)).ceil() as u32;
        let start = ((query.page - 1) * query.per_page) as usize;
        let end = (start + query.per_page as usize).min(filtered.len());

        let page_entries = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        SopSearchResult {
            entries: page_entries,
            total_count,
            page: query.page,
            per_page: query.per_page,
            total_pages,
        }
    }

    pub async fn get_statistics(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> SopStatistics {
        let entries = self.entries.read().await;

        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| e.timestamp >= from_date && e.timestamp <= to_date)
            .collect();

        let mut by_category: HashMap<String, u64> = HashMap::new();
        let mut by_severity: HashMap<String, u64> = HashMap::new();
        let mut by_outcome: HashMap<String, u64> = HashMap::new();
        let mut unique_users = std::collections::HashSet::new();
        let mut unique_orgs = std::collections::HashSet::new();

        for entry in &filtered {
            *by_category
                .entry(format!("{:?}", entry.category))
                .or_insert(0) += 1;
            *by_severity
                .entry(format!("{:?}", entry.severity))
                .or_insert(0) += 1;
            *by_outcome
                .entry(format!("{:?}", entry.outcome))
                .or_insert(0) += 1;

            if let Some(uid) = entry.user_id {
                unique_users.insert(uid);
            }
            if let Some(oid) = entry.organization_id {
                unique_orgs.insert(oid);
            }
        }

        SopStatistics {
            total_entries: filtered.len() as u64,
            entries_by_category: by_category,
            entries_by_severity: by_severity,
            entries_by_outcome: by_outcome,
            unique_users: unique_users.len() as u64,
            unique_organizations: unique_orgs.len() as u64,
            period_start: from_date,
            period_end: to_date,
        }
    }

    pub async fn export_for_audit(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
        compliance_control: Option<&str>,
    ) -> Vec<SopLogEntry> {
        let entries = self.entries.read().await;

        entries
            .iter()
            .filter(|e| {
                if e.timestamp < from_date || e.timestamp > to_date {
                    return false;
                }
                if let Some(ctrl) = compliance_control {
                    if !e.compliance_controls.contains(&ctrl.to_string()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    pub async fn cleanup_old_entries(&self) {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days);
        let mut entries = self.entries.write().await;
        entries.retain(|e| e.timestamp > cutoff);
    }

    fn should_alert(&self, entry: &SopLogEntry) -> bool {
        matches!(entry.severity, SopSeverity::High | SopSeverity::Critical)
            || matches!(entry.outcome, SopOutcome::Denied | SopOutcome::Error)
            || matches!(
                entry.category,
                SopCategory::SecurityEvent | SopCategory::IncidentResponse
            )
    }

    async fn send_alert(&self, entry: &SopLogEntry) {
        log::warn!(
            "SOP ALERT: [{:?}] {:?} - {} - {}",
            entry.severity,
            entry.category,
            entry.operation,
            entry.description
        );
    }

    async fn log_to_persistent_storage(&self, entry: &SopLogEntry) {
        log::info!(
            "SOP LOG: [{:?}] {:?}/{:?} - {} - {}",
            entry.severity,
            entry.category,
            entry.outcome,
            entry.operation,
            entry.description
        );
    }

    async fn send_to_webhook(&self, webhook_url: &str, entry: &SopLogEntry) {
        log::debug!("Sending SOP entry to webhook: {}", webhook_url);
        let _ = entry;
    }

    async fn send_to_siem(&self, siem_endpoint: &str, entry: &SopLogEntry) {
        log::debug!("Sending SOP entry to SIEM: {}", siem_endpoint);
        let _ = entry;
    }
}

fn categorize_request(method: &Method, path: &str) -> SopCategory {
    let path_lower = path.to_lowercase();

    if path_lower.contains("/auth") || path_lower.contains("/login") || path_lower.contains("/logout") {
        return SopCategory::Authentication;
    }

    if path_lower.contains("/users") || path_lower.contains("/members") {
        return SopCategory::UserManagement;
    }

    if path_lower.contains("/roles") || path_lower.contains("/permissions") {
        return SopCategory::RoleManagement;
    }

    if path_lower.contains("/billing") || path_lower.contains("/subscription") || path_lower.contains("/invoice") {
        return SopCategory::BillingOperation;
    }

    if path_lower.contains("/compliance") || path_lower.contains("/audit") {
        return SopCategory::ComplianceAction;
    }

    if path_lower.contains("/settings") || path_lower.contains("/config") {
        return SopCategory::ConfigurationChange;
    }

    if path_lower.contains("/files") || path_lower.contains("/documents") || path_lower.contains("/drive") {
        return SopCategory::FileAccess;
    }

    match *method {
        Method::GET | Method::HEAD => SopCategory::DataAccess,
        Method::POST => SopCategory::DataModification,
        Method::PUT | Method::PATCH => SopCategory::DataModification,
        Method::DELETE => SopCategory::DataDeletion,
        _ => SopCategory::ApiAccess,
    }
}

fn determine_severity(method: &Method, path: &str, status: StatusCode) -> SopSeverity {
    if status.is_server_error() {
        return SopSeverity::High;
    }

    if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        return SopSeverity::Medium;
    }

    let path_lower = path.to_lowercase();
    if path_lower.contains("/admin") || path_lower.contains("/security") {
        return SopSeverity::Medium;
    }

    if *method == Method::DELETE {
        return SopSeverity::Medium;
    }

    if path_lower.contains("/config") || path_lower.contains("/settings") {
        if *method != Method::GET {
            return SopSeverity::Medium;
        }
    }

    SopSeverity::Info
}

fn determine_outcome(status: StatusCode) -> SopOutcome {
    if status.is_success() {
        SopOutcome::Success
    } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        SopOutcome::Denied
    } else if status.is_server_error() {
        SopOutcome::Error
    } else if status == StatusCode::REQUEST_TIMEOUT || status == StatusCode::GATEWAY_TIMEOUT {
        SopOutcome::Timeout
    } else {
        SopOutcome::Failure
    }
}

pub async fn sop_logging_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start_time = std::time::Instant::now();
    let request_id = Uuid::new_v4().to_string();

    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let user_id = request
        .headers()
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let organization_id = request
        .headers()
        .get("x-organization-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let response = next.run(request).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let status = response.status();

    let category = categorize_request(&method, &path);
    let severity = determine_severity(&method, &path, status);
    let outcome = determine_outcome(status);

    let mut entry = SopLogEntry::new(
        category,
        &format!("{}_{}", method.as_str().to_lowercase(), sanitize_path(&path)),
        &format!("{} {}", method, path),
    )
    .with_severity(severity)
    .with_outcome(outcome)
    .with_request_info(method.as_str(), &path, Some(request_id))
    .with_client_info(ip_address, user_agent)
    .with_response(status.as_u16(), duration_ms);

    if let Some(uid) = user_id {
        entry = entry.with_user(uid, None);
    }

    if let Some(oid) = organization_id {
        entry = entry.with_organization(oid);
    }

    let logger = SopLogger::new();
    logger.log(entry).await;

    drop(state);

    response
}

fn sanitize_path(path: &str) -> String {
    let uuid_pattern = regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").ok();
    let numeric_pattern = regex::Regex::new(r"/\d+").ok();

    let mut sanitized = path.to_string();

    if let Some(re) = uuid_pattern {
        sanitized = re.replace_all(&sanitized, ":id").to_string();
    }

    if let Some(re) = numeric_pattern {
        sanitized = re.replace_all(&sanitized, "/:id").to_string();
    }

    sanitized
        .replace('/', "_")
        .trim_matches('_')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sop_log_entry_creation() {
        let entry = SopLogEntry::new(
            SopCategory::Authentication,
            "user_login",
            "User logged in",
        )
        .with_severity(SopSeverity::Info)
        .with_outcome(SopOutcome::Success)
        .with_user(Uuid::new_v4(), Some("test@example.com".to_string()));

        assert_eq!(entry.category, SopCategory::Authentication);
        assert_eq!(entry.severity, SopSeverity::Info);
        assert_eq!(entry.outcome, SopOutcome::Success);
        assert!(entry.user_id.is_some());
        assert_eq!(entry.user_email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test
