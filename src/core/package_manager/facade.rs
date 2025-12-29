use crate::package_manager::cache::{CacheResult, DownloadCache};
use crate::package_manager::component::{ComponentConfig, InstallResult};
use crate::package_manager::installer::PackageManager;
use crate::package_manager::InstallMode;
use crate::package_manager::OsType;
use crate::shared::utils::{self, get_database_url_sync, parse_database_url};
use anyhow::{Context, Result};
use log::{error, info, trace, warn};
use reqwest::Client;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;
use std::process::Command;
impl PackageManager {
    pub async fn install(&self, component_name: &str) -> Result<Option<InstallResult>> {
        let component = self
            .components
            .get(component_name)
            .context(format!("Component '{}' not found", component_name))?;
        trace!(
            "Starting installation of component '{}' in {:?} mode",
            component_name,
            self.mode
        );
        for dep in &component.dependencies {
            if !self.is_installed(dep) {
                warn!("Installing missing dependency: {}", dep);
                Box::pin(self.install(dep)).await?;
            }
        }
        let result = match self.mode {
            InstallMode::Local => {
                self.install_local(component).await?;
                None
            }
            InstallMode::Container => Some(self.install_container(component)?),
        };
        trace!(
            "Component '{}' installation completed successfully",
            component_name
        );
        Ok(result)
    }
    pub async fn install_local(&self, component: &ComponentConfig) -> Result<()> {
        trace!(
            "Installing component '{}' locally to {}",
            component.name,
            self.base_path.display()
        );
        self.create_directories(&component.name)?;
        let (pre_cmds, post_cmds) = match self.os_type {
            OsType::Linux => (
                &component.pre_install_cmds_linux,
                &component.post_install_cmds_linux,
            ),
            OsType::MacOS => (
                &component.pre_install_cmds_macos,
                &component.post_install_cmds_macos,
            ),
            OsType::Windows => (
                &component.pre_install_cmds_windows,
                &component.post_install_cmds_windows,
            ),
        };
        self.run_commands(pre_cmds, "local", &component.name)?;
        self.install_system_packages(component)?;
        if let Some(url) = &component.download_url {
            let url = url.clone();
            let name = component.name.clone();
            let binary_name = component.binary_name.clone();
            self.download_and_install(&url, &name, binary_name.as_deref())
                .await?;
        }
        if !component.data_download_list.is_empty() {
            let cache_base = self.base_path.parent().unwrap_or(&self.base_path);
            let cache = DownloadCache::new(cache_base).ok();

            for url in &component.data_download_list {
                let filename = DownloadCache::extract_filename(url);
                let output_path = self
                    .base_path
                    .join("data")
                    .join(&component.name)
                    .join(&filename);

                if output_path.exists() {
                    info!("Data file already exists: {}", output_path.display());
                    continue;
                }

                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                if let Some(ref c) = cache {
                    if let Some(cached_path) = c.get_cached_path(&filename) {
                        info!("Using cached data file: {}", cached_path.display());
                        std::fs::copy(&cached_path, &output_path)?;
                        continue;
                    }
                }

                let download_target = if let Some(ref c) = cache {
                    c.get_cache_path(&filename)
                } else {
                    output_path.clone()
                };

                info!("Downloading data file: {}", url);
                println!("Downloading {}", url);
                utils::download_file(url, download_target.to_str().unwrap_or_default()).await?;

                if cache.is_some() && download_target != output_path {
                    std::fs::copy(&download_target, &output_path)?;
                    info!("Copied cached file to: {}", output_path.display());
                }
            }
        }
        self.run_commands(post_cmds, "local", &component.name)?;
        Ok(())
    }
    pub fn install_container(&self, component: &ComponentConfig) -> Result<InstallResult> {
        let container_name = format!("{}-{}", self.tenant, component.name);

        let _ = Command::new("lxd").args(["init", "--auto"]).output();

        let images = [
            "ubuntu:24.04",
            "ubuntu:22.04",
            "images:debian/12",
            "images:debian/11",
        ];

        let mut last_error = String::new();
        let mut success = false;

        for image in &images {
            info!("Attempting to create container with image: {}", image);
            let output = Command::new("lxc")
                .args([
                    "launch",
                    image,
                    &container_name,
                    "-c",
                    "security.privileged=true",
                ])
                .output()?;

            if output.status.success() {
                info!("Successfully created container with image: {}", image);
                success = true;
                break;
            }
            last_error = String::from_utf8_lossy(&output.stderr).to_string();
            warn!("Failed to create container with {}: {}", image, last_error);

            let _ = Command::new("lxc")
                .args(["delete", &container_name, "--force"])
                .output();
        }

        if !success {
            return Err(anyhow::anyhow!(
                "LXC container creation failed with all images. Last error: {}",
                last_error
            ));
        }
        std::thread::sleep(std::time::Duration::from_secs(15));
        self.exec_in_container(&container_name, "mkdir -p /opt/gbo/{bin,data,conf,logs}")?;

        self.exec_in_container(
            &container_name,
            "echo 'nameserver 8.8.8.8' > /etc/resolv.conf",
        )?;
        self.exec_in_container(
            &container_name,
            "echo 'nameserver 8.8.4.4' >> /etc/resolv.conf",
        )?;

        self.exec_in_container(&container_name, "apt-get update -qq")?;
        self.exec_in_container(
            &container_name,
            "DEBIAN_FRONTEND=noninteractive apt-get install -y -qq wget unzip curl ca-certificates",
        )?;
        let (pre_cmds, post_cmds) = match self.os_type {
            OsType::Linux => (
                &component.pre_install_cmds_linux,
                &component.post_install_cmds_linux,
            ),
            OsType::MacOS => (
                &component.pre_install_cmds_macos,
                &component.post_install_cmds_macos,
            ),
            OsType::Windows => (
                &component.pre_install_cmds_windows,
                &component.post_install_cmds_windows,
            ),
        };
        self.run_commands(pre_cmds, &container_name, &component.name)?;
        let packages = match self.os_type {
            OsType::Linux => &component.linux_packages,
            OsType::MacOS => &component.macos_packages,
            OsType::Windows => &component.windows_packages,
        };
        if !packages.is_empty() {
            let pkg_list = packages.join(" ");
            self.exec_in_container(&container_name, "apt-get update -qq")?;
            self.exec_in_container(
                &container_name,
                &format!(
                    "DEBIAN_FRONTEND=noninteractive apt-get install -y -qq {}",
                    pkg_list
                ),
            )?;
        }
        if let Some(url) = &component.download_url {
            self.download_in_container(
                &container_name,
                url,
                &component.name,
                component.binary_name.as_deref(),
            )?;
        }
        self.run_commands(post_cmds, &container_name, &component.name)?;
        self.exec_in_container(
            &container_name,
            "useradd --system --no-create-home --shell /bin/false gbuser",
        )?;
        self.mount_container_directories(&container_name, &component.name)?;
        if !component.exec_cmd.is_empty() {
            self.create_container_service(
                &container_name,
                &component.name,
                &component.exec_cmd,
                &component.env_vars,
            )?;
        }
        self.setup_port_forwarding(&container_name, &component.ports)?;

        let container_ip = Self::get_container_ip(&container_name)?;

        if component.name == "vault" {
            Self::initialize_vault(&container_name, &container_ip)?;
        }

        let (connection_info, env_vars) =
            self.generate_connection_info(&component.name, &container_ip, &component.ports);

        trace!(
            "Container installation of '{}' completed in {}",
            component.name,
            container_name
        );

        Ok(InstallResult {
            component: component.name.clone(),
            container_name,
            container_ip,
            ports: component.ports.clone(),
            env_vars,
            connection_info,
        })
    }

    fn get_container_ip(container_name: &str) -> Result<String> {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let output = Command::new("lxc")
            .args(["list", container_name, "-c", "4", "--format", "csv"])
            .output()?;

        if output.status.success() {
            let ip_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !ip_output.is_empty() {
                let ip = ip_output.split([' ', '(']).next().unwrap_or("").trim();
                if !ip.is_empty() && ip.contains('.') {
                    return Ok(ip.to_string());
                }
            }
        }

        let output = Command::new("lxc")
            .args(["exec", container_name, "--", "hostname", "-I"])
            .output()?;

        if output.status.success() {
            let ip_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(ip) = ip_output.split_whitespace().next() {
                if ip.contains('.') {
                    return Ok(ip.to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    fn initialize_vault(container_name: &str, ip: &str) -> Result<()> {
        info!("Initializing Vault...");

        std::thread::sleep(std::time::Duration::from_secs(5));

        let output = Command::new("lxc")
            .args([
                "exec",
                container_name,
                "--",
                "bash",
                "-c",
                "VAULT_ADDR=http://127.0.0.1:8200 /opt/gbo/bin/vault operator init -key-shares=5 -key-threshold=3 -format=json",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            if stderr.contains("already initialized") {
                warn!("Vault already initialized, skipping file generation");
                return Ok(());
            }
            return Err(anyhow::anyhow!("Failed to initialize Vault: {}", stderr));
        }

        let init_output = String::from_utf8_lossy(&output.stdout);

        let init_json: serde_json::Value =
            serde_json::from_str(&init_output).context("Failed to parse Vault init output")?;

        let unseal_keys = init_json["unseal_keys_b64"]
            .as_array()
            .context("No unseal keys in output")?;
        let root_token = init_json["root_token"]
            .as_str()
            .context("No root token in output")?;

        let unseal_keys_file = PathBuf::from("vault-unseal-keys");
        let mut unseal_content = String::new();
        for (i, key) in unseal_keys.iter().enumerate() {
            if i < 3 {
                let _ = writeln!(
                    unseal_content,
                    "VAULT_UNSEAL_KEY_{}={}",
                    i + 1,
                    key.as_str().unwrap_or("")
                );
            }
        }
        std::fs::write(&unseal_keys_file, &unseal_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&unseal_keys_file, std::fs::Permissions::from_mode(0o600))?;
        }

        info!("Created {}", unseal_keys_file.display());

        let env_file = PathBuf::from(".env");
        let env_content = format!(
            "\n# Vault Configuration (auto-generated)\nVAULT_ADDR=http://{}:8200\nVAULT_TOKEN={}\nVAULT_UNSEAL_KEYS_FILE=vault-unseal-keys\n",
            ip, root_token
        );

        if env_file.exists() {
            let existing = std::fs::read_to_string(&env_file)?;

            if existing.contains("VAULT_ADDR=") {
                warn!(".env already contains VAULT_ADDR, not overwriting");
            } else {
                let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
                use std::io::Write;
                file.write_all(env_content.as_bytes())?;
                info!("Appended Vault config to .env");
            }
        } else {
            std::fs::write(&env_file, env_content.trim_start())?;
            info!("Created .env with Vault config");
        }

        for i in 0..3 {
            if let Some(key) = unseal_keys.get(i) {
                let key_str = key.as_str().unwrap_or("");
                let unseal_cmd = format!(
                    "VAULT_ADDR=http://127.0.0.1:8200 /opt/gbo/bin/vault operator unseal {}",
                    key_str
                );
                let unseal_output = Command::new("lxc")
                    .args(["exec", container_name, "--", "bash", "-c", &unseal_cmd])
                    .output()?;

                if !unseal_output.status.success() {
                    warn!("Unseal step {} may have failed", i + 1);
                }
            }
        }

        info!("Vault initialized and unsealed successfully");
        Ok(())
    }

    fn generate_connection_info(
        &self,
        component: &str,
        ip: &str,
        ports: &[u16],
    ) -> (String, HashMap<String, String>) {
        let env_vars = HashMap::new();
        let connection_info = match component {
            "vault" => {
                format!(
                    r"Vault Server:
  URL: http://{}:8200
  UI:  http://{}:8200/ui

✓ Vault initialized and unsealed automatically
✓ Created .env with VAULT_ADDR, VAULT_TOKEN
✓ Created vault-unseal-keys (chmod 600)

Files created:
  .env                 - Vault connection config
  vault-unseal-keys    - Unseal keys for auto-unseal

On server restart, run:
  botserver vault unseal

Or manually:
  lxc exec {}-vault -- /opt/gbo/bin/vault operator unseal <key>

For other auto-unseal options (TPM, HSM, Transit), see:
  https://generalbots.github.io/botbook/chapter-08/secrets-management.html",
                    ip, ip, self.tenant
                )
            }
            "vector_db" => {
                format!(
                    r"Qdrant Vector Database:
  REST API: http://{}:6333
  gRPC:     {}:6334
  Dashboard: http://{}:6333/dashboard

Store credentials in Vault:
  botserver vault put gbo/vectordb host={} port=6333",
                    ip, ip, ip, ip
                )
            }
            "tables" => {
                format!(
                    r"PostgreSQL Database:
  Host: {}
  Port: 5432
  Database: botserver
  User: gbuser

Store credentials in Vault:
  botserver vault put gbo/tables host={} port=5432 database=botserver username=gbuser password=<your-password>",
                    ip, ip
                )
            }
            "drive" => {
                format!(
                    r"MinIO Object Storage:
  API:     http://{}:9000
  Console: http://{}:9001

Store credentials in Vault:
  botserver vault put gbo/drive server={} port=9000 accesskey=minioadmin secret=<your-secret>",
                    ip, ip, ip
                )
            }
            "cache" => {
                format!(
                    r"Redis/Valkey Cache:
  Host: {}
  Port: 6379

Store credentials in Vault:
  botserver vault put gbo/cache host={} port=6379 password=<your-password>",
                    ip, ip
                )
            }
            "email" => {
                format!(
                    r"Email Server (Stalwart):
  SMTP: {}:25
  IMAP: {}:143
  Web:  http://{}:8080

Store credentials in Vault:
  botserver vault put gbo/email server={} port=25 username=admin password=<your-password>",
                    ip, ip, ip, ip
                )
            }
            "directory" => {
                format!(
                    r"Zitadel Identity Provider:
  URL: http://{}:8080
  Console: http://{}:8080/ui/console

Store credentials in Vault:
  botserver vault put gbo/directory url=http://{}:8080 client_id=<client-id> client_secret=<client-secret>",
                    ip, ip, ip
                )
            }
            "llm" => {
                format!(
                    r"LLM Server (llama.cpp):
  API: http://{}:8081

Test:
  curl http://{}:8081/v1/models

Store credentials in Vault:
  botserver vault put gbo/llm url=http://{}:8081 local=true",
                    ip, ip, ip
                )
            }
            "meeting" => {
                format!(
                    r"LiveKit Meeting Server:
  WebSocket: ws://{}:7880
  API: http://{}:7880

Store credentials in Vault:
  botserver vault put gbo/meet url=ws://{}:7880 api_key=<api-key> api_secret=<api-secret>",
                    ip, ip, ip
                )
            }
            "proxy" => {
                format!(
                    r"Caddy Reverse Proxy:
  HTTP:  http://{}:80
  HTTPS: https://{}:443
  Admin: http://{}:2019",
                    ip, ip, ip
                )
            }
            "timeseries_db" => {
                format!(
                    r"InfluxDB Time Series Database:
  API: http://{}:8086

Store credentials in Vault:
  botserver vault put gbo/observability url=http://{}:8086 token=<influx-token> org=pragmatismo bucket=metrics",
                    ip, ip
                )
            }
            "observability" => {
                format!(
                    r"Vector Log Aggregation:
  API: http://{}:8686

Store credentials in Vault:
  botserver vault put gbo/observability vector_url=http://{}:8686",
                    ip, ip
                )
            }
            "alm" => {
                format!(
                    r"Forgejo Git Server:
  Web: http://{}:3000
  SSH: {}:22

Store credentials in Vault:
  botserver vault put gbo/alm url=http://{}:3000 token=<api-token>",
                    ip, ip, ip
                )
            }
            _ => {
                let ports_str = ports
                    .iter()
                    .map(|p| format!("  - {}:{}", ip, p))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    r"Component: {}
  Container: {}-{}
  IP: {}
  Ports:
{}",
                    component, self.tenant, component, ip, ports_str
                )
            }
        };

        (connection_info, env_vars)
    }

    pub fn remove(&self, component_name: &str) -> Result<()> {
        let component = self
            .components
            .get(component_name)
            .context(format!("Component '{}' not found", component_name))?;
        match self.mode {
            InstallMode::Local => self.remove_local(component)?,
            InstallMode::Container => self.remove_container(component)?,
        }
        Ok(())
    }
    pub fn remove_local(&self, component: &ComponentConfig) -> Result<()> {
        let bin_path = self.base_path.join("bin").join(&component.name);
        let _ = std::fs::remove_dir_all(bin_path);
        Ok(())
    }
    pub fn remove_container(&self, component: &ComponentConfig) -> Result<()> {
        let container_name = format!("{}-{}", self.tenant, component.name);
        let _ = Command::new("lxc").args(["stop", &container_name]).output();
        let output = Command::new("lxc")
            .args(["delete", &container_name])
            .output()?;
        if !output.status.success() {
            warn!(
                "Container deletion had issues: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }
    pub fn list(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
    pub fn is_installed(&self, component_name: &str) -> bool {
        match self.mode {
            InstallMode::Local => {
                let bin_path = self.base_path.join("bin").join(component_name);
                bin_path.exists()
            }
            InstallMode::Container => {
                let container_name = format!("{}-{}", self.tenant, component_name);
                let output = match Command::new("lxc")
                    .args(["list", &container_name, "--format=json"])
                    .output()
                {
                    Ok(o) => o,
                    Err(e) => {
                        log::warn!("Failed to check container status: {}", e);
                        return false;
                    }
                };
                if !output.status.success() {
                    return false;
                }
                let output_str = String::from_utf8_lossy(&output.stdout);
                !output_str.contains("\"name\":\"") || output_str.contains("\"status\":\"Stopped\"")
            }
        }
    }
    pub fn create_directories(&self, component: &str) -> Result<()> {
        let dirs = ["bin", "data", "conf", "logs"];
        for dir in &dirs {
            let path = self.base_path.join(dir).join(component);
            std::fs::create_dir_all(&path)
                .context(format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }
    pub fn install_system_packages(&self, component: &ComponentConfig) -> Result<()> {
        let packages = match self.os_type {
            OsType::Linux => &component.linux_packages,
            OsType::MacOS => &component.macos_packages,
            OsType::Windows => &component.windows_packages,
        };
        if packages.is_empty() {
            return Ok(());
        }
        trace!(
            "Installing {} system packages for component '{}'",
            packages.len(),
            component.name
        );
        match self.os_type {
            OsType::Linux => {
                let output = Command::new("apt-get").args(["update"]).output()?;
                if !output.status.success() {
                    warn!("apt-get update had issues");
                }
                let output = Command::new("apt-get")
                    .args(["install", "-y"])
                    .args(packages)
                    .output()?;
                if !output.status.success() {
                    warn!("Some packages may have failed to install");
                }
            }
            OsType::MacOS => {
                let output = Command::new("brew")
                    .args(["install"])
                    .args(packages)
                    .output()?;
                if !output.status.success() {
                    warn!("Homebrew installation had warnings");
                }
            }
            OsType::Windows => {
                warn!("Windows package installation not implemented");
            }
        }
        Ok(())
    }
    pub async fn download_and_install(
        &self,
        url: &str,
        component: &str,
        binary_name: Option<&str>,
    ) -> Result<()> {
        let bin_path = self.base_path.join("bin").join(component);
        std::fs::create_dir_all(&bin_path)?;

        let cache_base = self.base_path.parent().unwrap_or(&self.base_path);
        let cache = match DownloadCache::new(cache_base) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to initialize download cache: {}", e);
                match DownloadCache::new(&self.base_path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to create fallback cache: {}", e);
                        return Err(anyhow::anyhow!("Failed to create download cache"));
                    }
                }
            }
        };

        let cache_result = cache.resolve_component_url(component, url);

        let source_file = match cache_result {
            CacheResult::Cached(cached_path) => {
                info!(
                    "Using cached file for {}: {}",
                    component,
                    cached_path.display()
                );
                cached_path
            }
            CacheResult::Download {
                url: download_url,
                cache_path,
            } => {
                info!("Downloading {} from {}", component, download_url);
                println!("Downloading {}", download_url);

                self.download_with_reqwest(&download_url, &cache_path, component)
                    .await?;

                info!("Cached {} to {}", component, cache_path.display());
                cache_path
            }
        };

        self.handle_downloaded_file(&source_file, &bin_path, binary_name)?;
        Ok(())
    }
    pub async fn download_with_reqwest(
        &self,
        url: &str,
        target_file: &std::path::Path,
        component: &str,
    ) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

        if let Some(parent) = target_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("botserver-package-manager/1.0")
            .build()?;
        let mut last_error = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                trace!(
                    "Retry attempt {}/{} for {}",
                    attempt,
                    MAX_RETRIES,
                    component
                );
                std::thread::sleep(RETRY_DELAY * attempt);
            }
            match self
                .attempt_reqwest_download(&client, url, target_file)
                .await
            {
                Ok(_size) => {
                    if attempt > 0 {
                        trace!("Download succeeded on retry attempt {}", attempt);
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    let _ = std::fs::remove_file(target_file);
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to download {} after {} attempts. Last error: {}",
            component,
            MAX_RETRIES + 1,
            last_error.unwrap_or_else(|| anyhow::anyhow!("unknown error"))
        ))
    }
    pub async fn attempt_reqwest_download(
        &self,
        _client: &Client,
        url: &str,
        temp_file: &std::path::Path,
    ) -> Result<u64> {
        let output_path = temp_file.to_str().context("Invalid temp file path")?;
        utils::download_file(url, output_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download file using shared utility: {}", e))?;
        let metadata = std::fs::metadata(temp_file).context("Failed to get file metadata")?;
        let size = metadata.len();
        Ok(size)
    }
    pub fn handle_downloaded_file(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        binary_name: Option<&str>,
    ) -> Result<()> {
        let metadata = std::fs::metadata(temp_file)?;
        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("Downloaded file is empty"));
        }
        let file_extension = temp_file
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        match file_extension {
            "gz" | "tgz" => {
                self.extract_tar_gz(temp_file, bin_path)?;
            }
            "zip" => {
                self.extract_zip(temp_file, bin_path)?;
            }
            _ => {
                if let Some(name) = binary_name {
                    self.install_binary(temp_file, bin_path, name)?;
                } else {
                    let final_path = bin_path.join(temp_file.file_name().unwrap_or_default());

                    if temp_file.to_string_lossy().contains("botserver-installers") {
                        std::fs::copy(temp_file, &final_path)?;
                    } else {
                        std::fs::rename(temp_file, &final_path)?;
                    }
                    self.make_executable(&final_path)?;
                }
            }
        }
        Ok(())
    }
    pub fn extract_tar_gz(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
    ) -> Result<()> {
        // Check if tarball has a top-level directory or files at root
        let list_output = Command::new("tar")
            .args(["-tzf", temp_file.to_str().unwrap_or_default()])
            .output()?;

        let has_subdir = if list_output.status.success() {
            let contents = String::from_utf8_lossy(&list_output.stdout);
            // If first entry contains '/', there's a subdirectory structure
            contents.lines().next().map(|l| l.contains('/')).unwrap_or(false)
        } else {
            false
        };

        let mut args = vec!["-xzf", temp_file.to_str().unwrap_or_default()];
        if has_subdir {
            args.push("--strip-components=1");
        }

        let output = Command::new("tar")
            .current_dir(bin_path)
            .args(&args)
            .output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "tar extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        if !temp_file.to_string_lossy().contains("botserver-installers") {
            std::fs::remove_file(temp_file)?;
        }
        Ok(())
    }
    pub fn extract_zip(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
    ) -> Result<()> {
        let output = Command::new("unzip")
            .current_dir(bin_path)
            .args(["-o", "-q", temp_file.to_str().unwrap_or_default()])
            .output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "unzip extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = std::fs::read_dir(bin_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let mut perms = metadata.permissions();

                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if ext.is_empty() || ext == "sh" || ext == "bash" {
                                perms.set_mode(0o755);
                                let _ = std::fs::set_permissions(&path, perms);
                                trace!("Made executable: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        if !temp_file.to_string_lossy().contains("botserver-installers") {
            std::fs::remove_file(temp_file)?;
        }
        Ok(())
    }
    pub fn install_binary(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        name: &str,
    ) -> Result<()> {
        let final_path = bin_path.join(name);

        if temp_file.to_string_lossy().contains("botserver-installers") {
            std::fs::copy(temp_file, &final_path)?;
        } else {
            std::fs::rename(temp_file, &final_path)?;
        }
        self.make_executable(&final_path)?;
        Ok(())
    }
    pub fn make_executable(&self, path: &std::path::Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(path, perms)?;
        }
        Ok(())
    }
    pub fn run_commands(&self, commands: &[String], target: &str, component: &str) -> Result<()> {
        let bin_path = if target == "local" {
            self.base_path.join("bin").join(component)
        } else {
            PathBuf::from("/opt/gbo/bin")
        };
        let data_path = if target == "local" {
            self.base_path.join("data").join(component)
        } else {
            PathBuf::from("/opt/gbo/data")
        };

        let conf_path = if target == "local" {
            self.base_path.join("conf")
        } else {
            PathBuf::from("/opt/gbo/conf")
        };
        let logs_path = if target == "local" {
            self.base_path.join("logs").join(component)
        } else {
            PathBuf::from("/opt/gbo/logs")
        };

        let db_password = match get_database_url_sync() {
            Ok(url) => {
                let (_, password, _, _, _) = parse_database_url(&url);
                password
            }
            Err(_) => {
                trace!("Vault not available for DB_PASSWORD, using empty string");
                String::new()
            }
        };

        for cmd in commands {
            let rendered_cmd = cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy())
                .replace("{{DB_PASSWORD}}", &db_password);
            if target == "local" {
                trace!("Executing command: {}", rendered_cmd);
                let output = Command::new("bash")
                    .current_dir(&bin_path)
                    .args(["-c", &rendered_cmd])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .with_context(|| {
                        format!("Failed to execute command for component '{}'", component)
                    })?;
                if !output.status.success() {
                    error!(
                        "Command had non-zero exit: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            } else {
                self.exec_in_container(target, &rendered_cmd)?;
            }
        }
        Ok(())
    }
    pub fn exec_in_container(&self, container: &str, command: &str) -> Result<()> {
        info!("Executing in container {}: {}", container, command);
        let output = Command::new("lxc")
            .args(["exec", container, "--", "bash", "-c", command])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!(
                "Container command failed.\nCommand: {}\nStderr: {}\nStdout: {}",
                command, stderr, stdout
            );
            return Err(anyhow::anyhow!(
                "Container command failed: {}",
                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    stderr.to_string()
                }
            ));
        }
        Ok(())
    }
    pub fn download_in_container(
        &self,
        container: &str,
        url: &str,
        _component: &str,
        binary_name: Option<&str>,
    ) -> Result<()> {
        let download_cmd = format!("wget -O /tmp/download.tmp {}", url);
        self.exec_in_container(container, &download_cmd)?;
        let path = std::path::Path::new(url);
        let is_tar_gz = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("tgz"))
            || (path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
                && path
                    .file_stem()
                    .and_then(|s| std::path::Path::new(s).extension())
                    .is_some_and(|e| e.eq_ignore_ascii_case("tar")));
        let is_zip = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));
        if is_tar_gz {
            self.exec_in_container(container, "tar -xzf /tmp/download.tmp -C /opt/gbo/bin")?;
        } else if is_zip {
            self.exec_in_container(container, "unzip -o /tmp/download.tmp -d /opt/gbo/bin")?;
        } else if let Some(name) = binary_name {
            let mv_cmd = format!(
                "mv /tmp/download.tmp /opt/gbo/bin/{} && chmod +x /opt/gbo/bin/{}",
                name, name
            );
            self.exec_in_container(container, &mv_cmd)?;
        }
        self.exec_in_container(container, "rm -f /tmp/download.tmp")?;
        Ok(())
    }
    pub fn mount_container_directories(&self, container: &str, component: &str) -> Result<()> {
        let host_base = format!("/opt/gbo/tenants/{}/{}", self.tenant, component);
        for dir in &["data", "conf", "logs"] {
            let host_path = format!("{}/{}", host_base, dir);
            std::fs::create_dir_all(&host_path)?;
            let device_name = format!("{}-{}", component, dir);
            let container_path = format!("/opt/gbo/{}", dir);
            let _ = Command::new("lxc")
                .args(["config", "device", "remove", container, &device_name])
                .output();
            let output = Command::new("lxc")
                .args([
                    "config",
                    "device",
                    "add",
                    container,
                    &device_name,
                    "disk",
                    &format!("source={}", host_path),
                    &format!("path={}", container_path),
                ])
                .output()?;
            if !output.status.success() {
                warn!("Failed to mount {} in container {}", dir, container);
            }
            trace!(
                "Mounted {} to {} in container {}",
                host_path,
                container_path,
                container
            );
        }
        Ok(())
    }
    pub fn create_container_service(
        &self,
        container: &str,
        component: &str,
        exec_cmd: &str,
        env_vars: &HashMap<String, String>,
    ) -> Result<()> {
        let db_password = match get_database_url_sync() {
            Ok(url) => {
                let (_, password, _, _, _) = parse_database_url(&url);
                password
            }
            Err(_) => {
                trace!(
                    "Vault not available for DB_PASSWORD in container service, using empty string"
                );
                String::new()
            }
        };

        let rendered_cmd = exec_cmd
            .replace("{{DB_PASSWORD}}", &db_password)
            .replace("{{BIN_PATH}}", "/opt/gbo/bin")
            .replace("{{DATA_PATH}}", "/opt/gbo/data")
            .replace("{{CONF_PATH}}", "/opt/gbo/conf")
            .replace("{{LOGS_PATH}}", "/opt/gbo/logs");
        let mut env_section = String::new();
        for (key, value) in env_vars {
            let rendered_value = value
                .replace("{{DATA_PATH}}", "/opt/gbo/data")
                .replace("{{BIN_PATH}}", "/opt/gbo/bin")
                .replace("{{CONF_PATH}}", "/opt/gbo/conf")
                .replace("{{LOGS_PATH}}", "/opt/gbo/logs");
            let _ = writeln!(env_section, "Environment={key}={rendered_value}");
        }
        let service_content = format!(
            "[Unit]\nDescription={} Service\nAfter=network.target\n\n[Service]\nType=simple\n{}ExecStart={}\nWorkingDirectory=/opt/gbo/data\nRestart=always\nRestartSec=10\nUser=root\n\n[Install]\nWantedBy=multi-user.target\n",
            component, env_section, rendered_cmd
        );
        let service_file = format!("/tmp/{}.service", component);
        std::fs::write(&service_file, &service_content)?;
        let output = Command::new("lxc")
            .args([
                "file",
                "push",
                &service_file,
                &format!("{container}/etc/systemd/system/{component}.service"),
            ])
            .output()?;
        if !output.status.success() {
            warn!("Failed to push service file to container");
        }
        self.exec_in_container(container, "systemctl daemon-reload")?;
        self.exec_in_container(container, &format!("systemctl enable {}", component))?;
        self.exec_in_container(container, &format!("systemctl start {}", component))?;
        std::fs::remove_file(&service_file)?;
        trace!(
            "Created and started service in container {}: {}",
            container,
            component
        );
        Ok(())
    }
    pub fn setup_port_forwarding(&self, container: &str, ports: &[u16]) -> Result<()> {
        for port in ports {
            let device_name = format!("port-{}", port);
            let _ = Command::new("lxc")
                .args(["config", "device", "remove", container, &device_name])
                .output();
            let output = Command::new("lxc")
                .args([
                    "config",
                    "device",
                    "add",
                    container,
                    &device_name,
                    "proxy",
                    &format!("listen=tcp:0.0.0.0:{port}"),
                    &format!("connect=tcp:127.0.0.1:{port}"),
                ])
                .output()?;
            if !output.status.success() {
                warn!("Failed to setup port forwarding for port {}", port);
            }
            trace!(
                "Port forwarding configured: {} -> container {}",
                port,
                container
            );
        }
        Ok(())
    }
}
