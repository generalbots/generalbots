use super::*;

// The table editor component was removed (#2582). The `table_editor` key stays
// only in the existing out-of-scope exclusion lists that guard legacy stacks.

pub fn register_remote_desktop(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "remote_desktop".to_string(),
        ComponentConfig {
            name: "remote_desktop".to_string(),
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
