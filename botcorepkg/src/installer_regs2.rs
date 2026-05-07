use crate::component::ComponentConfig;
use crate::installer::{get_component_url, safe_sh_command};
use botlib::security::get_stack_path;
use log::info;
use std::collections::HashMap;

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
            container: None,
        },
    );
}

pub fn register_doc_editor(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "doc_editor".to_string(),
        ComponentConfig {
            name: "doc_editor".to_string(),
            ports: vec![9980],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: None,
            binary_name: Some("coolwsd".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "coolwsd --config-file={{CONF_PATH}}/coolwsd.xml".to_string(),
            check_cmd:
                "curl -f -k --connect-timeout 2 -m 5 https://localhost:9980 >/dev/null 2>&1"
                    .to_string(),
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
            container: None,
        },
    );
}

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
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/qdrant --config-path {{CONF_PATH}}/vector_db/config.yaml > {{LOGS_PATH}}/qdrant.log 2>&1 &".to_string(),
            check_cmd: "pgrep -x qdrant >/dev/null 2>&1".to_string(),
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
            container: None,
        },
    );
}

pub fn register_vault(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "vault".to_string(),
        ComponentConfig {
            name: "vault".to_string(),
            ports: vec![8200],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("vault"),
            binary_name: Some("vault".to_string()),
            pre_install_cmds_linux: vec![
                "mkdir -p {{DATA_PATH}}/vault".to_string(),
                "mkdir -p {{CONF_PATH}}/vault".to_string(),
                "mkdir -p {{LOGS_PATH}}".to_string(),
                r#"cat > {{CONF_PATH}}/vault/config.hcl << 'EOF'
storage "file" {
  path = "{{DATA_PATH}}/vault"
}

listener "tcp" {
  address = "0.0.0.0:8200"
  tls_disable = false
  tls_cert_file = "{{CONF_PATH}}/system/certificates/vault/server.crt"
  tls_key_file = "{{CONF_PATH}}/system/certificates/vault/server.key"
  tls_client_ca_file = "{{CONF_PATH}}/system/certificates/ca/ca.crt"
}

api_addr = "https://localhost:8200"
cluster_addr = "https://localhost:8201"
ui = true
disable_mlock = true
EOF"#.to_string(),
            ],
            post_install_cmds_linux: vec![
                "mkdir -p {{CONF_PATH}}/system/certificates/ca".to_string(),
                "mkdir -p {{CONF_PATH}}/system/certificates/vault".to_string(),
                "mkdir -p {{CONF_PATH}}/system/certificates/botserver".to_string(),
                "mkdir -p {{CONF_PATH}}/system/certificates/tables".to_string(),
                "openssl genrsa -out {{CONF_PATH}}/system/certificates/ca/ca.key 4096 2>/dev/null".to_string(),
                "openssl req -new -x509 -days 3650 -key {{CONF_PATH}}/system/certificates/ca/ca.key -out {{CONF_PATH}}/system/certificates/ca/ca.crt -subj '/C=BR/ST=SP/L=São Paulo/O=BotServer Internal CA/CN=BotServer CA' 2>/dev/null".to_string(),
                "openssl genrsa -out {{CONF_PATH}}/system/certificates/vault/server.key 4096 2>/dev/null".to_string(),
                "openssl req -new -key {{CONF_PATH}}/system/certificates/vault/server.key -out {{CONF_PATH}}/system/certificates/vault/server.csr -subj '/C=BR/ST=SP/L=São Paulo/O=BotServer/CN=localhost' 2>/dev/null".to_string(),
                "echo -e 'subjectAltName = DNS:localhost,IP:127.0.0.1\\nkeyUsage = digitalSignature,keyEncipherment\\nextendedKeyUsage = serverAuth' > {{CONF_PATH}}/system/certificates/vault/server.ext".to_string(),
                "openssl x509 -req -days 3650 -in {{CONF_PATH}}/system/certificates/vault/server.csr -CA {{CONF_PATH}}/system/certificates/ca/ca.crt -CAkey {{CONF_PATH}}/system/certificates/ca/ca.key -CAcreateserial -out {{CONF_PATH}}/system/certificates/vault/server.crt -extfile {{CONF_PATH}}/system/certificates/vault/server.ext 2>/dev/null".to_string(),
                "openssl genrsa -out {{CONF_PATH}}/system/certificates/botserver/client.key 4096 2>/dev/null".to_string(),
                "openssl req -new -key {{CONF_PATH}}/system/certificates/botserver/client.key -out {{CONF_PATH}}/system/certificates/botserver/client.csr -subj '/C=BR/ST=SP/L=São Paulo/O=BotServer/CN=botserver' 2>/dev/null".to_string(),
                "openssl x509 -req -days 3650 -in {{CONF_PATH}}/system/certificates/botserver/client.csr -CA {{CONF_PATH}}/system/certificates/ca/ca.crt -CAkey {{CONF_PATH}}/system/certificates/ca/ca.key -CAcreateserial -out {{CONF_PATH}}/system/certificates/botserver/client.crt 2>/dev/null".to_string(),
                "openssl genrsa -out {{CONF_PATH}}/system/certificates/tables/server.key 4096 2>/dev/null".to_string(),
                "openssl req -new -key {{CONF_PATH}}/system/certificates/tables/server.key -out {{CONF_PATH}}/system/certificates/tables/server.csr -subj '/C=BR/ST=SP/L=São Paulo/O=BotServer/CN=localhost' 2>/dev/null".to_string(),
                "echo -e 'subjectAltName = DNS:localhost,IP:127.0.0.1\\nkeyUsage = digitalSignature,keyEncipherment\\nextendedKeyUsage = serverAuth' > {{CONF_PATH}}/system/certificates/tables/server.ext".to_string(),
                "openssl x509 -req -days 3650 -in {{CONF_PATH}}/system/certificates/tables/server.csr -CA {{CONF_PATH}}/system/certificates/ca/ca.crt -CAkey {{CONF_PATH}}/system/certificates/ca/ca.key -CAcreateserial -out {{CONF_PATH}}/system/certificates/tables/server.crt -extfile {{CONF_PATH}}/system/certificates/tables/server.ext 2>/dev/null".to_string(),
                "echo 'Certificates generated successfully'".to_string(),
            ],
            pre_install_cmds_macos: vec![
                "mkdir -p {{DATA_PATH}}/vault".to_string(),
                "mkdir -p {{CONF_PATH}}/vault".to_string(),
                "mkdir -p {{LOGS_PATH}}".to_string(),
                r#"cat > {{CONF_PATH}}/vault/config.hcl << 'EOF'
storage "file" {
  path = "{{DATA_PATH}}/vault"
}

listener "tcp" {
  address = "0.0.0.0:8200"
  tls_disable = false
  tls_cert_file = "{{CONF_PATH}}/system/certificates/vault/server.crt"
  tls_key_file = "{{CONF_PATH}}/system/certificates/vault/server.key"
  tls_client_ca_file = "{{CONF_PATH}}/system/certificates/ca/ca.crt"
}

api_addr = "https://localhost:8200"
cluster_addr = "https://localhost:8201"
ui = true
disable_mlock = true
EOF"#.to_string(),
            ],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: {
                let mut env = HashMap::new();
                env.insert(
                    "VAULT_ADDR".to_string(),
                    "https://localhost:8200".to_string(),
                );
                env.insert(
                    "VAULT_CACERT".to_string(),
                    format!("{}/conf/system/certificates/ca/ca.crt", get_stack_path()),
                );
                env
            },
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/vault server -config={{CONF_PATH}}/vault/config.hcl > {{LOGS_PATH}}/vault.log 2>&1 &"
                .to_string(),
            check_cmd: "if [ -f {{CONF_PATH}}/system/certificates/botserver/client.crt ]; then curl -f -sk --connect-timeout 2 -m 5 --cert {{CONF_PATH}}/system/certificates/botserver/client.crt --key {{CONF_PATH}}/system/certificates/botserver/client.key 'https://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1; else curl -f -sk --connect-timeout 2 -m 5 'https://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1; fi"
                .to_string(),
            container: None,
        },
    );
}

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
            container: None,
        },
    );
}
