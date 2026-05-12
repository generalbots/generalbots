use crate::cache::DownloadCache;
use crate::component::{ComponentConfig, InstallResult};
use crate::installer::PackageManager;
use crate::InstallMode;
use crate::OsType;
use botlib::security::SafeCommand;
use anyhow::{Context, Result};
use log::{info, trace, warn};
use std::collections::HashMap;

fn safe_lxc(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("lxc")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

fn safe_lxd(args: &[&str]) -> Option<std::process::Output> {
    let cmd_res = SafeCommand::new("lxd").and_then(|c| c.args(args));
    cmd_res.ok().and_then(|cmd| cmd.execute().ok())
}

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
        crate::facade_download::create_directories(&self.base_path, &component.name)?;
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
        crate::facade_download::run_commands(
            pre_cmds, "local", &component.name, &self.base_path, "",
        )?;
        crate::facade_download::install_system_packages(component, &self.os_type)?;
        if let Some(url) = &component.download_url {
            let url = url.clone();
            let name = component.name.clone();
            let binary_name = component.binary_name.clone();
            crate::facade_download::download_and_install(
                &self.base_path, &url, &name, binary_name.as_deref(),
            )
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
                crate::facade_download::download_file(
                    url,
                    download_target.to_str().unwrap_or_default(),
                )
                .await?;
                if cache.is_some() && download_target != output_path {
                    std::fs::copy(&download_target, &output_path)?;
                    info!("Copied cached file to: {}", output_path.display());
                }
            }
        }
        crate::facade_download::run_commands(
            post_cmds, "local", &component.name, &self.base_path, "",
        )?;
        Ok(())
    }

    pub fn install_container_only(&self, component_name: &str) -> Result<InstallResult> {
        let container_name = format!("{}-{}", self.tenant, component_name);
        let _ = safe_lxd(&["init", "--auto"]);
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
            let output = safe_lxc(&[
                "launch", image, &container_name, "-c", "security.privileged=true",
            ]);
            let output = match output {
                Some(o) => o,
                None => continue,
            };
            if output.status.success() {
                info!("Successfully created container with image: {}", image);
                success = true;
                break;
            }
            last_error = String::from_utf8_lossy(&output.stderr).to_string();
            warn!("Failed to create container with {}: {}", image, last_error);
            let _ = safe_lxc(&["delete", &container_name, "--force"]);
        }
        if !success {
            return Err(anyhow::anyhow!(
                "LXC container creation failed with all images. Last error: {}",
                last_error
            ));
        }
        std::thread::sleep(std::time::Duration::from_secs(15));
        let container_ip = crate::facade_container::get_container_ip(&container_name)?;
        info!("Container '{}' created successfully at IP: {}", container_name, container_ip);
        Ok(InstallResult {
            component: component_name.to_string(),
            container_name: container_name.clone(),
            container_ip: container_ip.clone(),
            ports: vec![],
            env_vars: HashMap::new(),
            connection_info: format!(
                "Container '{}' created successfully at IP: {}\nRun without --container-only to complete installation.",
                container_name, container_ip
            ),
        })
    }

    pub fn install_container(&self, component: &ComponentConfig) -> Result<InstallResult> {
        crate::facade_container::install_container(
            &self.tenant, component, &self.os_type, &self.base_path,
        )
    }

    pub fn generate_env_from_vault(container_name: &str) -> Result<String> {
        crate::facade_container::generate_env_from_vault(container_name)
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
        let _ = safe_lxc(&["stop", &container_name]);
        let output = safe_lxc(&["delete", &container_name]);
        if let Some(o) = output {
            if !o.status.success() {
                warn!(
                    "Container deletion had issues: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            }
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
                let output = match safe_lxc(&["list", &container_name, "--format=json"]) {
                    Some(o) => o,
                    None => {
                        log::warn!("Failed to check container status");
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
}
