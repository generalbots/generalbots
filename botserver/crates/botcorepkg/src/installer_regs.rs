use crate::component::ComponentConfig;
use crate::installer::{get_component_url, get_llama_cpp_url, get_model_url, LLAMA_CPP_VERSION};
use log::{info, warn};
use std::collections::HashMap;

pub fn register_drive(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "drive".to_string(),
        ComponentConfig {
            name: "drive".to_string(),
            ports: vec![9100, 9101],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("drive"),
            binary_name: Some("minio".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "cp {{DATA_PATH}}/mc {{BIN_PATH}}/mc && chmod +x {{BIN_PATH}}/mc".to_string(),
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("MINIO_ROOT_USER".to_string(), "$DRIVE_ACCESSKEY".to_string()),
                ("MINIO_ROOT_PASSWORD".to_string(), "$DRIVE_SECRET".to_string()),
            ]),
            data_download_list: get_component_url("mc").map(|url| vec![url]).unwrap_or_default(),
            exec_cmd: "nohup {{BIN_PATH}}/minio server {{DATA_PATH}} --address 127.0.0.1:9100 --console-address 127.0.0.1:9101 --certs-dir {{CONF_PATH}}/drive/certs > {{LOGS_PATH}}/minio.log 2>&1 &".to_string(),
            check_cmd: "curl -sf --cacert {{CONF_PATH}}/drive/certs/CAs/ca.crt https://127.0.0.1:9100/minio/health/live >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_tables(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "tables".to_string(),
        ComponentConfig {
            name: "tables".to_string(),
            ports: vec![5432],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("tables"),
            binary_name: Some("postgres".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "chmod +x ./bin/*".to_string(),
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then PG_PASSWORD='{{DB_PASSWORD}}' ./bin/initdb -D {{DATA_PATH}}/pgdata -U gbuser --pwfile=<(echo \"$PG_PASSWORD\"); fi".to_string(),
                "echo \"data_directory = '{{DATA_PATH}}/pgdata'\" > {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ident_file = '{{CONF_PATH}}/pg_ident.conf'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"port = 5432\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"listen_addresses = '*'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl = on\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_cert_file = '{{CONF_PATH}}/system/certificates/tables/server.crt'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_key_file = '{{CONF_PATH}}/system/certificates/tables/server.key'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_ca_file = '{{CONF_PATH}}/system/certificates/ca/ca.crt'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"log_directory = '{{LOGS_PATH}}'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"logging_collector = on\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"hostssl all all all md5\" > {{CONF_PATH}}/pg_hba.conf".to_string(),
                "touch {{CONF_PATH}}/pg_ident.conf".to_string(),
                "./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30".to_string(),
                "sleep 5".to_string(),
                "for i in $(seq 1 30); do ./bin/pg_isready -h localhost -p 5432 -d postgres >/dev/null 2>&1 && echo 'PostgreSQL is ready' && break || echo \"Waiting for PostgreSQL... attempt $i/30\" >&2; sleep 2; done".to_string(),
                "./bin/pg_isready -h localhost -p 5432 -d postgres || { echo 'ERROR: PostgreSQL failed to start properly' >&2; cat {{LOGS_PATH}}/postgres.log >&2; exit 1; }".to_string(),
                "PGPASSWORD='{{DB_PASSWORD}}' ./bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE DATABASE botserver WITH OWNER gbuser\" 2>&1 | grep -v 'already exists' || true".to_string(),
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![
                "chmod +x ./bin/*".to_string(),
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then ./bin/initdb -A -D {{DATA_PATH}}/pgdata -U postgres; fi".to_string(),
            ],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30 > {{LOGS_PATH}}/stdout.log 2>&1 &".to_string(),
            check_cmd: "{{BIN_PATH}}/bin/pg_isready -h localhost -p 5432 -d postgres >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_cache(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "cache".to_string(),
        ComponentConfig {
            name: "cache".to_string(),
            ports: vec![6379],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("cache"),
            binary_name: Some("valkey-server".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "mkdir -p {{BIN_PATH}}/bin && cd {{BIN_PATH}}/bin && tar -xzf {{CACHE_FILE}} --strip-components=1 -C {{BIN_PATH}}/bin 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-server 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-cli 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-benchmark 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-check-aof 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-check-rdb 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-sentinel 2>/dev/null || true".to_string(),
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/bin/valkey-server --port 6379 --bind 127.0.0.1 --dir {{DATA_PATH}} --logfile {{LOGS_PATH}}/valkey.log --daemonize yes > {{LOGS_PATH}}/valkey-startup.log 2>&1".to_string(),
            check_cmd: "pgrep -x valkey-server >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_llm(components: &mut HashMap<String, ComponentConfig>) {
    let download_url = get_llama_cpp_url();

    if download_url.is_none() {
        warn!("No llama.cpp binary available for this platform");
        warn!("Local LLM will not be available - use external API instead");
    }

    info!(
        "LLM component using llama.cpp {} for this platform",
        LLAMA_CPP_VERSION
    );

    components.insert(
        "llm".to_string(),
        ComponentConfig {
            name: "llm".to_string(),
            ports: vec![8081, 8082],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url,
            binary_name: Some("llama-server".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: vec![
                get_model_url("deepseek_small").unwrap_or_default(),
                get_model_url("bge_embedding").unwrap_or_default(),
            ],
            exec_cmd: "nohup {{BIN_PATH}}/build/bin/llama-server --port 8081 --ssl-key-file {{CONF_PATH}}/system/certificates/llm/server.key --ssl-cert-file {{CONF_PATH}}/system/certificates/llm/server.crt -m {{DATA_PATH}}/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf --ubatch-size 512 > {{LOGS_PATH}}/llm.log 2>&1 & nohup {{BIN_PATH}}/build/bin/llama-server --port 8082 --ssl-key-file {{CONF_PATH}}/system/certificates/embedding/server.key --ssl-cert-file {{CONF_PATH}}/system/certificates/embedding/server.crt -m {{DATA_PATH}}/bge-small-en-v1.5-f32.gguf --embeddings --pooling mean --n-gpu-layers 0 --ctx-size 512 --ubatch-size 512 > {{LOGS_PATH}}/embedding.log 2>&1 &".to_string(),
            check_cmd: "curl -f -k --connect-timeout 2 -m 5 https://localhost:8081/health >/dev/null 2>&1 && curl -f -k --connect-timeout 2 -m 5 https://localhost:8082/health >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_email(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "email".to_string(),
        ComponentConfig {
            name: "email".to_string(),
            ports: vec![25, 143, 465, 993, 8025],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("email"),
            binary_name: Some("stalwart-mail".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("STALWART_TLS_ENABLE".to_string(), "true".to_string()),
                ("STALWART_TLS_CERT".to_string(), "{{CONF_PATH}}/system/certificates/email/server.crt".to_string()),
                ("STALWART_TLS_KEY".to_string(), "{{CONF_PATH}}/system/certificates/email/server.key".to_string()),
            ]),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/stalwart-mail --config {{CONF_PATH}}/email/config.toml".to_string(),
            check_cmd: "curl -f -k --connect-timeout 2 -m 5 https://localhost:8025/health >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_proxy(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "proxy".to_string(),
        ComponentConfig {
            name: "proxy".to_string(),
            ports: vec![80, 443],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("proxy"),
            binary_name: Some("caddy".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/caddy".to_string(),
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([(
                "XDG_DATA_HOME".to_string(),
                "{{DATA_PATH}}".to_string(),
            )]),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/caddy run --config {{CONF_PATH}}/Caddyfile".to_string(),
            check_cmd: "curl -f --connect-timeout 2 -m 5 http://localhost >/dev/null 2>&1"
                .to_string(),
            container: None,
        },
    );
}

pub fn register_directory(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "directory".to_string(),
        ComponentConfig {
            name: "directory".to_string(),
            ports: vec![8300],
            dependencies: vec!["tables".to_string()],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("directory"),
            binary_name: Some("zitadel".to_string()),
            pre_install_cmds_linux: vec![
                "mkdir -p {{CONF_PATH}}/directory".to_string(),
                "mkdir -p {{LOGS_PATH}}".to_string(),
                concat!(
                    "cat > {{CONF_PATH}}/directory/zitadel-init-steps.yaml << 'STEPSEOF'\n",
                    "FirstInstance:\n",
                    " Org:\n",
                    " Machine:\n",
                    " Machine:\n",
                    " Username: gb-service-account\n",
                    " Name: General Bots Service Account\n",
                    " MachineKey:\n",
                    " Type: 1\n",
                    " Pat:\n",
                    " ExpirationDate: '2099-01-01T00:00:00Z'\n",
                    " PatPath: {{CONF_PATH}}/directory/admin-pat.txt\n",
                    " MachineKeyPath: {{CONF_PATH}}/directory/machine-key.json\n",
                    "STEPSEOF",
                ).to_string(),
            ],
            post_install_cmds_linux: vec![
                "PGPASSWORD='{{DB_PASSWORD}}' {{STACK_PATH}}/bin/tables/bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE ROLE zitadel WITH LOGIN PASSWORD 'zitadel'\" 2>&1 | grep -v 'already exists' || true".to_string(),
                "PGPASSWORD='{{DB_PASSWORD}}' {{STACK_PATH}}/bin/tables/bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE DATABASE zitadel WITH OWNER zitadel\" 2>&1 | grep -v 'already exists' || true".to_string(),
                "PGPASSWORD='{{DB_PASSWORD}}' {{STACK_PATH}}/bin/tables/bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \"GRANT ALL PRIVILEGES ON DATABASE zitadel TO zitadel\" 2>&1 || true".to_string(),
                concat!(
                    "ZITADEL_PORT=8300 ",
                    "ZITADEL_DATABASE_POSTGRES_HOST=localhost ",
                    "ZITADEL_DATABASE_POSTGRES_PORT=5432 ",
                    "ZITADEL_DATABASE_POSTGRES_DATABASE=zitadel ",
                    "ZITADEL_DATABASE_POSTGRES_USER_USERNAME=zitadel ",
                    "ZITADEL_DATABASE_POSTGRES_USER_PASSWORD=zitadel ",
                    "ZITADEL_DATABASE_POSTGRES_USER_SSL_MODE=disable ",
                    "ZITADEL_DATABASE_POSTGRES_ADMIN_USERNAME=gbuser ",
                    "ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD={{DB_PASSWORD}} ",
                    "ZITADEL_DATABASE_POSTGRES_ADMIN_SSL_MODE=disable ",
                    "ZITADEL_EXTERNALSECURE=false ",
                    "ZITADEL_EXTERNALDOMAIN=localhost ",
                    "ZITADEL_EXTERNALPORT=8300 ",
                    "ZITADEL_TLS_ENABLED=false ",
                    "nohup {{BIN_PATH}}/zitadel start-from-init ",
                    "--masterkey MasterkeyNeedsToHave32Characters ",
                    "--tlsMode disabled ",
                    "--steps {{CONF_PATH}}/directory/zitadel-init-steps.yaml ",
                    "> {{LOGS_PATH}}/zitadel.log 2>&1 &",
                ).to_string(),
                "for i in $(seq 1 120); do curl -sf http://localhost:8300/debug/healthz && echo 'Zitadel is ready!' && break || sleep 2; done".to_string(),
                "echo 'Waiting for PAT token in logs...'; for i in $(seq 1 30); do sync; if grep -q -E '^[A-Za-z0-9_-]{40,}$' {{LOGS_PATH}}/zitadel.log 2>/dev/null; then echo \"PAT token found in logs after $((i*2)) seconds\"; break; fi; sleep 2; done".to_string(),
                "if [ ! -f '{{CONF_PATH}}/directory/admin-pat.txt' ]; then grep -E '^[A-Za-z0-9_-]{40,}$' {{LOGS_PATH}}/zitadel.log 2>/dev/null | head -1 > {{CONF_PATH}}/directory/admin-pat.txt && echo 'PAT extracted from logs' || echo 'Could not extract PAT from logs'; fi".to_string(),
                "sync; sleep 1; if [ -f '{{CONF_PATH}}/directory/admin-pat.txt' ] && [ -s '{{CONF_PATH}}/directory/admin-pat.txt' ]; then echo 'PAT token created successfully'; cat {{CONF_PATH}}/directory/admin-pat.txt; else echo 'WARNING: PAT file not found or empty'; fi".to_string(),
            ],
            pre_install_cmds_macos: vec![
                "mkdir -p {{CONF_PATH}}/directory".to_string(),
            ],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("ZITADEL_PORT".to_string(), "8300".to_string()),
                ("ZITADEL_EXTERNALSECURE".to_string(), "false".to_string()),
                ("ZITADEL_EXTERNALDOMAIN".to_string(), "localhost".to_string()),
                ("ZITADEL_EXTERNALPORT".to_string(), "8300".to_string()),
                ("ZITADEL_TLS_ENABLED".to_string(), "false".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_HOST".to_string(), "localhost".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_PORT".to_string(), "5432".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_DATABASE".to_string(), "zitadel".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_USER_USERNAME".to_string(), "zitadel".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_USER_PASSWORD".to_string(), "zitadel".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_USER_SSL_MODE".to_string(), "disable".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_ADMIN_USERNAME".to_string(), "gbuser".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD".to_string(), "$DB_PASSWORD".to_string()),
                ("ZITADEL_DATABASE_POSTGRES_ADMIN_SSL_MODE".to_string(), "disable".to_string()),
            ]),
            data_download_list: Vec::new(),
            exec_cmd: concat!(
                "if [ -f {{CONF_PATH}}/directory/admin-pat.txt ]; then ",
                "nohup {{BIN_PATH}}/zitadel start ",
                "--masterkey MasterkeyNeedsToHave32Characters ",
                "--tlsMode disabled ",
                "> {{LOGS_PATH}}/zitadel.log 2>&1 & ",
                "else ",
                "ZITADEL_PORT=8300 ",
                "ZITADEL_DATABASE_POSTGRES_HOST=localhost ",
                "ZITADEL_DATABASE_POSTGRES_PORT=5432 ",
                "ZITADEL_DATABASE_POSTGRES_DATABASE=zitadel ",
                "ZITADEL_DATABASE_POSTGRES_USER_USERNAME=zitadel ",
                "ZITADEL_DATABASE_POSTGRES_USER_PASSWORD=zitadel ",
                "ZITADEL_DATABASE_POSTGRES_USER_SSL_MODE=disable ",
                "ZITADEL_DATABASE_POSTGRES_ADMIN_USERNAME=gbuser ",
                "ZITADEL_DATABASE_POSTGRES_ADMIN_PASSWORD={{DB_PASSWORD}} ",
                "ZITADEL_DATABASE_POSTGRES_ADMIN_SSL_MODE=disable ",
                "ZITADEL_EXTERNALSECURE=false ",
                "ZITADEL_EXTERNALDOMAIN=localhost ",
                "ZITADEL_EXTERNALPORT=8300 ",
                "ZITADEL_TLS_ENABLED=false ",
                "nohup {{BIN_PATH}}/zitadel start-from-init ",
                "--masterkey MasterkeyNeedsToHave32Characters ",
                "--tlsMode disabled ",
                "--steps {{CONF_PATH}}/directory/zitadel-init-steps.yaml ",
                "> {{LOGS_PATH}}/zitadel.log 2>&1 & ",
                "fi",
            ).to_string(),
            check_cmd: "curl -f --connect-timeout 2 -m 5 http://localhost:8300/debug/healthz >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_alm(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "alm".to_string(),
        ComponentConfig {
            name: "alm".to_string(),
            ports: vec![3000],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("alm"),
            binary_name: Some("forgejo".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("FORGEJO_RUN_USER".to_string(), "$USER".to_string()),
                ("HOME".to_string(), "{{DATA_PATH}}".to_string()),
            ]),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/forgejo web --work-path {{DATA_PATH}} --port 3000 --cert {{CONF_PATH}}/system/certificates/alm/server.crt --key {{CONF_PATH}}/system/certificates/alm/server.key > {{LOGS_PATH}}/forgejo.log 2>&1 &".to_string(),
            check_cmd: "curl -f -k --connect-timeout 2 -m 5 https://localhost:3000 >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_alm_ci(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "alm-ci".to_string(),
        ComponentConfig {
            name: "alm-ci".to_string(),
            ports: vec![],
            dependencies: vec!["alm".to_string()],
            linux_packages: vec![],
            macos_packages: vec!["git".to_string(), "node".to_string()],
            windows_packages: vec![],
            download_url: get_component_url("alm_ci"),
            binary_name: Some("forgejo-runner".to_string()),
            pre_install_cmds_linux: vec![
                "mkdir -p {{CONF_PATH}}/alm-ci".to_string(),
            ],
            post_install_cmds_linux: vec![
                "echo 'To register the runner, run:'".to_string(),
                "echo '{{BIN_PATH}}/forgejo-runner register --instance $ALM_URL --token $ALM_RUNNER_TOKEN --name gbo --labels ubuntu-latest:docker://node:20-bookworm'".to_string(),
                "echo 'Then start with: {{BIN_PATH}}/forgejo-runner daemon --config {{CONF_PATH}}/alm-ci/config.yaml'".to_string(),
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: {
                let mut env = HashMap::new();
                env.insert("ALM_URL".to_string(), "$ALM_URL".to_string());
                env.insert("ALM_RUNNER_TOKEN".to_string(), "$ALM_RUNNER_TOKEN".to_string());
                env
            },
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/forgejo-runner daemon --config {{CONF_PATH}}/alm-ci/config.yaml > {{LOGS_PATH}}/forgejo-runner.log 2>&1 &".to_string(),
            check_cmd: "ps -ef | grep forgejo-runner | grep -v grep | grep {{BIN_PATH}} >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_dns(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "dns".to_string(),
        ComponentConfig {
            name: "dns".to_string(),
            ports: vec![53],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("dns"),
            binary_name: Some("coredns".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/coredns -conf {{CONF_PATH}}/dns/Corefile".to_string(),
            check_cmd: "dig @localhost botserver.local >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}

pub fn register_webmail(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "webmail".to_string(),
        ComponentConfig {
            name: "webmail".to_string(),
            ports: vec![8300],
            dependencies: vec!["email".to_string()],
            linux_packages: vec![
                "ca-certificates".to_string(),
                "apt-transport-https".to_string(),
                "php8.1".to_string(),
                "php8.1-fpm".to_string(),
            ],
            macos_packages: vec!["php".to_string()],
            windows_packages: vec![],
            download_url: get_component_url("webmail"),
            binary_name: None,
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "php -S 0.0.0.0:9000 -t {{DATA_PATH}}/roundcubemail".to_string(),
            check_cmd:
                "curl -f -k --connect-timeout 2 -m 5 https://localhost:8300 >/dev/null 2>&1"
                    .to_string(),
            container: None,
        },
    );
}

pub fn register_meeting(components: &mut HashMap<String, ComponentConfig>) {
    components.insert(
        "meet".to_string(),
        ComponentConfig {
            name: "meet".to_string(),
            ports: vec![7880],
            dependencies: vec![],
            linux_packages: vec![],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: get_component_url("meet"),
            binary_name: Some("livekit-server".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/livekit-server --config {{CONF_PATH}}/meet/config.yaml --key-file {{CONF_PATH}}/system/certificates/meet/server.key --cert-file {{CONF_PATH}}/system/certificates/meet/server.crt".to_string(),
            check_cmd: "curl -f -k --connect-timeout 2 -m 5 https://localhost:7880 >/dev/null 2>&1".to_string(),
            container: None,
        },
    );
}
