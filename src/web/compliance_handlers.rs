//! Compliance API Handlers
//!
//! Provides REST endpoints for the compliance scanner that checks:
//! - Passwords in config files (not in vault)
//! - Fragile code patterns in .bas files
//! - Security issues and best practice violations
//!
//! ## Endpoints
//!
//! - `GET /api/compliance` - Get compliance summary
//! - `POST /api/compliance/scan` - Run a new compliance scan
//! - `GET /api/compliance/report/:id` - Get specific report
//! - `GET /api/compliance/export/:format` - Export report (json, csv, pdf)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::shared::state::AppState;

#[cfg(feature = "compliance")]
use crate::compliance::{
    CodeIssue, CodeScanner, ComplianceReporter, ComplianceScanResult, IssueSeverity, IssueType,
    ScanStats,
};

/// Compliance scan request
#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    /// Bot ID to scan (optional, scans all if not provided)
    pub bot_id: Option<String>,
    /// Specific paths to scan
    pub paths: Option<Vec<String>>,
    /// Whether to include info-level issues
    #[serde(default)]
    pub include_info: bool,
    /// Categories to scan (empty = all)
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Compliance scan response
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub scan_id: String,
    pub status: String,
    pub scanned_at: DateTime<Utc>,
    pub summary: ScanSummary,
    pub issues: Vec<IssueResponse>,
}

/// Summary of scan results
#[derive(Debug, Serialize, Default)]
pub struct ScanSummary {
    pub total_files_scanned: usize,
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub compliance_score: f64,
    pub categories: HashMap<String, usize>,
}

/// Individual issue in response
#[derive(Debug, Serialize)]
pub struct IssueResponse {
    pub id: String,
    pub severity: String,
    pub issue_type: String,
    pub title: String,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub code_snippet: Option<String>,
    pub remediation: String,
    pub category: String,
}

/// Query parameters for listing compliance reports
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub severity: Option<String>,
}

/// Export format options
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// Create compliance routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/compliance", get(get_compliance_summary))
        .route("/api/compliance/scan", post(run_compliance_scan))
        .route("/api/compliance/report/:id", get(get_report))
        .route("/api/compliance/issues", get(list_issues))
        .route("/api/compliance/export", get(export_report))
}

/// Get compliance summary - overview of current compliance status
async fn get_compliance_summary(
    State(_state): State<AppState>,
) -> Result<Json<ScanResponse>, (StatusCode, String)> {
    info!("Getting compliance summary");

    // Run a quick scan to get current status
    let scan_result = run_scan_internal(None, false).await?;

    Ok(Json(scan_result))
}

/// Run a new compliance scan
async fn run_compliance_scan(
    State(_state): State<AppState>,
    Json(request): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, (StatusCode, String)> {
    info!("Running compliance scan for bot: {:?}", request.bot_id);

    let scan_result = run_scan_internal(request.bot_id, request.include_info).await?;

    Ok(Json(scan_result))
}

/// Get a specific compliance report by ID
async fn get_report(
    State(_state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<ScanResponse>, (StatusCode, String)> {
    info!("Getting compliance report: {}", report_id);

    // For now, run a fresh scan
    // In production, this would retrieve from storage
    let scan_result = run_scan_internal(None, true).await?;

    Ok(Json(scan_result))
}

/// List all compliance issues with filtering
async fn list_issues(
    State(_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<IssueResponse>>, (StatusCode, String)> {
    info!("Listing compliance issues");

    let scan_result = run_scan_internal(None, true).await?;

    let mut issues = scan_result.issues;

    // Filter by severity if specified
    if let Some(severity) = query.severity {
        issues.retain(|i| i.severity.to_lowercase() == severity.to_lowercase());
    }

    // Apply pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let paginated: Vec<IssueResponse> = issues.into_iter().skip(offset).take(limit).collect();

    Ok(Json(paginated))
}

/// Export compliance report in various formats
async fn export_report(
    State(_state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> impl IntoResponse {
    let format = query.format.unwrap_or_else(|| "json".to_string());

    info!("Exporting compliance report as: {}", format);

    match run_scan_internal(None, true).await {
        Ok(scan_result) => match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&scan_result).unwrap_or_default();
                (
                    StatusCode::OK,
                    [
                        ("Content-Type", "application/json"),
                        (
                            "Content-Disposition",
                            "attachment; filename=\"compliance-report.json\"",
                        ),
                    ],
                    json,
                )
            }
            "csv" => {
                let csv = generate_csv(&scan_result);
                (
                    StatusCode::OK,
                    [
                        ("Content-Type", "text/csv"),
                        (
                            "Content-Disposition",
                            "attachment; filename=\"compliance-report.csv\"",
                        ),
                    ],
                    csv,
                )
            }
            _ => (
                StatusCode::BAD_REQUEST,
                [("Content-Type", "text/plain"), ("Content-Disposition", "")],
                format!("Unsupported format: {}. Use 'json' or 'csv'.", format),
            ),
        },
        Err((status, msg)) => (
            status,
            [("Content-Type", "text/plain"), ("Content-Disposition", "")],
            msg,
        ),
    }
}

/// Internal function to run the compliance scan
async fn run_scan_internal(
    bot_id: Option<String>,
    include_info: bool,
) -> Result<ScanResponse, (StatusCode, String)> {
    let scan_id = Uuid::new_v4().to_string();
    let scanned_at = Utc::now();

    // Collect issues from various scanners
    let mut all_issues: Vec<IssueResponse> = Vec::new();
    let mut files_scanned = 0;

    // Scan for passwords in config files
    let config_issues = scan_config_files().await;
    all_issues.extend(config_issues.iter().cloned());

    // Scan .bas files for fragile code
    let bas_issues = scan_bas_files().await;
    files_scanned += bas_issues.len();
    all_issues.extend(bas_issues);

    // Scan for security issues
    let security_issues = scan_security_issues().await;
    all_issues.extend(security_issues);

    // Filter out info level if not requested
    if !include_info {
        all_issues.retain(|i| i.severity.to_lowercase() != "info");
    }

    // Calculate summary
    let summary = calculate_summary(&all_issues, files_scanned);

    Ok(ScanResponse {
        scan_id,
        status: "completed".to_string(),
        scanned_at,
        summary,
        issues: all_issues,
    })
}

/// Scan config files for passwords and secrets
async fn scan_config_files() -> Vec<IssueResponse> {
    let mut issues = Vec::new();

    // Patterns that indicate passwords in config
    let password_patterns = [
        ("password", "Password field found in config"),
        ("api_key", "API key found in config"),
        ("secret", "Secret found in config"),
        ("token", "Token found in config"),
        ("private_key", "Private key found in config"),
    ];

    // Check common config file locations
    let config_paths = [
        ".env",
        "config.csv",
        "config.json",
        "settings.json",
        ".gbai/config.csv",
    ];

    for path in &config_paths {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            for (pattern, description) in &password_patterns {
                if content.to_lowercase().contains(pattern) {
                    // Check if it's using vault reference
                    if !content.contains("vault://") && !content.contains("${VAULT_") {
                        issues.push(IssueResponse {
                            id: Uuid::new_v4().to_string(),
                            severity: "critical".to_string(),
                            issue_type: "password_in_config".to_string(),
                            title: format!("{} not using vault", description),
                            description: format!(
                                "Found '{}' in {} without vault reference. Secrets should be stored in a vault, not in config files.",
                                pattern, path
                            ),
                            file_path: path.to_string(),
                            line_number: None,
                            code_snippet: None,
                            remediation: format!(
                                "Move the {} to a vault and reference it using vault://path/to/secret or ${{VAULT_SECRET_NAME}}",
                                pattern
                            ),
                            category: "secrets".to_string(),
                        });
                    }
                }
            }
        }
    }

    issues
}

/// Scan .bas files for fragile code patterns
async fn scan_bas_files() -> Vec<IssueResponse> {
    let mut issues = Vec::new();

    // Fragile code patterns to detect
    let fragile_patterns = [
        (
            r"IF\s+.+\s*=\s*input",
            "deprecated_if_input",
            "high",
            "Deprecated IF...input pattern",
            "Use HEAR keyword with validation instead of direct input comparison",
        ),
        (
            r"GOTO\s+\w+",
            "fragile_code",
            "medium",
            "GOTO statement found",
            "Replace GOTO with structured control flow (IF/FOR/SWITCH)",
        ),
        (
            r#"password\s*=\s*["'][^"']+["']"#,
            "hardcoded_secret",
            "critical",
            "Hardcoded password in code",
            "Use GET BOT MEMORY or vault references instead of hardcoding passwords",
        ),
        (
            r"[A-Z]+_[A-Z]+",
            "underscore_in_keyword",
            "info",
            "Keyword uses underscore instead of space",
            "Use spaces in keywords (e.g., 'GET BOT MEMORY' instead of 'GET_BOT_MEMORY')",
        ),
        (
            r"(?i)exec\s*\(",
            "insecure_pattern",
            "critical",
            "Dynamic code execution detected",
            "Avoid dynamic code execution. Use predefined procedures instead.",
        ),
        (
            r"(?i)eval\s*\(",
            "insecure_pattern",
            "critical",
            "Eval statement detected",
            "Avoid eval. Use structured data handling instead.",
        ),
    ];

    // Walk through .bas files
    let base_paths = [".", "dialogs", "templates"];

    for base_path in &base_paths {
        if let Ok(mut entries) = tokio::fs::read_dir(base_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "bas").unwrap_or(false) {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        for (line_num, line) in content.lines().enumerate() {
                            for (pattern, issue_type, severity, title, remediation) in
                                &fragile_patterns
                            {
                                if let Ok(re) = regex::Regex::new(pattern) {
                                    if re.is_match(line) {
                                        // Skip underscore warnings for internal function names
                                        if *issue_type == "underscore_in_keyword"
                                            && (line.contains("register_fn")
                                                || line.starts_with("'")
                                                || line.starts_with("REM"))
                                        {
                                            continue;
                                        }

                                        issues.push(IssueResponse {
                                            id: Uuid::new_v4().to_string(),
                                            severity: severity.to_string(),
                                            issue_type: issue_type.to_string(),
                                            title: title.to_string(),
                                            description: format!(
                                                "Found fragile code pattern at line {}",
                                                line_num + 1
                                            ),
                                            file_path: path.display().to_string(),
                                            line_number: Some(line_num + 1),
                                            code_snippet: Some(line.trim().to_string()),
                                            remediation: remediation.to_string(),
                                            category: "code_quality".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    issues
}

/// Scan for general security issues
async fn scan_security_issues() -> Vec<IssueResponse> {
    let mut issues = Vec::new();

    // Check for missing security configurations
    let security_checks = [
        (
            ".env",
            "ENCRYPTION_KEY",
            "Encryption key not configured",
            "high",
        ),
        (".env", "JWT_SECRET", "JWT secret not configured", "high"),
        (
            ".env",
            "RATE_LIMIT_ENABLED",
            "Rate limiting not enabled",
            "medium",
        ),
        (
            ".env",
            "CORS_ORIGINS",
            "CORS origins not configured",
            "medium",
        ),
    ];

    for (file, key, title, severity) in &security_checks {
        if let Ok(content) = tokio::fs::read_to_string(file).await {
            if !content.contains(key) {
                issues.push(IssueResponse {
                    id: Uuid::new_v4().to_string(),
                    severity: severity.to_string(),
                    issue_type: "configuration_issue".to_string(),
                    title: title.to_string(),
                    description: format!("{} is not set in {}", key, file),
                    file_path: file.to_string(),
                    line_number: None,
                    code_snippet: None,
                    remediation: format!("Add {} to your {} file", key, file),
                    category: "security".to_string(),
                });
            }
        }
    }

    issues
}

/// Calculate summary statistics
fn calculate_summary(issues: &[IssueResponse], files_scanned: usize) -> ScanSummary {
    let mut summary = ScanSummary {
        total_files_scanned: files_scanned,
        total_issues: issues.len(),
        ..Default::default()
    };

    let mut categories: HashMap<String, usize> = HashMap::new();

    for issue in issues {
        match issue.severity.to_lowercase().as_str() {
            "critical" => summary.critical_count += 1,
            "high" => summary.high_count += 1,
            "medium" => summary.medium_count += 1,
            "low" => summary.low_count += 1,
            "info" => summary.info_count += 1,
            _ => {}
        }

        *categories.entry(issue.category.clone()).or_insert(0) += 1;
    }

    summary.categories = categories;

    // Calculate compliance score (100 - weighted issues)
    let weighted_issues = (summary.critical_count * 25)
        + (summary.high_count * 15)
        + (summary.medium_count * 5)
        + (summary.low_count * 1);

    summary.compliance_score = (100.0 - weighted_issues as f64).max(0.0);

    summary
}

/// Generate CSV export
fn generate_csv(report: &ScanResponse) -> String {
    let mut csv = String::from("ID,Severity,Type,Title,File,Line,Category,Remediation\n");

    for issue in &report.issues {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            issue.id,
            issue.severity,
            issue.issue_type,
            issue.title.replace('"', "\"\""),
            issue.file_path,
            issue.line_number.map(|n| n.to_string()).unwrap_or_default(),
            issue.category,
            issue.remediation.replace('"', "\"\""),
        ));
    }

    csv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_summary() {
        let issues = vec![
            IssueResponse {
                id: "1".to_string(),
                severity: "critical".to_string(),
                issue_type: "test".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                file_path: "test.bas".to_string(),
                line_number: Some(1),
                code_snippet: None,
                remediation: "Fix it".to_string(),
                category: "security".to_string(),
            },
            IssueResponse {
                id: "2".to_string(),
                severity: "high".to_string(),
                issue_type: "test".to_string(),
                title: "Test 2".to_string(),
                description: "Test 2".to_string(),
                file_path: "test2.bas".to_string(),
                line_number: Some(5),
                code_snippet: None,
                remediation: "Fix it".to_string(),
                category: "code_quality".to_string(),
            },
        ];

        let summary = calculate_summary(&issues, 10);

        assert_eq!(summary.total_issues, 2);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.high_count, 1);
        assert_eq!(summary.total_files_scanned, 10);
        assert!(summary.compliance_score < 100.0);
    }

    #[test]
    fn test_generate_csv() {
        let report = ScanResponse {
            scan_id: "test-123".to_string(),
            status: "completed".to_string(),
            scanned_at: Utc::now(),
            summary: ScanSummary::default(),
            issues: vec![IssueResponse {
                id: "1".to_string(),
                severity: "high".to_string(),
                issue_type: "test".to_string(),
                title: "Test Issue".to_string(),
                description: "Description".to_string(),
                file_path: "test.bas".to_string(),
                line_number: Some(10),
                code_snippet: None,
                remediation: "Fix it".to_string(),
                category: "security".to_string(),
            }],
        };

        let csv = generate_csv(&report);
        assert!(csv.contains("ID,Severity"));
        assert!(csv.contains("Test Issue"));
        assert!(csv.contains("test.bas"));
    }
}
