use super::*;

pub fn register_table_editor(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "table_editor".to_string(),
        ComponentConfig {
            name: "table_editor".to_string(),
            ports: vec![5757],
            dependencies: vec!["tables".to_string()],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("table_editor"),
            binary_name: Some("nocodb".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/nocodb".to_string(),
            check_cmd:
                "curl -f -k --connect-timeout 2 -m 5 https://localhost:5757 >/dev/null 2>&1"
                    .to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\nocodb.exe --port 5757 > {{LOGS_PATH}}\\nocodb.log 2>&1".to_string()),
            check_cmd_windows: Some("curl -f http://localhost:5757/api/health >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

pub fn register_remote_terminal(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "remote_terminal".to_string(),
        ComponentConfig {
            name: "remote_terminal".to_string(),
            ports: vec![3389],
            dependencies: vec![],
            linux_packages: vec!["xvfb".to_string(), "xrdp".to_string(), "xfce4".to_string()],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: None,
            binary_name: None,
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "xrdp --nodaemon".to_string(),
            check_cmd: "netstat -tln | grep :3389 >/dev/null 2>&1".to_string(),
            exec_cmd_windows: None,
            check_cmd_windows: Some("netstat -ano | findstr :3389 >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

pub fn register_devtools(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "devtools".to_string(),
        ComponentConfig {
            name: "devtools".to_string(),
            ports: vec![],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: None,
            binary_name: None,
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "".to_string(),
            check_cmd: "".to_string(),
            exec_cmd_windows: None,
            check_cmd_windows: None,
            container: None,
        },
    );
}
