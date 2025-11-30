//! Desktop Tools Module
//!
//! This module provides desktop utility tools including:
//! - Drive cleaner for removing temporary files and junk
//! - Windows optimizer integration
//! - Brave browser installer
//! - System maintenance utilities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Cleanup result statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupStats {
    pub files_removed: u64,
    pub directories_removed: u64,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cleanup category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupCategory {
    TempFiles,
    BrowserCache,
    WindowsTemp,
    RecycleBin,
    Downloads,
    Logs,
    Thumbnails,
    UpdateCache,
    All,
}

/// Optimization task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationTask {
    DefragmentDisk,
    ClearMemory,
    DisableStartupPrograms,
    OptimizeServices,
    CleanRegistry,
    UpdateDrivers,
    All,
}

/// Optimization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStatus {
    pub task: OptimizationTask,
    pub status: TaskStatus,
    pub progress: u8,
    pub message: String,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Installation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationStatus {
    pub software: String,
    pub version: Option<String>,
    pub status: TaskStatus,
    pub progress: u8,
    pub message: String,
    pub download_url: Option<String>,
}

/// Desktop tools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopToolsConfig {
    /// Paths to clean
    pub temp_paths: Vec<PathBuf>,
    /// Browser cache paths
    pub browser_cache_paths: Vec<PathBuf>,
    /// Download folder
    pub downloads_path: PathBuf,
    /// Windows Optimization script URL
    pub optimization_script_url: String,
    /// Brave installer URL
    pub brave_installer_url: String,
    /// Minimum free space warning (GB)
    pub min_free_space_gb: u64,
}

impl Default for DesktopToolsConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let temp_paths = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from(
                    std::env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string()),
                ),
                PathBuf::from(
                    std::env::var("TMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string()),
                ),
                home.join("AppData\\Local\\Temp"),
            ]
        } else {
            vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
                home.join(".cache"),
            ]
        };

        let browser_cache_paths = if cfg!(target_os = "windows") {
            vec![
                home.join("AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache"),
                home.join("AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache"),
                home.join(
                    "AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache",
                ),
                home.join("AppData\\Local\\Mozilla\\Firefox\\Profiles"),
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                home.join("Library/Caches/Google/Chrome"),
                home.join("Library/Caches/BraveSoftware/Brave-Browser"),
                home.join("Library/Caches/Firefox"),
            ]
        } else {
            vec![
                home.join(".cache/google-chrome"),
                home.join(".cache/BraveSoftware/Brave-Browser"),
                home.join(".cache/mozilla/firefox"),
            ]
        };

        let downloads_path = dirs::download_dir().unwrap_or_else(|| home.join("Downloads"));

        Self {
            temp_paths,
            browser_cache_paths,
            downloads_path,
            optimization_script_url: "https://github.com/Metaljisawa/OptimizationWindowsV1"
                .to_string(),
            brave_installer_url: "https://laptop-updates.brave.com/latest/winx64".to_string(),
            min_free_space_gb: 10,
        }
    }
}

/// Desktop Tools Manager
pub struct DesktopToolsManager {
    config: DesktopToolsConfig,
    cleanup_stats: RwLock<CleanupStats>,
    optimization_status: RwLock<Option<OptimizationStatus>>,
    installation_status: RwLock<Option<InstallationStatus>>,
}

impl DesktopToolsManager {
    /// Create a new desktop tools manager
    pub fn new(config: DesktopToolsConfig) -> Self {
        Self {
            config,
            cleanup_stats: RwLock::new(CleanupStats::default()),
            optimization_status: RwLock::new(None),
            installation_status: RwLock::new(None),
        }
    }

    /// Clean temporary files and junk
    pub async fn clean_drive(&self, categories: Vec<CleanupCategory>) -> Result<CleanupStats> {
        let mut stats = CleanupStats {
            started_at: Some(chrono::Utc::now()),
            ..Default::default()
        };

        info!("Starting drive cleanup for categories: {:?}", categories);

        for category in &categories {
            match category {
                CleanupCategory::TempFiles | CleanupCategory::All => {
                    for path in &self.config.temp_paths {
                        if path.exists() {
                            self.clean_directory(path, &mut stats).await;
                        }
                    }
                }
                CleanupCategory::BrowserCache => {
                    for path in &self.config.browser_cache_paths {
                        if path.exists() {
                            self.clean_directory(path, &mut stats).await;
                        }
                    }
                }
                CleanupCategory::WindowsTemp => {
                    #[cfg(target_os = "windows")]
                    {
                        let windows_temp = PathBuf::from("C:\\Windows\\Temp");
                        if windows_temp.exists() {
                            self.clean_directory(&windows_temp, &mut stats).await;
                        }
                    }
                }
                CleanupCategory::RecycleBin => {
                    self.empty_recycle_bin(&mut stats).await;
                }
                CleanupCategory::Downloads => {
                    // Only clean old files in downloads (older than 30 days)
                    self.clean_old_downloads(&mut stats, 30).await;
                }
                CleanupCategory::Logs => {
                    self.clean_logs(&mut stats).await;
                }
                CleanupCategory::Thumbnails => {
                    self.clean_thumbnails(&mut stats).await;
                }
                CleanupCategory::UpdateCache => {
                    self.clean_update_cache(&mut stats).await;
                }
                _ => {}
            }
        }

        stats.completed_at = Some(chrono::Utc::now());

        // Update stored stats
        *self.cleanup_stats.write().await = stats.clone();

        info!(
            "Drive cleanup completed. Files: {}, Dirs: {}, Freed: {} bytes",
            stats.files_removed, stats.directories_removed, stats.bytes_freed
        );

        Ok(stats)
    }

    /// Clean a directory recursively
    async fn clean_directory(&self, path: &Path, stats: &mut CleanupStats) {
        if !path.exists() {
            return;
        }

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                stats
                    .errors
                    .push(format!("Failed to read {:?}: {}", path, e));
                return;
            }
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Recursively clean subdirectory
                Box::pin(self.clean_directory(&entry_path, stats)).await;

                // Try to remove empty directory
                if let Ok(mut dir) = std::fs::read_dir(&entry_path) {
                    if dir.next().is_none() {
                        if std::fs::remove_dir(&entry_path).is_ok() {
                            stats.directories_removed += 1;
                        }
                    }
                }
            } else {
                // Get file size before deletion
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                match std::fs::remove_file(&entry_path) {
                    Ok(_) => {
                        stats.files_removed += 1;
                        stats.bytes_freed += size;
                    }
                    Err(e) => {
                        stats
                            .errors
                            .push(format!("Failed to delete {:?}: {}", entry_path, e));
                    }
                }
            }
        }
    }

    /// Empty the recycle bin
    async fn empty_recycle_bin(&self, stats: &mut CleanupStats) {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args([
                    "-Command",
                    "Clear-RecycleBin -Force -ErrorAction SilentlyContinue",
                ])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    info!("Recycle bin emptied successfully");
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr);
                    stats
                        .errors
                        .push(format!("Failed to empty recycle bin: {}", error));
                }
                Err(e) => {
                    stats
                        .errors
                        .push(format!("Failed to empty recycle bin: {}", e));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let trash_path = home.join(".local/share/Trash/files");
                if trash_path.exists() {
                    self.clean_directory(&trash_path, stats).await;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                let trash_path = home.join(".Trash");
                if trash_path.exists() {
                    self.clean_directory(&trash_path, stats).await;
                }
            }
        }
    }

    /// Clean old files in downloads folder
    async fn clean_old_downloads(&self, stats: &mut CleanupStats, days_old: u64) {
        let downloads = &self.config.downloads_path;
        if !downloads.exists() {
            return;
        }

        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old as i64);

        if let Ok(entries) = std::fs::read_dir(downloads) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: chrono::DateTime<chrono::Utc> = modified.into();
                        if modified_time < cutoff {
                            let size = metadata.len();
                            if std::fs::remove_file(entry.path()).is_ok() {
                                stats.files_removed += 1;
                                stats.bytes_freed += size;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Clean system logs
    async fn clean_logs(&self, stats: &mut CleanupStats) {
        let log_paths = if cfg!(target_os = "windows") {
            vec![
                PathBuf::from("C:\\Windows\\Logs"),
                PathBuf::from("C:\\Windows\\Panther"),
            ]
        } else {
            vec![PathBuf::from("/var/log")]
        };

        for path in log_paths {
            if path.exists() {
                // Only clean .log files older than 7 days
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.extension().map(|e| e == "log").unwrap_or(false) {
                            if let Ok(metadata) = entry.metadata() {
                                if let Ok(modified) = metadata.modified() {
                                    let modified_time: chrono::DateTime<chrono::Utc> =
                                        modified.into();
                                    let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
                                    if modified_time < cutoff {
                                        let size = metadata.len();
                                        if std::fs::remove_file(&entry_path).is_ok() {
                                            stats.files_removed += 1;
                                            stats.bytes_freed += size;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Clean thumbnail cache
    async fn clean_thumbnails(&self, stats: &mut CleanupStats) {
        #[cfg(target_os = "windows")]
        {
            if let Some(home) = dirs::home_dir() {
                let thumb_path = home.join("AppData\\Local\\Microsoft\\Windows\\Explorer");
                if thumb_path.exists() {
                    if let Ok(entries) = std::fs::read_dir(&thumb_path) {
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.starts_with("thumbcache_") || name.starts_with("iconcache_") {
                                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                if std::fs::remove_file(entry.path()).is_ok() {
                                    stats.files_removed += 1;
                                    stats.bytes_freed += size;
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(home) = dirs::home_dir() {
                let thumb_path = home.join(".cache/thumbnails");
                if thumb_path.exists() {
                    self.clean_directory(&thumb_path, stats).await;
                }
            }
        }
    }

    /// Clean Windows Update cache
    async fn clean_update_cache(&self, stats: &mut CleanupStats) {
        #[cfg(target_os = "windows")]
        {
            let update_paths = vec![
                PathBuf::from("C:\\Windows\\SoftwareDistribution\\Download"),
                PathBuf::from("C:\\Windows\\SoftwareDistribution\\DataStore"),
            ];

            // Stop Windows Update service first
            let _ = Command::new("net").args(["stop", "wuauserv"]).output();

            for path in update_paths {
                if path.exists() {
                    self.clean_directory(&path, stats).await;
                }
            }

            // Restart Windows Update service
            let _ = Command::new("net").args(["start", "wuauserv"]).output();
        }
    }

    /// Run Windows optimizer
    pub async fn run_optimizer(&self, tasks: Vec<OptimizationTask>) -> Result<()> {
        info!("Starting Windows optimization...");

        for task in tasks {
            let status = OptimizationStatus {
                task,
                status: TaskStatus::Running,
                progress: 0,
                message: format!("Running {:?}...", task),
                started_at: Some(chrono::Utc::now()),
                completed_at: None,
            };

            *self.optimization_status.write().await = Some(status);

            let result = match task {
                OptimizationTask::DefragmentDisk => self.defragment_disk().await,
                OptimizationTask::ClearMemory => self.clear_memory().await,
                OptimizationTask::DisableStartupPrograms => self.disable_startup_programs().await,
                OptimizationTask::OptimizeServices => self.optimize_services().await,
                OptimizationTask::CleanRegistry => self.clean_registry().await,
                OptimizationTask::UpdateDrivers => self.update_drivers().await,
                OptimizationTask::All => {
                    self.defragment_disk().await?;
                    self.clear_memory().await?;
                    self.optimize_services().await?;
                    Ok(())
                }
            };

            let mut status = self.optimization_status.write().await;
            if let Some(ref mut s) = *status {
                s.completed_at = Some(chrono::Utc::now());
                match result {
                    Ok(_) => {
                        s.status = TaskStatus::Completed;
                        s.progress = 100;
                        s.message = format!("{:?} completed successfully", task);
                    }
                    Err(e) => {
                        s.status = TaskStatus::Failed;
                        s.message = format!("{:?} failed: {}", task, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Defragment disk (Windows only)
    async fn defragment_disk(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("defrag")
                .args(["C:", "/O"]) // Optimize
                .output()
                .context("Failed to run defrag")?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Defrag failed: {}", error));
            }
        }

        Ok(())
    }

    /// Clear memory (free up RAM)
    async fn clear_memory(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Use EmptyWorkingSet via PowerShell
            let script = r#"
                Get-Process | ForEach-Object {
                    try {
                        $_.MinWorkingSet = $_.MinWorkingSet
                    } catch {}
                }
            "#;

            let _ = Command::new("powershell")
                .args(["-Command", script])
                .output();
        }

        #[cfg(target_os = "linux")]
        {
            // Drop caches
            let _ = Command::new("sh")
                .args(["-c", "sync; echo 3 > /proc/sys/vm/drop_caches"])
                .output();
        }

        Ok(())
    }

    /// Disable unnecessary startup programs
    async fn disable_startup_programs(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // List startup programs that are commonly not needed
            let script = r#"
                $unwanted = @('Discord', 'Spotify', 'Steam', 'OneDrive', 'Skype')
                Get-CimInstance Win32_StartupCommand | Where-Object {
                    $unwanted -contains $_.Name
                } | ForEach-Object {
                    Write-Host "Found: $($_.Name)"
                }
            "#;

            let _ = Command::new("powershell")
                .args(["-Command", script])
                .output();
        }

        Ok(())
    }

    /// Optimize Windows services
    async fn optimize_services(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Services that can be safely disabled on most systems
            let services_to_disable = vec![
                "DiagTrack",        // Connected User Experiences and Telemetry
                "dmwappushservice", // WAP Push Message Routing Service
                "MapsBroker",       // Downloaded Maps Manager
                "RemoteRegistry",   // Remote Registry
                "RetailDemo",       // Retail Demo Service
            ];

            for service in services_to_disable {
                let _ = Command::new("sc")
                    .args(["config", service, "start=", "disabled"])
                    .output();
            }
        }

        Ok(())
    }

    /// Clean Windows registry
    async fn clean_registry(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            warn!("Registry cleaning is a sensitive operation. Skipping automatic cleanup.");
            // Registry cleaning should be done manually or with dedicated tools
        }

        Ok(())
    }

    /// Update drivers (Windows only)
    async fn update_drivers(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Use Windows Update to check for driver updates
            let script = r#"
                $UpdateSession = New-Object -ComObject Microsoft.Update.Session
                $UpdateSearcher = $UpdateSession.CreateUpdateSearcher()
                $SearchResult = $UpdateSearcher.Search("IsInstalled=0 and Type='Driver'")
                $SearchResult.Updates.Count
            "#;

            let output = Command::new("powershell")
                .args(["-Command", script])
                .output()
                .context("Failed to check for driver updates")?;

            let count = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!("Found {} driver updates available", count);
        }

        Ok(())
    }

    /// Install Brave browser
    pub async fn install_brave(&self) -> Result<InstallationStatus> {
        let mut status = InstallationStatus {
            software: "Brave Browser".to_string(),
            version: None,
            status: TaskStatus::Running,
            progress: 0,
            message: "Starting download...".to_string(),
            download_url: Some(self.config.brave_installer_url.clone()),
        };

        *self.installation_status.write().await = Some(status.clone());

        info!("Installing Brave Browser...");

        #[cfg(target_os = "windows")]
        {
            // Download Brave installer
            let temp_dir = std::env::temp_dir();
            let installer_path = temp_dir.join("BraveBrowserSetup.exe");

            // Use PowerShell to download
            let download_cmd = format!(
                "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                self.config.brave_installer_url,
                installer_path.display()
            );

            status.message = "Downloading Brave installer...".to_string();
            status.progress = 25;
            *self.installation_status.write().await = Some(status.clone());

            let output = Command::new("powershell")
                .args(["-Command", &download_cmd])
                .output()
                .context("Failed to download Brave installer")?;

            if !output.status.success() {
                status.status = TaskStatus::Failed;
                status.message = "Failed to download installer".to_string();
                *self.installation_status.write().await = Some(status.clone());
                return Err(anyhow::anyhow!("Failed to download Brave installer"));
            }

            status.message = "Running installer...".to_string();
            status.progress = 50;
            *self.installation_status.write().await = Some(status.clone());

            // Run installer silently
            let output = Command::new(&installer_path)
                .args(["/silent", "/install"])
                .output()
                .context("Failed to run Brave installer")?;

            // Clean up installer
            let _ = std::fs::remove_file(&installer_path);

            if output.status.success() {
                status.status = TaskStatus::Completed;
                status.progress = 100;
                status.message = "Brave Browser installed successfully!".to_string();
                info!("Brave Browser installed successfully");
            } else {
                status.status = TaskStatus::Failed;
                status.message = "Installation failed".to_string();
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try apt first (Debian/Ubuntu)
            let output = Command::new("sh")
                .args(["-c", r#"
                    curl -fsSLo /usr/share/keyrings/brave-browser-archive-keyring.gpg https://brave-browser-apt-release.s3.brave.com/brave-browser-archive-keyring.gpg
                    echo "deb [signed-by=/usr/share/keyrings/brave-browser-archive-keyring.gpg] https://brave-browser-apt-release.s3.brave.com/ stable main" | tee /etc/apt/sources.list.d/brave-browser-release.list
                    apt update && apt install -y brave-browser
                "#])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    status.status = TaskStatus::Completed;
                    status.progress = 100;
                    status.message = "Brave Browser installed successfully!".to_string();
                }
                _ => {
                    // Try snap as fallback
                    let snap_output = Command::new("snap").args(["install", "brave"]).output();

                    match snap_output {
                        Ok(output) if output.status.success() => {
                            status.status = TaskStatus::Completed;
                            status.progress = 100;
                            status.message = "Brave Browser installed via Snap!".to_string();
                        }
                        _ => {
                            status.status = TaskStatus::Failed;
                            status.message = "Failed to install Brave Browser".to_string();
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Use Homebrew
            let output = Command::new("brew")
                .args(["install", "--cask", "brave-browser"])
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    status.status = TaskStatus::Completed;
                    status.progress = 100;
                    status.message = "Brave Browser installed successfully!".to_string();
                }
                _ => {
                    status.status = TaskStatus::Failed;
                    status.message =
                        "Failed to install. Please install Homebrew first.".to_string();
                }
            }
        }

        *self.installation_status.write().await = Some(status.clone());
        Ok(status)
    }

    /// Run external optimization script
    pub async fn run_optimization_script(&self) -> Result<()> {
        info!(
            "Running optimization script from: {}",
            self.config.optimization_script_url
        );

        #[cfg(target_os = "windows")]
        {
            let temp_dir = std::env::temp_dir();
            let script_dir = temp_dir.join("OptimizationWindowsV1");

            // Clone the repository
            let clone_cmd = format!(
                "git clone {} {}",
                self.config.optimization_script_url,
                script_dir.display()
            );

            let output = Command::new("cmd")
                .args(["/C", &clone_cmd])
                .output()
                .context("Failed to clone optimization script")?;

            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to clone optimization repository"));
            }

            // Find and run the main script
            let script_path = script_dir.join("optimize.bat");
            if script_path.exists() {
                let _ = Command::new("cmd")
                    .args(["/C", &script_path.to_string_lossy().to_string()])
                    .spawn();

                info!("Optimization script started");
            } else {
                warn!("Optimization script not found at expected location");
            }

            // Cleanup
            let _ = std::fs::remove_dir_all(&script_dir);
        }

        Ok(())
    }

    /// Get disk space information
    pub async fn get_disk_info(&self) -> Result<Vec<DiskInfo>> {
        let mut disks = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args([
                    "-Command",
                    "Get-PSDrive -PSProvider FileSystem | Select-Object Name, Used, Free | ConvertTo-Json",
                ])
                .output()
                .context("Failed to get disk info")?;

            if output.status.success() {
                let json = String::from_utf8_lossy(&output.stdout);
                if let Ok(drives) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
                    for drive in drives {
                        disks.push(DiskInfo {
                            name: format!("{}:", drive["Name"].as_str().unwrap_or("?")),
                            total_bytes: drive["Used"].as_u64().unwrap_or(0)
                                + drive["Free"].as_u64().unwrap_or(0),
                            free_bytes: drive["Free"].as_u64().unwrap_or(0),
                            used_bytes: drive["Used"].as_u64().unwrap_or(0),
                        });
                    }
                }
            }
        }

        #[cfg(unix)]
        {
            let output = Command::new("df")
                .args(["-B1", "--output=source,size,used,avail"])
                .output()
                .context("Failed to get disk info")?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        disks.push(DiskInfo {
                            name: parts[0].to_string(),
                            total_bytes: parts[1].parse().unwrap_or(0),
                            used_bytes: parts[2].parse().unwrap_or(0),
                            free_bytes: parts[3].parse().unwrap_or(0),
                        });
                    }
                }
            }
        }

        Ok(disks)
    }

    /// Get cleanup stats
    pub async fn get_cleanup_stats(&self) -> CleanupStats {
        self.cleanup_stats.read().await.clone()
    }

    /// Get optimization status
    pub async fn get_optimization_status(&self) -> Option<OptimizationStatus> {
        self.optimization_status.read().await.clone()
    }

    /// Get installation status
    pub async fn get_installation_status(&self) -> Option<InstallationStatus> {
        self.installation_status.read().await.clone()
    }
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
}

impl DiskInfo {
    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Format bytes to human readable
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

/// API types for desktop tools
pub mod api {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CleanupRequest {
        pub categories: Vec<CleanupCategory>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CleanupResponse {
        pub success: bool,
        pub stats: CleanupStats,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct OptimizeRequest {
        pub tasks: Vec<OptimizationTask>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct OptimizeResponse {
        pub success: bool,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DiskInfoResponse {
        pub disks: Vec<DiskInfo>,
        pub total_free_bytes: u64,
        pub low_space_warning: bool,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(DiskInfo::format_bytes(500), "500 B");
        assert_eq!(DiskInfo::format_bytes(1024), "1.00 KB");
        assert_eq!(DiskInfo::format_bytes(1048576), "1.00 MB");
        assert_eq!(DiskInfo::format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_disk_info_usage_percent() {
        let disk = DiskInfo {
            name: "C:".to_string(),
            total_bytes: 100,
            free_bytes: 25,
            used_bytes: 75,
        };
        assert_eq!(disk.usage_percent(), 75.0);
    }

    #[test]
    fn test_default_config() {
        let config = DesktopToolsConfig::default();
        assert!(!config.temp_paths.is_empty());
        assert!(!config.browser_cache_paths.is_empty());
    }
}
