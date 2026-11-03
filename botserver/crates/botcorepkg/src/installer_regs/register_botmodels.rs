use super::*;

/// BotModels as a pythonbox: the archive carries its own CPython, the service
/// sources and `install.sh`/`run.sh`. Nothing is compiled or installed at this
/// step — `run.sh` resolves the virtual environment on first start, because the
/// dependency set (torch et al.) is far larger than the bundle itself.
pub fn register_botmodels(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "botmodels".to_string(),
        ComponentConfig {
            name: "botmodels".to_string(),
            ports: vec![8085],
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("botmodels"),
            binary_name: None,
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: vec![
                "chmod +x {{BIN_PATH}}/install.sh {{BIN_PATH}}/run.sh".to_string(),
            ],
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: vec![
                "chmod +x {{BIN_PATH}}/install.sh {{BIN_PATH}}/run.sh".to_string(),
            ],
            pre_install_cmds_windows: Vec::new(),
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::from([
                ("BOTMODELS_HOST".to_string(), "127.0.0.1".to_string()),
                ("BOTMODELS_PORT".to_string(), "8085".to_string()),
            ]),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/run.sh > {{LOGS_PATH}}/botmodels.log 2>&1 &".to_string(),
            check_cmd: "curl -f -s -o /dev/null --connect-timeout 2 -m 5 http://127.0.0.1:8085/api/health".to_string(),
            // No Windows bundle is published yet: the box is Linux x86_64 only.
            exec_cmd_windows: None,
            check_cmd_windows: None,
            container: None,
        },
    );
}
