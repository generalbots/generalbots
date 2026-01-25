use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::security::command_guard::SafeCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtectionTool {
    Lynis,
    RKHunter,
    Chkrootkit,
    Suricata,
    LMD,
    ClamAV,
}

impl std::fmt::Display for ProtectionTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lynis => write!(f, "lynis"),
            Self::RKHunter => write!(f, "rkhunter"),
            Self::Chkrootkit => write!(f, "chkrootkit"),
            Self::Suricata => write!(f, "suricata"),
            Self::LMD => write!(f, "lmd"),
            Self::ClamAV => write!(f, "clamav"),
        }
    }
}

impl std::str::FromStr for ProtectionTool {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lynis" => Ok(Self::Lynis),
            "rkhunter" => Ok(Self::RKHunter),
            "chkrootkit" => Ok(Self::Chkrootkit),
            "suricata" => Ok(Self::Suricata),
            "lmd" | "maldet" => Ok(Self::LMD),
            "clamav" | "clamscan" => Ok(Self::ClamAV),
            _ => Err(()),
        }
    }
}

impl ProtectionTool {

    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Lynis => "lynis",
            Self::RKHunter => "rkhunter",
            Self::Chkrootkit => "chkrootkit",
            Self::Suricata => "suricata",
            Self::LMD => "maldet",
            Self::ClamAV => "clamscan",
        }
    }

    pub fn service_name(&self) -> Option<&'static str> {
        match self {
            Self::Suricata => Some("suricata"),
            Self::ClamAV => Some("clamav-daemon"),
            _ => None,
        }
    }

    pub fn package_name(&self) -> &'static str {
        match self {
            Self::Lynis => "lynis",
            Self::RKHunter => "rkhunter",
            Self::Chkrootkit => "chkrootkit",
            Self::Suricata => "suricata",
            Self::LMD => "maldetect",
            Self::ClamAV => "clamav",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Lynis,
            Self::RKHunter,
            Self::Chkrootkit,
            Self::Suricata,
            Self::LMD,
            Self::ClamAV,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub tool: ProtectionTool,
    pub installed: bool,
    pub version: Option<String>,
    pub service_running: Option<bool>,
    pub last_scan: Option<DateTime<Utc>>,
    pub last_update: Option<DateTime<Utc>>,
    pub auto_update: bool,
    pub auto_remediate: bool,
    pub metrics: ToolMetrics,
}

impl ToolStatus {
    pub fn not_installed(tool: ProtectionTool) -> Self {
        Self {
            tool,
            installed: false,
            version: None,
            service_running: None,
            last_scan: None,
            last_update: None,
            auto_update: false,
            auto_remediate: false,
            metrics: ToolMetrics::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolMetrics {
    pub hardening_index: Option<u32>,
    pub warnings: u32,
    pub suggestions: u32,
    pub threats_found: u32,
    pub rules_count: Option<u32>,
    pub alerts_today: u32,
    pub blocked_today: u32,
    pub signatures_count: Option<u64>,
    pub quarantined_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub tool: ProtectionTool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ScanStatus,
    pub result: ScanResultStatus,
    pub findings: Vec<Finding>,
    pub warnings: u32,
    pub report_path: Option<String>,
    pub raw_output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanResultStatus {
    Clean,
    Warnings,
    Infected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub severity: FindingSeverity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    pub enabled_tools: Vec<ProtectionTool>,
    pub auto_scan_interval_hours: u32,
    pub auto_update_interval_hours: u32,
    pub quarantine_dir: String,
    pub log_dir: String,
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            enabled_tools: ProtectionTool::all(),
            auto_scan_interval_hours: 24,
            auto_update_interval_hours: 6,
            quarantine_dir: "/var/lib/gb/quarantine".to_string(),
            log_dir: "/var/log/gb/security".to_string(),
        }
    }
}

pub struct ProtectionManager {
    config: ProtectionConfig,
    tool_status: Arc<RwLock<HashMap<ProtectionTool, ToolStatus>>>,
    active_scans: Arc<RwLock<HashMap<String, ScanResult>>>,
    scan_history: Arc<RwLock<Vec<ScanResult>>>,
}

impl ProtectionManager {
    pub fn new(config: ProtectionConfig) -> Self {
        Self {
            config,
            tool_status: Arc::new(RwLock::new(HashMap::new())),
            active_scans: Arc::new(RwLock::new(HashMap::new())),
            scan_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Protection Manager");
        for tool in &self.config.enabled_tools {
            let status = self.check_tool_status(*tool).await?;
            self.tool_status.write().await.insert(*tool, status);
        }
        Ok(())
    }

    pub async fn check_tool_status(&self, tool: ProtectionTool) -> Result<ToolStatus> {
        let installed = self.is_tool_installed(tool).await;

        if !installed {
            return Ok(ToolStatus::not_installed(tool));
        }

        let version = self.get_tool_version(tool).await.ok();
        let service_running = if tool.service_name().is_some() {
            Some(self.is_service_running(tool).await)
        } else {
            None
        };

        let stored = self.tool_status.read().await;
        let existing = stored.get(&tool);

        Ok(ToolStatus {
            tool,
            installed: true,
            version,
            service_running,
            last_scan: existing.and_then(|s| s.last_scan),
            last_update: existing.and_then(|s| s.last_update),
            auto_update: existing.map(|s| s.auto_update).unwrap_or(false),
            auto_remediate: existing.map(|s| s.auto_remediate).unwrap_or(false),
            metrics: existing.map(|s| s.metrics.clone()).unwrap_or_default(),
        })
    }

    pub async fn is_tool_installed(&self, tool: ProtectionTool) -> bool {
        let binary = tool.binary_name();

        let result = SafeCommand::new("which")
            .and_then(|cmd| cmd.arg(binary))
            .and_then(|cmd| cmd.execute());

        match result {
            Ok(output) => output.status.success(),
            Err(e) => {
                warn!("Failed to check if {tool} is installed: {e}");
                false
            }
        }
    }

    pub async fn get_tool_version(&self, tool: ProtectionTool) -> Result<String> {
        let binary = tool.binary_name();
        let version_arg = match tool {
            ProtectionTool::Lynis => "--version",
            ProtectionTool::RKHunter => "--version",
            ProtectionTool::Chkrootkit => "-V",
            ProtectionTool::Suricata => "--build-info",
            ProtectionTool::LMD => "--version",
            ProtectionTool::ClamAV => "--version",
        };

        let output = SafeCommand::new(binary)?
            .arg(version_arg)?
            .execute()
            .context("Failed to get tool version")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = stdout.lines().next().unwrap_or("unknown").trim().to_string();
        Ok(version)
    }

    pub async fn is_service_running(&self, tool: ProtectionTool) -> bool {
        let Some(service_name) = tool.service_name() else {
            return false;
        };

        let result = SafeCommand::new("sudo")
            .and_then(|cmd| cmd.arg("systemctl"))
            .and_then(|cmd| cmd.arg("is-active"))
            .and_then(|cmd| cmd.arg(service_name));

        match result {
            Ok(cmd) => match cmd.execute() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.trim() == "active"
                }
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    pub async fn get_all_status(&self) -> HashMap<ProtectionTool, ToolStatus> {
        self.tool_status.read().await.clone()
    }

    pub async fn get_tool_status_by_name(&self, name: &str) -> Option<ToolStatus> {
        let tool: ProtectionTool = name.parse().ok()?;
        self.tool_status.read().await.get(&tool).cloned()
    }

    pub async fn install_tool(&self, tool: ProtectionTool) -> Result<()> {
        info!("Installing protection tool: {tool}");

        let package = tool.package_name();

        SafeCommand::new("apt-get")?
            .arg("install")?
            .arg("-y")?
            .arg(package)?
            .execute()
            .context("Failed to install tool")?;

        let status = self.check_tool_status(tool).await?;
        self.tool_status.write().await.insert(tool, status);

        info!("Successfully installed {tool}");
        Ok(())
    }

    pub async fn start_service(&self, tool: ProtectionTool) -> Result<()> {
        let Some(service_name) = tool.service_name() else {
            return Err(anyhow::anyhow!("{tool} does not have a service"));
        };

        info!("Starting service: {service_name}");

        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("start")?
            .arg(service_name)?
            .execute()
            .context("Failed to start service")?;

        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.service_running = Some(true);
        }

        Ok(())
    }

    pub async fn stop_service(&self, tool: ProtectionTool) -> Result<()> {
        let Some(service_name) = tool.service_name() else {
            return Err(anyhow::anyhow!("{tool} does not have a service"));
        };

        info!("Stopping service: {service_name}");

        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("stop")?
            .arg(service_name)?
            .execute()
            .context("Failed to stop service")?;

        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.service_running = Some(false);
        }

        Ok(())
    }

    pub async fn enable_service(&self, tool: ProtectionTool) -> Result<()> {
        let Some(service_name) = tool.service_name() else {
            return Err(anyhow::anyhow!("{tool} does not have a service"));
        };

        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("enable")?
            .arg(service_name)?
            .execute()
            .context("Failed to enable service")?;

        Ok(())
    }

    pub async fn disable_service(&self, tool: ProtectionTool) -> Result<()> {
        let Some(service_name) = tool.service_name() else {
            return Err(anyhow::anyhow!("{tool} does not have a service"));
        };

        SafeCommand::new("sudo")?
            .arg("systemctl")?
            .arg("disable")?
            .arg(service_name)?
            .execute()
            .context("Failed to disable service")?;

        Ok(())
    }

    pub async fn run_scan(&self, tool: ProtectionTool) -> Result<ScanResult> {
        let scan_id = uuid::Uuid::new_v4().to_string();
        let started_at = Utc::now();

        info!("Starting {tool} scan with ID: {scan_id}");

        let mut result = ScanResult {
            scan_id: scan_id.clone(),
            tool,
            started_at,
            completed_at: None,
            status: ScanStatus::Running,
            result: ScanResultStatus::Unknown,
            findings: Vec::new(),
            warnings: 0,
            report_path: None,
            raw_output: None,
        };

        self.active_scans.write().await.insert(scan_id.clone(), result.clone());

        let scan_output = match tool {
            ProtectionTool::Lynis => super::lynis::run_scan().await,
            ProtectionTool::RKHunter => super::rkhunter::run_scan().await,
            ProtectionTool::Chkrootkit => super::chkrootkit::run_scan().await,
            ProtectionTool::Suricata => super::suricata::get_alerts().await,
            ProtectionTool::LMD => super::lmd::run_scan(None).await,
            ProtectionTool::ClamAV => super::lynis::run_scan().await,
        };

        result.completed_at = Some(Utc::now());

        match scan_output {
            Ok((status, findings, raw)) => {
                result.status = ScanStatus::Completed;
                result.result = status;
                result.warnings = findings.iter().filter(|f| f.severity == FindingSeverity::Medium || f.severity == FindingSeverity::Low).count() as u32;
                result.findings = findings;
                result.raw_output = Some(raw);
            }
            Err(e) => {
                warn!("Scan failed for {tool}: {e}");
                result.status = ScanStatus::Failed;
                result.raw_output = Some(e.to_string());
            }
        }

        self.active_scans.write().await.remove(&scan_id);

        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.last_scan = Some(Utc::now());
            status.metrics.warnings = result.warnings;
            status.metrics.threats_found = result.findings.iter()
                .filter(|f| f.severity == FindingSeverity::High || f.severity == FindingSeverity::Critical)
                .count() as u32;
        }

        self.scan_history.write().await.push(result.clone());

        Ok(result)
    }

    pub async fn update_definitions(&self, tool: ProtectionTool) -> Result<()> {
        info!("Updating definitions for {tool}");

        match tool {
            ProtectionTool::RKHunter => {
                SafeCommand::new("sudo")?
                    .arg("rkhunter")?
                    .arg("--update")?
                    .execute()?;
            }
            ProtectionTool::ClamAV => {
                SafeCommand::new("sudo")?
                    .arg("freshclam")?
                    .execute()?;
            }
            ProtectionTool::Suricata => {
                SafeCommand::new("sudo")?
                    .arg("suricata-update")?
                    .execute()?;
            }
            ProtectionTool::LMD => {
                SafeCommand::new("sudo")?
                    .arg("maldet")?
                    .arg("--update-sigs")?
                    .execute()?;
            }
            _ => {
                return Err(anyhow::anyhow!("{tool} does not support definition updates"));
            }
        }

        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.last_update = Some(Utc::now());
        }

        Ok(())
    }

    pub async fn set_auto_update(&self, tool: ProtectionTool, enabled: bool) -> Result<()> {
        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.auto_update = enabled;
        }
        Ok(())
    }

    pub async fn set_auto_remediate(&self, tool: ProtectionTool, enabled: bool) -> Result<()> {
        if let Some(status) = self.tool_status.write().await.get_mut(&tool) {
            status.auto_remediate = enabled;
        }
        Ok(())
    }

    pub async fn get_scan_history(&self, tool: Option<ProtectionTool>, limit: usize) -> Vec<ScanResult> {
        let history = self.scan_history.read().await;
        history
            .iter()
            .filter(|s| tool.is_none() || Some(s.tool) == tool)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn get_active_scans(&self) -> Vec<ScanResult> {
        self.active_scans.read().await.values().cloned().collect()
    }

    pub async fn get_report(&self, tool: ProtectionTool) -> Result<String> {
        let history = self.scan_history.read().await;
        let latest = history
            .iter().rfind(|s| s.tool == tool)
            .ok_or_else(|| anyhow::anyhow!("No scan results found for {tool}"))?;

        latest.raw_output.clone().ok_or_else(|| anyhow::anyhow!("No report available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_tool_from_str() {
        assert_eq!("lynis".parse::<ProtectionTool>(), Ok(ProtectionTool::Lynis));
        assert_eq!("LYNIS".parse::<ProtectionTool>(), Ok(ProtectionTool::Lynis));
        assert_eq!("rkhunter".parse::<ProtectionTool>(), Ok(ProtectionTool::RKHunter));
        assert_eq!("clamav".parse::<ProtectionTool>(), Ok(ProtectionTool::ClamAV));
        assert_eq!("clamscan".parse::<ProtectionTool>(), Ok(ProtectionTool::ClamAV));
        assert_eq!("maldet".parse::<ProtectionTool>(), Ok(ProtectionTool::LMD));
        assert!("unknown".parse::<ProtectionTool>().is_err());
    }

    #[test]
    fn test_protection_tool_display() {
        assert_eq!(format!("{}", ProtectionTool::Lynis), "lynis");
        assert_eq!(format!("{}", ProtectionTool::ClamAV), "clamav");
    }

    #[test]
    fn test_tool_status_not_installed() {
        let status = ToolStatus::not_installed(ProtectionTool::Lynis);
        assert!(!status.installed);
        assert!(status.version.is_none());
        assert!(status.service_running.is_none());
    }

    #[test]
    fn test_protection_config_default() {
        let config = ProtectionConfig::default();
        assert_eq!(config.auto_scan_interval_hours, 24);
        assert_eq!(config.auto_update_interval_hours, 6);
        assert_eq!(config.enabled_tools.len(), 6);
    }

    #[test]
    fn test_protection_tool_all() {
        let all = ProtectionTool::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&ProtectionTool::Lynis));
        assert!(all.contains(&ProtectionTool::ClamAV));
    }

    #[test]
    fn test_finding_severity() {
        let finding = Finding {
            id: "test".to_string(),
            severity: FindingSeverity::High,
            category: "security".to_string(),
            title: "Test".to_string(),
            description: "Test finding".to_string(),
            file_path: None,
            remediation: None,
        };
        assert_eq!(finding.severity, FindingSeverity::High);
    }
}
