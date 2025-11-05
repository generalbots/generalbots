use std::collections::HashMap;

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
