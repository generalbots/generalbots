use anyhow::Result;
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::package_manager::component::ComponentConfig;
use crate::package_manager::os::detect_os;
use crate::package_manager::{InstallMode, OsType};

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
        } else {
            std::env::current_dir()?.join("botserver-stack")
        };

        let tenant = tenant.unwrap_or_else(|| "default".to_string());

        let mut pm = PackageManager {
            mode,
            os_type,
            base_path,
            tenant,
            components: HashMap::new(),
        };

        pm.register_components();
        info!(
            "PackageManager initialized with {} components in {:?} mode for tenant {}",
            pm.components.len(),
            pm.mode,
            pm.tenant
        );
        Ok(pm)
    }

    fn register_components(&mut self) {
        self.register_tables();
        self.register_cache();
        self.register_drive();
        self.register_llm();
        self.register_email();
        self.register_proxy();
        self.register_directory();
        self.register_alm();
        self.register_alm_ci();
        self.register_dns();
        self.register_webmail();
        self.register_meeting();
        self.register_table_editor();
        self.register_doc_editor();
        self.register_desktop();
        self.register_devtools();
        self.register_bot();
        self.register_system();
        self.register_vector_db();
        self.register_host();
    }

    fn register_drive(&mut self) {
        self.components.insert("drive".to_string(), ComponentConfig {
            name: "drive".to_string(),
            required: true,
            ports: vec![9000, 9001],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string()],
            macos_packages: vec!["wget".to_string()],
            windows_packages: vec![],
            download_url: Some("https://dl.min.io/server/minio/release/linux-amd64/minio".to_string()),
            binary_name: Some("minio".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "wget https://dl.min.io/client/mc/release/linux-amd64/mc -O {{BIN_PATH}}/mc".to_string(),
                "chmod +x {{BIN_PATH}}/mc".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![
                "wget https://dl.min.io/client/mc/release/darwin-amd64/mc -O {{BIN_PATH}}/mc".to_string(),
                "chmod +x {{BIN_PATH}}/mc".to_string()
            ],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("MINIO_ROOT_USER".to_string(), "minioadmin".to_string()),
                ("MINIO_ROOT_PASSWORD".to_string(), "minioadmin".to_string())
            ]),
            exec_cmd: "{{BIN_PATH}}/minio server {{DATA_PATH}} --address :9000 --console-address :9001".to_string(),
        });
    }

    fn register_cache(&mut self) {
        self.components.insert("cache".to_string(), ComponentConfig {
            name: "cache".to_string(),
            required: true,
            ports: vec![6379],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "curl".to_string(), "gnupg".to_string(), "lsb-release".to_string()],
            macos_packages: vec!["redis".to_string()],
            windows_packages: vec![],
            download_url: None,
            binary_name: Some("valkey-server".to_string()),
            pre_install_cmds_linux: vec![
                    "if [ ! -f /usr/share/keyrings/valkey.gpg ]; then curl -fsSL https://packages.redis.io/gpg | gpg --dearmor -o /usr/share/keyrings/valkey.gpg; fi".to_string(),
                    "if [ ! -f /etc/apt/sources.list.d/valkey.list ]; then echo 'deb [signed-by=/usr/share/keyrings/valkey.gpg] https://packages.redis.io/deb $(lsb_release -cs) main' | tee /etc/apt/sources.list.d/valkey.list; fi".to_string(),
                                "apt-get update && apt-get install -y valkey".to_string()
            ],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "valkey-server --port 6379 --dir {{DATA_PATH}}".to_string(),
        });
    }

    fn register_tables(&mut self) {
        self.components.insert("tables".to_string(), ComponentConfig {
            name: "tables".to_string(),
            required: true,
            ports: vec![5432],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string()],
            macos_packages: vec!["wget".to_string()],
            windows_packages: vec![],
            download_url: Some("https://github.com/theseus-rs/postgresql-binaries/releases/download/18.0.0/postgresql-18.0.0-x86_64-unknown-linux-gnu.tar.gz".to_string()),
            binary_name: Some("postgres".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then ./bin/initdb -D {{DATA_PATH}}/pgdata -U postgres; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"data_directory = '{{DATA_PATH}}/pgdata'\" > {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"hba_file = '{{CONF_PATH}}/pg_hba.conf'\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"ident_file = '{{CONF_PATH}}/pg_ident.conf'\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"port = 5432\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"listen_addresses = '*'\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"log_directory = '{{LOGS_PATH}}'\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/postgresql.conf\" ]; then echo \"logging_collector = on\" >> {{CONF_PATH}}/postgresql.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/pg_hba.conf\" ]; then echo \"host all all all md5\" > {{CONF_PATH}}/pg_hba.conf; fi".to_string(),
                "if [ ! -f \"{{CONF_PATH}}/pg_ident.conf\" ]; then touch {{CONF_PATH}}/pg_ident.conf; fi".to_string(),
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then ./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start; sleep 5; ./bin/psql -p 5432 -d postgres -c \" CREATE USER default WITH PASSWORD 'defaultpass'\"; ./bin/psql -p 5432 -d postgres -c \"CREATE DATABASE default_db OWNER default\"; ./bin/psql -p 5432 -d postgres -c \"GRANT ALL PRIVILEGES ON DATABASE default_db TO default\"; pkill; fi".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then ./bin/initdb -D {{DATA_PATH}}/pgdata -U postgres; fi".to_string(),
            ],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start".to_string(),
        });
    }

    fn register_llm(&mut self) {
        self.components.insert("llm".to_string(), ComponentConfig {
            name: "llm".to_string(),
            required: true,
            ports: vec![8081],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "unzip".to_string()],
            macos_packages: vec!["wget".to_string(), "unzip".to_string()],
            windows_packages: vec![],
            download_url: Some("https://github.com/ggml-org/llama.cpp/releases/download/b6148/llama-b6148-bin-ubuntu-x64.zip".to_string()),
            binary_name: Some("llama-server".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "wget https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf -P {{DATA_PATH}}".to_string(),
                "wget https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-f32.gguf -P {{DATA_PATH}}".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![
                "wget https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf -P {{DATA_PATH}}".to_string(),
                "wget https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf/resolve/main/bge-small-en-v1.5-f32.gguf -P {{DATA_PATH}}".to_string()
            ],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/llama-server -m {{DATA_PATH}}/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf --port 8081".to_string(),
        });
    }

    fn register_email(&mut self) {
        self.components.insert("email".to_string(), ComponentConfig {
            name: "email".to_string(),
            required: false,
            ports: vec![25, 80, 110, 143, 465, 587, 993, 995, 4190],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "libcap2-bin".to_string(), "resolvconf".to_string()],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: Some("https://github.com/stalwartlabs/stalwart/releases/download/v0.13.1/stalwart-x86_64-unknown-linux-gnu.tar.gz".to_string()),
            binary_name: Some("stalwart".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/stalwart".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/stalwart --config {{CONF_PATH}}/config.toml".to_string(),
        });
    }

    fn register_proxy(&mut self) {
        self.components.insert("proxy".to_string(), ComponentConfig {
            name: "proxy".to_string(),
            required: false,
            ports: vec![80, 443],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "libcap2-bin".to_string()],
            macos_packages: vec!["wget".to_string()],
            windows_packages: vec![],
            download_url: Some("https://github.com/caddyserver/caddy/releases/download/v2.10.0-beta.3/caddy_2.10.0-beta.3_linux_amd64.tar.gz".to_string()),
            binary_name: Some("caddy".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/caddy".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("XDG_DATA_HOME".to_string(), "{{DATA_PATH}}".to_string())
            ]),
            exec_cmd: "{{BIN_PATH}}/caddy run --config {{CONF_PATH}}/Caddyfile".to_string(),
        });
    }

    fn register_directory(&mut self) {
        self.components.insert("directory".to_string(), ComponentConfig {
            name: "directory".to_string(),
            required: false,
            ports: vec![8080],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "libcap2-bin".to_string()],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: Some("https://github.com/zitadel/zitadel/releases/download/v2.71.2/zitadel-linux-amd64.tar.gz".to_string()),
            binary_name: Some("zitadel".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/zitadel".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/zitadel start --config {{CONF_PATH}}/zitadel.yaml".to_string(),
        });
    }

    fn register_alm(&mut self) {
        self.components.insert("alm".to_string(), ComponentConfig {
            name: "alm".to_string(),
            required: false,
            ports: vec![3000],
            dependencies: vec![],
            linux_packages: vec!["git".to_string(), "git-lfs".to_string(), "wget".to_string()],
            macos_packages: vec!["git".to_string(), "git-lfs".to_string()],
            windows_packages: vec![],
            download_url: Some("https://codeberg.org/forgejo/forgejo/releases/download/v10.0.2/forgejo-10.0.2-linux-amd64".to_string()),
            binary_name: Some("forgejo".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::from([
                ("USER".to_string(), "alm".to_string()),
                ("HOME".to_string(), "{{DATA_PATH}}".to_string())
            ]),
            exec_cmd: "{{BIN_PATH}}/forgejo web --work-path {{DATA_PATH}}".to_string(),
        });
    }

    fn register_alm_ci(&mut self) {
        self.components.insert("alm-ci".to_string(), ComponentConfig {
            name: "alm-ci".to_string(),
            required: false,
            ports: vec![],
            dependencies: vec!["alm".to_string()],
            linux_packages: vec!["wget".to_string(), "git".to_string(), "curl".to_string(), "gnupg".to_string(), "ca-certificates".to_string(), "build-essential".to_string()],
            macos_packages: vec!["git".to_string(), "node".to_string()],
            windows_packages: vec![],
            download_url: Some("https://code.forgejo.org/forgejo/runner/releases/download/v6.3.1/forgejo-runner-6.3.1-linux-amd64".to_string()),
            binary_name: Some("forgejo-runner".to_string()),
            pre_install_cmds_linux: vec![
                "curl -fsSL https://deb.nodesource.com/setup_22.x | bash -".to_string(),
                "apt-get update && apt-get install -y nodejs".to_string()
            ],
            post_install_cmds_linux: vec![
                "npm install -g pnpm@latest".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![
                "npm install -g pnpm@latest".to_string()
            ],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/forgejo-runner daemon --config {{CONF_PATH}}/config.yaml".to_string(),
        });
    }

    fn register_dns(&mut self) {
        self.components.insert("dns".to_string(), ComponentConfig {
            name: "dns".to_string(),
            required: false,
            ports: vec![53],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string()],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: Some("https://github.com/coredns/coredns/releases/download/v1.12.4/coredns_1.12.4_linux_amd64.tgz".to_string()),
            binary_name: Some("coredns".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![
                "setcap cap_net_bind_service=+ep {{BIN_PATH}}/coredns".to_string()
            ],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/coredns -conf {{CONF_PATH}}/Corefile".to_string(),
        });
    }

    fn register_webmail(&mut self) {
        self.components.insert("webmail".to_string(), ComponentConfig {
            name: "webmail".to_string(),
            required: false,
            ports: vec![8080],
            dependencies: vec!["email".to_string()],
            linux_packages: vec!["ca-certificates".to_string(), "apt-transport-https".to_string(), "php8.1".to_string(), "php8.1-fpm".to_string()],
            macos_packages: vec!["php".to_string()],
            windows_packages: vec![],
            download_url: Some("https://github.com/roundcube/roundcubemail/releases/download/1.6.6/roundcubemail-1.6.6-complete.tar.gz".to_string()),
            binary_name: None,
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "php -S 0.0.0.0:8080 -t {{DATA_PATH}}/roundcubemail".to_string(),
        });
    }

    fn register_meeting(&mut self) {
        self.components.insert("meeting".to_string(), ComponentConfig {
            name: "meeting".to_string(),
            required: false,
            ports: vec![7880, 3478],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string(), "coturn".to_string()],
            macos_packages: vec![],
            windows_packages: vec![],
            download_url: Some("https://github.com/livekit/livekit/releases/download/v1.8.4/livekit_1.8.4_linux_amd64.tar.gz".to_string()),
            binary_name: Some("livekit-server".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/livekit-server --config {{CONF_PATH}}/config.yaml".to_string(),
        });
    }

    fn register_table_editor(&mut self) {
        self.components.insert(
            "table-editor".to_string(),
            ComponentConfig {
                name: "table-editor".to_string(),
                required: false,
                ports: vec![5757],
                dependencies: vec!["tables".to_string()],
                linux_packages: vec!["wget".to_string(), "curl".to_string()],
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
                exec_cmd: "{{BIN_PATH}}/nocodb".to_string(),
            },
        );
    }

    fn register_doc_editor(&mut self) {
        self.components.insert(
            "doc-editor".to_string(),
            ComponentConfig {
                name: "doc-editor".to_string(),
                required: false,
                ports: vec![9980],
                dependencies: vec![],
                linux_packages: vec!["wget".to_string(), "gnupg".to_string()],
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
                exec_cmd: "coolwsd --config-file={{CONF_PATH}}/coolwsd.xml".to_string(),
            },
        );
    }

    fn register_desktop(&mut self) {
        self.components.insert(
            "desktop".to_string(),
            ComponentConfig {
                name: "desktop".to_string(),
                required: false,
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
                exec_cmd: "xrdp --nodaemon".to_string(),
            },
        );
    }

    fn register_devtools(&mut self) {
        self.components.insert(
            "devtools".to_string(),
            ComponentConfig {
                name: "devtools".to_string(),
                required: false,
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
                exec_cmd: "".to_string(),
            },
        );
    }

    fn register_bot(&mut self) {
        self.components.insert(
            "bot".to_string(),
            ComponentConfig {
                name: "bot".to_string(),
                required: false,
                ports: vec![3000],
                dependencies: vec![],
                linux_packages: vec![
                    "curl".to_string(),
                    "gnupg".to_string(),
                    "ca-certificates".to_string(),
                    "git".to_string(),
                ],
                macos_packages: vec!["node".to_string()],
                windows_packages: vec![],
                download_url: None,
                binary_name: None,
                pre_install_cmds_linux: vec![
                    "curl -fsSL https://deb.nodesource.com/setup_22.x | bash -".to_string(),
                    "apt-get update && apt-get install -y nodejs".to_string(),
                ],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::from([("DISPLAY".to_string(), ":99".to_string())]),
                exec_cmd: "".to_string(),
            },
        );
    }

    fn register_system(&mut self) {
        self.components.insert(
            "system".to_string(),
            ComponentConfig {
                name: "system".to_string(),
                required: false,
                ports: vec![8000],
                dependencies: vec![],
                linux_packages: vec![
                    "wget".to_string(),
                    "curl".to_string(),
                    "unzip".to_string(),
                    "git".to_string(),
                ],
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
                exec_cmd: "".to_string(),
            },
        );
    }

    fn register_vector_db(&mut self) {
        self.components.insert("vector-db".to_string(), ComponentConfig {
            name: "vector-db".to_string(),
            required: false,
            ports: vec![6333],
            dependencies: vec![],
            linux_packages: vec!["wget".to_string()],
            macos_packages: vec!["wget".to_string()],
            windows_packages: vec![],
            download_url: Some("https://github.com/qdrant/qdrant/releases/latest/download/qdrant-x86_64-unknown-linux-gnu.tar.gz".to_string()),
            binary_name: Some("qdrant".to_string()),
            pre_install_cmds_linux: vec![],
            post_install_cmds_linux: vec![],
            pre_install_cmds_macos: vec![],
            post_install_cmds_macos: vec![],
            pre_install_cmds_windows: vec![],
            post_install_cmds_windows: vec![],
            env_vars: HashMap::new(),
            exec_cmd: "{{BIN_PATH}}/qdrant --storage-path {{DATA_PATH}}".to_string(),
        });
    }

    fn register_host(&mut self) {
        self.components.insert(
            "host".to_string(),
            ComponentConfig {
                name: "host".to_string(),
                required: false,
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
                exec_cmd: "".to_string(),
            },
        );
    }

    pub(crate) fn start(&self, component: &str) -> Result<std::process::Child> {
        if let Some(component) = self.components.get(component) {
            Ok(std::process::Command::new("sh")
                .arg("-c")
                .arg(&component.exec_cmd)
                .spawn()?)
        } else {
            Err(anyhow::anyhow!("Component {} not found", component))
        }
    }
}
