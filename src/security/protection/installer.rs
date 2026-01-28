use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{error, info, warn};

use crate::security::command_guard::SafeCommand;

#[cfg(not(windows))]
const SUDOERS_FILE: &str = "/etc/sudoers.d/gb-protection";

const SUDOERS_CONTENT: &str = r#"# General Bots Security Protection Tools
# This file is managed by botserver install protection
# DO NOT EDIT MANUALLY

# Lynis - security auditing
{user} ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system
{user} ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --quick
{user} ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --quick --no-colors
{user} ALL=(ALL) NOPASSWD: /usr/bin/lynis audit system --no-colors

# RKHunter - rootkit detection
{user} ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --check --skip-keypress
{user} ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --check --skip-keypress --report-warnings-only
{user} ALL=(ALL) NOPASSWD: /usr/bin/rkhunter --update

# Chkrootkit - rootkit detection
{user} ALL=(ALL) NOPASSWD: /usr/bin/chkrootkit
{user} ALL=(ALL) NOPASSWD: /usr/bin/chkrootkit -q

# Suricata - IDS/IPS
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl start suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl disable suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active suricata
{user} ALL=(ALL) NOPASSWD: /usr/bin/suricata-update

# ClamAV - antivirus
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl start clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl disable clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active clamav-daemon
{user} ALL=(ALL) NOPASSWD: /usr/bin/freshclam

# LMD (Linux Malware Detect)
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /home
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /var/www
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet -a /tmp
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet --update-sigs
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/maldet --update-ver
"#;

#[cfg(not(windows))]
const PACKAGES: &[&str] = &[
    "lynis",
    "rkhunter",
    "chkrootkit",
    "suricata",
    "clamav",
    "clamav-daemon",
];

#[cfg(windows)]
const WINDOWS_TOOLS: &[(&str, &str)] = &[
    ("Windows Defender", "MpCmdRun"),
    ("PowerShell", "powershell"),
    ("Windows Firewall", "netsh"),
];

pub struct ProtectionInstaller {
    user: String,
}

impl ProtectionInstaller {
    pub fn new() -> Result<Self> {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        Ok(Self { user })
    }

    #[cfg(windows)]
    pub fn check_admin() -> bool {
        let result = Command::new("powershell")
            .args([
                "-Command",
                "([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)"
            ])
            .output();

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.trim() == "True"
            }
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    pub fn check_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false)
    }

    pub fn install(&self) -> Result<InstallResult> {
        #[cfg(windows)]
        {
            if !Self::check_admin() {
                return Err(anyhow::anyhow!(
                    "This command requires administrator privileges. Run as Administrator."
                ));
            }
        }

        #[cfg(not(windows))]
        {
            if !Self::check_root() {
                return Err(anyhow::anyhow!(
                    "This command requires root privileges. Run with: sudo botserver install protection"
                ));
            }
        }

        info!(
            "Starting security protection installation for user: {}",
            self.user
        );

        let mut result = InstallResult::default();

        #[cfg(windows)]
        {
            match self.configure_windows_security() {
                Ok(()) => {
                    result
                        .packages_installed
                        .push("Windows Defender".to_string());
                    result
                        .packages_installed
                        .push("Windows Firewall".to_string());
                    info!("Windows security configured successfully");
                }
                Err(e) => {
                    warn!("Windows security configuration had issues: {e}");
                    result
                        .warnings
                        .push(format!("Windows security configuration: {e}"));
                }
            }

            match self.update_windows_signatures() {
                Ok(()) => {
                    result.databases_updated = true;
                    info!("Windows security signatures updated");
                }
                Err(e) => {
                    warn!("Windows signature update failed: {e}");
                    result
                        .warnings
                        .push(format!("Windows signature update: {e}"));
                }
            }
        }

        #[cfg(not(windows))]
        {
            match self.install_packages() {
                Ok(installed) => {
                    result.packages_installed = installed;
                    info!("Packages installed: {:?}", result.packages_installed);
                }
                Err(e) => {
                    error!("Failed to install packages: {e}");
                    result
                        .errors
                        .push(format!("Package installation failed: {e}"));
                }
            }

            match self.create_sudoers() {
                Ok(()) => {
                    result.sudoers_created = true;
                    info!("Sudoers file created successfully");
                }
                Err(e) => {
                    error!("Failed to create sudoers file: {e}");
                    result.errors.push(format!("Sudoers creation failed: {e}"));
                }
            }

            match self.install_lmd() {
                Ok(installed) => {
                    if installed {
                        result.packages_installed.push("maldetect".to_string());
                        info!("LMD (maldetect) installed successfully");
                    }
                }
                Err(e) => {
                    warn!("LMD installation skipped: {e}");
                    result
                        .warnings
                        .push(format!("LMD installation skipped: {e}"));
                }
            }

            match self.update_databases() {
                Ok(()) => {
                    result.databases_updated = true;
                    info!("Security databases updated");
                }
                Err(e) => {
                    warn!("Database update failed: {e}");
                    result.warnings.push(format!("Database update failed: {e}"));
                }
            }
        }

        result.success = result.errors.is_empty();
        Ok(result)
    }

    #[cfg(not(windows))]
    fn install_packages(&self) -> Result<Vec<String>> {
        info!("Updating package lists...");

        SafeCommand::new("apt-get")?
            .arg("update")?
            .execute()
            .context("Failed to update package lists")?;

        let mut installed = Vec::new();

        for package in PACKAGES {
            info!("Installing package: {package}");

            let result = SafeCommand::new("apt-get")?
                .arg("install")?
                .arg("-y")?
                .arg(package)?
                .execute();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        installed.push((*package).to_string());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        warn!("Package {package} installation had issues: {stderr}");
                    }
                }
                Err(e) => {
                    warn!("Failed to install {package}: {e}");
                }
            }
        }

        Ok(installed)
    }

    #[cfg(windows)]
    fn configure_windows_security(&self) -> Result<()> {
        info!("Configuring Windows security settings...");

        // Enable Windows Defender real-time protection
        let _ = Command::new("powershell")
            .args([
                "-Command",
                "Set-MpPreference -DisableRealtimeMonitoring $false; Set-MpPreference -DisableIOAVProtection $false; Set-MpPreference -DisableScriptScanning $false"
            ])
            .output();

        // Enable Windows Firewall
        let _ = Command::new("netsh")
            .args(["advfirewall", "set", "allprofiles", "state", "on"])
            .output();

        // Enable Windows Defender scanning for mapped drives
        let _ = Command::new("powershell")
            .args([
                "-Command",
                "Set-MpPreference -DisableRemovableDriveScanning $false -DisableScanningMappedNetworkDrivesForFullScan $false"
            ])
            .output();

        info!("Windows security configuration completed");
        Ok(())
    }

    #[cfg(not(windows))]
    fn create_sudoers(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let content = SUDOERS_CONTENT.replace("{user}", &self.user);

        info!("Creating sudoers file at {SUDOERS_FILE}");

        fs::write(SUDOERS_FILE, &content).context("Failed to write sudoers file")?;

        let permissions = fs::Permissions::from_mode(0o440);
        fs::set_permissions(SUDOERS_FILE, permissions)
            .context("Failed to set sudoers file permissions")?;

        self.validate_sudoers()?;

        info!("Sudoers file created and validated");
        Ok(())
    }

    #[cfg(not(windows))]
    #[cfg(not(windows))]
    fn validate_sudoers(&self) -> Result<()> {
        let output = std::process::Command::new("visudo")
            .args(["-c", "-f", SUDOERS_FILE])
            .output()
            .context("Failed to run visudo validation")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            fs::remove_file(SUDOERS_FILE).ok();
            return Err(anyhow::anyhow!("Invalid sudoers file syntax: {stderr}"));
        }

        Ok(())
    }

    #[cfg(not(windows))]
    #[cfg(not(windows))]
    fn install_lmd(&self) -> Result<bool> {
        let maldet_path = Path::new("/usr/local/sbin/maldet");
        if maldet_path.exists() {
            info!("LMD already installed");
            return Ok(false);
        }

        info!("Installing Linux Malware Detect (LMD)...");

        let temp_dir = "/tmp/maldetect_install";
        fs::create_dir_all(temp_dir).ok();

        let download_result = SafeCommand::new("curl")?
            .arg("-sL")?
            .arg("-o")?
            .arg("/tmp/maldetect-current.tar.gz")?
            .arg("https://www.rfxn.com/downloads/maldetect-current.tar.gz")?
            .execute();

        if download_result.is_err() {
            return Err(anyhow::anyhow!("Failed to download LMD"));
        }

        SafeCommand::new("tar")?
            .arg("-xzf")?
            .arg("/tmp/maldetect-current.tar.gz")?
            .arg("-C")?
            .arg(temp_dir)?
            .execute()
            .context("Failed to extract LMD archive")?;

        let entries = fs::read_dir(temp_dir)?;
        let mut install_dir = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("maldetect"))
            {
                install_dir = Some(path);
                break;
            }
        }

        let install_dir =
            install_dir.ok_or_else(|| anyhow::anyhow!("LMD install directory not found"))?;
        let install_script = install_dir.join("install.sh");

        if !install_script.exists() {
            return Err(anyhow::anyhow!("LMD install.sh not found"));
        }

        SafeCommand::new("bash")?
            .arg("-c")?
            .shell_script_arg(&format!("cd {} && ./install.sh", install_dir.display()))?
            .execute()
            .context("Failed to run LMD installer")?;

        fs::remove_dir_all(temp_dir).ok();
        fs::remove_file("/tmp/maldetect-current.tar.gz").ok();

        Ok(true)
    }

    #[cfg(not(windows))]
    #[cfg(not(windows))]
    fn update_databases(&self) -> Result<()> {
        info!("Updating security tool databases...");

        if Path::new("/usr/bin/rkhunter").exists() {
            info!("Updating RKHunter database...");
            let result = SafeCommand::new("rkhunter")?.arg("--update")?.execute();
            if let Err(e) = result {
                warn!("RKHunter update failed: {e}");
            }
        }

        if Path::new("/usr/bin/freshclam").exists() {
            info!("Updating ClamAV signatures...");
            let result = SafeCommand::new("freshclam")?.execute();
            if let Err(e) = result {
                warn!("ClamAV update failed: {e}");
            }
        }

        if Path::new("/usr/bin/suricata-update").exists() {
            info!("Updating Suricata rules...");
            let result = SafeCommand::new("suricata-update")?.execute();
            if let Err(e) = result {
                warn!("Suricata update failed: {e}");
            }
        }

        if Path::new("/usr/local/sbin/maldet").exists() {
            info!("Updating LMD signatures...");
            let result = SafeCommand::new("maldet")?.arg("--update-sigs")?.execute();
            if let Err(e) = result {
                warn!("LMD update failed: {e}");
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    fn update_windows_signatures(&self) -> Result<()> {
        info!("Updating Windows Defender signatures...");

        let result = Command::new("powershell")
            .args([
                "-Command",
                "Update-MpSignature; Write-Host 'Windows Defender signatures updated'",
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    info!("Windows Defender signatures updated successfully");
                } else {
                    warn!("Windows Defender signature update had issues");
                }
            }
            Err(e) => {
                warn!("Failed to update Windows Defender signatures: {e}");
            }
        }

        Ok(())
    }

    pub fn uninstall(&self) -> Result<UninstallResult> {
        #[cfg(windows)]
        {
            if !Self::check_admin() {
                return Err(anyhow::anyhow!(
                    "This command requires administrator privileges. Run as Administrator."
                ));
            }
        }

        #[cfg(not(windows))]
        {
            if !Self::check_root() {
                return Err(anyhow::anyhow!(
                    "This command requires root privileges. Run with: sudo botserver remove protection"
                ));
            }
        }

        info!("Removing security protection components...");

        let mut result = UninstallResult::default();

        #[cfg(not(windows))]
        {
            if Path::new(SUDOERS_FILE).exists() {
                match fs::remove_file(SUDOERS_FILE) {
                    Ok(()) => {
                        result.sudoers_removed = true;
                        info!("Removed sudoers file");
                    }
                    Err(e) => {
                        result.errors.push(format!("Failed to remove sudoers: {e}"));
                    }
                }
            }

            result.message = "Protection sudoers removed. Packages were NOT uninstalled - remove manually if needed.".to_string();
        }

        #[cfg(windows)]
        {
            result.message = "Windows protection uninstalled. Windows Defender and Firewall settings were not modified - remove manually if needed.".to_string();
        }

        result.success = result.errors.is_empty();
        Ok(result)
    }

    pub fn verify(&self) -> VerifyResult {
        let mut result = VerifyResult::default();

        #[cfg(not(windows))]
        {
            for package in PACKAGES {
                let binary = match *package {
                    "clamav" | "clamav-daemon" => "clamscan",
                    other => other,
                };

                let check = SafeCommand::new("which")
                    .and_then(|cmd| cmd.arg(binary))
                    .and_then(|cmd| cmd.execute());

                let installed = check.map(|o| o.status.success()).unwrap_or(false);
                result.tools.push(ToolVerification {
                    name: (*package).to_string(),
                    installed,
                    sudo_configured: false,
                });
            }

            let maldet_installed = Path::new("/usr/local/sbin/maldet").exists();
            result.tools.push(ToolVerification {
                name: "maldetect".to_string(),
                installed: maldet_installed,
                sudo_configured: false,
            });

            result.sudoers_exists = Path::new(SUDOERS_FILE).exists();

            if result.sudoers_exists {
                if let Ok(content) = fs::read_to_string(SUDOERS_FILE) {
                    for tool in &mut result.tools {
                        tool.sudo_configured = content.contains(&tool.name)
                            || (tool.name == "clamav" && content.contains("clamav-daemon"))
                            || (tool.name == "clamav-daemon" && content.contains("clamav-daemon"));
                    }
                }
            }

            result.all_installed = result
                .tools
                .iter()
                .filter(|t| t.name != "clamav-daemon")
                .all(|t| t.installed);
            result.all_configured = result.sudoers_exists
                && result
                    .tools
                    .iter()
                    .all(|t| t.sudo_configured || !t.installed);
        }

        #[cfg(windows)]
        {
            for (tool_name, tool_cmd) in WINDOWS_TOOLS {
                let check = Command::new(tool_cmd)
                    .arg("--version")
                    .or_else(|_| {
                        Command::new("powershell")
                            .args(["-Command", &format!("Get-Command {}", tool_cmd)])
                    })
                    .output();

                let installed = check.map(|o| o.status.success()).unwrap_or(false);
                result.tools.push(ToolVerification {
                    name: tool_name.to_string(),
                    installed,
                    sudo_configured: true, // Windows tools are typically pre-configured
                });
            }

            result.sudoers_exists = false; // No sudoers on Windows
            result.all_installed = result.tools.iter().all(|t| t.installed);
            result.all_configured = true; // Windows tools are pre-configured
        }

        result
    }
}

impl Default for ProtectionInstaller {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            user: "unknown".to_string(),
        })
    }
}

pub struct InstallResult {
    pub success: bool,
    pub packages_installed: Vec<String>,
    pub sudoers_created: bool,
    pub databases_updated: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for InstallResult {
    fn default() -> Self {
        Self {
            success: false,
            packages_installed: Vec::new(),
            sudoers_created: false,
            databases_updated: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl InstallResult {
    pub fn print(&self) {
        println!("\n=== Security Protection Installation Result ===");
        println!(
            "Status: {}",
            if self.success {
                "✓ SUCCESS"
            } else {
                "✗ FAILED"
            }
        );

        if !self.packages_installed.is_empty() {
            println!("\nInstalled Packages:");
            for pkg in &self.packages_installed {
                println!("  - {pkg}");
            }
        }

        #[cfg(not(windows))]
        {
            println!("\nSudoers Configuration:");
            println!(
                "  - File created: {}",
                if self.sudoers_created { "✓" } else { "✗" }
            );
        }

        println!(
            "\nDatabases Updated: {}",
            if self.databases_updated { "✓" } else { "✗" }
        );

        if !self.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &self.warnings {
                println!("  ! {warning}");
            }
        }

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  ✗ {error}");
            }
        }

        println!("\n");
    }
}

pub struct UninstallResult {
    pub success: bool,
    pub sudoers_removed: bool,
    pub message: String,
    pub errors: Vec<String>,
}

impl Default for UninstallResult {
    fn default() -> Self {
        Self {
            success: false,
            sudoers_removed: false,
            message: String::new(),
            errors: Vec::new(),
        }
    }
}

impl UninstallResult {
    pub fn print(&self) {
        println!("\n=== Security Protection Uninstallation Result ===");
        println!(
            "Status: {}",
            if self.success {
                "✓ SUCCESS"
            } else {
                "✗ FAILED"
            }
        );

        #[cfg(not(windows))]
        {
            println!(
                "Sudoers removed: {}",
                if self.sudoers_removed { "✓" } else { "✗" }
            );
        }

        println!("\nMessage:");
        println!("  {}", self.message);

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  ✗ {error}");
            }
        }

        println!("\n");
    }
}

pub struct VerifyResult {
    pub all_installed: bool,
    pub all_configured: bool,
    pub sudoers_exists: bool,
    pub tools: Vec<ToolVerification>,
}

impl Default for VerifyResult {
    fn default() -> Self {
        Self {
            all_installed: false,
            all_configured: false,
            sudoers_exists: false,
            tools: Vec::new(),
        }
    }
}

pub struct ToolVerification {
    pub name: String,
    pub installed: bool,
    pub sudo_configured: bool,
}

impl VerifyResult {
    pub fn print(&self) {
        println!("\n=== Security Protection Verification ===");
        println!(
            "All tools installed: {}",
            if self.all_installed { "✓" } else { "✗" }
        );
        println!(
            "All tools configured: {}",
            if self.all_configured { "✓" } else { "✗" }
        );

        #[cfg(not(windows))]
        {
            println!(
                "Sudoers file exists: {}",
                if self.sudoers_exists { "✓" } else { "✗" }
            );
        }

        println!("\nTool Status:");
        for tool in &self.tools {
            println!(
                "  {} {}: {} {}",
                if tool.installed { "✓" } else { "✗" },
                tool.name,
                if tool.installed {
                    "installed"
                } else {
                    "not installed"
                },
                if tool.sudo_configured {
                    "(configured)"
                } else {
                    "(not configured)"
                }
            );
        }

        println!("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_result_default() {
        let result = InstallResult::default();
        assert!(!result.success);
        assert!(result.packages_installed.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_verify_result_default() {
        let result = VerifyResult::default();
        assert!(!result.all_installed);
        assert!(!result.all_configured);
        assert!(result.tools.is_empty());
    }

    #[test]
    fn test_tool_verification_default() {
        let tool = ToolVerification {
            name: "test".to_string(),
            installed: false,
            sudo_configured: false,
        };
        assert!(!tool.installed);
        assert!(!tool.sudo_configured);
    }

    #[test]
    fn test_uninstall_result_default() {
        let result = UninstallResult::default();
        assert!(!result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_protection_installer_default() {
        let installer = ProtectionInstaller::default();
        assert!(!installer.user.is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn test_sudoers_content_has_placeholder() {
        assert!(SUDOERS_CONTENT.contains("{user}"));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_sudoers_content_no_wildcards() {
        // Ensure no dangerous wildcards in sudoers
        assert!(!SUDOERS_CONTENT.contains(" ALL=(ALL) ALL"));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_packages_list() {
        assert!(PACKAGES.len() > 0);
        assert!(PACKAGES.contains(&"lynis"));
        assert!(PACKAGES.contains(&"rkhunter"));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_tools_list() {
        assert!(WINDOWS_TOOLS.len() > 0);
        assert!(WINDOWS_TOOLS.iter().any(|t| t.0 == "Windows Defender"));
    }
}
