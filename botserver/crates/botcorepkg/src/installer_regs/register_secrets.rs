use super::*;

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

api_addr = "https://127.0.0.1:8200"
cluster_addr = "https://127.0.0.1:8201"
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

api_addr = "https://127.0.0.1:8200"
cluster_addr = "https://127.0.0.1:8201"
ui = true
disable_mlock = true
EOF"#.to_string(),
            ],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![
                "if not exist {{DATA_PATH}}\\vault mkdir {{DATA_PATH}}\\vault".to_string(),
                "if not exist {{CONF_PATH}}\\vault mkdir {{CONF_PATH}}\\vault".to_string(),
                "if not exist {{LOGS_PATH}}\\vault mkdir {{LOGS_PATH}}\\vault".to_string(),
                "echo storage \"file\" { > {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo   path = \"{{DATA_PATH_FWD}}\" >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo } >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo. >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo listener \"tcp\" { >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo   address = \"0.0.0.0:8200\" >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo   tls_disable = true >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo } >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo. >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo api_addr = \"http://127.0.0.1:8200\" >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo cluster_addr = \"http://127.0.0.1:8201\" >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo ui = true >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
                "echo disable_mlock = true >> {{CONF_PATH}}\\vault\\config.hcl".to_string(),
            ],
            post_install_cmds_windows: vec![],
            env_vars: {
                let mut env = HashMap::new();
                // #1960 — one source of truth for the local scheme: Windows
                // runs Vault with `tls_disable = true`, so HTTPS there probes a
                // listener that does not exist.
                env.insert(
                    "VAULT_ADDR".to_string(),
                    botcoresecrets::local_vault_addr().to_string(),
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
            check_cmd: "if [ -f {{CONF_PATH}}/system/certificates/botserver/client.crt ]; then curl -f -sk --connect-timeout 2 -m 5 --cert {{CONF_PATH}}/system/certificates/botserver/client.crt --key {{CONF_PATH}}/system/certificates/botserver/client.key \"${VAULT_ADDR}/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200\" >/dev/null 2>&1; else curl -f -sk --connect-timeout 2 -m 5 \"${VAULT_ADDR}/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200\" >/dev/null 2>&1; fi"
                .to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\vault.exe server -config={{CONF_PATH}}\\vault\\config.hcl > {{LOGS_PATH}}\\vault.log 2>&1".to_string()),
            // #1960 — the Windows listener is plain HTTP (tls_disable = true).
            // Probe the literal address instead of `%VAULT_ADDR%`: the check can
            // run before the component's env block is applied, and a stale or
            // external VAULT_ADDR (HTTPS) made a healthy vault look dead.
            check_cmd_windows: Some("curl -f -sk --connect-timeout 2 -m 5 http://127.0.0.1:8200/v1/sys/health?standbyok=true >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

