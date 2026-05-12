use crate::component::ComponentConfig;
use crate::os::detect_os;
use crate::{InstallMode, OsType};
use crate::installer_regs;
use crate::installer_regs2;
use crate::installer_vault;
use crate::installer_vault2;
use botlib::security::SafeCommand;
use anyhow::Result;
use log::{error, info, trace, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;



#[derive(Deserialize, Debug)]
struct ComponentEntry {
    url: String,
}

#[derive(Deserialize, Debug)]
struct ThirdPartyConfig {
    components: HashMap<String, ComponentEntry>,
}

static THIRDPARTY_CONFIG: std::sync::OnceLock<ThirdPartyConfig> = std::sync::OnceLock::new();

fn get_thirdparty_config() -> &'static ThirdPartyConfig {
    THIRDPARTY_CONFIG.get_or_init(|| {
        let toml_str = include_str!("../3rdparty.toml");
        match toml::from_str::<ThirdPartyConfig>(toml_str) {
            Ok(config) => config,
            Err(e) => {
                error!("CRITICAL: Failed to parse embedded 3rdparty.toml: {e}");
                ThirdPartyConfig {
                    components: HashMap::new(),
                }
            }
        }
    })
}

pub fn get_component_url(name: &str) -> Option<String> {
    get_thirdparty_config()
        .components
        .get(name)
        .map(|c| c.url.clone())
}

#[cfg(target_os = "windows")]
fn safe_nvcc_version() -> Option<std::process::Output> {
    SafeCommand::new("nvcc")
        .and_then(|c| c.arg("--version"))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub fn safe_sh_command(script: &str) -> Option<std::process::Output> {
    SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_shell_script_arg(script))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub fn safe_pgrep(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("pgrep")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

pub const LLAMA_CPP_VERSION: &str = "b7345";

pub fn get_llama_cpp_url() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        {
        let _ = std::path::Path::new("/usr/local/cuda").exists()
            || std::path::Path::new("/opt/cuda").exists()
            || std::env::var("CUDA_HOME").is_ok();

            if std::path::Path::new("/usr/share/vulkan").exists()
                || std::env::var("VULKAN_SDK").is_ok()
            {
                info!("Detected Vulkan - using Vulkan build");
                return get_component_url("llm_linux_vulkan");
            }

            info!("Using standard Ubuntu x64 build (CPU)");
            get_component_url("llm")
        }

        #[cfg(target_arch = "s390x")]
        {
            info!("Detected s390x architecture");
            return get_component_url("llm_linux_s390x");
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected ARM64 architecture on Linux");
            warn!("No pre-built llama.cpp for Linux ARM64 - LLM will not be available");
            return None;
        }
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected macOS ARM64 (Apple Silicon)");
            return get_component_url("llm_macos_arm64");
        }

        #[cfg(target_arch = "x86_64")]
        {
            info!("Detected macOS x64 (Intel)");
            return get_component_url("llm_macos_x64");
        }
    }

    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if std::env::var("CUDA_PATH").is_ok() {
                if let Some(output) = safe_nvcc_version() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    if version_str.contains("13.") {
                        info!("Detected CUDA 13.x on Windows");
                        return get_component_url("llm_win_cuda13");
                    } else if version_str.contains("12.") {
                        info!("Detected CUDA 12.x on Windows");
                        return get_component_url("llm_win_cuda12");
                    }
                }
            }

            if std::env::var("VULKAN_SDK").is_ok() {
                info!("Detected Vulkan SDK on Windows");
                return get_component_url("llm_win_vulkan");
            }

            info!("Using standard Windows x64 CPU build");
            return get_component_url("llm_win_cpu_x64");
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected Windows ARM64");
            return get_component_url("llm_win_cpu_arm64");
        }
    }
}

#[derive(Debug)]
pub struct PackageManager {
    pub mode: InstallMode,
    pub os_type: OsType,
    pub base_path: PathBuf,
    pub tenant: String,
    pub components: HashMap<String, ComponentConfig>,
}

impl PackageManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Result<Self> {
        let os_type = detect_os();
        let base_path = if mode == InstallMode::Container {
            PathBuf::from("/opt/gbo")
        } else if let Ok(custom_path) = std::env::var("BOTSERVER_STACK_PATH") {
            PathBuf::from(custom_path)
        } else {
            std::env::current_dir()?.join("botserver-stack")
        };
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    pub fn with_base_path(
        mode: InstallMode,
        tenant: Option<String>,
        base_path: PathBuf,
    ) -> Result<Self> {
        let os_type = detect_os();
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    fn register_components(&mut self) {
        installer_regs::register_drive(&mut self.components);
        installer_regs::register_tables(&mut self.components);
        installer_regs::register_cache(&mut self.components);
        installer_regs::register_llm(&mut self.components);
        installer_regs::register_email(&mut self.components);
        installer_regs::register_proxy(&mut self.components);
        installer_regs::register_directory(&mut self.components);
        installer_regs::register_alm(&mut self.components);
        installer_regs::register_alm_ci(&mut self.components);
        installer_regs::register_dns(&mut self.components);
        installer_regs::register_meeting(&mut self.components);
        installer_regs::register_webmail(&mut self.components);
        installer_regs2::register_table_editor(&mut self.components);
        installer_regs2::register_doc_editor(&mut self.components);
        installer_regs2::register_remote_terminal(&mut self.components);
        installer_regs2::register_devtools(&mut self.components);
        installer_regs2::register_vector_db(&mut self.components);
        installer_regs2::register_timeseries_db(&mut self.components);
        installer_regs2::register_vault(&mut self.components);
        installer_regs2::register_observability(&mut self.components);
        installer_regs2::register_host(&mut self.components);
    }

    pub fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            let bin_path = self.base_path.join("bin").join(&component.name);
            let data_path = self.base_path.join("data").join(&component.name);
            let conf_path = self.base_path.join("conf");
            let logs_path = self.base_path.join("logs").join(&component.name);

            let check_cmd = component
                .check_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            let check_output = safe_sh_command(&check_cmd)
                .map(|o| o.status.success())
                .unwrap_or(false);

            if check_output {
                info!(
                    "Component {} is already running, skipping start",
                    component.name
                );
                return SafeCommand::noop_child()
                    .map_err(|e| anyhow::anyhow!("Failed to create noop process: {}", e));
            }

            if component.name == "vector_db" {
                let qdrant_conf = conf_path.join("vector_db/config.yaml");
                if !qdrant_conf.exists() {
                    let storage = data_path.join("storage");
                    let snapshots = data_path.join("snapshots");
                    let _ = std::fs::create_dir_all(&storage);
                    let _ = std::fs::create_dir_all(&snapshots);
                    let yaml = format!(
                        "storage:\n storage_path: {}\n snapshots_path: {}\n\nservice:\n host: 0.0.0.0\n http_port: 6333\n grpc_port: 6334\n enable_tls: false\n\nlog_level: INFO\n",
                        storage.display(),
                        snapshots.display()
                    );
                    if let Err(e) = std::fs::write(&qdrant_conf, yaml) {
                        warn!("Failed to write qdrant config: {}", e);
                    } else {
                        info!("Generated qdrant config at {}", qdrant_conf.display());
                    }
                }
            }

            let rendered_cmd = component
                .exec_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            if let Err(e) = std::fs::create_dir_all(&logs_path) {
                warn!("Failed to create log directory {}: {}", logs_path.display(), e);
            }

            trace!(
                "Starting component {} with command: {}",
                component.name,
                rendered_cmd
            );
            trace!(
                "Working directory: {}, logs_path: {}",
                bin_path.display(),
                logs_path.display()
            );

            let vault_credentials = installer_vault::fetch_vault_credentials();

            let mut evaluated_envs = HashMap::new();
            for (k, v) in &component.env_vars {
                if let Some(var_name) = v.strip_prefix('$') {
                    let value = vault_credentials
                        .get(var_name)
                        .cloned()
                        .or_else(|| std::env::var(var_name).ok())
                        .unwrap_or_default();
                    evaluated_envs.insert(k.clone(), value);
                } else {
                    evaluated_envs.insert(k.clone(), v.clone());
                }
            }

            trace!(
                "About to spawn shell command for {}: {}",
                component.name,
                rendered_cmd
            );
            trace!("Working dir: {}", bin_path.display());
            let child = SafeCommand::new("sh")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| c.trusted_shell_script_arg(&rendered_cmd))
                .and_then(|c| c.working_dir(&bin_path))
                .and_then(|cmd| cmd.spawn_with_envs(&evaluated_envs))
                .map_err(|e| anyhow::anyhow!("Failed to spawn process: {}", e));

            trace!("Spawn result for {}: {:?}", component.name, child.is_ok());
            std::thread::sleep(std::time::Duration::from_secs(2));

            trace!(
                "Checking if {} process exists after 2s sleep...",
                component.name
            );
            let check_proc = safe_pgrep(&["-f", &component.name]);
            if let Some(output) = check_proc {
                let pids = String::from_utf8_lossy(&output.stdout);
                trace!("pgrep '{}' result: '{}'", component.name, pids.trim());
            }

            match child {
                Ok(c) => {
                    trace!("Component {} started successfully", component.name);

                    if component.name == "vault" && self.mode == InstallMode::Local {
                        if let Err(e) = installer_vault::initialize_vault_local(&self.base_path) {
                            warn!("Failed to initialize Vault: {}", e);
                            warn!("Vault started but may need manual initialization");
                        }
                    }

                    Ok(c)
                }
                Err(e) => {
                    error!("Spawn failed for {}: {}", component.name, e);
                    let err_msg = e.to_string();
                    if err_msg.contains("already running")
                        || err_msg.contains("be running")
                        || component.name == "tables"
                    {
                        trace!(
                            "Component {} may already be running, continuing anyway",
                            component.name
                        );

                        if component.name == "vault" && self.mode == InstallMode::Local {
                            let _ = installer_vault2::ensure_env_file_exists(&self.base_path);
                        }

                        SafeCommand::noop_child()
                            .map_err(|e| anyhow::anyhow!("Failed to create noop process: {}", e))
                    } else {
                        Err(e)
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Component not found: {}", component))
        }
    }
}
