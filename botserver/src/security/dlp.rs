use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

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

impl Default for DlpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_inbound: true,
            scan_outbound: true,
            block_on_violation: false,
            log_violations: true,
            redact_sensitive_data: true,
            max_content_length: 10 * 1024 * 1024,
        }
    }
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

impl SensitiveDataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::PhoneNumber => "phone_number",
            Self::CreditCard => "credit_card",
            Self::Ssn => "ssn",
            Self::IpAddress => "ip_address",
            Self::ApiKey => "api_key",
            Self::Password => "password",
            Self::AwsAccessKey => "aws_access_key",
            Self::AwsSecretKey => "aws_secret_key",
            Self::PrivateKey => "private_key",
            Self::JwtToken => "jwt_token",
            Self::BankAccount => "bank_account",
            Self::PassportNumber => "passport_number",
            Self::DriversLicense => "drivers_license",
            Self::HealthId => "health_id",
            Self::Custom => "custom",
        }
    }

    pub fn severity(&self) -> DlpSeverity {
        match self {
            Self::CreditCard | Self::Ssn | Self::BankAccount | Self::PrivateKey => {
                DlpSeverity::Critical
            }
            Self::AwsAccessKey
            | Self::AwsSecretKey
            | Self::ApiKey
            | Self::Password
            | Self::JwtToken => DlpSeverity::High,
            Self::PassportNumber | Self::DriversLicense | Self::HealthId => DlpSeverity::High,
            Self::Email | Self::PhoneNumber => DlpSeverity::Medium,
            Self::IpAddress => DlpSeverity::Low,
            Self::Custom => DlpSeverity::Medium,
        }
    }

    pub fn redaction_pattern(&self) -> &'static str {
        match self {
            Self::Email => "[EMAIL REDACTED]",
            Self::PhoneNumber => "[PHONE REDACTED]",
            Self::CreditCard => "[CARD REDACTED]",
            Self::Ssn => "[SSN REDACTED]",
            Self::IpAddress => "[IP REDACTED]",
            Self::ApiKey => "[API KEY REDACTED]",
            Self::Password => "[PASSWORD REDACTED]",
            Self::AwsAccessKey => "[AWS KEY REDACTED]",
            Self::AwsSecretKey => "[AWS SECRET REDACTED]",
            Self::PrivateKey => "[PRIVATE KEY REDACTED]",
            Self::JwtToken => "[TOKEN REDACTED]",
            Self::BankAccount => "[BANK ACCOUNT REDACTED]",
            Self::PassportNumber => "[PASSPORT REDACTED]",
            Self::DriversLicense => "[LICENSE REDACTED]",
            Self::HealthId => "[HEALTH ID REDACTED]",
            Self::Custom => "[REDACTED]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum DlpSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DlpSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 25,
            Self::Medium => 50,
            Self::High => 75,
            Self::Critical => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DlpAction {
    Allow,
    Warn,
    Redact,
    Block,
    Quarantine,
}

impl DlpAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Redact => "redact",
            Self::Block => "block",
            Self::Quarantine => "quarantine",
        }
    }
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

impl DlpMatch {
    pub fn new(
        data_type: SensitiveDataType,
        matched_text: &str,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            data_type,
            matched_text: matched_text.to_string(),
            start_position: start,
            end_position: end,
            confidence: 1.0,
            context: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn masked_value(&self) -> String {
        let len = self.matched_text.len();
        if len <= 4 {
            "*".repeat(len)
        } else {
            format!(
                "{}{}{}",
                &self.matched_text[..2],
                "*".repeat(len - 4),
                &self.matched_text[len - 2..]
            )
        }
    }
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

impl DlpScanResult {
    pub fn clean() -> Self {
        Self {
            matches: Vec::new(),
            severity: DlpSeverity::Low,
            action: DlpAction::Allow,
            blocked: false,
            redacted_content: None,
            policy_violations: Vec::new(),
        }
    }

    pub fn has_sensitive_data(&self) -> bool {
        !self.matches.is_empty()
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn data_types_found(&self) -> HashSet<SensitiveDataType> {
        self.matches.iter().map(|m| m.data_type).collect()
    }
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

impl DlpPolicy {
    pub fn new(name: &str, data_types: Vec<SensitiveDataType>, action: DlpAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: None,
            enabled: true,
            data_types,
            action,
            severity_threshold: DlpSeverity::Low,
            exceptions: Vec::new(),
            applies_to: PolicyScope::All,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_severity_threshold(mut self, threshold: DlpSeverity) -> Self {
        self.severity_threshold = threshold;
        self
    }

    pub fn with_scope(mut self, scope: PolicyScope) -> Self {
        self.applies_to = scope;
        self
    }

    pub fn add_exception(&mut self, exception: DlpException) {
        self.exceptions.push(exception);
    }

    pub fn matches_data_type(&self, data_type: SensitiveDataType) -> bool {
        self.data_types.contains(&data_type)
    }

    pub fn is_exception(&self, content: &str, user_id: Option<Uuid>) -> bool {
        for exception in &self.exceptions {
            match exception {
                DlpException::User(uid) => {
                    if user_id == Some(*uid) {
                        return true;
                    }
                }
                DlpException::Pattern(pattern) => {
                    if let Ok(re) = Regex::new(pattern) {
                        if re.is_match(content) {
                            return true;
                        }
                    }
                }
                DlpException::Domain(domain) => {
                    if content.contains(domain) {
                        return true;
                    }
                }
            }
        }
        false
    }
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

impl DlpManager {
    pub fn new(config: DlpConfig) -> Result<Self> {
        let patterns = Self::build_detection_patterns()?;

        Ok(Self {
            config,
            patterns,
            policies: Arc::new(RwLock::new(Vec::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            custom_patterns: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn with_defaults() -> Result<Self> {
        Self::new(DlpConfig::default())
    }

    fn build_detection_patterns() -> Result<HashMap<SensitiveDataType, Vec<Regex>>> {
        let mut patterns = HashMap::new();

        patterns.insert(
            SensitiveDataType::Email,
            vec![Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?],
        );

        patterns.insert(
            SensitiveDataType::PhoneNumber,
            vec![
                Regex::new(r"\b\d{3}[-.\s]?\d{3}[-.\s]?\d{4}\b")?,
                Regex::new(r"\b\+\d{1,3}[-.\s]?\d{1,4}[-.\s]?\d{1,4}[-.\s]?\d{1,9}\b")?,
                Regex::new(r"\(\d{3}\)\s*\d{3}[-.\s]?\d{4}")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::CreditCard,
            vec![
                Regex::new(r"\b4\d{3}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b")?,
                Regex::new(r"\b5[1-5]\d{2}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b")?,
                Regex::new(r"\b3[47]\d{2}[-\s]?\d{6}[-\s]?\d{5}\b")?,
                Regex::new(r"\b6(?:011|5\d{2})[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::Ssn,
            vec![
                Regex::new(r"\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b")?,
                Regex::new(r"(?i)\bssn[:\s]*\d{3}[-\s]?\d{2}[-\s]?\d{4}\b")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::IpAddress,
            vec![
                Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b")?,
                Regex::new(r"\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::ApiKey,
            vec![
                Regex::new(r#"(?i)(?:api[_-]?key|apikey)["\s:=]+["']?([a-zA-Z0-9_\-]{20,})["']?"#)?,
                Regex::new(r#"(?i)(?:secret|token)[_\-]?key[:\s=]+['\"]?([a-zA-Z0-9_\-]{16,})['\"]?"#)?,
            ],
        );

        patterns.insert(
            SensitiveDataType::Password,
            vec![
                Regex::new(r#"(?i)password["\s:=]+["']?([^\s"']{8,})["']?"#)?,
                Regex::new(r#"(?i)(?:passwd|pwd)[:\s=]+['\"]?([^\s'\"]{8,})['\"]?"#)?,
            ],
        );

        patterns.insert(
            SensitiveDataType::AwsAccessKey,
            vec![Regex::new(r"\b(?:AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b")?],
        );

        patterns.insert(
            SensitiveDataType::AwsSecretKey,
            vec![Regex::new(r#"(?i)aws[_\-]?secret[_\-]?(?:access[_\-]?)?key[:\s=]+['\"]?([a-zA-Z0-9/+=]{40})['\"]?"#)?],
        );

        patterns.insert(
            SensitiveDataType::PrivateKey,
            vec![
                Regex::new(r"-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----")?,
                Regex::new(r"-----BEGIN PGP PRIVATE KEY BLOCK-----")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::JwtToken,
            vec![Regex::new(r"\beyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*\b")?],
        );

        patterns.insert(
            SensitiveDataType::BankAccount,
            vec![
                Regex::new(r#"(?i)(?:account|acct)[_\-\s]?(?:number|no|#)?[:\s]*\d{8,17}"#)?,
                Regex::new(r#"(?i)(?:routing|aba)[_\-\s]?(?:number|no|#)?[:\s]*\d{9}"#)?,
                Regex::new(r"\b[A-Z]{2}\d{2}[A-Z0-9]{4}\d{7}(?:[A-Z0-9]?){0,16}\b")?,
            ],
        );

        patterns.insert(
            SensitiveDataType::PassportNumber,
            vec![Regex::new(r"(?i)passport[_\-\s]?(?:number|no|#)?[:\s]*[A-Z0-9]{6,9}")?],
        );

        patterns.insert(
            SensitiveDataType::DriversLicense,
            vec![Regex::new(r"(?i)(?:driver'?s?|dl)[_\-\s]?(?:license|lic)[_\-\s]?(?:number|no|#)?[:\s]*[A-Z0-9]{5,15}")?],
        );

        patterns.insert(
            SensitiveDataType::HealthId,
            vec![
                Regex::new(r"(?i)(?:medicare|medicaid)[_\-\s]?(?:id|number|no|#)?[:\s]*[A-Z0-9]{10,12}")?,
                Regex::new(r"(?i)(?:health|medical)[_\-\s]?(?:id|record)[_\-\s]?(?:number|no|#)?[:\s]*[A-Z0-9]{8,15}")?,
            ],
        );

        Ok(patterns)
    }

    pub async fn scan(&self, content: &str) -> DlpScanResult {
        if !self.config.enabled {
            return DlpScanResult::clean();
        }

        if content.len() > self.config.max_content_length {
            return DlpScanResult {
                matches: Vec::new(),
                severity: DlpSeverity::Medium,
                action: DlpAction::Block,
                blocked: true,
                redacted_content: None,
                policy_violations: vec!["Content exceeds maximum allowed length".into()],
            };
        }

        let mut all_matches = Vec::new();

        for (data_type, patterns) in &self.patterns {
            for pattern in patterns {
                for mat in pattern.find_iter(content) {
                    let context = extract_context(content, mat.start(), mat.end(), 20);
                    let dlp_match = DlpMatch::new(*data_type, mat.as_str(), mat.start(), mat.end())
                        .with_context(context);
                    all_matches.push(dlp_match);
                }
            }
        }

        let custom_patterns = self.custom_patterns.read().await;
        for (_, pattern, data_type) in custom_patterns.iter() {
            for mat in pattern.find_iter(content) {
                let context = extract_context(content, mat.start(), mat.end(), 20);
                let dlp_match = DlpMatch::new(*data_type, mat.as_str(), mat.start(), mat.end())
                    .with_context(context);
                all_matches.push(dlp_match);
            }
        }
        drop(custom_patterns);

        if all_matches.is_empty() {
            return DlpScanResult::clean();
        }

        let severity = all_matches
            .iter()
            .map(|m| m.data_type.severity())
            .max()
            .unwrap_or(DlpSeverity::Low);

        let (action, blocked, policy_violations) =
            self.evaluate_policies(&all_matches, None).await;

        let redacted_content = if self.config.redact_sensitive_data {
            Some(self.redact_content(content, &all_matches))
        } else {
            None
        };

        if self.config.log_violations && !all_matches.is_empty() {
            warn!(
                "DLP scan found {} sensitive data matches, severity: {}, action: {}",
                all_matches.len(),
                severity.as_str(),
                action.as_str()
            );
        }

        DlpScanResult {
            matches: all_matches,
            severity,
            action,
            blocked,
            redacted_content,
            policy_violations,
        }
    }

    pub async fn scan_with_user(
        &self,
        content: &str,
        user_id: Uuid,
        direction: ScanDirection,
    ) -> DlpScanResult {
        if !self.config.enabled {
            return DlpScanResult::clean();
        }

        match direction {
            ScanDirection::Inbound if !self.config.scan_inbound => return DlpScanResult::clean(),
            ScanDirection::Outbound if !self.config.scan_outbound => return DlpScanResult::clean(),
            _ => {}
        }

        let mut result = self.scan(content).await;

        if result.has_sensitive_data() {
            let (action, blocked, violations) =
                self.evaluate_policies(&result.matches, Some(user_id)).await;
            result.action = action;
            result.blocked = blocked;
            result.policy_violations = violations;
        }

        result
    }

    async fn evaluate_policies(
        &self,
        matches: &[DlpMatch],
        user_id: Option<Uuid>,
    ) -> (DlpAction, bool, Vec<String>) {
        let policies = self.policies.read().await;
        let mut highest_action = DlpAction::Allow;
        let mut blocked = false;
        let mut violations = Vec::new();

        for policy in policies.iter().filter(|p| p.enabled) {
            for dlp_match in matches {
                if !policy.matches_data_type(dlp_match.data_type) {
                    continue;
                }

                if dlp_match.data_type.severity() < policy.severity_threshold {
                    continue;
                }

                if policy.is_exception(&dlp_match.matched_text, user_id) {
                    continue;
                }

                violations.push(format!(
                    "Policy '{}' violated: {} detected",
                    policy.name,
                    dlp_match.data_type.as_str()
                ));

                if policy.action as u8 > highest_action as u8 {
                    highest_action = policy.action;
                }

                if policy.action == DlpAction::Block || policy.action == DlpAction::Quarantine {
                    blocked = true;
                }
            }
        }

        if violations.is_empty() && self.config.block_on_violation && !matches.is_empty() {
            let critical_found = matches
                .iter()
                .any(|m| m.data_type.severity() >= DlpSeverity::Critical);

            if critical_found {
                blocked = true;
                highest_action = DlpAction::Block;
                violations.push("Critical sensitive data detected".into());
            }
        }

        (highest_action, blocked, violations)
    }

    pub fn redact_content(&self, content: &str, matches: &[DlpMatch]) -> String {
        let mut redacted = content.to_string();
        let mut sorted_matches: Vec<&DlpMatch> = matches.iter().collect();
        sorted_matches.sort_by(|a, b| b.start_position.cmp(&a.start_position));

        for dlp_match in sorted_matches {
            let redaction = dlp_match.data_type.redaction_pattern();
            redacted.replace_range(dlp_match.start_position..dlp_match.end_position, redaction);
        }

        redacted
    }

    pub fn partial_redact_content(&self, content: &str, matches: &[DlpMatch]) -> String {
        let mut redacted = content.to_string();
        let mut sorted_matches: Vec<&DlpMatch> = matches.iter().collect();
        sorted_matches.sort_by(|a, b| b.start_position.cmp(&a.start_position));

        for dlp_match in sorted_matches {
            let masked = dlp_match.masked_value();
            redacted.replace_range(dlp_match.start_position..dlp_match.end_position, &masked);
        }

        redacted
    }

    pub async fn add_policy(&self, policy: DlpPolicy) {
        let mut policies = self.policies.write().await;
        policies.push(policy);
    }

    pub async fn remove_policy(&self, policy_id: Uuid) -> bool {
        let mut policies = self.policies.write().await;
        let initial_len = policies.len();
        policies.retain(|p| p.id != policy_id);
        policies.len() < initial_len
    }

    pub async fn get_policies(&self) -> Vec<DlpPolicy> {
        let policies = self.policies.read().await;
        policies.clone()
    }

    pub async fn add_custom_pattern(
        &self,
        name: &str,
        pattern: &str,
        data_type: SensitiveDataType,
    ) -> Result<()> {
        let regex =
            Regex::new(pattern).map_err(|e| anyhow!("Invalid regex pattern: {e}"))?;

        let mut patterns = self.custom_patterns.write().await;
        patterns.push((name.to_string(), regex, data_type));

        info!("Added custom DLP pattern: {}", name);
        Ok(())
    }

    pub async fn record_violation(
        &self,
        policy_id: Uuid,
        policy_name: &str,
        user_id: Option<Uuid>,
        result: &DlpScanResult,
        content: &str,
    ) {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        let violation = DlpViolation {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            policy_id,
            policy_name: policy_name.to_string(),
            user_id,
            data_types: result.data_types_found().into_iter().collect(),
            action_taken: result.action,
            severity: result.severity,
            content_hash,
            match_count: result.match_count(),
        };

        let mut violations = self.violations.write().await;
        violations.push(violation);
    }

    pub async fn get_violations(
        &self,
        limit: usize,
        user_id: Option<Uuid>,
    ) -> Vec<DlpViolation> {
        let violations = self.violations.read().await;

        let filtered: Vec<DlpViolation> = if let Some(uid) = user_id {
            violations
                .iter()
                .filter(|v| v.user_id == Some(uid))
                .cloned()
                .collect()
        } else {
            violations.clone()
        };

        filtered.into_iter().rev().take(limit).collect()
    }

    pub fn config(&self) -> &DlpConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    Inbound,
    Outbound,
}

fn extract_context(content: &str, start: usize, end: usize, padding: usize) -> String {
    let context_start = start.saturating_sub(padding);
    let context_end = (end + padding).min(content.len());

    let mut context = String::new();

    if context_start > 0 {
        context.push_str("...");
    }

    context.push_str(&content[context_start..context_end]);

    if context_end < content.len() {
        context.push_str("...");
    }

    context
}

pub fn validate_credit_card_checksum(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for digit in digits.iter().rev() {
        let mut d = *digit;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum % 10 == 0
}

pub fn create_default_policies() -> Vec<DlpPolicy> {
    vec![
        DlpPolicy::new(
            "Block Credit Cards",
            vec![SensitiveDataType::CreditCard],
            DlpAction::Block,
        )
        .with_description("Block transmission of credit card numbers"),
        DlpPolicy::new(
            "Block SSN",
            vec![SensitiveDataType::Ssn],
            DlpAction::Block,
        )
        .with_description("Block transmission of Social Security Numbers"),
        DlpPolicy::new(
            "Warn on API Keys",
            vec![
                SensitiveDataType::ApiKey,
                SensitiveDataType::AwsAccessKey,
                SensitiveDataType::AwsSecretKey,
            ],
            DlpAction::Warn,
        )
        .with_description("Warn when API keys or secrets are detected"),
        DlpPolicy::new(
            "Redact Emails",
            vec![SensitiveDataType::Email],
            DlpAction::Redact,
        )
        .with_description("Redact email addresses in outbound content")
        .with_scope(PolicyScope::Outbound),
        DlpPolicy::new(
            "Block Private Keys",
            vec![SensitiveDataType::PrivateKey],
            DlpAction::Block,
        )
        .with_description("Block transmission of private keys"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_detection() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "Contact me at test@example.com for more info";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::Email));
    }

    #[tokio::test]
    async fn test_credit_card_detection() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "My card number is 4111-1111-1111-1111";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::CreditCard));
    }

    #[tokio::test]
    async fn test_ssn_detection() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "SSN: 123-45-6789";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::Ssn));
    }

    #[tokio::test]
    async fn test_aws_key_detection() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "AWS key: AKIAIOSFODNN7EXAMPLE";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::AwsAccessKey));
    }

    #[tokio::test]
    async fn test_jwt_detection() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::JwtToken));
    }

    #[tokio::test]
    async fn test_clean_content() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "This is just a regular message with no sensitive data.";
        let result = manager.scan(content).await;

        assert!(!result.has_sensitive_data());
        assert_eq!(result.action, DlpAction::Allow);
    }

    #[tokio::test]
    async fn test_redaction() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "Email me at user@example.com please";
        let result = manager.scan(content).await;

        assert!(result.redacted_content.is_some());
        let redacted = result.redacted_content.unwrap();
        assert!(redacted.contains("[EMAIL REDACTED]"));
        assert!(!redacted.contains("user@example.com"));
    }

    #[tokio::test]
    async fn test_partial_redaction() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "Card: 4111111111111111";
        let result = manager.scan(content).await;

        let partial = manager.partial_redact_content(content, &result.matches);
        assert!(partial.contains("41"));
        assert!(partial.contains("11"));
        assert!(partial.contains("*"));
    }

    #[tokio::test]
    async fn test_multiple_matches() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let content = "Contact: test@example.com, phone: 555-123-4567, card: 4111111111111111";
        let result = manager.scan(content).await;

        assert!(result.match_count() >= 3);
        assert!(result.data_types_found().contains(&SensitiveDataType::Email));
        assert!(result.data_types_found().contains(&SensitiveDataType::PhoneNumber));
        assert!(result.data_types_found().contains(&SensitiveDataType::CreditCard));
    }

    #[tokio::test]
    async fn test_policy_evaluation() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        let policy = DlpPolicy::new(
            "Block SSN",
            vec![SensitiveDataType::Ssn],
            DlpAction::Block,
        );
        manager.add_policy(policy).await;

        let content = "SSN: 123-45-6789";
        let result = manager.scan(content).await;

        assert!(result.blocked);
        assert_eq!(result.action, DlpAction::Block);
    }

    #[test]
    fn test_credit_card_checksum_valid() {
        assert!(validate_credit_card_checksum("4111111111111111"));
        assert!(validate_credit_card_checksum("5500000000000004"));
    }

    #[test]
    fn test_credit_card_checksum_invalid() {
        assert!(!validate_credit_card_checksum("4111111111111112"));
        assert!(!validate_credit_card_checksum("1234567890123456"));
    }

    #[test]
    fn test_severity_levels() {
        assert_eq!(SensitiveDataType::CreditCard.severity(), DlpSeverity::Critical);
        assert_eq!(SensitiveDataType::Email.severity(), DlpSeverity::Medium);
        assert_eq!(SensitiveDataType::IpAddress.severity(), DlpSeverity::Low);
    }

    #[test]
    fn test_dlp_match_masking() {
        let dlp_match = DlpMatch::new(
            SensitiveDataType::CreditCard,
            "4111111111111111",
            0,
            16,
        );

        let masked = dlp_match.masked_value();
        assert!(masked.starts_with("41"));
        assert!(masked.ends_with("11"));
        assert!(masked.contains("*"));
    }

    #[test]
    fn test_default_policies() {
        let policies = create_default_policies();

        assert!(!policies.is_empty());
        assert!(policies.iter().any(|p| p.name == "Block Credit Cards"));
        assert!(policies.iter().any(|p| p.name == "Block SSN"));
    }

    #[tokio::test]
    async fn test_custom_pattern() {
        let manager = DlpManager::with_defaults().expect("Failed to create manager");

        manager
            .add_custom_pattern("Custom ID", r"CUSTOM-\d{6}", SensitiveDataType::Custom)
            .await
            .expect("Failed to add pattern");

        let content = "ID: CUSTOM-123456";
        let result = manager.scan(content).await;

        assert!(result.has_sensitive_data());
        assert!(result.data_types_found().contains(&SensitiveDataType::Custom));
    }

    #[test]
    fn test_policy_exception() {
        let mut policy = DlpPolicy::new(
            "Test Policy",
            vec![SensitiveDataType::Email],
            DlpAction::Block,
        );

        policy.add_exception(DlpException::Domain("@internal.com".into()));

        assert!(policy.is_exception("user@internal.com", None));
        assert!(!policy.is_exception("user@external.com", None));
    }

    #[test]
    fn test_extract_context() {
        let content = "This is a test email@example.com in the middle";
        let context = extract_context(content, 15, 33, 10);

        assert!(context.contains("email@example.com"));
        assert!(context.len() < content.len() + 10);
    }

    #[tokio::test]
    async fn test_scan_direction() {
        let mut config = DlpConfig::default();
        config.scan_inbound = false;
        let manager = DlpManager::new(config).expect("Failed to create manager");

        let content = "Email: test@example.com";
        let user_id = Uuid::new_v4();

        let inbound_result = manager
            .scan_with_user(content, user_id, ScanDirection::Inbound)
            .await;
        assert!(!inbound_result.has_sensitive_data());

        let outbound_result = manager
            .scan_with_user(content, user_id, ScanDirection::Outbound)
            .await;
        assert!(outbound_result.has_sensitive_data());
    }
}
