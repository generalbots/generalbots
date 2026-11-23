use super::*;

impl AntivirusManager {
    pub async fn update_definitions(&self) -> Result<()> {
        info!("Updating virus definitions...");

        let freshclam = if cfg!(target_os = "windows") {
            "freshclam.exe"
        } else {
            "freshclam"
        };

        let output = SafeCommand::new(freshclam)
            .and_then(|cmd| cmd.execute())
            .map_err(|e| anyhow::anyhow!("Failed to run freshclam: {}", e))?;

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
        // #2601: a setter must not fabricate protection. Derive the claim from
        // the live scanner registration the same way `new` does, so requesting
        // "on" with no registered scanner reports disabled instead.
        let scanner_installed = crate::upload_hostile::malware_scanner_installed();
        let flags = super::check_windows_defender_status::derive_protection_flags(
            enabled,
            scanner_installed,
        );
        if enabled && !scanner_installed {
            warn!("Refusing to report real-time protection: no malware scanner is registered");
        }
        let mut status = self.protection_status.write().await;
        status.real_time_protection = flags.real_time_protection;
        status.general_bots_protection = flags.general_bots_protection;
        info!(
            "Real-time protection set to: {}",
            status.real_time_protection
        );
        Ok(())
    }
}
