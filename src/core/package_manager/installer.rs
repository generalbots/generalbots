use crate::package_manager::component::ComponentConfig;
use crate::package_manager::os::detect_os;
use crate::package_manager::{InstallMode, OsType};
use anyhow::Result;
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::path::PathBuf;

const LLAMA_CPP_VERSION: &str = "b7345";

fn get_llama_cpp_url() -> Option<String> {
    let base_url = format!(
        "https://github.com/ggml-org/llama.cpp/releases/download/{}",
        LLAMA_CPP_VERSION
    );

    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if std::path::Path::new("/usr/local/cuda").exists()
                || std::path::Path::new("/opt/cuda").exists()
                || std::env::var("CUDA_HOME").is_ok()
            {
                if let Ok(output) = std::process::Command::new("nvcc").arg("--version").output() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    if version_str.contains("13.") {
                        info!("Detected CUDA 13.x - using CUDA 13.1 build");
                        return Some(format!(
                            "{}/llama-{}-bin-linux-cuda-13.1-x64.zip",
                            base_url, LLAMA_CPP_VERSION
                        ));
                    } else if version_str.contains("12.") {
                        info!("Detected CUDA 12.x - using CUDA 12.4 build");
                        return Some(format!(
                            "{}/llama-{}-bin-linux-cuda-12.4-x64.zip",
                            base_url, LLAMA_CPP_VERSION
                        ));
                    }
                }
            }

            if std::path::Path::new("/usr/share/vulkan").exists()
                || std::env::var("VULKAN_SDK").is_ok()
            {
                info!("Detected Vulkan - using Vulkan build");
                return Some(format!(
                    "{}/llama-{}-bin-ubuntu-vulkan-x64.zip",
                    base_url, LLAMA_CPP_VERSION
                ));
            }

            info!("Using standard Ubuntu x64 build (CPU)");
            return Some(format!(
                "{}/llama-{}-bin-ubuntu-x64.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }

        #[cfg(target_arch = "s390x")]
        {
            info!("Detected s390x architecture");
            return Some(format!(
                "{}/llama-{}-bin-ubuntu-s390x.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected ARM64 architecture on Linux");

            warn!("No pre-built llama.cpp for Linux ARM64 - LLM will not be available");
            return None;
        }
    }

    #[cfg(target_os = "macos")]
    {
        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected macOS ARM64 (Apple Silicon)");
            return Some(format!(
                "{}/llama-{}-bin-macos-arm64.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }

        #[cfg(target_arch = "x86_64")]
        {
            info!("Detected macOS x64 (Intel)");
            return Some(format!(
                "{}/llama-{}-bin-macos-x64.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if std::env::var("CUDA_PATH").is_ok() {
                if let Ok(output) = std::process::Command::new("nvcc").arg("--version").output() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    if version_str.contains("13.") {
                        info!("Detected CUDA 13.x on Windows");
                        return Some(format!(
                            "{}/llama-{}-bin-win-cuda-13.1-x64.zip",
                            base_url, LLAMA_CPP_VERSION
                        ));
                    } else if version_str.contains("12.") {
                        info!("Detected CUDA 12.x on Windows");
                        return Some(format!(
                            "{}/llama-{}-bin-win-cuda-12.4-x64.zip",
                            base_url, LLAMA_CPP_VERSION
                        ));
                    }
                }
            }

            if std::env::var("VULKAN_SDK").is_ok() {
                info!("Detected Vulkan SDK on Windows");
                return Some(format!(
                    "{}/llama-{}-bin-win-vulkan-x64.zip",
                    base_url, LLAMA_CPP_VERSION
                ));
            }

            info!("Using standard Windows x64 CPU build");
            return Some(format!(
                "{}/llama-{}-bin-win-cpu-x64.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }

        #[cfg(target_arch = "aarch64")]
        {
            info!("Detected Windows ARM64");
            return Some(format!(
                "{}/llama-{}-bin-win-cpu-arm64.zip",
                base_url, LLAMA_CPP_VERSION
            ));
        }
    }

    #[allow(unreachable_code)]
    {
        warn!("Unknown platform - no llama.cpp binary available");
        None
    }
}

#[derive(Debug)]
pub struct PackageManager {
    pub mode: InstallMode,
    pub os_type: OsType,
    pub base_path: PathBuf,
    pub tenant: String,
    pub components: HashMap<String, ComponentConfig>,
}

impl PackageManager {
    pub fn new(mode: InstallMode, tenant: Option<String>) -> Result<Self> {
        let os_type = detect_os();
        let base_path = if mode == InstallMode::Container {
            PathBuf::from("/opt/gbo")
        } else if let Ok(custom_path) = std::env::var("BOTSERVER_STACK_PATH") {
            PathBuf::from(custom_path)
        } else {
            std::env::current_dir()?.join("botserver-stack")
        };
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    pub fn with_base_path(
        mode: InstallMode,
        tenant: Option<String>,
        base_path: PathBuf,
    ) -> Result<Self> {
        let os_type = detect_os();
        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = Self {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };
        pm.register_components();
        Ok(pm)
    }

    fn register_components(&mut self) {
        self.register_vault();
        self.register_tables();
        self.register_cache();
        self.register_drive();
        self.register_llm();
        self.register_email();
        self.register_proxy();
        self.register_dns();
        self.register_directory();
        self.register_alm();
        self.register_alm_ci();
        self.register_meeting();
        self.register_remote_terminal();
        self.register_devtools();
        self.register_vector_db();
        self.register_timeseries_db();
        self.register_observability();
        self.register_host();
        self.register_webmail();
        self.register_table_editor();
        self.register_doc_editor();
    }

    fn register_drive(&mut self) {
        self.components.insert(
            "drive".to_string(),
            ComponentConfig {
                name: "drive".to_string(),
                ports: vec![9000, 9001],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://dl.min.io/server/minio/release/linux-amd64/minio".to_string(),
                ),
                binary_name: Some("minio".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::from([
                    ("MINIO_ROOT_USER".to_string(), "$DRIVE_ACCESSKEY".to_string()),
                    ("MINIO_ROOT_PASSWORD".to_string(), "$DRIVE_SECRET".to_string()),
                ]),
                data_download_list: Vec::new(),
                exec_cmd: "nohup {{BIN_PATH}}/minio server {{DATA_PATH}} --address :9000 --console-address :9001 > {{LOGS_PATH}}/minio.log 2>&1 &".to_string(),
                check_cmd: "pgrep -f 'minio server' >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_tables(&mut self) {
        self.components.insert(
            "tables".to_string(),
            ComponentConfig {
                name: "tables".to_string(),
                ports: vec![5432],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/theseus-rs/postgresql-binaries/releases/download/17.2.0/postgresql-17.2.0-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                ),
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
                    "for i in $(seq 1 30); do ./bin/pg_isready -h localhost -p 5432 -U gbuser >/dev/null 2>&1 && echo 'PostgreSQL is ready' && break || echo \"Waiting for PostgreSQL... attempt $i/30\" >&2; sleep 2; done".to_string(),
                    "./bin/pg_isready -h localhost -p 5432 -U gbuser || { echo 'ERROR: PostgreSQL failed to start properly' >&2; cat {{LOGS_PATH}}/postgres.log >&2; exit 1; }".to_string(),
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
                check_cmd: "{{BIN_PATH}}/bin/pg_isready -h localhost -p 5432 -U gbuser >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_cache(&mut self) {
        self.components.insert(
            "cache".to_string(),
            ComponentConfig {
                name: "cache".to_string(),
                ports: vec![6379],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/valkey-io/valkey/archive/refs/tags/8.0.2.tar.gz".to_string(),
                ),
                binary_name: Some("valkey-server".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![

                    "ln -sf {{BIN_PATH}}/valkey-server {{BIN_PATH}}/redis-server 2>/dev/null || true".to_string(),
                    "ln -sf {{BIN_PATH}}/valkey-cli {{BIN_PATH}}/redis-cli 2>/dev/null || true".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "nohup {{BIN_PATH}}/valkey-server --port 6379 --dir {{DATA_PATH}} --logfile {{LOGS_PATH}}/valkey.log --daemonize yes > {{LOGS_PATH}}/valkey-startup.log 2>&1".to_string(),
                check_cmd: "pgrep -f 'valkey-server' >/dev/null 2>&1 || {{BIN_PATH}}/valkey-cli ping 2>/dev/null | grep -q PONG".to_string(),
            },
        );
    }

    fn register_llm(&mut self) {
        let download_url = get_llama_cpp_url();

        if download_url.is_none() {
            warn!("No llama.cpp binary available for this platform");
            warn!("Local LLM will not be available - use external API instead");
        }

        info!(
            "LLM component using llama.cpp {} for this platform",
            LLAMA_CPP_VERSION
        );

        self.components.insert(
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

                    "https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf".to_string(),

                    "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-f32.gguf".to_string(),
                ],
                exec_cmd: "nohup {{BIN_PATH}}/llama-server --port 8081 --ssl-key-file {{CONF_PATH}}/system/certificates/llm/server.key --ssl-cert-file {{CONF_PATH}}/system/certificates/llm/server.crt -m {{DATA_PATH}}/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf > {{LOGS_PATH}}/llm.log 2>&1 & nohup {{BIN_PATH}}/llama-server --port 8082 --ssl-key-file {{CONF_PATH}}/system/certificates/embedding/server.key --ssl-cert-file {{CONF_PATH}}/system/certificates/embedding/server.crt -m {{DATA_PATH}}/bge-small-en-v1.5-f32.gguf --embedding > {{LOGS_PATH}}/embedding.log 2>&1 &".to_string(),
                check_cmd: "curl -f -k https://localhost:8081/health >/dev/null 2>&1 && curl -f -k https://localhost:8082/health >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_email(&mut self) {
        self.components.insert(
            "email".to_string(),
            ComponentConfig {
                name: "email".to_string(),
                ports: vec![25, 143, 465, 993, 8025],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/stalwartlabs/mail-server/releases/download/v0.10.7/stalwart-mail-x86_64-linux.tar.gz"
                        .to_string(),
                ),
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
                check_cmd: "curl -f -k https://localhost:8025/health >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_proxy(&mut self) {
        self.components.insert(
            "proxy".to_string(),
            ComponentConfig {
                name: "proxy".to_string(),
                ports: vec![80, 443],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/caddyserver/caddy/releases/download/v2.10.0-beta.3/caddy_2.10.0-beta.3_linux_amd64.tar.gz".to_string(),
                ),
                binary_name: Some("caddy".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/caddy".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::from([("XDG_DATA_HOME".to_string(), "{{DATA_PATH}}".to_string())]),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/caddy run --config {{CONF_PATH}}/Caddyfile".to_string(),
                check_cmd: "curl -f http://localhost >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_directory(&mut self) {
        self.components.insert(
            "directory".to_string(),
            ComponentConfig {
                name: "directory".to_string(),
                ports: vec![8300],
                dependencies: vec!["tables".to_string()],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/zitadel/zitadel/releases/download/v2.70.4/zitadel-linux-amd64.tar.gz"
                        .to_string(),
                ),
                binary_name: Some("zitadel".to_string()),
                pre_install_cmds_linux: vec![
                    "mkdir -p {{CONF_PATH}}/directory".to_string(),
                    "mkdir -p {{LOGS_PATH}}".to_string(),
                ],
                post_install_cmds_linux: vec![



                    "ZITADEL_MASTERKEY=$(VAULT_ADDR=http://localhost:8200 vault kv get -field=masterkey secret/gbo/directory 2>/dev/null || echo 'MasterkeyNeedsToHave32Characters') nohup {{BIN_PATH}}/zitadel start-from-init --config {{CONF_PATH}}/directory/zitadel.yaml --masterkeyFromEnv --tlsMode disabled --steps {{CONF_PATH}}/directory/steps.yaml > {{LOGS_PATH}}/zitadel.log 2>&1 &".to_string(),

                    "for i in $(seq 1 90); do curl -sf http://localhost:8300/debug/ready && break || sleep 1; done".to_string(),
                ],
                pre_install_cmds_macos: vec![
                    "mkdir -p {{CONF_PATH}}/directory".to_string(),
                ],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::from([
                    ("ZITADEL_EXTERNALSECURE".to_string(), "false".to_string()),
                    ("ZITADEL_EXTERNALDOMAIN".to_string(), "localhost".to_string()),
                    ("ZITADEL_EXTERNALPORT".to_string(), "8300".to_string()),
                    ("ZITADEL_TLS_ENABLED".to_string(), "false".to_string()),
                ]),
                data_download_list: Vec::new(),
                exec_cmd: "ZITADEL_MASTERKEY=$(VAULT_ADDR=http://localhost:8200 vault kv get -field=masterkey secret/gbo/directory 2>/dev/null || echo 'MasterkeyNeedsToHave32Characters') nohup {{BIN_PATH}}/zitadel start --config {{CONF_PATH}}/directory/zitadel.yaml --masterkeyFromEnv --tlsMode disabled > {{LOGS_PATH}}/zitadel.log 2>&1 &".to_string(),
                check_cmd: "curl -f http://localhost:8300/healthz >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_alm(&mut self) {
        self.components.insert(
            "alm".to_string(),
            ComponentConfig {
                name: "alm".to_string(),
                ports: vec![3000],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://codeberg.org/forgejo/forgejo/releases/download/v10.0.2/forgejo-10.0.2-linux-amd64".to_string(),
                ),
                binary_name: Some("forgejo".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::from([
                    ("USER".to_string(), "alm".to_string()),
                    ("HOME".to_string(), "{{DATA_PATH}}".to_string()),
                ]),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/forgejo web --work-path {{DATA_PATH}} --port 3000 --cert {{CONF_PATH}}/system/certificates/alm/server.crt --key {{CONF_PATH}}/system/certificates/alm/server.key".to_string(),
                check_cmd: "curl -f -k https://localhost:3000 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_alm_ci(&mut self) {
        self.components.insert(
            "alm-ci".to_string(),
            ComponentConfig {
                name: "alm-ci".to_string(),

                ports: vec![],
                dependencies: vec!["alm".to_string()],
                linux_packages: vec![],
                macos_packages: vec!["git".to_string(), "node".to_string()],
                windows_packages: vec![],
                download_url: Some(
                    "https://code.forgejo.org/forgejo/runner/releases/download/v6.3.1/forgejo-runner-6.3.1-linux-amd64".to_string(),
                ),
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
                exec_cmd: "{{BIN_PATH}}/forgejo-runner daemon --config {{CONF_PATH}}/alm-ci/config.yaml".to_string(),
                check_cmd: "ps -ef | grep forgejo-runner | grep -v grep | grep {{BIN_PATH}} >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_dns(&mut self) {
        self.components.insert(
            "dns".to_string(),
            ComponentConfig {
                name: "dns".to_string(),
                ports: vec![53],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/coredns/coredns/releases/download/v1.11.1/coredns_1.11.1_linux_amd64.tgz".to_string(),
                ),
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
            },
        );
    }

    fn register_webmail(&mut self) {
        self.components.insert(
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
                download_url: Some(
                    "https://github.com/roundcube/roundcubemail/releases/download/1.6.6/roundcubemail-1.6.6-complete.tar.gz".to_string(),
                ),
                binary_name: None,
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "php -S 0.0.0.0:8080 -t {{DATA_PATH}}/roundcubemail".to_string(),
                check_cmd: "curl -f -k https://localhost:8300 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_meeting(&mut self) {
        self.components.insert(
            "meet".to_string(),
            ComponentConfig {
                name: "meet".to_string(),
                ports: vec![7880],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/livekit/livekit/releases/download/v2.8.2/livekit_2.8.2_linux_amd64.tar.gz"
                        .to_string(),
                ),
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
                check_cmd: "curl -f -k https://localhost:7880 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_table_editor(&mut self) {
        self.components.insert(
            "table_editor".to_string(),
            ComponentConfig {
                name: "table_editor".to_string(),

                ports: vec![5757],
                dependencies: vec!["tables".to_string()],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some("http://get.nocodb.com/linux-x64".to_string()),
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
                check_cmd: "curl -f -k https://localhost:5757 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_doc_editor(&mut self) {
        self.components.insert(
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
                check_cmd: "curl -f -k https://localhost:9980 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_remote_terminal(&mut self) {
        self.components.insert(
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
            },
        );
    }

    fn register_devtools(&mut self) {
        self.components.insert(
            "devtools".to_string(),
            ComponentConfig {
                name: "devtools".to_string(),

                ports: vec![],
                dependencies: vec![],
                linux_packages: vec!["xclip".to_string(), "git".to_string(), "curl".to_string()],
                macos_packages: vec!["git".to_string()],
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
            },
        );
    }

    fn _register_botserver(&mut self) {
        self.components.insert(
            "system".to_string(),
            ComponentConfig {
                name: "system".to_string(),

                ports: vec![8000],
                dependencies: vec![],
                linux_packages: vec!["curl".to_string(), "unzip".to_string(), "git".to_string()],
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
            },
        );
    }

    fn register_vector_db(&mut self) {
        self.components.insert(
            "vector_db".to_string(),
            ComponentConfig {
                name: "vector_db".to_string(),

                ports: vec![6333],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/qdrant/qdrant/releases/latest/download/qdrant-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                ),
                binary_name: Some("qdrant".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/qdrant --storage-path {{DATA_PATH}} --enable-tls --cert {{CONF_PATH}}/system/certificates/qdrant/server.crt --key {{CONF_PATH}}/system/certificates/qdrant/server.key".to_string(),
                check_cmd: "curl -f -k https://localhost:6334/metrics >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_timeseries_db(&mut self) {
        self.components.insert(
            "timeseries_db".to_string(),
            ComponentConfig {
                name: "timeseries_db".to_string(),
                ports: vec![8086, 8083],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://download.influxdata.com/influxdb/releases/influxdb2-2.7.5-linux-amd64.tar.gz".to_string(),
                ),
                binary_name: Some("influxd".to_string()),
                pre_install_cmds_linux: vec![
                    "mkdir -p {{DATA_PATH}}/influxdb".to_string(),
                    "mkdir -p {{CONF_PATH}}/influxdb".to_string(),
                ],
                post_install_cmds_linux: vec![
                    "{{BIN_PATH}}/influx setup --org pragmatismo --bucket metrics --username admin --password {{GENERATED_PASSWORD}} --force".to_string(),
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
                check_cmd: "curl -f http://localhost:8086/health >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_vault(&mut self) {
        self.components.insert(
            "vault".to_string(),
            ComponentConfig {
                name: "vault".to_string(),
                ports: vec![8200],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://releases.hashicorp.com/vault/1.15.4/vault_1.15.4_linux_amd64.zip"
                        .to_string(),
                ),
                binary_name: Some("vault".to_string()),
                pre_install_cmds_linux: vec![
                    "mkdir -p {{DATA_PATH}}/vault".to_string(),
                    "mkdir -p {{CONF_PATH}}/vault".to_string(),
                    "mkdir -p {{LOGS_PATH}}".to_string(),
                    r#"cat > {{CONF_PATH}}/vault/config.hcl << 'EOF'
storage "file" {
  path = "/opt/gbo/data/vault"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 1
}

api_addr = "http://0.0.0.0:8200"
cluster_addr = "http://0.0.0.0:8201"
ui = true
disable_mlock = true
EOF"#.to_string(),
                ],


                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![
                    "mkdir -p {{DATA_PATH}}/vault".to_string(),
                    "mkdir -p {{CONF_PATH}}/vault".to_string(),
                    "mkdir -p {{LOGS_PATH}}".to_string(),
                    r#"cat > {{CONF_PATH}}/vault/config.hcl << 'EOF'
storage "file" {
  path = "{{DATA_PATH}}/vault"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 1
}

api_addr = "http://0.0.0.0:8200"
cluster_addr = "http://0.0.0.0:8201"
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
                    env.insert("VAULT_SKIP_VERIFY".to_string(), "true".to_string());
                    env
                },
                data_download_list: Vec::new(),
                exec_cmd: "nohup {{BIN_PATH}}/vault server -config={{CONF_PATH}}/vault/config.hcl > {{LOGS_PATH}}/vault.log 2>&1 &"
                    .to_string(),
                check_cmd: "curl -f -s 'http://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200' >/dev/null 2>&1"
                    .to_string(),
            },
        );
    }

    fn register_observability(&mut self) {
        self.components.insert(
            "observability".to_string(),
            ComponentConfig {
                name: "observability".to_string(),
                ports: vec![8686],
                dependencies: vec!["timeseries_db".to_string()],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://packages.timber.io/vector/0.35.0/vector-0.35.0-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                ),
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






                exec_cmd: "{{BIN_PATH}}/vector --config {{CONF_PATH}}/monitoring/vector.toml".to_string(),
                check_cmd: "curl -f http://localhost:8686/health >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_host(&mut self) {
        self.components.insert(
            "host".to_string(),
            ComponentConfig {
                name: "host".to_string(),

                ports: vec![],
                dependencies: vec![],
                linux_packages: vec!["sshfs".to_string(), "bridge-utils".to_string()],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: None,
                binary_name: None,
                pre_install_cmds_linux: vec![
                    "echo 'net.ipv4.ip_forward=1' | tee -a /etc/sysctl.conf".to_string(),
                    "sysctl -p".to_string(),
                ],
                post_install_cmds_linux: vec![
                    "lxd init --auto".to_string(),
                    "lxc storage create default dir".to_string(),
                    "lxc profile device add default root disk path=/ pool=default".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "".to_string(),
                check_cmd: "".to_string(),
            },
        );
    }

    pub fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            let bin_path = self.base_path.join("bin").join(&component.name);
            let data_path = self.base_path.join("data").join(&component.name);
            let conf_path = self.base_path.join("conf");
            let logs_path = self.base_path.join("logs").join(&component.name);

            let check_cmd = component
                .check_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            let check_output = std::process::Command::new("sh")
                .current_dir(&bin_path)
                .arg("-c")
                .arg(&check_cmd)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if check_output.is_ok() && check_output.unwrap().success() {
                info!(
                    "Component {} is already running, skipping start",
                    component.name
                );
                return Ok(std::process::Command::new("sh")
                    .arg("-c")
                    .arg("true")
                    .spawn()?);
            }

            let rendered_cmd = component
                .exec_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            eprintln!(
                "[START DEBUG] Starting component {} with command: {}",
                component.name, rendered_cmd
            );
            eprintln!(
                "[START DEBUG] Working directory: {}, logs_path: {}",
                bin_path.display(),
                logs_path.display()
            );
            info!(
                "Starting component {} with command: {}",
                component.name, rendered_cmd
            );
            info!(
                "Working directory: {}, logs_path: {}",
                bin_path.display(),
                logs_path.display()
            );

            let vault_credentials = Self::fetch_vault_credentials();

            let mut evaluated_envs = HashMap::new();
            for (k, v) in &component.env_vars {
                if let Some(var_name) = v.strip_prefix('$') {
                    let value = vault_credentials
                        .get(var_name)
                        .cloned()
                        .or_else(|| std::env::var(var_name).ok())
                        .unwrap_or_default();
                    evaluated_envs.insert(k.clone(), value);
                } else {
                    evaluated_envs.insert(k.clone(), v.clone());
                }
            }

            info!(
                "[START] About to spawn shell command for {}: {}",
                component.name, rendered_cmd
            );
            info!("[START] Working dir: {}", bin_path.display());
            let child = std::process::Command::new("sh")
                .current_dir(&bin_path)
                .arg("-c")
                .arg(&rendered_cmd)
                .envs(&evaluated_envs)
                .spawn();

            info!(
                "[START] Spawn result for {}: {:?}",
                component.name,
                child.is_ok()
            );
            std::thread::sleep(std::time::Duration::from_secs(2));

            info!(
                "[START] Checking if {} process exists after 2s sleep...",
                component.name
            );
            let check_proc = std::process::Command::new("pgrep")
                .args(["-f", &component.name])
                .output();
            if let Ok(output) = check_proc {
                let pids = String::from_utf8_lossy(&output.stdout);
                info!(
                    "[START] pgrep '{}' result: '{}'",
                    component.name,
                    pids.trim()
                );
            }

            match child {
                Ok(c) => {
                    info!("[START] Component {} started successfully", component.name);
                    Ok(c)
                }
                Err(e) => {
                    error!("[START] Spawn failed for {}: {}", component.name, e);
                    let err_msg = e.to_string();
                    if err_msg.contains("already running")
                        || err_msg.contains("be running")
                        || component.name == "tables"
                    {
                        trace!(
                            "Component {} may already be running, continuing anyway",
                            component.name
                        );
                        Ok(std::process::Command::new("sh")
                            .arg("-c")
                            .arg("true")
                            .spawn()?)
                    } else {
                        Err(e.into())
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Component not found: {}", component))
        }
    }

    fn fetch_vault_credentials() -> HashMap<String, String> {
        let mut credentials = HashMap::new();

        let vault_addr =
            std::env::var("VAULT_ADDR").unwrap_or_else(|_| "http://localhost:8200".to_string());
        let vault_token = std::env::var("VAULT_TOKEN").unwrap_or_default();

        if vault_token.is_empty() {
            trace!("VAULT_TOKEN not set, skipping Vault credential fetch");
            return credentials;
        }

        if let Ok(output) = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv get -format=json secret/gbo/drive 2>/dev/null",
                vault_addr, vault_token
            ))
            .output()
        {
            if output.status.success() {
                if let Ok(json_str) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                            if let Some(accesskey) = data.get("accesskey").and_then(|v| v.as_str()) {
                                credentials.insert("DRIVE_ACCESSKEY".to_string(), accesskey.to_string());
                            }
                            if let Some(secret) = data.get("secret").and_then(|v| v.as_str()) {
                                credentials.insert("DRIVE_SECRET".to_string(), secret.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "unset VAULT_CLIENT_CERT VAULT_CLIENT_KEY VAULT_CACERT; VAULT_ADDR={} VAULT_TOKEN={} ./botserver-stack/bin/vault/vault kv get -format=json secret/gbo/cache 2>/dev/null",
                vault_addr, vault_token
            ))
            .output()
        {
            if output.status.success() {
                if let Ok(json_str) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                            if let Some(password) = data.get("password").and_then(|v| v.as_str()) {
                                credentials.insert("CACHE_PASSWORD".to_string(), password.to_string());
                            }
                        }
                    }
                }
            }
        }

        trace!("Fetched {} credentials from Vault", credentials.len());
        credentials
    }
}
