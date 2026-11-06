use super::*;

pub fn register_observability(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "observability".to_string(),
        ComponentConfig {
            name: "observability".to_string(),
            ports: vec![8686],
            dependencies: vec!["timeseries_db".to_string()],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("observability"),
            binary_name: Some("vector".to_string()),
            pre_install_cmds_linux: vec![
                "mkdir -p {{CONF_PATH}}/monitoring".to_string(),
                "mkdir -p {{DATA_PATH}}/vector".to_string(),
            ],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![
                "mkdir -p {{CONF_PATH}}/monitoring".to_string(),
                "mkdir -p {{DATA_PATH}}/vector".to_string(),
            ],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/vector --config {{CONF_PATH}}/monitoring/vector.toml"
                .to_string(),
            check_cmd: "curl -f --connect-timeout 2 -m 5 /health >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\vector.exe --config {{CONF_PATH}}\\monitoring\\vector.toml".to_string()),
            check_cmd_windows: Some("curl -f --connect-timeout 2 -m 5 http://localhost:8686/health >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

pub fn register_host(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "host".to_string(),
        ComponentConfig {
            name: "host".to_string(),
            ports: vec![],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: None,
            binary_name: None,
            pre_install_cmds_linux: vec![
                "echo 'net.ipv4.ip_forward=1' | tee -a /etc/sysctl.conf".to_string(),
                "sysctl -p".to_string(),
            ],
            post_install_cmds_linux: vec![
                "lxd init --dump >/dev/null 2>&1 || lxd init --auto".to_string(),
                "lxc storage show default >/dev/null 2>&1 || lxc storage create default dir".to_string(),
                "lxc profile device include default root >/dev/null 2>&1 || lxc profile device add default root disk path=/ pool=default".to_string(),
                "lxc profile device show default | grep lxd-sock >/dev/null 2>&1 || lxc profile device add default lxd-sock proxy connect=unix:/var/lib/lxd/unix.socket listen=unix:/tmp/lxd.sock bind=container uid=0 gid=0 mode=0660".to_string(),
            ],
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
