use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatStatus {
    Detected,
    Quarantined,
    Removed,
    Allowed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id: String,
    pub name: String,
    pub threat_type: String,
    pub severity: ThreatSeverity,
    pub status: ThreatStatus,
    pub file_path: Option<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub action_taken: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub cve_id: Option<String>,
    pub name: String,
    pub severity: ThreatSeverity,
    pub affected_component: String,
    pub description: String,
    pub remediation: Option<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub is_patched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ScanStatus,
    pub files_scanned: u64,
    pub threats_found: Vec<Threat>,
    pub scan_type: ScanType,
    pub target_path: Option<String>,
    pub error_message: Option<String>,
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
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Quick,
    Full,
    Custom,
    Memory,
    Rootkit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionStatus {
    pub real_time_protection: bool,
    pub windows_defender_enabled: bool,
    pub general_bots_protection: bool,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
    pub last_definition_update: Option<chrono::DateTime<chrono::Utc>>,
    pub threats_blocked_today: u32,
    pub quarantined_items: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntivirusConfig {
    pub clamav_path: Option<PathBuf>,

    pub real_time_protection: bool,

    pub quarantine_dir: PathBuf,

    pub log_dir: PathBuf,

    pub auto_quarantine: bool,

    pub excluded_paths: Vec<PathBuf>,

    pub excluded_extensions: Vec<String>,

    pub max_file_size_mb: u64,

    pub scan_archives: bool,

    pub definition_update_url: Option<String>,
}

impl Default for AntivirusConfig {
    fn default() -> Self {
        Self {
            clamav_path: None,
            real_time_protection: true,
            quarantine_dir: PathBuf::from("./data/quarantine"),
            log_dir: PathBuf::from("./logs/antivirus"),
            auto_quarantine: true,
            excluded_paths: vec![],
            excluded_extensions: vec![],
            max_file_size_mb: 100,
            scan_archives: true,
            definition_update_url: None,
        }
    }
}

#[derive(Debug)]
pub struct AntivirusManager {
    config: AntivirusConfig,
    threats: Arc<RwLock<Vec<Threat>>>,
    vulnerabilities: Arc<RwLock<Vec<Vulnerability>>>,
    active_scans: Arc<RwLock<HashMap<String, ScanResult>>>,
    protection_status: Arc<RwLock<ProtectionStatus>>,
}

impl AntivirusManager {
    pub fn new(config: AntivirusConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.quarantine_dir)
            .context("Failed to create quarantine directory")?;
        std::fs::create_dir_all(&config.log_dir).context("Failed to create log directory")?;

        let protection_status = ProtectionStatus {
            real_time_protection: config.real_time_protection,
            windows_defender_enabled: Self::check_windows_defender_status(),
            general_bots_protection: true,
            last_scan: None,
            last_definition_update: None,
            threats_blocked_today: 0,
            quarantined_items: 0,
        };

        Ok(Self {
            config,
            threats: Arc::new(RwLock::new(Vec::new())),
            vulnerabilities: Arc::new(RwLock::new(Vec::new())),
            active_scans: Arc::new(RwLock::new(HashMap::new())),
            protection_status: Arc::new(RwLock::new(protection_status)),
        })
    }

    #[cfg(target_os = "windows")]
    fn check_windows_defender_status() -> bool {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-MpPreference | Select-Object -ExpandProperty DisableRealtimeMonitoring",
            ])
            .output();

        match output {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                !result.trim().eq_ignore_ascii_case("true")
            }
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn check_windows_defender_status() -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    pub async fn disable_windows_defender(&self) -> Result<bool> {
        info!("Attempting to disable Windows Defender...");

        let script = r#"
            Set-MpPreference -DisableRealtimeMonitoring $true
            Set-MpPreference -DisableBehaviorMonitoring $true
            Set-MpPreference -DisableBlockAtFirstSeen $true
            Set-MpPreference -DisableIOAVProtection $true
            Set-MpPreference -DisablePrivacyMode $true
            Set-MpPreference -SignatureDisableUpdateOnStartupWithoutEngine $true
            Set-MpPreference -DisableArchiveScanning $true
            Set-MpPreference -DisableIntrusionPreventionSystem $true
            Set-MpPreference -DisableScriptScanning $true
        "#;

        let output = Command::new("powershell")
            .args(["-Command", script])
            .output()
            .context("Failed to execute PowerShell command")?;

        if output.status.success() {
            let mut status = self.protection_status.write().await;
            status.windows_defender_enabled = false;
            info!("Windows Defender disabled successfully");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("Failed to disable Windows Defender: {}", error);
            Err(anyhow::anyhow!(
                "Failed to disable Windows Defender: {}",
                error
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn disable_windows_defender(&self) -> Result<bool> {
        warn!("Windows Defender management is only available on Windows");
        Ok(false)
    }

    #[cfg(target_os = "windows")]
    pub async fn enable_windows_defender(&self) -> Result<bool> {
        info!("Attempting to enable Windows Defender...");

        let script = r#"
            Set-MpPreference -DisableRealtimeMonitoring $false
            Set-MpPreference -DisableBehaviorMonitoring $false
            Set-MpPreference -DisableBlockAtFirstSeen $false
            Set-MpPreference -DisableIOAVProtection $false
            Set-MpPreference -DisableArchiveScanning $false
            Set-MpPreference -DisableIntrusionPreventionSystem $false
            Set-MpPreference -DisableScriptScanning $false
        "#;

        let output = Command::new("powershell")
            .args(["-Command", script])
            .output()
            .context("Failed to execute PowerShell command")?;

        if output.status.success() {
            let mut status = self.protection_status.write().await;
            status.windows_defender_enabled = true;
            info!("Windows Defender enabled successfully");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            error!("Failed to enable Windows Defender: {}", error);
            Err(anyhow::anyhow!(
                "Failed to enable Windows Defender: {}",
                error
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn enable_windows_defender(&self) -> Result<bool> {
        warn!("Windows Defender management is only available on Windows");
        Ok(false)
    }

    pub async fn start_scan(
        &self,
        scan_type: ScanType,
        target_path: Option<&str>,
    ) -> Result<String> {
        let scan_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let scan_result = ScanResult {
            scan_id: scan_id.clone(),
            started_at: now,
            completed_at: None,
            status: ScanStatus::Pending,
            files_scanned: 0,
            threats_found: vec![],
            scan_type,
            target_path: target_path.map(|s| s.to_string()),
            error_message: None,
        };

        {
            let mut scans = self.active_scans.write().await;
            scans.insert(scan_id.clone(), scan_result);
        }

        let scan_id_clone = scan_id.clone();
        let scans = self.active_scans.clone();
        let threats = self.threats.clone();
        let config = self.config.clone();
        let target = target_path.map(|s| s.to_string());

        tokio::spawn(async move {
            Self::run_scan(scan_id_clone, scan_type, target, scans, threats, config).await;
        });

        info!("Started {:?} scan with ID: {}", scan_type, scan_id);
        Ok(scan_id)
    }

    async fn run_scan(
        scan_id: String,
        scan_type: ScanType,
        target_path: Option<String>,
        scans: Arc<RwLock<HashMap<String, ScanResult>>>,
        threats: Arc<RwLock<Vec<Threat>>>,
        config: AntivirusConfig,
    ) {
        {
            let mut scans_guard = scans.write().await;
            if let Some(scan) = scans_guard.get_mut(&scan_id) {
                scan.status = ScanStatus::Running;
            }
        }

        let scan_path = match scan_type {
            ScanType::Quick => target_path.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    "C:\\Users".to_string()
                } else {
                    "/home".to_string()
                }
            }),
            ScanType::Full => target_path.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    "C:\\".to_string()
                } else {
                    "/".to_string()
                }
            }),
            ScanType::Custom => target_path.unwrap_or_else(|| ".".to_string()),
            ScanType::Memory => "memory".to_string(),
            ScanType::Rootkit => "/".to_string(),
        };

        let result = Self::run_clamav_scan(&scan_path, &config).await;

        let mut scans_guard = scans.write().await;
        if let Some(scan) = scans_guard.get_mut(&scan_id) {
            scan.completed_at = Some(chrono::Utc::now());

            match result {
                Ok((files_scanned, found_threats)) => {
                    scan.status = ScanStatus::Completed;
                    scan.files_scanned = files_scanned;
                    scan.threats_found.clone_from(&found_threats);

                    if !found_threats.is_empty() {
                        let mut threats_guard = threats.blocking_write();
                        threats_guard.extend(found_threats);
                    }
                }
                Err(e) => {
                    scan.status = ScanStatus::Failed;
                    scan.error_message = Some(e.to_string());
                }
            }
        }
    }

    async fn run_clamav_scan(path: &str, config: &AntivirusConfig) -> Result<(u64, Vec<Threat>)> {
        let clamscan = config
            .clamav_path
            .clone()
            .map(|p| p.join("clamscan"))
            .unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    PathBuf::from("C:\\Program Files\\ClamAV\\clamscan.exe")
                } else {
                    PathBuf::from("/usr/bin/clamscan")
                }
            });

        if !clamscan.exists() {
            let output = Command::new("which")
                .arg("clamscan")
                .output()
                .unwrap_or_else(|_| {
                    Command::new("where")
                        .arg("clamscan")
                        .output()
                        .unwrap_or_else(|_| std::process::Output {
                            status: std::process::ExitStatus::default(),
                            stdout: vec![],
                            stderr: vec![],
                        })
                });

            if output.stdout.is_empty() {
                return Err(anyhow::anyhow!(
                    "ClamAV not found. Please install ClamAV first."
                ));
            }
        }

        let mut cmd = Command::new(&clamscan);
        cmd.arg("-r").arg("--infected").arg("--no-summary");

        if config.scan_archives {
            cmd.arg("--scan-archive=yes");
        }

        for excluded in &config.excluded_paths {
            cmd.arg(format!("--exclude-dir={}", excluded.display()));
        }

        cmd.arg(path);

        let output = cmd.output().context("Failed to run ClamAV scan")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut threats = Vec::new();
        let mut files_scanned: u64 = 0;

        for line in stdout.lines() {
            if line.contains("FOUND") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let file_path = parts[0].trim();
                    let threat_name = parts[1].trim().replace(" FOUND", "");

                    threats.push(Threat {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: threat_name.clone(),
                        threat_type: Self::classify_threat(&threat_name),
                        severity: Self::assess_severity(&threat_name),
                        status: ThreatStatus::Detected,
                        file_path: Some(file_path.to_string()),
                        detected_at: chrono::Utc::now(),
                        description: Some(format!("Detected by ClamAV: {}", threat_name)),
                        action_taken: None,
                    });
                }
            }
            files_scanned += 1;
        }

        Ok((files_scanned, threats))
    }

    fn classify_threat(name: &str) -> String {
        let name_lower = name.to_lowercase();
        if name_lower.contains("trojan") {
            "Trojan".to_string()
        } else if name_lower.contains("virus") {
            "Virus".to_string()
        } else if name_lower.contains("worm") {
            "Worm".to_string()
        } else if name_lower.contains("ransomware") {
            "Ransomware".to_string()
        } else if name_lower.contains("spyware") {
            "Spyware".to_string()
        } else if name_lower.contains("adware") {
            "Adware".to_string()
        } else if name_lower.contains("rootkit") {
            "Rootkit".to_string()
        } else if name_lower.contains("pup") || name_lower.contains("pua") {
            "PUP".to_string()
        } else {
            "Malware".to_string()
        }
    }

    fn assess_severity(name: &str) -> ThreatSeverity {
        let name_lower = name.to_lowercase();
        if name_lower.contains("ransomware") || name_lower.contains("rootkit") {
            ThreatSeverity::Critical
        } else if name_lower.contains("trojan") || name_lower.contains("backdoor") {
            ThreatSeverity::High
        } else if name_lower.contains("virus") || name_lower.contains("worm") {
            ThreatSeverity::Medium
        } else {
            ThreatSeverity::Low
        }
    }

    pub async fn quarantine_file(&self, file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File not found: {:?}", file_path));
        }

        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let quarantine_path = self.config.quarantine_dir.join(format!(
            "{}_{}",
            chrono::Utc::now().timestamp(),
            file_name
        ));

        std::fs::rename(file_path, &quarantine_path)
            .context("Failed to move file to quarantine")?;

        info!("File quarantined: {:?} -> {:?}", file_path, quarantine_path);

        let mut status = self.protection_status.write().await;
        status.quarantined_items += 1;

        Ok(())
    }

    pub async fn remove_threat(&self, threat_id: &str) -> Result<()> {
        let mut threats = self.threats.write().await;

        if let Some(pos) = threats.iter().position(|t| t.id == threat_id) {
            let threat = &threats[pos];

            if let Some(ref file_path) = threat.file_path {
                let path = Path::new(file_path);
                if path.exists() {
                    std::fs::remove_file(path).context("Failed to remove infected file")?;
                    info!("Removed infected file: {}", file_path);
                }
            }

            threats[pos].status = ThreatStatus::Removed;
            threats[pos].action_taken = Some("File removed".to_string());
        }

        Ok(())
    }

    pub async fn get_threats(&self) -> Vec<Threat> {
        self.threats.read().await.clone()
    }

    pub async fn get_threats_by_status(&self, status: ThreatStatus) -> Vec<Threat> {
        self.threats
            .read()
            .await
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    pub async fn get_vulnerabilities(&self) -> Vec<Vulnerability> {
        self.vulnerabilities.read().await.clone()
    }

    pub async fn scan_vulnerabilities(&self) -> Result<Vec<Vulnerability>> {
        let mut vulnerabilities = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args([
                    "-Command",
                    "Get-HotFix | Sort-Object -Property InstalledOn -Descending | Select-Object -First 1",
                ])
                .output();

            if let Ok(output) = output {
                let result = String::from_utf8_lossy(&output.stdout);

                if result.is_empty() {
                    vulnerabilities.push(Vulnerability {
                        id: uuid::Uuid::new_v4().to_string(),
                        cve_id: None,
                        name: "Missing Windows Updates".to_string(),
                        severity: ThreatSeverity::High,
                        affected_component: "Windows Update".to_string(),
                        description: "System may be missing critical security updates".to_string(),
                        remediation: Some(
                            "Run Windows Update to install latest patches".to_string(),
                        ),
                        detected_at: chrono::Utc::now(),
                        is_patched: false,
                    });
                }
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let sensitive_paths = vec!["/etc/passwd", "/etc/shadow", "/etc/ssh/sshd_config"];

            for path_str in sensitive_paths {
                let path = Path::new(path_str);
                if path.exists() {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let mode = metadata.permissions().mode();
                        if mode & 0o002 != 0 {
                            vulnerabilities.push(Vulnerability {
                                id: uuid::Uuid::new_v4().to_string(),
                                cve_id: None,
                                name: format!("Weak permissions on {}", path_str),
                                severity: ThreatSeverity::High,
                                affected_component: path_str.to_string(),
                                description: "Sensitive file has world-writable permissions"
                                    .to_string(),
                                remediation: Some(format!("chmod o-w {}", path_str)),
                                detected_at: chrono::Utc::now(),
                                is_patched: false,
                            });
                        }
                    }
                }
            }
        }

        {
            let mut vulns = self.vulnerabilities.write().await;
            vulns.extend(vulnerabilities.clone());
        }

        Ok(vulnerabilities)
    }

    pub async fn get_protection_status(&self) -> ProtectionStatus {
        self.protection_status.read().await.clone()
    }

    pub async fn get_scan_result(&self, scan_id: &str) -> Option<ScanResult> {
        self.active_scans.read().await.get(scan_id).cloned()
    }

    pub async fn get_all_scans(&self) -> Vec<ScanResult> {
        self.active_scans.read().await.values().cloned().collect()
    }

    pub async fn cancel_scan(&self, scan_id: &str) -> Result<()> {
        let mut scans = self.active_scans.write().await;
        if let Some(scan) = scans.get_mut(scan_id) {
            if scan.status == ScanStatus::Running || scan.status == ScanStatus::Pending {
                scan.status = ScanStatus::Cancelled;
                scan.completed_at = Some(chrono::Utc::now());
                info!("Scan {} cancelled", scan_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Scan is not running"))
            }
        } else {
            Err(anyhow::anyhow!("Scan not found"))
        }
    }

    pub async fn update_definitions(&self) -> Result<()> {
        info!("Updating virus definitions...");

        let freshclam = if cfg!(target_os = "windows") {
            "freshclam.exe"
        } else {
            "freshclam"
        };

        let output = Command::new(freshclam)
            .output()
            .context("Failed to run freshclam")?;

        if output.status.success() {
            let mut status = self.protection_status.write().await;
            status.last_definition_update = Some(chrono::Utc::now());
            info!("Virus definitions updated successfully");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to update definitions: {}", error))
        }
    }

    pub async fn set_realtime_protection(&self, enabled: bool) -> Result<()> {
        let mut status = self.protection_status.write().await;
        status.real_time_protection = enabled;
        info!("Real-time protection set to: {}", enabled);
        Ok(())
    }
}

pub mod api {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ThreatsResponse {
        pub threats: Vec<Threat>,
        pub total: usize,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct VulnerabilitiesResponse {
        pub vulnerabilities: Vec<Vulnerability>,
        pub total: usize,
        pub critical: usize,
        pub high: usize,
        pub medium: usize,
        pub low: usize,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ScanRequest {
        pub scan_type: ScanType,
        pub target_path: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ScanResponse {
        pub scan_id: String,
        pub status: ScanStatus,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ActionResponse {
        pub success: bool,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DefenderStatusRequest {
        pub enabled: bool,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_threat() {
        assert_eq!(
            AntivirusManager::classify_threat("Win.Trojan.Generic"),
            "Trojan"
        );
        assert_eq!(
            AntivirusManager::classify_threat("Ransomware.WannaCry"),
            "Ransomware"
        );
        assert_eq!(
            AntivirusManager::classify_threat("PUP.Optional.Adware"),
            "PUP"
        );
        assert_eq!(
            AntivirusManager::classify_threat("Unknown.Malware"),
            "Malware"
        );
        assert_eq!(AntivirusManager::classify_threat("Worm.Conficker"), "Worm");
        assert_eq!(
            AntivirusManager::classify_threat("Spyware.Keylogger"),
            "Spyware"
        );
        assert_eq!(
            AntivirusManager::classify_threat("Rootkit.Hidden"),
            "Rootkit"
        );
    }

    #[test]
    fn test_assess_severity() {
        assert_eq!(
            AntivirusManager::assess_severity("Ransomware.Test"),
            ThreatSeverity::Critical
        );
        assert_eq!(
            AntivirusManager::assess_severity("Rootkit.Hidden"),
            ThreatSeverity::Critical
        );
        assert_eq!(
            AntivirusManager::assess_severity("Trojan.Generic"),
            ThreatSeverity::High
        );
        assert_eq!(
            AntivirusManager::assess_severity("Backdoor.RAT"),
            ThreatSeverity::High
        );
        assert_eq!(
            AntivirusManager::assess_severity("Virus.Test"),
            ThreatSeverity::Medium
        );
        assert_eq!(
            AntivirusManager::assess_severity("Worm.Spread"),
            ThreatSeverity::Medium
        );
        assert_eq!(
            AntivirusManager::assess_severity("PUP.Adware"),
            ThreatSeverity::Low
        );
    }

    #[test]
    fn test_antivirus_config_default() {
        let config = AntivirusConfig::default();
        assert!(config.scan_archives);
        assert!(config.excluded_paths.is_empty());
    }

    #[test]
    fn test_threat_severity_ordering() {
        assert!(matches!(ThreatSeverity::Critical, ThreatSeverity::Critical));
        assert!(matches!(ThreatSeverity::High, ThreatSeverity::High));
        assert!(matches!(ThreatSeverity::Medium, ThreatSeverity::Medium));
        assert!(matches!(ThreatSeverity::Low, ThreatSeverity::Low));
    }

    #[test]
    fn test_scan_status_variants() {
        assert!(matches!(ScanStatus::Pending, ScanStatus::Pending));
        assert!(matches!(ScanStatus::Running, ScanStatus::Running));
        assert!(matches!(ScanStatus::Completed, ScanStatus::Completed));
        assert!(matches!(ScanStatus::Failed, ScanStatus::Failed));
        assert!(matches!(ScanStatus::Cancelled, ScanStatus::Cancelled));
    }

    #[test]
    fn test_threat_status_variants() {
        assert!(matches!(ThreatStatus::Detected, ThreatStatus::Detected));
        assert!(matches!(
            ThreatStatus::Quarantined,
            ThreatStatus::Quarantined
        ));
        assert!(matches!(ThreatStatus::Removed, ThreatStatus::Removed));
        assert!(matches!(ThreatStatus::Allowed, ThreatStatus::Allowed));
        assert!(matches!(ThreatStatus::Failed, ThreatStatus::Failed));
    }
}
