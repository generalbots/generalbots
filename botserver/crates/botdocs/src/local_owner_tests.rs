//! Per-install owner expectations (#2135).

use super::*;
use std::path::PathBuf;
use uuid::Uuid;

struct TempConfig {
    dir: PathBuf,
}

impl TempConfig {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("gb-docs-owner-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("a temp directory can be created");
        Self { dir }
    }

    fn path(&self) -> PathBuf {
        self.dir.join("directory_config.json")
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn an_install_keeps_its_owner_across_resolutions() {
    let config = TempConfig::new();
    let first = resolve_from_config(&config.path());
    let second = resolve_from_config(&config.path());
    assert_eq!(first, second);
    assert!(first.starts_with(LOCAL_OWNER_PREFIX));
    assert_ne!(first, "default-user");
}

#[test]
fn two_installs_get_different_owners() {
    let first_config = TempConfig::new();
    let second_config = TempConfig::new();
    let first = resolve_from_config(&first_config.path());
    let second = resolve_from_config(&second_config.path());
    assert_ne!(first, second);
    assert_ne!(
        crate::storage_core::get_user_docs_path(&first),
        crate::storage_core::get_user_docs_path(&second)
    );
}

#[test]
fn a_persisted_owner_is_reused() {
    let config = TempConfig::new();
    std::fs::write(
        config.path(),
        r#"{"saas_jwt_secret":"keep-me","docs_local_owner":"local-existing"}"#,
    )
    .expect("the fixture can be written");
    assert_eq!(resolve_from_config(&config.path()), "local-existing");

    // The rest of the install configuration survives the write.
    let stored = std::fs::read_to_string(config.path()).expect("the config is readable");
    assert!(stored.contains("keep-me"));
}

#[test]
fn a_legacy_persisted_owner_is_replaced() {
    let config = TempConfig::new();
    std::fs::write(config.path(), r#"{"docs_local_owner":"default-user"}"#)
        .expect("the fixture can be written");
    let owner = resolve_from_config(&config.path());
    assert!(owner.starts_with(LOCAL_OWNER_PREFIX));
    assert_ne!(owner, "default-user");
}

#[test]
fn only_a_usable_id_can_be_an_owner() {
    assert!(is_usable_owner("local-abc"));
    assert!(is_usable_owner("11111111-1111-1111-1111-111111111111"));
    assert!(!is_usable_owner(""));
    assert!(!is_usable_owner("   "));
    assert!(!is_usable_owner("default-user"));
    assert!(!is_usable_owner("Default-User"));
    assert!(!is_usable_owner("///"));
}
