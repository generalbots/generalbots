pub(crate) use anyhow::{anyhow, Result};
pub(crate) use regex::Regex;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::{HashMap, HashSet};
pub(crate) use std::sync::Arc;
pub(crate) use tokio::sync::RwLock;
pub(crate) use tracing::{info, warn};
pub(crate) use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpConfig {
    pub enabled: bool,
    pub scan_inbound: bool,
    pub scan_outbound: bool,
    pub block_on_violation: bool,
    pub log_violations: bool,
    pub redact_sensitive_data: bool,
    pub max_content_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensitiveDataType {
    Email,
    PhoneNumber,
    CreditCard,
    Ssn,
    IpAddress,
    ApiKey,
    Password,
    AwsAccessKey,
    AwsSecretKey,
    PrivateKey,
    JwtToken,
    BankAccount,
    PassportNumber,
    DriversLicense,
    HealthId,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum DlpSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DlpAction {
    Allow,
    Warn,
    Redact,
    Block,
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpMatch {
    pub data_type: SensitiveDataType,
    pub matched_text: String,
    pub start_position: usize,
    pub end_position: usize,
    pub confidence: f32,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpScanResult {
    pub matches: Vec<DlpMatch>,
    pub severity: DlpSeverity,
    pub action: DlpAction,
    pub blocked: bool,
    pub redacted_content: Option<String>,
    pub policy_violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub data_types: Vec<SensitiveDataType>,
    pub action: DlpAction,
    pub severity_threshold: DlpSeverity,
    pub exceptions: Vec<DlpException>,
    pub applies_to: PolicyScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    All,
    Inbound,
    Outbound,
    Users(Vec<Uuid>),
    Channels(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DlpException {
    User(Uuid),
    Pattern(String),
    Domain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlpViolation {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub policy_id: Uuid,
    pub policy_name: String,
    pub user_id: Option<Uuid>,
    pub data_types: Vec<SensitiveDataType>,
    pub action_taken: DlpAction,
    pub severity: DlpSeverity,
    pub content_hash: String,
    pub match_count: usize,
}

pub struct DlpManager {
    config: DlpConfig,
    patterns: HashMap<SensitiveDataType, Vec<Regex>>,
    policies: Arc<RwLock<Vec<DlpPolicy>>>,
    violations: Arc<RwLock<Vec<DlpViolation>>>,
    custom_patterns: Arc<RwLock<Vec<(String, Regex, SensitiveDataType)>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    Inbound,
    Outbound,
}

mod dlp_config;
mod severity;
mod score;
mod dlp_action;
mod with_confidence;
mod clean;
mod with_description;
mod build_detection_patterns;
mod redact_content;
mod extract_context;

pub use extract_context::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "adversarial_tests.rs"]
mod adversarial_tests;
