use crate::package_manager::component::ComponentConfig;
use crate::package_manager::installer::PackageManager;
use crate::package_manager::InstallMode;
use crate::package_manager::OsType;
use crate::shared::utils::{self, get_database_url_sync, parse_database_url};
use anyhow::{Context, Result};
use log::{error, trace, warn};
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
impl PackageManager {
    pub async fn install(&self, component_name: &str) -> Result<()> {
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
        match self.mode {
            InstallMode::Local => self.install_local(component).await?,
            InstallMode::Container => self.install_container(component)?,
        }
        trace!(
            "Component '{}' installation completed successfully",
            component_name
        );
        Ok(())
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
            for url in &component.data_download_list {
                let filename = url.split('/').last().unwrap_or("download.tmp");
                let output_path = self
                    .base_path
                    .join("data")
                    .join(&component.name)
                    .join(filename);
                utils::download_file(url, output_path.to_str().unwrap()).await?;
            }
        }
        self.run_commands(post_cmds, "local", &component.name)?;
        Ok(())
    }
    pub fn install_container(&self, component: &ComponentConfig) -> Result<()> {
        let container_name = format!("{}-{}", self.tenant, component.name);
        let output = Command::new("lxc")
            .args(&[
                "launch",
                "images:debian/12",
                &container_name,
                "-c",
                "security.privileged=true",
            ])
            .output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "LXC container creation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        std::thread::sleep(std::time::Duration::from_secs(15));
        self.exec_in_container(&container_name, "mkdir -p /opt/gbo/{bin,data,conf,logs}")?;
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
            self.exec_in_container(
                &container_name,
                &format!("apt-get install -y {}", pkg_list),
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
        trace!(
            "Container installation of '{}' completed in {}",
            component.name,
            container_name
        );
        Ok(())
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
        let _ = Command::new("lxc")
            .args(&["stop", &container_name])
            .output();
        let output = Command::new("lxc")
            .args(&["delete", &container_name])
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
                let output = Command::new("lxc")
                    .args(&["list", &container_name, "--format=json"])
                    .output()
                    .unwrap();
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
                .context(format!("Failed to create directory: {:?}", path))?;
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
                let output = Command::new("apt-get").args(&["update"]).output()?;
                if !output.status.success() {
                    warn!("apt-get update had issues");
                }
                let output = Command::new("apt-get")
                    .args(&["install", "-y"])
                    .args(packages)
                    .output()?;
                if !output.status.success() {
                    warn!("Some packages may have failed to install");
                }
            }
            OsType::MacOS => {
                let output = Command::new("brew")
                    .args(&["install"])
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
        let filename = url.split('/').last().unwrap_or("download.tmp");
        let temp_file = if filename.starts_with('/') {
            PathBuf::from(filename)
        } else {
            bin_path.join(filename)
        };
        self.download_with_reqwest(url, &temp_file, component)
            .await?;
        self.handle_downloaded_file(&temp_file, &bin_path, binary_name)?;
        Ok(())
    }
    pub async fn download_with_reqwest(
        &self,
        url: &str,
        temp_file: &PathBuf,
        component: &str,
    ) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
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
            match self.attempt_reqwest_download(&client, url, temp_file).await {
                Ok(_size) => {
                    if attempt > 0 {
                        trace!("Download succeeded on retry attempt {}", attempt);
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    let _ = std::fs::remove_file(temp_file);
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to download {} after {} attempts. Last error: {}",
            component,
            MAX_RETRIES + 1,
            last_error.unwrap()
        ))
    }
    pub async fn attempt_reqwest_download(
        &self,
        _client: &Client,
        url: &str,
        temp_file: &PathBuf,
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
        temp_file: &PathBuf,
        bin_path: &PathBuf,
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
                    let final_path = bin_path.join(temp_file.file_name().unwrap());
                    std::fs::rename(temp_file, &final_path)?;
                    self.make_executable(&final_path)?;
                }
            }
        }
        Ok(())
    }
    pub fn extract_tar_gz(&self, temp_file: &PathBuf, bin_path: &PathBuf) -> Result<()> {
        let output = Command::new("tar")
            .current_dir(bin_path)
            .args(&["-xzf", temp_file.to_str().unwrap(), "--strip-components=1"])
            .output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "tar extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        std::fs::remove_file(temp_file)?;
        Ok(())
    }
    pub fn extract_zip(&self, temp_file: &PathBuf, bin_path: &PathBuf) -> Result<()> {
        let output = Command::new("unzip")
            .current_dir(bin_path)
            .args(&["-o", "-q", temp_file.to_str().unwrap()])
            .output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "unzip extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        std::fs::remove_file(temp_file)?;
        Ok(())
    }
    pub fn install_binary(
        &self,
        temp_file: &PathBuf,
        bin_path: &PathBuf,
        name: &str,
    ) -> Result<()> {
        let final_path = bin_path.join(name);
        std::fs::rename(temp_file, &final_path)?;
        self.make_executable(&final_path)?;
        Ok(())
    }
    pub fn make_executable(&self, path: &PathBuf) -> Result<()> {
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
            self.base_path.join("conf").join(component)
        } else {
            PathBuf::from("/opt/gbo/conf")
        };
        let logs_path = if target == "local" {
            self.base_path.join("logs").join(component)
        } else {
            PathBuf::from("/opt/gbo/logs")
        };
        for cmd in commands {
            let rendered_cmd = cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());
            if target == "local" {
                trace!("Executing command: {}", rendered_cmd);
                let child = Command::new("bash")
                    .current_dir(&bin_path)
                    .args(&["-c", &rendered_cmd])
                    .spawn()
                    .with_context(|| {
                        format!("Failed to spawn command for component '{}'", component)
                    })?;
                let output = child.wait_with_output().with_context(|| {
                    format!(
                        "Failed while waiting for command to finish for component '{}'",
                        component
                    )
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
        let output = Command::new("lxc")
            .args(&["exec", container, "--", "bash", "-c", command])
            .output()?;
        if !output.status.success() {
            warn!(
                "Container command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
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
        if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
            self.exec_in_container(container, "tar -xzf /tmp/download.tmp -C /opt/gbo/bin")?;
        } else if url.ends_with(".zip") {
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
                .args(&["config", "device", "remove", container, &device_name])
                .output();
            let output = Command::new("lxc")
                .args(&[
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
        let database_url = get_database_url_sync()
            .context("Failed to get DATABASE_URL from Vault. Ensure Vault is configured.")?;
        let (_db_username, db_password, _db_server, _db_port, _db_name) =
            parse_database_url(&database_url);

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
            env_section.push_str(&format!("Environment={}={}\n", key, rendered_value));
        }
        let service_content = format!(
            "[Unit]\nDescription={} Service\nAfter=network.target\n\n[Service]\nType=simple\n{}ExecStart={}\nWorkingDirectory=/opt/gbo/data\nRestart=always\nRestartSec=10\nUser=root\n\n[Install]\nWantedBy=multi-user.target\n",
            component, env_section, rendered_cmd
        );
        let service_file = format!("/tmp/{}.service", component);
        std::fs::write(&service_file, &service_content)?;
        let output = Command::new("lxc")
            .args(&[
                "file",
                "push",
                &service_file,
                &format!("{}/etc/systemd/system/{}.service", container, component),
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
                .args(&["config", "device", "remove", container, &device_name])
                .output();
            let output = Command::new("lxc")
                .args(&[
                    "config",
                    "device",
                    "add",
                    container,
                    &device_name,
                    "proxy",
                    &format!("listen=tcp:0.0.0.0:{}", port),
                    &format!("connect=tcp:127.0.0.1:{}", port),
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
