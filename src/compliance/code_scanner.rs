use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    PasswordInConfig,
    HardcodedSecret,
    DeprecatedKeyword,
    FragileCode,
    ConfigurationIssue,
    UnderscoreInKeyword,
    MissingVault,
    InsecurePattern,
    DeprecatedIfInput,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PasswordInConfig => write!(f, "Password in Config"),
            Self::HardcodedSecret => write!(f, "Hardcoded Secret"),
            Self::DeprecatedKeyword => write!(f, "Deprecated Keyword"),
            Self::FragileCode => write!(f, "Fragile Code"),
            Self::ConfigurationIssue => write!(f, "Configuration Issue"),
            Self::UnderscoreInKeyword => write!(f, "Underscore in Keyword"),
            Self::MissingVault => write!(f, "Missing Vault Config"),
            Self::InsecurePattern => write!(f, "Insecure Pattern"),
            Self::DeprecatedIfInput => write!(f, "Deprecated IF...input Pattern"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub code_snippet: Option<String>,
    pub remediation: String,
    pub category: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotScanResult {
    pub bot_id: String,
    pub bot_name: String,
    pub scanned_at: DateTime<Utc>,
    pub files_scanned: usize,
    pub issues: Vec<CodeIssue>,
    pub stats: ScanStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanStats {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub total: usize,
}

impl ScanStats {
    pub fn add_issue(&mut self, severity: &IssueSeverity) {
        match severity {
            IssueSeverity::Critical => self.critical += 1,
            IssueSeverity::High => self.high += 1,
            IssueSeverity::Medium => self.medium += 1,
            IssueSeverity::Low => self.low += 1,
            IssueSeverity::Info => self.info += 1,
        }
        self.total += 1;
    }

    pub fn merge(&mut self, other: &Self) {
        self.critical += other.critical;
        self.high += other.high;
        self.medium += other.medium;
        self.low += other.low;
        self.info += other.info;
        self.total += other.total;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScanResult {
    pub scanned_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub bots_scanned: usize,
    pub total_files: usize,
    pub stats: ScanStats,
    pub bot_results: Vec<BotScanResult>,
}

struct ScanPattern {
    regex: Regex,
    issue_type: IssueType,
    severity: IssueSeverity,
    title: String,
    description: String,
    remediation: String,
    category: String,
}

pub struct CodeScanner {
    patterns: Vec<ScanPattern>,
    base_path: PathBuf,
}

impl CodeScanner {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        let patterns = Self::build_patterns();
        Self {
            patterns,
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn build_patterns() -> Vec<ScanPattern> {
        vec![
        ScanPattern {
            regex: Regex::new(r#"(?i)password\s*=\s*["'][^"']+["']"#).unwrap(),
            issue_type: IssueType::PasswordInConfig,
            severity: IssueSeverity::Critical,
            title: "Hardcoded Password".to_string(),
            description: "A password is hardcoded in the source code. This is a critical security risk.".to_string(),
            remediation: "Move the password to Vault using: vault_password = GET VAULT SECRET \"password_key\"".to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r#"(?i)(api[_-]?key|apikey|secret[_-]?key|client[_-]?secret)\s*=\s*["'][^"']{8,}["']"#).unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::Critical,
            title: "Hardcoded API Key/Secret".to_string(),
            description: "An API key or secret is hardcoded in the source code.".to_string(),
            remediation: "Store secrets in Vault and retrieve with GET VAULT SECRET".to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r#"(?i)token\s*=\s*["'][a-zA-Z0-9_\-]{20,}["']"#).unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::High,
            title: "Hardcoded Token".to_string(),
            description: "A token appears to be hardcoded in the source code.".to_string(),
            remediation: "Store tokens securely in Vault".to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)IF\s+.*\binput\b").unwrap(),
            issue_type: IssueType::DeprecatedIfInput,
            severity: IssueSeverity::Medium,
            title: "Deprecated IF...input Pattern".to_string(),
            description:
                "Using IF with raw input variable. Prefer HEAR AS for type-safe input handling."
                    .to_string(),
            remediation: "Replace with: HEAR response AS STRING\nIF response = \"value\" THEN"
                .to_string(),
            category: "Code Quality".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)\b(GET_BOT_MEMORY|SET_BOT_MEMORY|GET_USER_MEMORY|SET_USER_MEMORY|USE_KB|USE_TOOL|SEND_MAIL|CREATE_TASK)\b").unwrap(),
            issue_type: IssueType::UnderscoreInKeyword,
            severity: IssueSeverity::Low,
            title: "Underscore in Keyword".to_string(),
            description: "Keywords should use spaces instead of underscores for consistency.".to_string(),
            remediation: "Use spaces: GET BOT MEMORY, SET BOT MEMORY, etc.".to_string(),
            category: "Naming Convention".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)POST\s+TO\s+INSTAGRAM\s+\w+\s*,\s*\w+").unwrap(),
            issue_type: IssueType::InsecurePattern,
            severity: IssueSeverity::High,
            title: "Instagram Credentials in Code".to_string(),
            description:
                "Instagram username/password passed directly. Use secure credential storage."
                    .to_string(),
            remediation: "Store Instagram credentials in Vault and retrieve securely.".to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE)\s+.*(FROM|INTO|SET)\s+").unwrap(),
            issue_type: IssueType::FragileCode,
            severity: IssueSeverity::Medium,
            title: "Raw SQL Query".to_string(),
            description: "Raw SQL queries in BASIC code may be vulnerable to injection."
                .to_string(),
            remediation:
                "Use parameterized queries or the built-in data operations (SAVE, GET, etc.)"
                    .to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)\bEVAL\s*\(").unwrap(),
            issue_type: IssueType::FragileCode,
            severity: IssueSeverity::High,
            title: "Dynamic Code Execution".to_string(),
            description: "EVAL can execute arbitrary code and is a security risk.".to_string(),
            remediation: "Avoid EVAL. Use structured control flow instead.".to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(
                r#"(?i)(password|secret|key|token)\s*=\s*["'][A-Za-z0-9+/=]{40,}["']"#,
            )
            .unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::High,
            title: "Potential Encoded Secret".to_string(),
            description: "A base64-like string is assigned to a sensitive variable.".to_string(),
            remediation: "Remove encoded secrets from code. Use Vault for secret management."
                .to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::Critical,
            title: "AWS Access Key".to_string(),
            description: "An AWS access key ID is hardcoded in the source code.".to_string(),
            remediation: "Remove immediately and rotate the key. Use IAM roles or Vault."
                .to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::Critical,
            title: "Private Key in Code".to_string(),
            description: "A private key is embedded in the source code.".to_string(),
            remediation: "Remove private key immediately. Store in secure key management system."
                .to_string(),
            category: "Security".to_string(),
        },
        ScanPattern {
            regex: Regex::new(r"(?i)(postgres|mysql|mongodb|redis)://[^:]+:[^@]+@").unwrap(),
            issue_type: IssueType::HardcodedSecret,
            severity: IssueSeverity::Critical,
            title: "Database Credentials in Connection String".to_string(),
            description: "Database connection string contains embedded credentials.".to_string(),
            remediation: "Use environment variables or Vault for database credentials.".to_string(),
            category: "Security".to_string(),
        },
        ]
    }

    pub async fn scan_all(
        &self,
    ) -> Result<ComplianceScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        let mut bot_results = Vec::new();
        let mut total_stats = ScanStats::default();
        let mut total_files = 0;

        let templates_path = self.base_path.join("templates");
        let work_path = self.base_path.join("work");

        let mut bot_paths = Vec::new();

        if templates_path.exists() {
            for entry in WalkDir::new(&templates_path)
                .max_depth(3)
                .into_iter()
                .flatten()
            {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.ends_with(".gbai") || name.ends_with(".gbdialog") {
                        bot_paths.push(path.to_path_buf());
                    }
                }
            }
        }

        if work_path.exists() {
            for entry in WalkDir::new(&work_path).max_depth(3).into_iter().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name.ends_with(".gbai") || name.ends_with(".gbdialog") {
                        bot_paths.push(path.to_path_buf());
                    }
                }
            }
        }

        for bot_path in &bot_paths {
            let result = self.scan_bot(bot_path).await?;
            total_files += result.files_scanned;
            total_stats.merge(&result.stats);
            bot_results.push(result);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ComplianceScanResult {
            scanned_at: Utc::now(),
            duration_ms,
            bots_scanned: bot_results.len(),
            total_files,
            stats: total_stats,
            bot_results,
        })
    }

    pub async fn scan_bot(
        &self,
        bot_path: &Path,
    ) -> Result<BotScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let bot_name = bot_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let bot_id =
            uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, bot_name.as_bytes()).to_string();

        let mut issues = Vec::new();
        let mut stats = ScanStats::default();
        let mut files_scanned = 0;

        for entry in WalkDir::new(bot_path).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                let extension = path.extension().unwrap_or_default().to_string_lossy();
                if extension == "bas" || extension == "csv" {
                    files_scanned += 1;
                    let file_issues = self.scan_file(path).await?;
                    for issue in file_issues {
                        stats.add_issue(&issue.severity);
                        issues.push(issue);
                    }
                }
            }
        }

        let config_path = bot_path.join("config.csv");
        if config_path.exists() {
            let vault_configured = self.check_vault_config(&config_path).await?;
            if !vault_configured {
                let issue = CodeIssue {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Info,
                    issue_type: IssueType::MissingVault,
                    title: "Vault Not Configured".to_string(),
                    description: "This bot is not configured to use Vault for secrets management.".to_string(),
                    file_path: config_path.to_string_lossy().to_string(),
                    line_number: None,
                    code_snippet: None,
                    remediation: "Add VAULT_ADDR and VAULT_TOKEN to configuration for secure secret management.".to_string(),
                    category: "Configuration".to_string(),
                    detected_at: Utc::now(),
                };
                stats.add_issue(&issue.severity);
                issues.push(issue);
            }
        }

        issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(BotScanResult {
            bot_id,
            bot_name,
            scanned_at: Utc::now(),
            files_scanned,
            issues,
            stats,
        })
    }

    async fn scan_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<CodeIssue>, Box<dyn std::error::Error + Send + Sync>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let mut issues = Vec::new();

        let relative_path = file_path
            .strip_prefix(&self.base_path)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        for (line_number, line) in content.lines().enumerate() {
            let line_num = line_number + 1;

            let trimmed = line.trim();
            if trimmed.starts_with("REM") || trimmed.starts_with('\'') || trimmed.starts_with("//")
            {
                continue;
            }

            for pattern in &self.patterns {
                if pattern.regex.is_match(line) {
                    let snippet = Self::redact_sensitive(line);

                    let issue = CodeIssue {
                        id: uuid::Uuid::new_v4().to_string(),
                        severity: pattern.severity.clone(),
                        issue_type: pattern.issue_type.clone(),
                        title: pattern.title.clone(),
                        description: pattern.description.clone(),
                        file_path: relative_path.clone(),
                        line_number: Some(line_num),
                        code_snippet: Some(snippet),
                        remediation: pattern.remediation.clone(),
                        category: pattern.category.clone(),
                        detected_at: Utc::now(),
                    };
                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    fn redact_sensitive(line: &str) -> String {
        let mut result = line.to_string();

        let secret_pattern = Regex::new(r#"(["'])[^"']{8,}(["'])"#).unwrap();
        result = secret_pattern
            .replace_all(&result, "$1***REDACTED***$2")
            .to_string();

        let aws_pattern = Regex::new(r"AKIA[0-9A-Z]{16}").unwrap();
        result = aws_pattern
            .replace_all(&result, "AKIA***REDACTED***")
            .to_string();

        result
    }

    async fn check_vault_config(
        &self,
        config_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let content = tokio::fs::read_to_string(config_path).await?;

        let has_vault = content.to_lowercase().contains("vault_addr")
            || content.to_lowercase().contains("vault_token")
            || content.to_lowercase().contains("vault-");

        Ok(has_vault)
    }

    pub async fn scan_bots(
        &self,
        bot_ids: &[String],
    ) -> Result<ComplianceScanResult, Box<dyn std::error::Error + Send + Sync>> {
        if bot_ids.is_empty() || bot_ids.contains(&"all".to_string()) {
            return self.scan_all().await;
        }

        let mut full_result = self.scan_all().await?;
        full_result
            .bot_results
            .retain(|r| bot_ids.contains(&r.bot_id) || bot_ids.contains(&r.bot_name));

        let mut new_stats = ScanStats::default();
        for bot in &full_result.bot_results {
            new_stats.merge(&bot.stats);
        }
        full_result.stats = new_stats;
        full_result.bots_scanned = full_result.bot_results.len();

        Ok(full_result)
    }
}

pub struct ComplianceReporter;

impl ComplianceReporter {
    pub fn to_html(result: &ComplianceScanResult) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html><html><head><title>Compliance Report</title>");
        html.push_str("<style>body{font-family:system-ui;margin:20px;}table{border-collapse:collapse;width:100%;}th,td{border:1px solid #ddd;padding:8px;text-align:left;}.critical{color:#dc2626;}.high{color:#ea580c;}.medium{color:#d97706;}.low{color:#65a30d;}.info{color:#0891b2;}</style>");
        html.push_str("</head><body>");

        html.push_str("<h1>Compliance Scan Report</h1>");
        use std::fmt::Write;
        let _ = write!(html, "<p>Scanned at: {}</p>", result.scanned_at);
        let _ = write!(html, "<p>Duration: {}ms</p>", result.duration_ms);
        let _ = write!(html, "<p>Bots scanned: {}</p>", result.bots_scanned);
        let _ = write!(html, "<p>Files scanned: {}</p>", result.total_files);

        html.push_str("<h2>Summary</h2>");
        let _ = write!(
            html,
            "<p class='critical'>Critical: {}</p>",
            result.stats.critical
        );
        let _ = write!(html, "<p class='high'>High: {}</p>", result.stats.high);
        let _ = write!(
            html,
            "<p class='medium'>Medium: {}</p>",
            result.stats.medium
        );
        let _ = write!(html, "<p class='low'>Low: {}</p>", result.stats.low);
        let _ = write!(html, "<p class='info'>Info: {}</p>", result.stats.info);

        html.push_str("<h2>Issues</h2>");
        html.push_str("<table><tr><th>Severity</th><th>Type</th><th>File</th><th>Line</th><th>Description</th></tr>");

        for bot in &result.bot_results {
            for issue in &bot.issues {
                let _ = write!(
                    html,
                    "<tr><td class='{}'>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    issue.severity,
                    issue.severity,
                    issue.issue_type,
                    issue.file_path,
                    issue
                        .line_number
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    issue.description
                );
            }
        }

        html.push_str("</table></body></html>");
        html
    }

    pub fn to_json(result: &ComplianceScanResult) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(result)
    }

    pub fn to_csv(result: &ComplianceScanResult) -> String {
        let mut csv = String::new();
        csv.push_str("Severity,Type,Category,File,Line,Title,Description,Remediation\n");

        for bot in &result.bot_results {
            for issue in &bot.issues {
                use std::fmt::Write;
                let _ = writeln!(
                    csv,
                    "{},{},{},{},{},{},{},{}",
                    issue.severity,
                    issue.issue_type,
                    issue.category,
                    issue.file_path,
                    issue
                        .line_number
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    escape_csv(&issue.title),
                    escape_csv(&issue.description),
                    escape_csv(&issue.remediation)
                );
            }
        }

        csv
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
