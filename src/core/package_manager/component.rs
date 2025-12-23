use std::collections::HashMap;



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
        println!("\n========================================");
        println!("  {} Installation Complete", self.component.to_uppercase());
        println!("========================================\n");
        println!("Container: {}", self.container_name);
        println!("IP Address: {}", self.container_ip);
        println!("Ports: {:?}", self.ports);
        println!("\n--- Connection Info ---\n");
        println!("{}", self.connection_info);
        if !self.env_vars.is_empty() {
            println!("\n--- Environment Variables (.env) ---\n");
            for (key, value) in &self.env_vars {
                println!("{}={}", key, value);
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
}
