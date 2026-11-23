use super::*;

/// Pure protection claims derived from observable facts (#2601). A config flag
/// alone never grants "protection": the process-wide upload scanner registered
/// by `install_scanner_at_boot` is the source of truth. Both flags fail closed
/// when it is absent, so the manager cannot advertise protection it lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ProtectionFlags {
    pub real_time_protection: bool,
    pub general_bots_protection: bool,
}

pub(super) fn derive_protection_flags(
    real_time_config: bool,
    scanner_installed: bool,
) -> ProtectionFlags {
    ProtectionFlags {
        general_bots_protection: scanner_installed,
        real_time_protection: real_time_config && scanner_installed,
    }
}

fn parsed_quarantined_at(
    entry: &crate::upload_hostile::malware::QuarantineEntry,
) -> Option<chrono::DateTime<chrono::Utc>> {
    entry
        .quarantined_at
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&chrono::Utc))
}

/// Pure status projection from the registered scanner and the canonical
/// quarantine records. No I/O and no wall-clock reads: `now` is injected so the
/// fail-closed policy and the reported counts are unit-testable offline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DerivedProtection {
    pub flags: ProtectionFlags,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
    pub threats_blocked_today: u32,
    pub quarantined_items: u32,
}

pub(super) fn derive_protection_status(
    real_time_config: bool,
    scanner_installed: bool,
    records: &[crate::upload_hostile::malware::QuarantineEntry],
    now: chrono::DateTime<chrono::Utc>,
) -> DerivedProtection {
    let today = now.date_naive();
    let threats_blocked_today = records
        .iter()
        .filter(|entry| entry.verdict == "infected")
        .filter(|entry| {
            parsed_quarantined_at(*entry)
                .map(|value| value.date_naive() == today)
                .unwrap_or(false)
        })
        .count() as u32;
    DerivedProtection {
        flags: derive_protection_flags(real_time_config, scanner_installed),
        last_scan: records.iter().filter_map(parsed_quarantined_at).max(),
        threats_blocked_today,
        quarantined_items: records.len() as u32,
    }
}

impl AntivirusManager {
    pub fn new(config: AntivirusConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.quarantine_dir)
            .context("Failed to create quarantine directory")?;
        std::fs::create_dir_all(&config.log_dir).context("Failed to create log directory")?;

        // Report only what is actually installed and recorded (#2601). The
        // upload gate's registered scanner is the source of truth; the legacy
        // `real_time_protection` config flag alone is not protection.
        let scanner_installed = crate::upload_hostile::malware_scanner_installed();
        let records = match crate::upload_hostile::malware::list_quarantine_records() {
            Ok(records) => records,
            Err(error) => {
                warn!("Cannot read malware quarantine for protection status: {error}");
                Vec::new()
            }
        };
        let derived = derive_protection_status(
            config.real_time_protection,
            scanner_installed,
            &records,
            chrono::Utc::now(),
        );
        let protection_status = ProtectionStatus {
            real_time_protection: derived.flags.real_time_protection,
            windows_defender_enabled: Self::check_windows_defender_status(),
            general_bots_protection: derived.flags.general_bots_protection,
            last_scan: derived.last_scan,
            last_definition_update: None,
            threats_blocked_today: derived.threats_blocked_today,
            quarantined_items: derived.quarantined_items,
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
    pub(super) fn check_windows_defender_status() -> bool {
        let result = SafeCommand::new("powershell")
            .and_then(|cmd| cmd.arg("-Command"))
            .and_then(|cmd| cmd.arg("Get-MpPreference | Select-Object -ExpandProperty DisableRealtimeMonitoring"))
            .and_then(|cmd| cmd.execute());

        match result {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout);
                !result.trim().eq_ignore_ascii_case("true")
            }
            Err(e) => {
                warn!("Failed to check Windows Defender status: {}", e);
                false
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub(super) fn check_windows_defender_status() -> bool {
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

        let output = SafeCommand::new("powershell")
            .and_then(|c| c.arg("-Command"))
            .and_then(|c| c.arg(script))
            .map_err(|e| anyhow::anyhow!("Failed to build PowerShell command: {}", e))?
            .execute()
            .context("Failed to execute PowerShell command")?;

        if output.status.success() {
            let mut status = self.protection_status.write().await;
            status.windows_defender_enabled = false;
            info!("Windows Defender disabled successfully");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            log::error!("Failed to disable Windows Defender: {}", error);
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

        let output = SafeCommand::new("powershell")
            .and_then(|c| c.arg("-Command"))
            .and_then(|c| c.arg(script))
            .map_err(|e| anyhow::anyhow!("Failed to build PowerShell command: {}", e))?
            .execute()
            .context("Failed to execute PowerShell command")?;

        if output.status.success() {
            let mut status = self.protection_status.write().await;
            status.windows_defender_enabled = true;
            info!("Windows Defender enabled successfully");
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            log::error!("Failed to enable Windows Defender: {}", error);
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

        let result = Self::run_clamav_scan(&scan_path, &config);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload_hostile::malware::QuarantineEntry;

    fn entry(id: &str, verdict: &str, quarantined_at: Option<&str>) -> QuarantineEntry {
        QuarantineEntry {
            id: id.to_string(),
            original_path: format!("{id}.docx"),
            quarantine_path: String::new(),
            size: 0,
            quarantined_at: quarantined_at.map(str::to_string),
            threat_name: None,
            verdict: verdict.to_string(),
            scanner: "clamav".to_string(),
            organization_id: None,
        }
    }

    fn fixed_now() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-11-23T12:00:00+00:00")
            .map(|value| value.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now())
    }

    #[test]
    fn protection_flags_fail_closed_without_a_scanner() {
        let expected = ProtectionFlags {
            real_time_protection: false,
            general_bots_protection: false,
        };
        assert_eq!(derive_protection_flags(true, false), expected);
        assert_eq!(derive_protection_flags(false, false), expected);
    }

    #[test]
    fn protection_flags_require_both_the_flag_and_the_scanner() {
        assert_eq!(
            derive_protection_flags(true, true),
            ProtectionFlags {
                real_time_protection: true,
                general_bots_protection: true,
            }
        );
        assert_eq!(
            derive_protection_flags(false, true),
            ProtectionFlags {
                real_time_protection: false,
                general_bots_protection: true,
            }
        );
    }

    #[test]
    fn status_counts_only_todays_infected_records_and_latest_scan() {
        let records = vec![
            entry("1", "infected", Some("2026-11-23T09:00:00+00:00")),
            entry("2", "infected", Some("2026-11-22T09:00:00+00:00")),
            entry("3", "clean", Some("2026-11-23T10:00:00+00:00")),
            entry("4", "infected", None),
        ];
        let status = derive_protection_status(true, true, &records, fixed_now());
        assert_eq!(
            status.flags,
            ProtectionFlags {
                real_time_protection: true,
                general_bots_protection: true,
            }
        );
        assert_eq!(status.quarantined_items, 4);
        assert_eq!(status.threats_blocked_today, 1);
        assert_eq!(
            status.last_scan,
            Some(
                chrono::DateTime::parse_from_rfc3339("2026-11-23T10:00:00+00:00")
                    .map(|value| value.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| fixed_now())
            )
        );
    }

    #[test]
    fn status_without_a_scanner_never_claims_protection() {
        let status = derive_protection_status(true, false, &[], fixed_now());
        assert!(!status.flags.real_time_protection);
        assert!(!status.flags.general_bots_protection);
        assert_eq!(status.threats_blocked_today, 0);
        assert_eq!(status.quarantined_items, 0);
        assert_eq!(status.last_scan, None);
    }
}
