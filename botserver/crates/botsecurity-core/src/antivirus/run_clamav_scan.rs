use std::fs;

use super::*;

impl AntivirusManager {
    /// Filesystem-path entry point that delegates to the single shared byte
    /// scanner (#2601). No second ClamAV invocation path is kept here.
    pub(super) fn run_clamav_scan(path: &str, config: &AntivirusConfig) -> Result<(u64, Vec<Threat>)> {
        let file = Path::new(path);
        let metadata = fs::metadata(file).context("ClamAV scan target metadata")?;
        let configured_limit = config.max_file_size_mb.saturating_mul(1024 * 1024);
        if metadata.len() > configured_limit {
            return Err(anyhow::anyhow!("file exceeds configured antivirus size limit"));
        }
        let data = fs::read(file).context("ClamAV scan target read")?;

        match super::byte_scan::scan_bytes(&data) {
            super::byte_scan::ByteScanVerdict::Clean { .. } => Ok((1, Vec::new())),
            super::byte_scan::ByteScanVerdict::Infected { signature, .. } => {
                let threat = Threat {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: signature.clone(),
                    threat_type: Self::classify_threat(&signature),
                    severity: Self::assess_severity(&signature),
                    status: ThreatStatus::Detected,
                    file_path: Some(path.to_string()),
                    detected_at: chrono::Utc::now(),
                    description: Some(format!("Detected by ClamAV: {signature}")),
                    action_taken: None,
                };
                Ok((1, vec![threat]))
            }
            super::byte_scan::ByteScanVerdict::Failed { detail } => {
                Err(anyhow::anyhow!("ClamAV scan failed: {detail}"))
            }
        }
    }

    pub(super) fn classify_threat(name: &str) -> String {
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

    pub(super) fn assess_severity(name: &str) -> ThreatSeverity {
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
            return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
        }

        let file_name = match file_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                log::warn!(
                    "quarantine: {} has no final path component, using an empty file name",
                    file_path.display()
                );
                String::new()
            }
        };

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
            let script = "Get-HotFix | Sort-Object -Property InstalledOn -Descending | Select-Object -First 1";
            let output = SafeCommand::new("powershell")
                .and_then(|c| c.arg("-Command"))
                .and_then(|c| c.arg(script));

            if let Ok(cmd) = output {
                if let Ok(result) = cmd.execute() {
                    let output_str = String::from_utf8_lossy(&result.stdout);

                    if output_str.is_empty() {
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
        }

        #[cfg(unix)]
        {
            

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
}
