use super::*;

const WINDOWS_VAULT: &str = "storage \"file\" { \r\n  path = \"C:/stack/data/vault\" \r\n} \r\n \r\nlistener \"tcp\" { \r\n  address = \"0.0.0.0:8200\" \r\n  tls_disable = true \r\n} \r\n \r\napi_addr = \"http://127.0.0.1:8200\" \r\ncluster_addr = \"http://127.0.0.1:8201\" \r\n";

#[test]
fn vault_listener_and_cluster_listener_move_to_loopback() {
    let hardened = harden_vault_hcl(WINDOWS_VAULT).unwrap_or_default();
    assert!(hardened.contains("  address = \"127.0.0.1:8200\""));
    assert!(hardened.contains("  cluster_address = \"127.0.0.1:8201\""));
    assert!(!hardened.contains("0.0.0.0"));
    assert!(hardened.contains("tls_disable = true"));
    assert!(hardened.contains("cluster_addr = \"http://127.0.0.1:8201\""));
}

#[test]
fn vault_hardening_is_idempotent() {
    let once = harden_vault_hcl(WINDOWS_VAULT).unwrap_or_default();
    assert_eq!(harden_vault_hcl(&once), None);
}

#[test]
fn vault_existing_cluster_address_is_rewritten_not_duplicated() {
    let text = "listener \"tcp\" {\n  address = \"0.0.0.0:8200\"\n  cluster_address = \"0.0.0.0:8201\"\n}\n";
    let hardened = harden_vault_hcl(text).unwrap_or_default();
    assert_eq!(hardened.matches("cluster_address").count(), 1);
    assert!(hardened.contains("cluster_address = \"127.0.0.1:8201\""));
}

#[test]
fn qdrant_gets_a_loopback_host_in_its_service_block() {
    let text = "service: \r\n  grpc_port: 6334 \r\n  http_port: 6333 \r\nstorage: \r\n  storage_path: C:\\stack\\data\\vector_db \r\nlog_level: INFO \r\n";
    let hardened = harden_qdrant_yaml(text).unwrap_or_default();
    let lines: Vec<&str> = hardened.lines().collect();
    assert_eq!(lines[0], "service:");
    assert_eq!(lines[1], "  host: 127.0.0.1");
    assert!(lines.contains(&"  http_port: 6333"));
    assert_eq!(harden_qdrant_yaml(&hardened), None);
}

#[test]
fn qdrant_wildcard_host_is_replaced() {
    let text = "storage:\n  storage_path: /s\n\nservice:\n  host: 0.0.0.0\n  http_port: 6333\n";
    let hardened = harden_qdrant_yaml(text).unwrap_or_default();
    assert!(hardened.contains("  host: 127.0.0.1"));
    assert!(!hardened.contains("0.0.0.0"));
    assert!(hardened.contains("  storage_path: /s"));
}

#[test]
fn qdrant_without_service_block_gets_one() {
    let hardened = harden_qdrant_yaml("log_level: INFO\n").unwrap_or_default();
    assert!(hardened.ends_with("service:\n  host: 127.0.0.1\n"));
}

#[test]
fn postgres_listen_addresses_is_set_once_to_loopback() {
    let text = "data_directory = 'x' \r\nport = 5432 \r\nlisten_addresses = '*' \r\nlisten_addresses = 'localhost'\r\n";
    let hardened = harden_postgresql_conf(text).unwrap_or_default();
    assert_eq!(hardened.matches("listen_addresses").count(), 1);
    assert!(hardened.contains("listen_addresses = '127.0.0.1,::1'"));
    assert_eq!(harden_postgresql_conf(&hardened), None);
}

#[test]
fn postgres_commented_default_gets_an_active_line() {
    let text = "#listen_addresses = 'localhost'\t\t# what IP address(es)\nport = 5432\n";
    let hardened = harden_postgresql_conf(text).unwrap_or_default();
    assert!(hardened.contains("#listen_addresses = 'localhost'"));
    assert!(hardened.ends_with("listen_addresses = '127.0.0.1,::1'\n"));
}

#[test]
fn pg_hba_trust_host_rules_become_scram_and_local_rules_stay() {
    let text = "# comment trust\nlocal   all  all                trust\nhost    all  all  127.0.0.1/32  trust\nhost    all  all  ::1/128       trust\nhost replication all 127.0.0.1 255.255.255.255 trust\n";
    let hardened = harden_pg_hba(text).unwrap_or_default();
    assert!(hardened.contains("# comment trust"));
    assert!(hardened.contains("local   all  all                trust"));
    assert!(hardened.contains("host\tall\tall\t127.0.0.1/32\tscram-sha-256"));
    assert!(hardened.contains("host\tall\tall\t::1/128\tscram-sha-256"));
    assert!(hardened.contains("255.255.255.255\tscram-sha-256"));
    assert!(pg_hba_is_hardened(&hardened));
    assert!(!pg_hba_is_hardened(text));
}

#[test]
fn pg_hba_password_rules_are_left_alone() {
    assert!(pg_hba_is_hardened("hostssl all all all md5\nhost all all 127.0.0.1/32 scram-sha-256\n"));
}

#[test]
fn apply_rewrites_only_when_needed_and_skips_missing_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yaml");
    assert!(!apply(&path, harden_qdrant_yaml).unwrap());
    std::fs::write(&path, "service:\n  http_port: 6333\n").unwrap();
    assert!(apply(&path, harden_qdrant_yaml).unwrap());
    assert!(!apply(&path, harden_qdrant_yaml).unwrap());
}

#[test]
fn plain_vault_secret_files_are_sealed_and_still_readable() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("conf/vault")).unwrap();
    let init = dir.path().join("conf/vault/init.json");
    std::fs::write(&init, "{}").unwrap();
    seal_plain_secret_files(dir.path());
    seal_plain_secret_files(dir.path());
    assert!(init.exists());
    let sealed = crate::protected_file::is_sealed(&std::fs::read(&init).unwrap());
    assert_eq!(sealed, cfg!(windows));
    assert_eq!(crate::protected_file::read_secret(&init).unwrap(), "{}");
    assert!(!dir.path().join("vault-unseal-keys").exists());
}
