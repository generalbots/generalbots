//! A real move on a throw-away stack, run by hand:
//!
//! `GB_OPENBAO_REHEARSAL=<dir with vault.exe and bao.exe> cargo test -p
//! botcorepkg rehearsal -- --ignored --nocapture --test-threads=1`
//!
//! Each case builds `<dir>/<case>/stack` with copies of both programs, a
//! former Vault on port `GB_OPENBAO_REHEARSAL_PORT` (default 18300) holding
//! made-up secrets, and moves it. Only counts and comparisons are printed.

use super::api::Api;
use super::layout::Layout;
use super::server;
use super::tree::{Fields, KvStore, KvVersion, Mount};
use super::{Fault, Migration, Outcome, Stage};
use crate::protected_file::write_secret;
use crate::secrets_engine::{active_engine, Engine, BIN_DIR, MIGRATED_MARKER, MIGRATING_MARKER};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(30);

fn mounts() -> Vec<Mount> {
    vec![
        Mount { path: "secret/".into(), version: KvVersion::V2 },
        Mount { path: "gbo-extra/".into(), version: KvVersion::V2 },
        Mount { path: "kv1/".into(), version: KvVersion::V1 },
    ]
}

fn fake_secrets() -> Vec<(usize, &'static str, serde_json::Value)> {
    vec![
        (0, "gbo/tables", json!({"username": "gbuser", "password": "fake-tables-pw", "port": "5432"})),
        (0, "gbo/llm", json!({"url": "http://127.0.0.1:1/v1", "openai_key": "fake-llm-key"})),
        (0, "gbo/drive", json!({"accesskey": "fake-ak", "secret": "fake-sk"})),
        (0, "gbo/00000000-0000-0000-0000-000000000000/branch/bot1", json!({"llm-key": "fake-bot-key", "llm-model": "m"})),
        (0, "gbo/directory/admin", json!({"password": "fake-admin"})),
        (1, "a/b/c/deep", json!({"x": "1", "nested": {"k": ["v"]}})),
        (1, "top", json!({"y": "2"})),
        (2, "one", json!({"v": "kv1-value"})),
    ]
}

fn fields(value: &serde_json::Value) -> Fields {
    value.as_object().cloned().unwrap_or_default()
}

fn port() -> u16 {
    std::env::var("GB_OPENBAO_REHEARSAL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(18300)
}

fn build_stack(root: &Path, case: &str) -> PathBuf {
    let stack = root.join(case).join("stack");
    let _ = std::fs::remove_dir_all(root.join(case));
    let bin = stack.join(BIN_DIR);
    assert!(std::fs::create_dir_all(&bin).is_ok());
    for engine in [Engine::LegacyVault, Engine::OpenBao] {
        assert!(std::fs::copy(root.join(engine.file_name()), engine.path_in(&stack)).is_ok());
    }
    assert!(std::fs::create_dir_all(stack.join("data/vault")).is_ok());
    let data = stack.join("data/vault").to_string_lossy().replace('\\', "/");
    let p = port();
    let config = format!(
        "storage \"file\" {{\n  path = \"{data}\"\n}}\nlistener \"tcp\" {{\n  address = \"127.0.0.1:{p}\"\n  cluster_address = \"127.0.0.1:{}\"\n  tls_disable = true\n}}\napi_addr = \"http://127.0.0.1:{p}\"\ncluster_addr = \"http://127.0.0.1:{}\"\nui = false\ndisable_mlock = true\n",
        p + 1,
        p + 1
    );
    assert!(std::fs::create_dir_all(stack.join("conf/vault")).is_ok());
    assert!(std::fs::write(stack.join(server::CONFIG_FILE), config).is_ok());
    stack
}

fn addr() -> String {
    format!("http://127.0.0.1:{}", port())
}

/// Former Vault initialised the way the stack does it, with the made-up
/// secrets, a policy and a sealed server token; stopped afterwards.
async fn seed_former_vault(stack: &Path) {
    let Ok(api) = Api::new(&addr()) else { panic!("client") };
    let Ok(mut child) = server::start(Engine::LegacyVault, stack) else { panic!("start vault") };
    assert!(api.wait_answering(WAIT).await.is_ok());
    let Ok(init) = api.init(5, 3).await else { panic!("init") };
    assert!(Layout::new(stack).write_init(&init, 3).is_ok());
    assert!(write_secret(&stack.join(super::TOKEN_FILE), "s.fake-scoped-token").is_ok());
    assert!(api.unseal(&init.keys_b64[..3]).await.is_ok());
    let root = api.with_token(&init.root_token);
    for mount in mounts() {
        assert!(root.enable_kv(&mount).await.is_ok(), "mount {}", mount.path);
    }
    for (i, path, value) in fake_secrets() {
        assert!(root.write(&mounts()[i], path, &fields(&value)).await.is_ok(), "write {path}");
    }
    assert!(root.put_policy("gbo-server", "path \"secret/data/gbo/*\" { capabilities = [\"read\"] }").await.is_ok());
    server::stop(&mut child);
    assert!(api.wait_silent(WAIT).await.is_ok());
}

/// Starts whatever the stack runs now, unseals it with its sealed keys and
/// checks every made-up secret; returns the process for the caller to stop.
async fn check_serving(stack: &Path, expect: Engine) -> std::process::Child {
    assert_eq!(active_engine(stack), expect);
    let Ok(api) = Api::new(&addr()) else { panic!("client") };
    let Ok(child) = server::start(expect, stack) else { panic!("start") };
    assert!(api.wait_answering(WAIT).await.is_ok());
    let layout = Layout::new(stack);
    let keys = layout.unseal_keys().unwrap_or_default();
    assert!(api.unseal(&keys[..3]).await.is_ok());
    let root = api.with_token(&layout.root_token().unwrap_or_default());
    let mut equal = 0;
    for (i, path, value) in fake_secrets() {
        if root.read(&mounts()[i], path).await.ok().flatten() == Some(fields(&value)) {
            equal += 1;
        }
    }
    println!("  {} serving: {equal}/{} made-up secrets read back equal", expect.binary_name(), fake_secrets().len());
    assert_eq!(equal, fake_secrets().len());
    child
}

fn root_dir() -> Option<PathBuf> {
    std::env::var("GB_OPENBAO_REHEARSAL").ok().map(PathBuf::from)
}

fn migration(stack: &Path, fault: Option<Fault>) -> Migration {
    let mut m = Migration::new(stack.to_path_buf(), addr());
    m.ready_timeout = WAIT;
    m.fault = fault;
    m
}

fn kill(pid: u32, stack: &Path) {
    server::stop_pid(pid, stack);
}

#[tokio::test]
#[ignore = "runs real secrets servers; see the module docs"]
async fn rehearsal_move_succeeds() {
    let Some(root) = root_dir() else { return };
    let stack = build_stack(&root, "migrate");
    seed_former_vault(&stack).await;
    let outcome = migration(&stack, None).run().await;
    let Outcome::Migrated(report) = outcome else { panic!("not migrated: {outcome:?}") };
    println!("migrate: before {:?}", report.before);
    println!("migrate: after  {:?}, policies {}", report.after, report.policies);
    assert_eq!(report.before, report.after);
    kill(report.openbao_pid, &stack);
    assert!(stack.join(MIGRATED_MARKER).is_file());
    let backups = std::fs::read_dir(stack.join("data")).map(|d| d.count()).unwrap_or_default();
    println!("  data folders now: {backups} (storage + kept former storage)");
    assert!(!stack.join(super::TOKEN_FILE).exists());
    let mut child = check_serving(&stack, Engine::OpenBao).await;
    server::stop(&mut child);
}

#[tokio::test]
#[ignore = "runs real secrets servers; see the module docs"]
async fn rehearsal_killed_openbao_rolls_back() {
    let Some(root) = root_dir() else { return };
    let stack = build_stack(&root, "rollback");
    seed_former_vault(&stack).await;
    let outcome = migration(&stack, Some(Fault::KillOpenBaoAfterWrites(3))).run().await;
    println!("rollback: outcome {outcome:?}");
    assert!(matches!(outcome, Outcome::RolledBack(_)));
    assert!(!stack.join(MIGRATING_MARKER).exists());
    assert!(stack.join(super::TOKEN_FILE).is_file());
    let mut child = check_serving(&stack, Engine::LegacyVault).await;
    server::stop(&mut child);
}

#[tokio::test]
#[ignore = "runs real secrets servers; see the module docs"]
async fn rehearsal_crash_is_recovered_then_retried() {
    let Some(root) = root_dir() else { return };
    let stack = build_stack(&root, "crash");
    seed_former_vault(&stack).await;
    let crashed = migration(&stack, Some(Fault::CrashAt(Stage::OpenBaoInitialised))).run().await;
    println!("crash: first run {crashed:?}, migrating marker {}", stack.join(MIGRATING_MARKER).exists());
    assert_eq!(crashed, Outcome::Crashed);
    assert_eq!(active_engine(&stack), Engine::LegacyVault);
    let outcome = migration(&stack, None).run().await;
    let Outcome::Migrated(report) = outcome else { panic!("not migrated: {outcome:?}") };
    println!("crash: second run before {:?} after {:?}", report.before, report.after);
    assert_eq!(report.before, report.after);
    kill(report.openbao_pid, &stack);
    let mut child = check_serving(&stack, Engine::OpenBao).await;
    server::stop(&mut child);
}
