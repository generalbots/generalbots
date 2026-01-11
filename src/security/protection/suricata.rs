use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::security::command_guard::SafeCommand;
use super::manager::{Finding, FindingSeverity, ScanResultStatus};

const SURICATA_EVE_LOG: &str = "/var/log/suricata/eve.json";
const SURICATA_RULES_DIR: &str = "/var/lib/suricata/rules";

pub async fn get_alerts() -> Result<(ScanResultStatus, Vec<Finding>, String)> {
    info!("Retrieving Suricata alerts");

    let alerts = parse_eve_log(100).await.unwrap_or_default();
    let findings = alerts_to_findings(&alerts);
    let status = determine_result_status(&findings);

    let raw_output = serde_json::to_string_pretty(&alerts).unwrap_or_default();

    Ok((status, findings, raw_output))
}

pub async fn start_service() -> Result<()> {
    info!("Starting Suricata service");

    SafeCommand::new("sudo")?
        .arg("systemctl")?
        .arg("start")?
        .arg("suricata")?
        .execute()
        .context("Failed to start Suricata service")?;

    info!("Suricata service started successfully");
    Ok(())
}

pub async fn stop_service() -> Result<()> {
    info!("Stopping Suricata service");

    SafeCommand::new("sudo")?
        .arg("systemctl")?
        .arg("stop")?
        .arg("suricata")?
        .execute()
        .context("Failed to stop Suricata service")?;

    info!("Suricata service stopped successfully");
    Ok(())
}

pub async fn restart_service() -> Result<()> {
    info!("Restarting Suricata service");

    SafeCommand::new("sudo")?
        .arg("systemctl")?
        .arg("restart")?
        .arg("suricata")?
        .execute()
        .context("Failed to restart Suricata service")?;

    info!("Suricata service restarted successfully");
    Ok(())
}

pub async fn update_rules() -> Result<()> {
    info!("Updating Suricata rules");

    SafeCommand::new("sudo")?
        .arg("suricata-update")?
        .execute()
        .context("Failed to update Suricata rules")?;

    SafeCommand::new("sudo")?
        .arg("systemctl")?
        .arg("reload")?
        .arg("suricata")?
        .execute()
        .context("Failed to reload Suricata after rule update")?;

    info!("Suricata rules updated successfully");
    Ok(())
}

pub async fn get_version() -> Result<String> {
    let output = SafeCommand::new("suricata")?
        .arg("--build-info")?
        .execute()
        .context("Failed to get Suricata version")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .lines()
        .find(|l| l.contains("Suricata version"))
        .and_then(|l| l.split(':').nth(1))
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(version)
}

pub async fn get_rule_count() -> Result<u32> {
    let mut count = 0u32;

    if let Ok(entries) = std::fs::read_dir(SURICATA_RULES_DIR) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rules") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    count += content
                        .lines()
                        .filter(|l| !l.trim().starts_with('#') && !l.trim().is_empty())
                        .count() as u32;
                }
            }
        }
    }

    Ok(count)
}

pub async fn get_stats() -> Result<SuricataStats> {
    let alerts = parse_eve_log(1000).await.unwrap_or_default();
    let today = Utc::now().date_naive();

    let alerts_today = alerts
        .iter()
        .filter(|a| a.timestamp.date_naive() == today)
        .count() as u32;

    let blocked_today = alerts
        .iter()
        .filter(|a| a.timestamp.date_naive() == today && a.action == "blocked")
        .count() as u32;

    let rule_count = get_rule_count().await.unwrap_or(0);

    Ok(SuricataStats {
        alerts_today,
        blocked_today,
        rule_count,
        total_alerts: alerts.len() as u32,
    })
}

pub async fn parse_eve_log(limit: usize) -> Result<Vec<SuricataAlert>> {
    let content = std::fs::read_to_string(SURICATA_EVE_LOG)
        .context("Failed to read Suricata EVE log")?;

    let mut alerts = Vec::new();

    for line in content.lines().rev().take(limit * 2) {
        if let Ok(event) = serde_json::from_str::<EveEvent>(line) {
            if event.event_type == "alert" {
                if let Some(alert_data) = event.alert {
                    let alert = SuricataAlert {
                        timestamp: event.timestamp,
                        src_ip: event.src_ip.unwrap_or_default(),
                        src_port: event.src_port.unwrap_or(0),
                        dest_ip: event.dest_ip.unwrap_or_default(),
                        dest_port: event.dest_port.unwrap_or(0),
                        protocol: event.proto.unwrap_or_default(),
                        signature: alert_data.signature,
                        signature_id: alert_data.signature_id,
                        severity: alert_data.severity,
                        category: alert_data.category,
                        action: alert_data.action,
                    };
                    alerts.push(alert);
                    if alerts.len() >= limit {
                        break;
                    }
                }
            }
        }
    }

    alerts.reverse();
    Ok(alerts)
}

fn alerts_to_findings(alerts: &[SuricataAlert]) -> Vec<Finding> {
    alerts
        .iter()
        .map(|alert| {
            let severity = match alert.severity {
                1 => FindingSeverity::Critical,
                2 => FindingSeverity::High,
                3 => FindingSeverity::Medium,
                _ => FindingSeverity::Low,
            };

            Finding {
                id: format!("suricata-{}", alert.signature_id),
                severity,
                category: alert.category.clone(),
                title: alert.signature.clone(),
                description: format!(
                    "{}:{} -> {}:{} ({})",
                    alert.src_ip, alert.src_port, alert.dest_ip, alert.dest_port, alert.protocol
                ),
                file_path: None,
                remediation: if alert.action == "blocked" {
                    Some("Traffic was automatically blocked".to_string())
                } else {
                    Some("Review the alert and consider blocking the source".to_string())
                },
            }
        })
        .collect()
}

fn determine_result_status(findings: &[Finding]) -> ScanResultStatus {
    let has_critical = findings.iter().any(|f| f.severity == FindingSeverity::Critical);
    let has_high = findings.iter().any(|f| f.severity == FindingSeverity::High);

    if has_critical {
        ScanResultStatus::Infected
    } else if has_high || !findings.is_empty() {
        ScanResultStatus::Warnings
    } else {
        ScanResultStatus::Clean
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuricataAlert {
    pub timestamp: DateTime<Utc>,
    pub src_ip: String,
    pub src_port: u16,
    pub dest_ip: String,
    pub dest_port: u16,
    pub protocol: String,
    pub signature: String,
    pub signature_id: u64,
    pub severity: u8,
    pub category: String,
    pub action: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuricataStats {
    pub alerts_today: u32,
    pub blocked_today: u32,
    pub rule_count: u32,
    pub total_alerts: u32,
}

#[derive(Debug, Deserialize)]
struct EveEvent {
    timestamp: DateTime<Utc>,
    event_type: String,
    src_ip: Option<String>,
    src_port: Option<u16>,
    dest_ip: Option<String>,
    dest_port: Option<u16>,
    proto: Option<String>,
    alert: Option<EveAlert>,
}

#[derive(Debug, Deserialize)]
struct EveAlert {
    signature: String,
    signature_id: u64,
    severity: u8,
    category: String,
    action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alerts_to_findings_empty() {
        let alerts: Vec<SuricataAlert> = vec![];
        let findings = alerts_to_findings(&alerts);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_alerts_to_findings_severity_mapping() {
        let alert = SuricataAlert {
            timestamp: Utc::now(),
            src_ip: "192.168.1.1".to_string(),
            src_port: 12345,
            dest_ip: "10.0.0.1".to_string(),
            dest_port: 80,
            protocol: "TCP".to_string(),
            signature: "Test Alert".to_string(),
            signature_id: 1000001,
            severity: 1,
            category: "Test".to_string(),
            action: "allowed".to_string(),
        };

        let findings = alerts_to_findings(&[alert]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, FindingSeverity::Critical);
    }

    #[test]
    fn test_alerts_to_findings_severity_high() {
        let alert = SuricataAlert {
            timestamp: Utc::now(),
            src_ip: "192.168.1.1".to_string(),
            src_port: 12345,
            dest_ip: "10.0.0.1".to_string(),
            dest_port: 443,
            protocol: "TCP".to_string(),
            signature: "High Severity Alert".to_string(),
            signature_id: 1000002,
            severity: 2,
            category: "Test".to_string(),
            action: "blocked".to_string(),
        };

        let findings = alerts_to_findings(&[alert]);
        assert_eq!(findings[0].severity, FindingSeverity::High);
        assert!(findings[0].remediation.as_ref().unwrap().contains("blocked"));
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
    fn test_determine_result_status_warnings() {
        let findings = vec![Finding {
            id: "test".to_string(),
            severity: FindingSeverity::Medium,
            category: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            file_path: None,
            remediation: None,
        }];
        assert_eq!(determine_result_status(&findings), ScanResultStatus::Warnings);
    }

    #[test]
    fn test_suricata_stats_default() {
        let stats = SuricataStats::default();
        assert_eq!(stats.alerts_today, 0);
        assert_eq!(stats.blocked_today, 0);
        assert_eq!(stats.rule_count, 0);
    }

    #[test]
    fn test_suricata_alert_serialization() {
        let alert = SuricataAlert {
            timestamp: Utc::now(),
            src_ip: "192.168.1.1".to_string(),
            src_port: 12345,
            dest_ip: "10.0.0.1".to_string(),
            dest_port: 80,
            protocol: "TCP".to_string(),
            signature: "Test".to_string(),
            signature_id: 1,
            severity: 3,
            category: "Test".to_string(),
            action: "allowed".to_string(),
        };

        let json = serde_json::to_string(&alert);
        assert!(json.is_ok());
    }
}
