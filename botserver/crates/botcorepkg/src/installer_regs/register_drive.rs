use super::*;

pub fn register_drive(components: &mut HashMap<String, ComponentConfig>) {
    // OC ships per-platform archives (`*.tar.gz` / `*.zip`), not a raw binary.
    // Extract the `oc` member into BIN_PATH so every caller can invoke it by
    // name; the component key stays `mc` in 3rdparty.toml for compatibility.
    let extract_oc = "oc_archive=$(ls {{DATA_PATH}}/oc-*.tar.gz 2>/dev/null | head -1); if [ -z \"$oc_archive\" ]; then oc_archive=$(ls {{DATA_PATH}}/oc-* 2>/dev/null | head -1); fi; if [ -n \"$oc_archive\" ]; then oc_tmp=$(mktemp -d); tar -xzf \"$oc_archive\" -C \"$oc_tmp\" 2>/dev/null; oc_bin=$(find \"$oc_tmp\" -type f -name oc -print -quit); if [ -n \"$oc_bin\" ]; then cp \"$oc_bin\" {{BIN_PATH}}/oc && chmod +x {{BIN_PATH}}/oc; fi; rm -rf \"$oc_tmp\"; fi".to_string();
    let extract_oc_windows = "powershell -NoProfile -Command \"$z = Get-ChildItem '{{DATA_PATH}}\\oc-*.zip' -ErrorAction SilentlyContinue | Select-Object -First 1; if ($z) { $d = '{{DATA_PATH}}\\oc-extract'; Expand-Archive -Path $z.FullName -DestinationPath $d -Force; $b = Get-ChildItem -Path $d -Filter oc.exe -Recurse -File | Select-Object -First 1; if ($b) { Copy-Item -Path $b.FullName -Destination '{{BIN_PATH}}\\oc.exe' -Force } }\"".to_string();
    components.insert(
        "drive".to_string(),
        ComponentConfig {
            name: "drive".to_string(),
            ports: vec![9100, 9101],
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("drive"),
            binary_name: Some("otterio".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: vec![extract_oc.clone()],
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: vec![extract_oc],
            pre_install_cmds_windows: vec![
                "if not exist {{DATA_PATH}} mkdir {{DATA_PATH}}".to_string(),
                "if not exist {{LOGS_PATH}} mkdir {{LOGS_PATH}}".to_string(),
                // #1964 — Windows runs OtterIO in plain HTTP: no `--certs-dir`
                // and no CA is provisioned, so there is no certs directory.
            ],
            post_install_cmds_windows: vec![extract_oc_windows],
            env_vars: HashMap::from([
                ("OTTERIO_ROOT_USER".to_string(), "$DRIVE_ACCESSKEY".to_string()),
                ("OTTERIO_ROOT_PASSWORD".to_string(), "$DRIVE_SECRET".to_string()),
            ]),
            data_download_list: get_component_url("mc").map(|url| vec![url]).unwrap_or_else(Vec::new),
            exec_cmd: "nohup {{BIN_PATH}}/otterio server {{DATA_PATH}} --address 127.0.0.1:9100 --console-address 127.0.0.1:9101 --certs-dir {{CONF_PATH}}/drive/certs > {{LOGS_PATH}}/otterio.log 2>&1 &".to_string(),
            check_cmd: "curl -sf --cacert {{CONF_PATH}}/drive/certs/CAs/ca.crt https://127.0.0.1:9100/otterio/health/live >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\otterio.exe server {{DATA_PATH}} --address 127.0.0.1:9100 --console-address 127.0.0.1:9101 > {{LOGS_PATH}}\\otterio.log 2>&1".to_string()),
            check_cmd_windows: Some("curl -sf --connect-timeout 2 -m 5 http://127.0.0.1:9100/otterio/health/live >NUL 2>&1".to_string()),
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
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("tables"),
            binary_name: Some("postgres".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: vec![
                "chmod +x ./bin/*".to_string(),
                // theseus-rs PostgreSQL 18.x bundles libstdc++ 12 from
                // Debian 12, whose only newer glibc dependency is
                // `arc4random` (GLIBC_2.36). Ubuntu 22.04 ships glibc 2.35,
                // so the dynamic loader aborts before initdb can print its
                // usage and the database is never initialised. Keep the
                // bundled copy when it loads; otherwise fall back to the
                // host libstdc++ (present on every WebKitGTK-capable
                // desktop, i.e. wherever this edition runs).
                "if ! ./bin/initdb --version >/dev/null 2>&1; then rm -f ./lib/libstdc++.so.6; fi".to_string(),
                "id postgres 2>/dev/null || useradd -r -s /bin/false postgres 2>/dev/null; [ \"$(id -u)\" -eq 0 ] && chown -R postgres:postgres {{DATA_PATH}} {{LOGS_PATH}} 2>/dev/null || true".to_string(),
                // Root-aware wrapper: run as the `postgres` user when root, otherwise directly as the launching user (allows non-root dev stacks where no `postgres` user exists).
                "cat > {{BIN_PATH}}/pgwrap.sh <<'PGB_WRAP'\n#!/bin/bash\ncmd=\"$*\"\nif [ \"$(id -u)\" -eq 0 ]; then su -s /bin/bash postgres -c \"$cmd\"; else bash -c \"$cmd\"; fi\nPGB_WRAP\nchmod +x {{BIN_PATH}}/pgwrap.sh".to_string(),
                // #2785 — pin SCRAM as the password format before initdb
                // creates the gbuser verifier (`-c` is written into the cluster's
                // postgresql.conf before CREATE/ALTER ROLE runs). `--auth-*`
                // makes the generated pg_hba reject MD5 and trust connections.
                // Migrating an existing pre-SCRAM cluster: set
                // password_encryption = 'scram-sha-256' + reload, re-run
                // `ALTER ROLE <role> WITH PASSWORD '<same password>'` for every
                // login role (gbuser, zitadel, ...), replace md5 with
                // scram-sha-256 in pg_hba.conf, then reload again.
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then printf '%s\\n' '{{DB_PASSWORD}}' > {{LOGS_PATH}}/pwfile && {{BIN_PATH}}/pgwrap.sh \"cd {{BIN_PATH}} && ./bin/initdb -D {{DATA_PATH}}/pgdata -U gbuser --pwfile={{LOGS_PATH}}/pwfile --auth-local=scram-sha-256 --auth-host=scram-sha-256 -c password_encryption=scram-sha-256\" && rm -f {{LOGS_PATH}}/pwfile; fi".to_string(),
                "echo \"data_directory = '{{DATA_PATH}}/pgdata'\" > {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"password_encryption = 'scram-sha-256'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ident_file = '{{CONF_PATH}}/pg_ident.conf'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"port = 5432\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"listen_addresses = '*'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl = on\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_cert_file = '{{CONF_PATH}}/system/certificates/tables/server.crt'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_key_file = '{{CONF_PATH}}/system/certificates/tables/server.key'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"ssl_ca_file = '{{CONF_PATH}}/system/certificates/ca/ca.crt'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"log_directory = '{{LOGS_PATH}}'\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"logging_collector = on\" >> {{CONF_PATH}}/postgresql.conf".to_string(),
                "echo \"hostssl all all all scram-sha-256\" > {{CONF_PATH}}/pg_hba.conf".to_string(),
                "touch {{CONF_PATH}}/pg_ident.conf".to_string(),
                "{{BIN_PATH}}/pgwrap.sh \"cd {{BIN_PATH}} && ./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30\"".to_string(),
                "sleep 5".to_string(),
                "for i in $(seq 1 30); do {{BIN_PATH}}/pgwrap.sh \"cd {{BIN_PATH}} && ./bin/pg_isready -h localhost -p 5432 -d postgres\" >/dev/null 2>&1 && echo 'PostgreSQL is ready' && break || echo \"Waiting for PostgreSQL... attempt $i/30\" >&2; sleep 2; done".to_string(),
                "{{BIN_PATH}}/pgwrap.sh \"cd {{BIN_PATH}} && ./bin/pg_isready -h localhost -p 5432 -d postgres\" || { echo 'ERROR: PostgreSQL failed to start properly' >&2; cat {{LOGS_PATH}}/postgres.log >&2; exit 1; }".to_string(),
                "{{BIN_PATH}}/pgwrap.sh \"cd {{BIN_PATH}} && PGPASSWORD='{{DB_PASSWORD}}' ./bin/psql -h localhost -p 5432 -U gbuser -d postgres -c \\\"CREATE DATABASE botserver WITH OWNER gbuser\\\"\" 2>&1 | grep -v 'already exists' || true".to_string(),
            ],
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: vec![
                "chmod +x ./bin/*".to_string(),
                "if [ ! -d \"{{DATA_PATH}}/pgdata\" ]; then ./bin/initdb -A -D {{DATA_PATH}}/pgdata -U postgres; fi".to_string(),
            ],
            pre_install_cmds_windows: vec![
                "if not exist {{DATA_PATH}} mkdir {{DATA_PATH}}".to_string(),
                "if not exist {{LOGS_PATH}} mkdir {{LOGS_PATH}}".to_string(),
                "if not exist {{CONF_PATH}} mkdir {{CONF_PATH}}".to_string(),
            ],
            post_install_cmds_windows: vec![
                "if not exist \"{{DATA_PATH}}\\pgdata\\PG_VERSION\" \"{{BIN_PATH}}\\bin\\initdb.exe\" --encoding=UTF8 --no-locale -D \"{{DATA_PATH}}\\pgdata\" -U gbuser --auth=trust".to_string(),
                "echo data_directory = '{{DATA_PATH}}\\pgdata' > \"{{CONF_PATH}}\\postgresql.conf\"".to_string(),
                "echo port = 5432 >> \"{{CONF_PATH}}\\postgresql.conf\"".to_string(),
                "echo listen_addresses = '127.0.0.1,::1' >> \"{{CONF_PATH}}\\postgresql.conf\"".to_string(),
                "echo log_directory = '{{LOGS_PATH}}' >> \"{{CONF_PATH}}\\postgresql.conf\"".to_string(),
                "echo logging_collector = on >> \"{{CONF_PATH}}\\postgresql.conf\"".to_string(),
                "echo host all all 127.0.0.1/32 scram-sha-256 > \"{{CONF_PATH}}\\pg_hba.conf\"".to_string(),
                "type nul > \"{{CONF_PATH}}\\pg_ident.conf\"".to_string(),
            ],
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "if [ $(id -u) -eq 0 ]; then id postgres 2>/dev/null || useradd -r -s /bin/false postgres; chown -R postgres:postgres {{DATA_PATH}} {{LOGS_PATH}} 2>/dev/null; su -s /bin/bash postgres -c \"cd {{BIN_PATH}} && ./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30\"; else cd {{BIN_PATH}} && ./bin/pg_ctl -D {{DATA_PATH}}/pgdata -l {{LOGS_PATH}}/postgres.log start -w -t 30; fi > {{LOGS_PATH}}/stdout.log 2>&1 &".to_string(),
            check_cmd: "if [ $(id -u) -eq 0 ]; then su -s /bin/bash postgres -c \"{{BIN_PATH}}/bin/pg_isready -h localhost -p 5432 -d postgres\"; else {{BIN_PATH}}/bin/pg_isready -h localhost -p 5432 -d postgres; fi >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\bin\\pg_ctl -D {{DATA_PATH}}\\pgdata -l {{LOGS_PATH}}\\postgresql.log start -w -t 30 > {{LOGS_PATH}}\\stdout.log 2>&1".to_string()),
            // Logins need the Vault password; readiness does not.
            check_cmd_windows: Some("\"{{BIN_PATH}}\\bin\\pg_isready.exe\" -h localhost -p 5432 -t 5 >NUL 2>&1".to_string()),
            container: None,
        },
    );
}
pub fn register_cache(components: &mut HashMap<String, ComponentConfig>) {
    // On Windows no external cache binary is bundled or downloaded: the server
    // provides a built-in RESP cache (main_module/embedded_cache.rs). The
    // Windows exec/check commands are cleared so the installer never tries to
    // start a binary that does not exist. Linux keeps Valkey as before.
    components.insert(
        "cache".to_string(),
        ComponentConfig {
            name: "cache".to_string(),
            ports: vec![6379],
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("cache"),
            binary_name: Some("valkey-server".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: vec![
                "mkdir -p {{BIN_PATH}}/bin && cd {{BIN_PATH}}/bin && tar -xzf {{CACHE_FILE}} --strip-components=1 -C {{BIN_PATH}}/bin 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-server 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-cli 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-benchmark 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-check-aof 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-check-rdb 2>/dev/null || true".to_string(),
                "chmod +x {{BIN_PATH}}/bin/valkey-sentinel 2>/dev/null || true".to_string(),
            ],
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: Vec::new(),
            pre_install_cmds_windows: Vec::new(),
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::new(),
            data_download_list: Vec::new(),
            exec_cmd: "nohup {{BIN_PATH}}/bin/valkey-server --port 6379 --bind 127.0.0.1 --dir {{DATA_PATH}} --logfile {{LOGS_PATH}}/valkey.log --daemonize yes > {{LOGS_PATH}}/valkey-startup.log 2>&1".to_string(),
            check_cmd: "pgrep -x valkey-server >/dev/null 2>&1".to_string(),
            exec_cmd_windows: None,
            check_cmd_windows: None,
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
        download_url
            .as_deref()
            .and_then(crate::installer::llama_build_tag)
            .unwrap_or_else(|| LLAMA_CPP_VERSION.to_string())
    );

    components.insert(
        "llm".to_string(),
        ComponentConfig {
            name: "llm".to_string(),
            ports: vec![8081, 8082],
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url,
            binary_name: Some("llama-server".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: Vec::new(),
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: Vec::new(),
            pre_install_cmds_windows: vec![
                "if not exist {{DATA_PATH}} mkdir {{DATA_PATH}}".to_string(),
                "if not exist {{LOGS_PATH}} mkdir {{LOGS_PATH}}".to_string(),
            ],
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::new(),
            data_download_list: vec![
                get_model_url("deepseek_small").unwrap_or_else(String::new),
                get_model_url("bge_embedding").unwrap_or_else(String::new),
            ],
            // The llama.cpp release archives unpack to `bin/`, and every consumer
            // of the local model calls it over plain HTTP (`http://127.0.0.1:8081`,
            // `http://localhost:8082/v1/embeddings`). Do not pass TLS flags: the
            // stack has no certificates for these ports.
            exec_cmd: "nohup {{BIN_PATH}}/bin/llama-server --port 8081 --host 127.0.0.1 -m {{DATA_PATH}}/DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf --ubatch-size 512 > {{LOGS_PATH}}/llm.log 2>&1 & nohup {{BIN_PATH}}/bin/llama-server --port 8082 --host 127.0.0.1 -m {{DATA_PATH}}/bge-small-en-v1.5-f32.gguf --embeddings --pooling mean --n-gpu-layers 0 --ctx-size 512 --ubatch-size 512 > {{LOGS_PATH}}/embedding.log 2>&1 &".to_string(),
            check_cmd: "curl -fsS --connect-timeout 2 -m 5 http://127.0.0.1:8081/health >/dev/null 2>&1 && curl -fsS --connect-timeout 2 -m 5 http://127.0.0.1:8082/health >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\bin\\llama-server.exe --port 8081 --host 127.0.0.1 -m {{DATA_PATH}}\\DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf --ubatch-size 512 > {{LOGS_PATH}}\\llm.log 2>&1".to_string()),
            check_cmd_windows: Some("curl -fsS --connect-timeout 2 -m 5 http://127.0.0.1:8081/health >NUL 2>&1 && curl -fsS --connect-timeout 2 -m 5 http://127.0.0.1:8082/health >NUL 2>&1".to_string()),
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
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("email"),
            binary_name: Some("stalwart-mail".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: Vec::new(),
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: Vec::new(),
            pre_install_cmds_windows: Vec::new(),
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::from([
                ("STALWART_TLS_ENABLE".to_string(), "true".to_string()),
                ("STALWART_TLS_CERT".to_string(), "{{CONF_PATH}}/system/certificates/email/server.crt".to_string()),
                ("STALWART_TLS_KEY".to_string(), "{{CONF_PATH}}/system/certificates/email/server.key".to_string()),
            ]),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/stalwart-mail --config {{CONF_PATH}}/email/config.toml".to_string(),
            check_cmd: "curl -f -k --connect-timeout 2 -m 5 https://localhost:8025/health >/dev/null 2>&1".to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\stalwart-mail.exe --config {{CONF_PATH}}\\email\\config.toml".to_string()),
            check_cmd_windows: Some("curl -f -k --connect-timeout 2 -m 5 https://localhost:8025/health >NUL 2>&1".to_string()),
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
            dependencies: Vec::new(),
            linux_packages: Vec::new(),
            macos_packages: Vec::new(),
            windows_packages: Vec::new(),
            download_url: get_component_url("proxy"),
            binary_name: Some("caddy".to_string()),
            pre_install_cmds_linux: Vec::new(),
            post_install_cmds_linux: vec![
                "setcap 'cap_net_bind_service=+ep' {{BIN_PATH}}/caddy".to_string(),
            ],
            pre_install_cmds_macos: Vec::new(),
            post_install_cmds_macos: Vec::new(),
            pre_install_cmds_windows: Vec::new(),
            post_install_cmds_windows: Vec::new(),
            env_vars: HashMap::from([(
                "XDG_DATA_HOME".to_string(),
                "{{DATA_PATH}}".to_string(),
            )]),
            data_download_list: Vec::new(),
            exec_cmd: "{{BIN_PATH}}/caddy run --config {{CONF_PATH}}/Caddyfile".to_string(),
            check_cmd: "curl -f --connect-timeout 2 -m 5 http://localhost >/dev/null 2>&1"
                .to_string(),
            exec_cmd_windows: Some("start /B {{BIN_PATH}}\\caddy.exe run --config {{CONF_PATH}}\\Caddyfile".to_string()),
            check_cmd_windows: Some("curl -f --connect-timeout 2 -m 5 http://localhost >NUL 2>&1".to_string()),
            container: None,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_component_uses_otterio_binary_and_oc_archive_extraction() {
        let mut components = HashMap::new();
        register_drive(&mut components);
        let registered = components.get("drive").is_some_and(|drive| {
            drive.binary_name.as_deref() == Some("otterio")
                && drive.env_vars.contains_key("OTTERIO_ROOT_USER")
                && drive.env_vars.contains_key("OTTERIO_ROOT_PASSWORD")
                && drive.check_cmd.contains("/otterio/health/live")
                && drive.exec_cmd.contains("{{BIN_PATH}}/otterio server")
                && drive.post_install_cmds_linux.iter().any(|c| {
                    c.contains("tar -xzf") && c.contains("name oc")
                })
                && drive.post_install_cmds_windows.iter().any(|c| {
                    c.contains("Expand-Archive") && c.contains("oc.exe")
                })
                // The installer key is stable even though the payload is now
                // the OC archive.
                && !drive.data_download_list.is_empty()
        });
        assert!(registered, "drive component must use OtterIO and OC");
    }

    /// #2562 — the cache ships as `valkey-server` (Linux) / `redis-server.exe`
    /// (Windows). `windows_binary_name_from_exec` must return the Windows name
    /// so `is_installed` finds the binary even though `binary_name` says valkey.
    #[test]
    fn cache_has_no_windows_exec_command() {
        let mut components = HashMap::new();
        register_cache(&mut components);
        let cache = components.get("cache").expect("cache component");
        assert_eq!(cache.binary_name.as_deref(), Some("valkey-server"));
        assert!(cache.exec_cmd_windows.is_none(), "Windows has no bundled cache binary");
        assert!(cache.check_cmd_windows.is_none());
    }

    #[test]
    fn tables_probes_initdb_before_creating_the_cluster() {
        let mut components = HashMap::new();
        register_tables(&mut components);
        let tables = components.get("tables").expect("tables component");
        let probe = tables
            .post_install_cmds_linux
            .iter()
            .position(|cmd| cmd.contains("initdb --version"))
            .expect("libstdc++ compatibility probe");
        let initdb = tables
            .post_install_cmds_linux
            .iter()
            .position(|cmd| cmd.contains("initdb -D"))
            .expect("initdb command");
        assert!(
            probe < initdb,
            "the libstdc++ probe must run before initdb creates pgdata"
        );
    }

    /// #2785 — a fresh install must never emit deprecated MD5 authentication.
    #[test]
    fn tables_fresh_install_uses_scram_and_no_md5() {
        let mut components = HashMap::new();
        register_tables(&mut components);
        let tables = components.get("tables").expect("tables component");

        for cmd in tables
            .post_install_cmds_linux
            .iter()
            .chain(tables.post_install_cmds_windows.iter())
        {
            assert!(
                !cmd.contains("md5"),
                "generated PostgreSQL config must not contain md5: {cmd}"
            );
        }

        let hba = tables
            .post_install_cmds_linux
            .iter()
            .find(|cmd| cmd.contains("pg_hba.conf"))
            .expect("pg_hba.conf generation command");
        assert!(
            hba.contains("hostssl all all all scram-sha-256"),
            "pg_hba must use SCRAM: {hba}"
        );

        assert!(
            tables
                .post_install_cmds_linux
                .iter()
                .any(|cmd| cmd.contains("password_encryption = 'scram-sha-256'")),
            "postgresql.conf must pin password_encryption to scram-sha-256"
        );

        let initdb = tables
            .post_install_cmds_linux
            .iter()
            .find(|cmd| cmd.contains("initdb -D"))
            .expect("initdb command");
        assert!(
            initdb.contains("--auth-local=scram-sha-256")
                && initdb.contains("--auth-host=scram-sha-256")
                && initdb.contains("-c password_encryption=scram-sha-256"),
            "initdb must create the SCRAM verifier and cluster config: {initdb}"
        );
    }
}
