use super::*;

/// BotModels as a pythonbox: the archive carries its own CPython, the service
/// sources, a CPU-tier wheelhouse and platform entry points (`install.sh` /
/// `run.sh` on Linux, `install.ps1` / `run.ps1` on Windows). Nothing is
/// compiled or installed at this step — the run script materialises the
/// virtual environment from the bundled wheels on first start, so a clean
/// machine needs no network access (the dependency set is far larger than the
/// bundle itself).
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
            // The Windows pythonbox ships run.ps1; `start /B ""` detaches the
            // service so provisioning is not blocked while the venv is built.
            exec_cmd_windows: Some(
                "start /B \"\" powershell -NoProfile -ExecutionPolicy Bypass -File \"{{BIN_PATH}}\\run.ps1\" >> \"{{LOGS_PATH}}\\botmodels.log\" 2>&1".to_string(),
            ),
            check_cmd_windows: Some(
                "curl.exe -f -s -o NUL --connect-timeout 2 -m 5 http://127.0.0.1:8085/api/health".to_string(),
            ),
            container: None,
        },
    );
}
