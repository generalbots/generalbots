use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;
use super::manager::{Finding, FindingSeverity, ScanResultStatus};

const LMD_LOG_DIR: &str = "/usr/local/maldetect/logs";
const LMD_QUARANTINE_DIR: &str = "/usr/local/maldetect/quarantine";

pub async fn run_scan(path: Option<&str>) -> Result<(ScanResultStatus, Vec<Finding>, String)> {
    info!("Running Linux Malware Detect scan");

    let scan_path = path.unwrap_or("/var/www");

    let output = SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("-a")?
        .arg(scan_path)?
        .execute()
        .context("Failed to run LMD scan")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let raw_output = format!("{stdout}\n{stderr}");

    let findings = parse_lmd_output(&stdout);
    let status = determine_result_status(&findings);

    Ok((status, findings, raw_output))
}

pub async fn run_background_scan(path: Option<&str>) -> Result<String> {
    info!("Starting LMD background scan");

    let scan_path = path.unwrap_or("/var/www");

    let output = SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("-b")?
        .arg("-a")?
        .arg(scan_path)?
        .execute()
        .context("Failed to start LMD background scan")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let scan_id = extract_scan_id(&stdout).unwrap_or_else(|| "unknown".to_string());

    info!("LMD background scan started with ID: {scan_id}");
    Ok(scan_id)
}

pub async fn update_signatures() -> Result<()> {
    info!("Updating LMD signatures");

    SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("--update-sigs")?
        .execute()
        .context("Failed to update LMD signatures")?;

    info!("LMD signatures updated successfully");
    Ok(())
}

pub async fn update_version() -> Result<()> {
    info!("Updating LMD version");

    SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("--update-ver")?
        .execute()
        .context("Failed to update LMD version")?;

    info!("LMD version updated successfully");
    Ok(())
}

pub async fn get_version() -> Result<String> {
    let output = SafeCommand::new("maldet")?
        .arg("--version")?
        .execute()
        .context("Failed to get LMD version")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .lines()
        .find(|l| l.contains("maldet") || l.contains("version"))
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

pub async fn get_signature_count() -> Result<u64> {
    let sig_dir = "/usr/local/maldetect/sigs";

    let mut count = 0u64;

    if let Ok(entries) = std::fs::read_dir(sig_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += content.lines().filter(|l| !l.trim().is_empty()).count() as u64;
                }
            }
        }
    }

    Ok(count)
}

pub async fn quarantine_file(file_path: &str) -> Result<()> {
    info!("Quarantining file: {file_path}");

    SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("-q")?
        .arg(file_path)?
        .execute()
        .context("Failed to quarantine file")?;

    info!("File quarantined successfully: {file_path}");
    Ok(())
}

pub async fn restore_file(file_path: &str) -> Result<()> {
    info!("Restoring file from quarantine: {file_path}");

    SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("--restore")?
        .arg(file_path)?
        .execute()
        .context("Failed to restore file from quarantine")?;

    info!("File restored successfully: {file_path}");
    Ok(())
}

pub async fn clean_file(file_path: &str) -> Result<()> {
    info!("Cleaning infected file: {file_path}");

    SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("-n")?
        .arg(file_path)?
        .execute()
        .context("Failed to clean file")?;

    info!("File cleaned successfully: {file_path}");
    Ok(())
}

pub async fn get_report(scan_id: &str) -> Result<String> {
    info!("Retrieving LMD report for scan: {scan_id}");

    let output = SafeCommand::new("sudo")?
        .arg("maldet")?
        .arg("--report")?
        .arg(scan_id)?
        .execute()
        .context("Failed to get LMD report")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

pub async fn list_quarantined() -> Result<Vec<QuarantinedFile>> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(LMD_QUARANTINE_DIR) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let metadata = std::fs::metadata(&path).ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let quarantined_at = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t));

                files.push(QuarantinedFile {
                    id: filename.clone(),
                    original_path: extract_original_path(&filename),
                    quarantine_path: path.to_string_lossy().to_string(),
                    size,
                    quarantined_at,
                    threat_name: None,
                });
            }
        }
    }

    Ok(files)
}

pub fn parse_lmd_output(output: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.contains("HIT") || trimmed.contains("FOUND") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let file_path = parts.iter().find(|p| p.starts_with('/')).map(|s| s.to_string());
            let threat_name = parts.iter()
                .find(|p| p.contains("malware") || p.contains("backdoor") || p.contains("trojan"))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Malware".to_string());

            let finding = Finding {
                id: format!("lmd-hit-{}", findings.len()),
                severity: FindingSeverity::Critical,
                category: "Malware Detection".to_string(),
                title: format!("Malware Detected: {threat_name}"),
                description: trimmed.to_string(),
                file_path,
                remediation: Some("Quarantine or remove the infected file immediately".to_string()),
            };
            findings.push(finding);
        }

        if trimmed.contains("suspicious") || trimmed.contains("Suspicious") {
            let file_path = extract_file_path_from_line(trimmed);

            let finding = Finding {
                id: format!("lmd-susp-{}", findings.len()),
                severity: FindingSeverity::High,
                category: "Suspicious Activity".to_string(),
                title: "Suspicious File Detected".to_string(),
                description: trimmed.to_string(),
                file_path,
                remediation: Some("Review the file and consider quarantine if malicious".to_string()),
            };
            findings.push(finding);
        }

        if trimmed.contains("warning") || trimmed.contains("Warning") {
            let finding = Finding {
                id: format!("lmd-warn-{}", findings.len()),
                severity: FindingSeverity::Medium,
                category: "Warning".to_string(),
                title: "LMD Warning".to_string(),
                description: trimmed.to_string(),
                file_path: None,
                remediation: None,
            };
            findings.push(finding);
        }
    }

    findings
}

fn extract_scan_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("scan id:") || line.contains("SCAN ID:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
        if line.contains("report") && line.contains(".") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.contains('.') && part.chars().all(|c| c.is_numeric() || c == '.') {
                    return Some(part.to_string());
                }
            }
        }
    }
    None
}

fn extract_file_path_from_line(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    for word in words {
        if word.starts_with('/') {
            return Some(word.trim_matches(|c| c == ':' || c == ',' || c == ';').to_string());
        }
    }
    None
}

fn extract_original_path(quarantine_filename: &str) -> String {
    quarantine_filename
        .replace(".", "/")
        .trim_start_matches('/')
        .to_string()
}

fn determine_result_status(findings: &[Finding]) -> ScanResultStatus {
    let has_critical = findings.iter().any(|f| f.severity == FindingSeverity::Critical);
    let has_high = findings.iter().any(|f| f.severity == FindingSeverity::High);
    let has_medium = findings.iter().any(|f| f.severity == FindingSeverity::Medium);

    if has_critical {
        ScanResultStatus::Infected
    } else if has_high {
        ScanResultStatus::Warnings
    } else if has_medium {
        ScanResultStatus::Warnings
    } else {
        ScanResultStatus::Clean
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarantinedFile {
    pub id: String,
    pub original_path: String,
    pub quarantine_path: String,
    pub size: u64,
    pub quarantined_at: Option<chrono::DateTime<chrono::Utc>>,
    pub threat_name: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LMDStats {
    pub signature_count: u64,
    pub quarantined_count: u32,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
    pub threats_found: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lmd_output_clean() {
        let output = r#"
Linux Malware Detect v1.6.5
Scanning /var/www
Total files scanned: 1234
Total hits: 0
Total cleaned: 0
"#;
        let findings = parse_lmd_output(output);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_lmd_output_hit() {
        let output = r#"
Linux Malware Detect v1.6.5
Scanning /var/www
{HIT} /var/www/uploads/shell.php : php.cmdshell.unclassed.6
Total hits: 1
"#;
        let findings = parse_lmd_output(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
    }

    #[test]
    fn test_parse_lmd_output_suspicious() {
        let output = r#"
Linux Malware Detect v1.6.5
Scanning /var/www
suspicious file found: /var/www/uploads/unknown.php
"#;
        let findings = parse_lmd_output(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::High);
    }

    #[test]
    fn test_extract_scan_id() {
        assert_eq!(
            extract_scan_id("scan id: 123456.789"),
            Some("123456.789".to_string())
        );
        assert_eq!(
            extract_scan_id("SCAN ID: abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(extract_scan_id("no scan id here"), None);
    }

    #[test]
    fn test_extract_file_path_from_line() {
        assert_eq!(
            extract_file_path_from_line("Found malware in /var/www/shell.php"),
            Some("/var/www/shell.php".to_string())
        );
        assert_eq!(
            extract_file_path_from_line("No path here"),
            None
        );
    }

    #[test]
    fn test_extract_original_path() {
        assert_eq!(
            extract_original_path("var.www.uploads.shell.php"),
            "var/www/uploads/shell/php"
        );
    }

    #[test]
    fn test_determine_result_status_clean() {
        let findings: Vec<Finding> = vec![];
        assert_eq!(determine_result_status(&findings), ScanResultStatus::Clean);
    }

    #[test]
    fn test_determine_result_status_infected() {
        let findings = vec![Finding {
            id: "test".to_string(),
            severity: FindingSeverity::Critical,
            category: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            file_path: None,
            remediation: None,
        }];
        assert_eq!(determine_result_status(&findings), ScanResultStatus::Infected);
    }

    #[test]
    fn test_determine_result_status_warnings() {
        let findings = vec![Finding {
            id: "test".to_string(),
            severity: FindingSeverity::High,
            category: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            file_path: None,
            remediation: None,
        }];
        assert_eq!(determine_result_status(&findings), ScanResultStatus::Warnings);
    }

    #[test]
    fn test_quarantined_file_struct() {
        let file = QuarantinedFile {
            id: "test".to_string(),
            original_path: "/var/www/shell.php".to_string(),
            quarantine_path: "/usr/local/maldetect/quarantine/test".to_string(),
            size: 1024,
            quarantined_at: None,
            threat_name: Some("php.cmdshell".to_string()),
        };
        assert_eq!(file.size, 1024);
        assert!(file.threat_name.is_some());
    }

    #[test]
    fn test_lmd_stats_default() {
        let stats = LMDStats::default();
        assert_eq!(stats.signature_count, 0);
        assert_eq!(stats.quarantined_count, 0);
        assert!(stats.last_scan.is_none());
    }

    #[test]
    fn test_parse_lmd_output_warning() {
        let output = r#"
Linux Malware Detect v1.6.5
Warning: signature database may be outdated
Scanning /var/www
"#;
        let findings = parse_lmd_output(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Medium);
    }

    #[test]
    fn test_parse_lmd_output_found() {
        let output = r#"
FOUND: /var/www/malicious.php : malware.backdoor.123
"#;
        let findings = parse_lmd_output(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
    }
}
