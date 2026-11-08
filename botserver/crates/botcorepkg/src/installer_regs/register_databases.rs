use super::*;

pub fn register_vector_db(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "vector_db".to_string(),
        ComponentConfig {
            name: "vector_db".to_string(),
            ports: vec![6334],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("vector_db"),
            binary_name: Some("qdrant".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![
                "if not exist {{DATA_PATH}} mkdir {{DATA_PATH}}".to_string(),
                "if not exist {{LOGS_PATH}} mkdir {{LOGS_PATH}}".to_string(),
                "if not exist {{CONF_PATH}}\\vector_db mkdir {{CONF_PATH}}\\vector_db".to_string(),
                "echo service: > {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
                "echo   grpc_port: 6334 >> {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
                "echo   http_port: 6333 >> {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
                "echo storage: >> {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
                "echo   storage_path: {{DATA_PATH}} >> {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
                "echo log_level: INFO >> {{CONF_PATH}}\\vector_db\\config.yaml".to_string(),
            ],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/qdrant --config-path {{CONF_PATH}}/vector_db/config.yaml > {{LOGS_PATH}}/qdrant.log 2>&1 &".to_string(),
            check_cmd: "pgrep -x qdrant >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\qdrant.exe --config-path {{CONF_PATH}}\\vector_db\\config.yaml > {{LOGS_PATH}}\\qdrant.log 2>&1".to_string()),
            check_cmd_windows: Some("tasklist /FI \"IMAGENAME eq qdrant*\" 2>NUL | findstr /I \"qdrant\" >NUL".to_string()),
            container: None,
        },
    );
}

pub fn register_timeseries_db(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "timeseries_db".to_string(),
        ComponentConfig {
            name: "timeseries_db".to_string(),
            ports: vec![8086, 8083],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("timeseries_db"),
            binary_name: Some("influxd".to_string()),
            pre_install_cmds_linux: vec![
                "mkdir -p {{DATA_PATH}}/influxdb".to_string(),
                "mkdir -p {{CONF_PATH}}/influxdb".to_string(),
            ],
            post_install_cmds_linux: vec![
                "{{BIN_PATH}}/influx setup --org system --bucket metrics --username admin --password {{GENERATED_PASSWORD}} --force".to_string(),
            ],
            pre_install_cmds_macos: vec![
                "mkdir -p {{DATA_PATH}}/influxdb".to_string(),
            ],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: {
                let mut env = HashMap::new();
                env.insert("INFLUXD_ENGINE_PATH".to_string(), "{{DATA_PATH}}/influxdb/engine".to_string());
                env.insert("INFLUXD_BOLT_PATH".to_string(), "{{DATA_PATH}}/influxdb/influxd.bolt".to_string());
                env.insert("INFLUXD_HTTP_BIND_ADDRESS".to_string(), ":8086".to_string());
                env.insert("INFLUXD_REPORTING_DISABLED".to_string(), "true".to_string());
                env
            },
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/influxd --bolt-path={{DATA_PATH}}/influxdb/influxd.bolt --engine-path={{DATA_PATH}}/influxdb/engine --http-bind-address=:8086".to_string(),
            check_cmd: "curl -f --connect-timeout 2 -m 5 /health >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\influxd.exe --bolt-path={{DATA_PATH}}\\influxdb\\influxd.bolt --engine-path={{DATA_PATH}}\\influxdb\\engine --http-bind-address=:8086".to_string()),
            check_cmd_windows: Some("curl -f --connect-timeout 2 -m 5 http://localhost:8086/health >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

