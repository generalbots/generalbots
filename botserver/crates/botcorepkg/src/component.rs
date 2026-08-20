use std::collections::HashMap;
use crate::container::ContainerSettings;

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub component: String,
    pub container_name: String,
    pub container_ip: String,
    pub ports: Vec<u16>,
    pub env_vars: HashMap<String, String>,
    pub connection_info: String,
}

impl InstallResult {
    pub fn print(&self) {
        let component_upper = self.component.to_uppercase();
        println!("\n========================================");
        println!(" {component_upper} Installation Complete");
        println!("========================================\n");
        println!("Container: {}", self.container_name);
        println!("IP Address: {}", self.container_ip);
        println!("Ports: {:?}", self.ports);
        println!("\n--- Connection Info ---\n");
        println!("{}", self.connection_info);
        if !self.env_vars.is_empty() {
            println!("\n--- Environment Variables (.env) ---\n");
            for (key, value) in &self.env_vars {
                println!("{key}={value}");
            }
        }
        println!("\n========================================\n");
    }
}

#[derive(Debug, Clone)]
pub struct ComponentConfig {
    pub name: String,
    pub ports: Vec<u16>,
    pub dependencies: Vec<String>,
    pub linux_packages: Vec<String>,
    pub macos_packages: Vec<String>,
    pub windows_packages: Vec<String>,
    pub download_url: Option<String>,
    pub binary_name: Option<String>,
    pub pre_install_cmds_linux: Vec<String>,
    pub post_install_cmds_linux: Vec<String>,
    pub pre_install_cmds_macos: Vec<String>,
    pub post_install_cmds_macos: Vec<String>,
    pub pre_install_cmds_windows: Vec<String>,
    pub post_install_cmds_windows: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub data_download_list: Vec<String>,
    pub exec_cmd: String,
    pub check_cmd: String,
    pub exec_cmd_windows: Option<String>,
    pub check_cmd_windows: Option<String>,
    pub container: Option<ContainerSettings>,
}

impl ComponentConfig {
    pub fn effective_exec_cmd(&self) -> &str {
        if cfg!(target_os = "windows") {
            self.exec_cmd_windows.as_deref().unwrap_or("")
        } else {
            &self.exec_cmd
        }
    }

    pub fn effective_check_cmd(&self) -> &str {
        if cfg!(target_os = "windows") {
            self.check_cmd_windows.as_deref().unwrap_or("")
        } else {
            &self.check_cmd
        }
    }

    pub fn effective_binary_name(&self) -> Option<String> {
        if cfg!(target_os = "windows") {
            self.binary_name.as_ref().map(|n| {
                if !n.ends_with(".exe") {
                    format!("{}.exe", n)
                } else {
                    n.clone()
                }
            })
        } else {
            self.binary_name.clone()
        }
    }
}
