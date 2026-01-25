use anyhow::{Context, Result};
use tracing::info;

use crate::security::command_guard::SafeCommand;
use super::manager::{Finding, FindingSeverity, ScanResultStatus};

const RKHUNTER_LOG_PATH: &str = "/var/log/rkhunter.log";

pub async fn run_scan() -> Result<(ScanResultStatus, Vec<Finding>, String)> {
    info!("Running RKHunter rootkit scan");

    let output = SafeCommand::new("sudo")?
        .arg("rkhunter")?
        .arg("--check")?
        .arg("--skip-keypress")?
        .arg("--report-warnings-only")?
        .execute()
        .context("Failed to run RKHunter scan")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let raw_output = format!("{stdout}\n{stderr}");

    let findings = parse_rkhunter_output(&stdout);
    let status = determine_result_status(&findings);

    Ok((status, findings, raw_output))
}

pub async fn update_database() -> Result<()> {
    info!("Updating RKHunter database");

    SafeCommand::new("sudo")?
        .arg("rkhunter")?
        .arg("--update")?
        .execute()
        .context("Failed to update RKHunter database")?;

    info!("RKHunter database updated successfully");
    Ok(())
}

pub async fn update_properties() -> Result<()> {
    info!("Updating RKHunter file properties database");

    SafeCommand::new("sudo")?
        .arg("rkhunter")?
        .arg("--propupd")?
        .execute()
        .context("Failed to update RKHunter properties")?;

    info!("RKHunter properties updated successfully");
    Ok(())
}

pub fn parse_rkhunter_output(output: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut current_section = String::new();

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Checking") {
            current_section = trimmed.replace("Checking", "").trim().to_string();
            continue;
        }

        if trimmed.contains("[ Warning ]") || trimmed.contains("[Warning]") {
            let description = extract_warning_description(trimmed);
            let finding = Finding {
                id: format!("rkhunter-warn-{}", findings.len()),
                severity: FindingSeverity::High,
                category: current_section.clone(),
                title: "RKHunter Warning".to_string(),
                description,
                file_path: extract_file_path(trimmed),
                remediation: Some("Investigate the flagged file or configuration".to_string()),
            };
            findings.push(finding);
        }

        if trimmed.contains("[ Rootkit ]") || trimmed.to_lowercase().contains("rootkit found") {
            let finding = Finding {
                id: format!("rkhunter-rootkit-{}", findings.len()),
                severity: FindingSeverity::Critical,
                category: "Rootkit Detection".to_string(),
                title: "Potential Rootkit Detected".to_string(),
                description: trimmed.to_string(),
                file_path: extract_file_path(trimmed),
                remediation: Some("Immediately investigate and consider system recovery".to_string()),
            };
            findings.push(finding);
        }

        if trimmed.contains("Suspicious file") || trimmed.contains("suspicious") {
            let finding = Finding {
                id: format!("rkhunter-susp-{}", findings.len()),
                severity: FindingSeverity::High,
                category: current_section.clone(),
                title: "Suspicious File Detected".to_string(),
                description: trimmed.to_string(),
                file_path: extract_file_path(trimmed),
                remediation: Some("Verify the file integrity and source".to_string()),
            };
            findings.push(finding);
        }

        if trimmed.contains("[ Bad ]") {
            let finding = Finding {
                id: format!("rkhunter-bad-{}", findings.len()),
                severity: FindingSeverity::High,
                category: current_section.clone(),
                title: "Bad Configuration or File".to_string(),
                description: trimmed.to_string(),
                file_path: extract_file_path(trimmed),
                remediation: Some("Review and correct the flagged item".to_string()),
            };
            findings.push(finding);
        }
    }

    findings
}

pub fn parse_log_file() -> Result<RKHunterReport> {
    let content = std::fs::read_to_string(RKHUNTER_LOG_PATH)
        .context("Failed to read RKHunter log file")?;

    let mut report = RKHunterReport::default();

    for line in content.lines() {
        if line.contains("Rootkits checked") {
            if let Some(count) = extract_number_from_line(line) {
                report.rootkits_checked = count;
            }
        }

        if line.contains("Possible rootkits") {
            if let Some(count) = extract_number_from_line(line) {
                report.possible_rootkits = count;
            }
        }

        if line.contains("Suspect files") {
            if let Some(count) = extract_number_from_line(line) {
                report.suspect_files = count;
            }
        }

        if line.contains("Warning:") {
            report.warnings.push(line.replace("Warning:", "").trim().to_string());
        }

        if line.contains("rkhunter version") {
            report.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    Ok(report)
}

pub async fn get_version() -> Result<String> {
    let output = SafeCommand::new("rkhunter")?
        .arg("--version")?
        .execute()
        .context("Failed to get RKHunter version")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .lines()
        .find(|l| l.contains("version"))
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

fn extract_warning_description(line: &str) -> String {
    line.replace("[ Warning ]", "")
        .replace("[Warning]", "")
        .trim()
        .to_string()
}

fn extract_file_path(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split_whitespace().collect();
    for word in words {
        if word.starts_with('/') {
            return Some(word.trim_matches(|c| c == ':' || c == ',' || c == ';').to_string());
        }
    }
    None
}

fn extract_number_from_line(line: &str) -> Option<u32> {
    line.split_whitespace()
        .find_map(|word| word.parse::<u32>().ok())
}

fn determine_result_status(findings: &[Finding]) -> ScanResultStatus {
    let has_critical = findings.iter().any(|f| f.severity == FindingSeverity::Critical);
    let has_high = findings.iter().any(|f| f.severity == FindingSeverity::High);
    let has_medium = findings.iter().any(|f| f.severity == FindingSeverity::Medium);

    if has_critical {
        ScanResultStatus::Infected
    } else if has_high || has_medium {
        ScanResultStatus::Warnings
    } else {
        ScanResultStatus::Clean
    }
}

#[derive(Debug, Clone, Default)]
pub struct RKHunterReport {
    pub version: String,
    pub rootkits_checked: u32,
    pub possible_rootkits: u32,
    pub suspect_files: u32,
    pub warnings: Vec<String>,
    pub scan_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rkhunter_output_clean() {
        let output = r#"
Checking for rootkits...
  Performing check of known rootkit files and directories
    55808 Trojan - Variant A                         [ Not found ]
    ADM Worm                                         [ Not found ]
    AjaKit Rootkit                                   [ Not found ]
System checks summary
=====================
File properties checks...
    Files checked: 142
    Suspect files: 0
"#;
        let findings = parse_rkhunter_output(output);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_parse_rkhunter_output_warning() {
        let output = r#"
Checking for rootkits...
  Checking /dev for suspicious file types           [ Warning ]
  Suspicious file found: /dev/.udev/something
"#;
        let findings = parse_rkhunter_output(output);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, FindingSeverity::High);
    }

    #[test]
    fn test_extract_file_path() {
        assert_eq!(
            extract_file_path("Suspicious file found: /etc/passwd"),
            Some("/etc/passwd".to_string())
        );
        assert_eq!(
            extract_file_path("Checking /dev/sda for issues"),
            Some("/dev/sda".to_string())
        );
        assert_eq!(extract_file_path("No path here"), None);
    }

    #[test]
    fn test_extract_number_from_line() {
        assert_eq!(extract_number_from_line("Rootkits checked: 42"), Some(42));
        assert_eq!(extract_number_from_line("Found 5 issues"), Some(5));
        assert_eq!(extract_number_from_line("No numbers here"), None);
    }

    #[test]
    fn test_determine_result_status_clean() {
        let findings: Vec<Finding> = vec![];
        assert_eq!(determine_result_status(&findings), ScanResultStatus::Clean);
    }

    #[test]
    fn test_determine_result_status_critical() {
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
    fn test_rkhunter_report_default() {
        let report = RKHunterReport::default();
        assert_eq!(report.rootkits_checked, 0);
        assert_eq!(report.possible_rootkits, 0);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_extract_warning_description() {
        assert_eq!(
            extract_warning_description("[ Warning ] Some issue here"),
            "Some issue here"
        );
        assert_eq!(
            extract_warning_description("[Warning] Another issue"),
            "Another issue"
        );
    }
}
