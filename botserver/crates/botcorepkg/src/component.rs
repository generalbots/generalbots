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

    /// Returns a secondary binary name to check on Windows when the primary
    /// `binary_name` does not match. The cache ships as `valkey-server` on
    /// Linux but `redis-server.exe` on Windows (Microsoft port); the
    /// `exec_cmd_windows` already uses the correct name, so we extract it.
    pub fn windows_binary_name_from_exec(&self) -> Option<String> {
        let cmd = self.exec_cmd_windows.as_deref()?;
        // `start /B {{BIN_PATH}}\\redis-server.exe …`  — find the first
        // token ending in `.exe` after template expansion markers.
        cmd.split_whitespace()
            .find(|token| token.ends_with(".exe"))
            .and_then(|token| {
                // Strip path: `{{BIN_PATH}}\\redis-server.exe` → `redis-server.exe`
                token
                    .rsplit(|c| c == '\\' || c == '/')
                    .next()
            })
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal `ComponentConfig` for testing.
    fn stub(binary_name: Option<&str>, exec_cmd_windows: Option<&str>) -> ComponentConfig {
        ComponentConfig {
            name: "test".into(),
            ports: Vec::new(),
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: None,
            binary_name: binary_name.map(String::from),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: Vec::new(),
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: Vec::new(),
            pre_install_cmds_windows: Vec::new(),
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: String::new(),
            check_cmd: String::new(),
            exec_cmd_windows: exec_cmd_windows.map(String::from),
            check_cmd_windows: None,
            container: None,
        }
    }

    #[test]
    fn windows_binary_name_from_cache_exec() {
        let c = stub(
            Some("valkey-server"),
            Some("start /B {{BIN_PATH}}\\redis-server.exe --port 6379"),
        );
        assert_eq!(
            c.windows_binary_name_from_exec(),
            Some("redis-server.exe".into())
        );
    }

    #[test]
    fn windows_binary_name_from_exec_with_forward_slash() {
        let c = stub(None, Some("start /B {{BIN_PATH}}/otterio.exe server data"));
        assert_eq!(
            c.windows_binary_name_from_exec(),
            Some("otterio.exe".into())
        );
    }

    #[test]
    fn windows_binary_name_from_exec_none_when_no_exe() {
        let c = stub(None, Some("echo hello"));
        assert_eq!(c.windows_binary_name_from_exec(), None);
    }

    #[test]
    fn windows_binary_name_from_exec_none_when_no_cmd() {
        let c = stub(None, None);
        assert_eq!(c.windows_binary_name_from_exec(), None);
    }
}
