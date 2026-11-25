use super::layout::{parse_unseal_keys, Layout, MarkerData};
use super::tree::{export, import, verify, Fields, KvStore, KvVersion, Mount, Snapshot};
use crate::secrets_engine::{StackFacts, DATA_DIR, MIGRATED_MARKER, MIGRATING_MARKER};
use anyhow::Result;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// An in-memory KV store answering LIST like the real API: direct children,
/// folders with a trailing `/`.
#[derive(Default)]
struct FakeStore {
    data: Mutex<BTreeMap<(String, String), Fields>>,
    deleted: Vec<(String, String)>,
}

impl FakeStore {
    fn with(entries: &[(&str, &str, serde_json::Value)]) -> Self {
        let store = Self::default();
        if let Ok(mut data) = store.data.lock() {
            for (mount, path, value) in entries {
                if let Some(fields) = value.as_object() {
                    data.insert(((*mount).to_string(), (*path).to_string()), fields.clone());
                }
            }
        }
        store
    }
}

impl KvStore for FakeStore {
    async fn list(&self, mount: &Mount, dir: &str) -> Result<Vec<String>> {
        let data = self.data.lock().map_err(|_| anyhow::anyhow!("lock"))?;
        let mut keys: Vec<String> = data
            .keys()
            .chain(self.deleted.iter())
            .filter(|(m, p)| *m == mount.path && p.starts_with(dir))
            .map(|(_, p)| {
                let rest = &p[dir.len()..];
                match rest.split_once('/') {
                    Some((folder, _)) => format!("{folder}/"),
                    None => rest.to_string(),
                }
            })
            .collect();
        keys.sort();
        keys.dedup();
        Ok(keys)
    }

    async fn read(&self, mount: &Mount, path: &str) -> Result<Option<Fields>> {
        let data = self.data.lock().map_err(|_| anyhow::anyhow!("lock"))?;
        Ok(data.get(&(mount.path.clone(), path.to_string())).cloned())
    }

    async fn write(&self, mount: &Mount, path: &str, fields: &Fields) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow::anyhow!("lock"))?;
        data.insert((mount.path.clone(), path.to_string()), fields.clone());
        Ok(())
    }
}

fn mounts() -> Vec<Mount> {
    vec![
        Mount { path: "secret/".into(), version: KvVersion::V2 },
        Mount { path: "extra/".into(), version: KvVersion::V1 },
    ]
}

fn source() -> FakeStore {
    let mut store = FakeStore::with(&[
        ("secret/", "gbo/tables", json!({"password": "p1", "port": 5432})),
        ("secret/", "gbo/llm", json!({"url": "u", "key": "k"})),
        ("secret/", "gbo/org/branch/bot", json!({"llm-key": "b"})),
        ("extra/", "a/b/c", json!({"x": "1"})),
        ("other/", "ignored", json!({"y": "2"})),
    ]);
    // A path whose latest version was deleted: listed, but reads nothing.
    store.deleted.push(("secret/".into(), "gbo/gone".into()));
    store
}

#[tokio::test]
async fn the_walk_reads_every_nested_secret_of_every_mount() {
    let snapshot = export(&source(), &mounts()).await;
    let counts = snapshot.map(|s| s.counts()).unwrap_or_default();
    assert_eq!(counts, vec![("secret/".to_string(), 3), ("extra/".to_string(), 1)]);
}

#[tokio::test]
async fn writing_back_gives_an_equal_copy() {
    let Ok(snapshot) = export(&source(), &mounts()).await else { panic!("export") };
    let target = FakeStore::default();
    let mut seen = Vec::new();
    let written = import(&target, &snapshot, |n| seen.push(n)).await.unwrap_or_default();
    assert_eq!(written, 4);
    assert_eq!(seen, vec![1, 2, 3, 4]);
    let Ok(back) = export(&target, &mounts()).await else { panic!("export back") };
    assert!(verify(&snapshot, &back).is_ok());
}

#[tokio::test]
async fn verification_catches_a_missing_or_changed_secret() {
    let Ok(snapshot) = export(&source(), &mounts()).await else { panic!("export") };
    let partial = FakeStore::with(&[
        ("secret/", "gbo/tables", json!({"password": "p1", "port": 5432})),
        ("secret/", "gbo/llm", json!({"url": "u", "key": "k"})),
        ("extra/", "a/b/c", json!({"x": "1"})),
    ]);
    let Ok(short) = export(&partial, &mounts()).await else { panic!("export short") };
    let message = verify(&snapshot, &short).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("2 secrets after the move, 3 before"), "{message}");

    let changed = FakeStore::with(&[
        ("secret/", "gbo/tables", json!({"password": "other", "port": 5432})),
        ("secret/", "gbo/llm", json!({"url": "u", "key": "k"})),
        ("secret/", "gbo/org/branch/bot", json!({"llm-key": "b"})),
        ("extra/", "a/b/c", json!({"x": "1"})),
    ]);
    let Ok(diff) = export(&changed, &mounts()).await else { panic!("export diff") };
    let message = verify(&snapshot, &diff).err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("secret/gbo/tables differs"), "{message}");
    assert!(!message.contains("other"), "values must not appear in errors");
}

#[test]
fn urls_follow_the_kv_version() {
    let v2 = Mount { path: "secret/".into(), version: KvVersion::V2 };
    assert_eq!(v2.list_segments("gbo/"), vec!["secret", "metadata", "gbo", ""]);
    assert_eq!(v2.data_segments("gbo/tables"), vec!["secret", "data", "gbo", "tables"]);
    let v1 = Mount { path: "kv/".into(), version: KvVersion::V1 };
    assert_eq!(v1.list_segments(""), vec!["kv", ""]);
    assert_eq!(v1.data_segments("a"), vec!["kv", "a"]);
    let fields = json!({"a": "1"}).as_object().cloned().unwrap_or_default();
    assert_eq!(v2.write_body(&fields), json!({"data": {"a": "1"}}));
    assert_eq!(v1.fields_of(&json!({"data": {"a": "1"}})), Some(fields.clone()));
    assert_eq!(v2.fields_of(&json!({"data": {"data": null}})), None);
}

#[test]
fn only_kv_mounts_are_moved() {
    let table = json!({
        "secret/": {"type": "kv", "options": {"version": "2"}},
        "legacy/": {"type": "kv", "options": null},
        "cubbyhole/": {"type": "cubbyhole"},
        "sys/": {"type": "system"},
        "request_id": "x"
    });
    let found = super::api::kv_mounts_in(&table);
    assert_eq!(found, vec![
        Mount { path: "legacy/".into(), version: KvVersion::V1 },
        Mount { path: "secret/".into(), version: KvVersion::V2 },
    ]);
}

#[test]
fn unseal_keys_are_read_in_order() {
    let keys = parse_unseal_keys("VAULT_UNSEAL_KEY_1=a\r\nVAULT_UNSEAL_KEY_2=b\n\nOTHER=c\nVAULT_UNSEAL_KEY_3=\n");
    assert_eq!(keys, vec!["a", "b"]);
}

#[test]
fn a_snapshot_wipes_its_values() {
    let mut value = json!({"k": "secret", "n": ["x"]});
    super::tree::wipe_value(&mut value);
    assert_eq!(value, json!({"k": "", "n": [""]}));
    drop(Snapshot::default());
}

fn write(path: &std::path::Path, text: &str) {
    if let Some(parent) = path.parent() {
        assert!(std::fs::create_dir_all(parent).is_ok());
    }
    assert!(std::fs::write(path, text).is_ok());
}

#[test]
fn rollback_restores_storage_and_files_and_is_repeatable() {
    let Ok(dir) = tempfile::tempdir() else { return };
    let stack = dir.path();
    let layout = Layout::new(stack);
    write(&stack.join(DATA_DIR).join("core/old"), "former");
    write(&stack.join(super::INIT_FILE), "former-init");
    write(&stack.join(super::KEYS_FILE), "former-keys");
    write(&stack.join(super::TOKEN_FILE), "former-token");
    assert!(layout.begin("s1").is_ok());
    assert!(layout.record_pid(42).is_ok());
    assert_eq!(layout.marker().ok(), Some(MarkerData { stamp: "s1".into(), pids: vec![42] }));
    assert!(layout.set_aside_storage("s1").is_ok());
    assert!(layout.set_aside_files("s1").is_ok());
    assert!(!stack.join(super::INIT_FILE).exists());
    write(&stack.join(DATA_DIR).join("core/new"), "partial");
    write(&stack.join(super::INIT_FILE), "new-init");

    assert!(StackFacts::read(stack).migrating);
    assert!(layout.roll_back().is_ok());
    let read = |rel: &str| std::fs::read_to_string(stack.join(rel)).unwrap_or_default();
    assert_eq!(read("data/vault/core/old"), "former");
    assert!(!stack.join("data/vault/core/new").exists());
    assert_eq!(read(super::INIT_FILE), "former-init");
    assert_eq!(read(super::KEYS_FILE), "former-keys");
    assert_eq!(read(super::TOKEN_FILE), "former-token");
    assert!(!layout.backup_dir("s1").exists());
    assert!(!layout.aside_dir("s1").exists());
    assert!(!stack.join(MIGRATING_MARKER).exists());
    // Without a marker there is nothing left to roll back.
    assert!(layout.roll_back().is_err());
}

#[test]
fn a_crash_before_the_storage_moved_rolls_back_to_the_same_stack() {
    let Ok(dir) = tempfile::tempdir() else { return };
    let stack = dir.path();
    let layout = Layout::new(stack);
    write(&stack.join(DATA_DIR).join("core/old"), "former");
    assert!(layout.begin("s2").is_ok());
    assert!(layout.roll_back().is_ok());
    assert!(stack.join("data/vault/core/old").is_file());
    assert!(!stack.join(MIGRATING_MARKER).exists());
}

#[test]
fn finishing_writes_the_migrated_marker_last() {
    let Ok(dir) = tempfile::tempdir() else { return };
    let layout = Layout::new(dir.path());
    assert!(layout.begin("s3").is_ok());
    assert!(layout.finish().is_ok());
    assert!(dir.path().join(MIGRATED_MARKER).is_file());
    assert!(!dir.path().join(MIGRATING_MARKER).exists());
}
