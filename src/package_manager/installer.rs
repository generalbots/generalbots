use crate::package_manager::component::ComponentConfig;
use crate::package_manager::os::detect_os;
use crate::package_manager::{InstallMode, OsType};
use anyhow::Result;
use log::trace;
use std::collections::HashMap;
use std::path::PathBuf;

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
        self.register_botserver();
        self.register_vector_db();
        self.register_host();
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
                check_cmd: "ps -ef | grep minio | grep -v grep | grep {{BIN_PATH}}".to_string(),
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
                    "https://github.com/theseus-rs/postgresql-binaries/releases/download/18.0.0/postgresql-18.0.0-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                ),
                binary_name: Some("postgres".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "chmod +x ./bin/*".to_string(),
                    format!("if [ ! -d \"{{{{DATA_PATH}}}}/pgdata\" ]; then PG_PASSWORD={{DB_PASSWORD}} ./bin/initdb -D {{{{DATA_PATH}}}}/pgdata -U gbuser --pwfile=<(echo $PG_PASSWORD); fi"),
                    "echo \"data_directory = '{{DATA_PATH}}/pgdata'\" > {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"ident_file = '{{CONF_PATH}}/pg_ident.conf'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"port = 5432\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"listen_addresses = '*'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"log_directory = '{{LOGS_PATH}}'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"logging_collector = on\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                    "echo \"host all all all md5\" > {{CONF_PATH}}/pg_hba.conf".to_string(),
                    "touch {{CONF_PATH}}/pg_ident.conf".to_string(),
                    "./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30".to_string(),
                    "sleep 5".to_string(),
                    "for i in $(seq 1 30); do ./bin/pg_isready -h localhost -p 5432 -U gbuser >/dev/null 2>&1 && echo 'PostgreSQL is ready' && break || echo \"Waiting for PostgreSQL... attempt $i/30\" >&2; sleep 2; done".to_string(),
                    "./bin/pg_isready -h localhost -p 5432 -U gbuser || { echo 'ERROR: PostgreSQL failed to start properly' >&2; cat {{LOGS_PATH}}/postgres.log >&2; exit 1; }".to_string(),
                    format!("PGPASSWORD={{DB_PASSWORD}} ./bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \"CREATE DATABASE botserver WITH OWNER gbuser\" 2>&1 | grep -v 'already exists' || true"),
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
                    "https://download.valkey.io/releases/valkey-9.0.0-jammy-x86_64.tar.gz".to_string(),
                ),
                binary_name: Some("valkey-server".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "chmod +x {{BIN_PATH}}/bin/valkey-server".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "nohup {{BIN_PATH}}/bin/valkey-server --port 6379 --dir {{DATA_PATH}} > {{LOGS_PATH}}/valkey.log 2>&1 && {{BIN_PATH}}/bin/valkey-cli CONFIG SET stop-writes-on-bgsave-error no 2>&1 &".to_string(),
                check_cmd: "{{BIN_PATH}}/bin/valkey-cli ping | grep -q PONG".to_string(),
            },
        );
    }

    fn register_llm(&mut self) {
        self.components.insert(
            "llm".to_string(),
            ComponentConfig {
                name: "llm".to_string(),
                
                ports: vec![8081, 8082],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/ggml-org/llama.cpp/releases/download/b6148/llama-b6148-bin-ubuntu-x64.zip".to_string(),
                ),
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
                exec_cmd: "".to_string(),
                check_cmd: "".to_string(),
            },
        );
    }

    fn register_email(&mut self) {
        self.components.insert(
            "email".to_string(),
            ComponentConfig {
                name: "email".to_string(),
                ports: vec![25, 80, 110, 143, 465, 587, 993, 995, 4190],
                dependencies: vec![],
                linux_packages: vec![], 
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/stalwartlabs/stalwart/releases/download/v0.13.1/stalwart-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                ),
                binary_name: Some("stalwart".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/stalwart".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/stalwart --config {{CONF_PATH}}/config.toml".to_string(),
                check_cmd: "curl -f http://localhost:25 >/dev/null 2>&1".to_string(),
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
                
                ports: vec![8080],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/zitadel/zitadel/releases/download/v2.71.2/zitadel-linux-amd64.tar.gz".to_string(),
                ),
                binary_name: Some("zitadel".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/zitadel".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/zitadel start --config {{CONF_PATH}}/zitadel.yaml".to_string(),
                check_cmd: "curl -f http://localhost:8080 >/dev/null 2>&1".to_string(),
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
                exec_cmd: "{{BIN_PATH}}/forgejo web --work-path {{DATA_PATH}}".to_string(),
                check_cmd: "curl -f http://localhost:3000 >/dev/null 2>&1".to_string(),
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
                linux_packages: vec![
                ],
                macos_packages: vec!["git".to_string(), "node".to_string()],
                windows_packages: vec![],
                download_url: Some(
                    "https://code.forgejo.org/forgejo/runner/releases/download/v6.3.1/forgejo-runner-6.3.1-linux-amd64".to_string(),
                ),
                binary_name: Some("forgejo-runner".to_string()),
                pre_install_cmds_linux: vec![
                ],
                post_install_cmds_linux: vec![],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/forgejo-runner daemon --config {{CONF_PATH}}/config.yaml".to_string(),
                check_cmd: "ps -ef | grep forgejo-runner | grep -v grep | grep {{BIN_PATH}}".to_string(),
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
                    "https://github.com/coredns/coredns/releases/download/v1.12.4/coredns_1.12.4_linux_amd64.tgz".to_string(),
                ),
                binary_name: Some("coredns".to_string()),
                pre_install_cmds_linux: vec![],
                post_install_cmds_linux: vec![
                    "setcap cap_net_bind_service=+ep {{BIN_PATH}}/coredns".to_string(),
                ],
                pre_install_cmds_macos: vec![],
                post_install_cmds_macos: vec![],
                pre_install_cmds_windows: vec![],
                post_install_cmds_windows: vec![],
                env_vars: HashMap::new(),
                data_download_list: Vec::new(),
                exec_cmd: "{{BIN_PATH}}/coredns -conf {{CONF_PATH}}/Corefile".to_string(),
                check_cmd: "dig @localhost example.com >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_webmail(&mut self) {
        self.components.insert(
            "webmail".to_string(),
            ComponentConfig {
                name: "webmail".to_string(),
                
                ports: vec![8080],
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
                check_cmd: "curl -f http://localhost:8080 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_meeting(&mut self) {
        self.components.insert(
            "meeting".to_string(),
            ComponentConfig {
                name: "meeting".to_string(),
                
                ports: vec![7880, 3478],
                dependencies: vec![],
                linux_packages: vec![],
                macos_packages: vec![],
                windows_packages: vec![],
                download_url: Some(
                    "https://github.com/livekit/livekit/releases/download/v1.8.4/livekit_1.8.4_linux_amd64.tar.gz".to_string(),
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
                exec_cmd: "{{BIN_PATH}}/livekit-server --config {{CONF_PATH}}/config.yaml".to_string(),
                check_cmd: "curl -f http://localhost:7880 >/dev/null 2>&1".to_string(),
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
                check_cmd: "curl -f http://localhost:5757 >/dev/null 2>&1".to_string(),
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
                check_cmd: "curl -f http://localhost:9980 >/dev/null 2>&1".to_string(),
            },
        );
    }

    fn register_desktop(&mut self) {
        self.components.insert(
            "desktop".to_string(),
            ComponentConfig {
                name: "desktop".to_string(),
                
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

    fn register_botserver(&mut self) {
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
                exec_cmd: "{{BIN_PATH}}/qdrant --storage-path {{DATA_PATH}}".to_string(),
                check_cmd: "curl -f http://localhost:6333 >/dev/null 2>&1".to_string(),
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
            let conf_path = self.base_path.join("conf").join(&component.name);
            let logs_path = self.base_path.join("logs").join(&component.name);

            // First check if the service is already running
            let check_cmd = component
                .check_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            let check_status = std::process::Command::new("sh")
                .current_dir(&bin_path)
                .arg("-c")
                .arg(&check_cmd)
                .status();

            if check_status.is_ok() && check_status.unwrap().success() {
                trace!("Component {} is already running", component.name);
                return Ok(std::process::Command::new("sh").arg("-c").spawn()?);
            }

            // If not running, execute the main command
            let rendered_cmd = component
                .exec_cmd
                .replace("{{BIN_PATH}}", &bin_path.to_string_lossy())
                .replace("{{DATA_PATH}}", &data_path.to_string_lossy())
                .replace("{{CONF_PATH}}", &conf_path.to_string_lossy())
                .replace("{{LOGS_PATH}}", &logs_path.to_string_lossy());

            trace!(
                "Starting component {} with command: {}",
                component.name,
                rendered_cmd
            );

            // Create new env vars map with evaluated $VAR references
            let mut evaluated_envs = HashMap::new();
            for (k, v) in C&component.env_vars {
                if v.starts_with('$') {
                    let var_name = &v[1..];
                    evaluated_envs.insert(k.clone(), std::env::var(var_name).unwrap_or_default());
                } else {
                    evaluated_envs.insert(k.clone(), v.clone());
                }
            }

            let child = std::process::Command::new("sh")
                .current_dir(&bin_path)
                .arg("-c")
                .arg(&rendered_cmd)
                .envs(&evaluated_envs)
                .spawn();

            std::thread::sleep(std::time::Duration::from_secs(2));

            match child {
                Ok(c) => Ok(c),
                Err(e) => {
                    let err_msg = e.to_string();
                    if err_msg.contains("already running")
                        || err_msg.contains("be running")
                        || component.name == "tables"
                    {
                        trace!(
                            "Component {} may already be running, continuing anyway",
                            component.name
                        );
                        Ok(std::process::Command::new("sh").arg("-c").spawn()?)
                    } else {
                        Err(e.into())
                    }
                }
            }
        } else {
            Err(anyhow::anyhow!("Component {} not found", component))
        }
    }

}
