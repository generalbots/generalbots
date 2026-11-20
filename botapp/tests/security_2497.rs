//! Regression tests for #2497 — desktop shell filesystem and origin hardening.
//!
//! The shell used to hand every `http://localhost:*` page an unrestricted
//! `invoke` bridge and let it pass arbitrary absolute paths to the file
//! commands. These tests pin the two halves of the fix: the commands reject
//! traversal/relative input before touching the filesystem, and the shipped
//! configuration keeps a real CSP plus a narrow remote-origin ACL.

use std::path::Path;

const MANIFEST: &str = env!("CARGO_MANIFEST_DIR");

fn read_config(file: &str) -> String {
    let path = Path::new(MANIFEST).join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn config_json(file: &str) -> serde_json::Value {
    let raw = read_config(file);
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("invalid JSON in {file}: {e}"))
}

/// The issue's proof-of-concept: an injected script calling
/// `invoke('delete_path', { path: '/var/lib/gbdesk' })` must not wipe the
/// appliance state. The path is either outside a configured scope or is a
/// scope root itself, and both cases are refused.
#[test]
fn the_appliance_state_root_cannot_be_deleted() {
    assert!(
        botapp::desktop::drive::delete_path("/var/lib/gbdesk").is_err(),
        "delete_path accepted the appliance state root"
    );
}

/// `..` and relative inputs must be refused by every file command before any
/// filesystem access: no canonicalisation, scope or confirmation is involved
/// because the input itself is rejected outright (#2497).
#[test]
fn file_commands_reject_traversal_and_relative_paths() {
    const BAD: [&str; 3] = ["../etc/passwd", "drive/../../etc/passwd", "/tmp/../etc/passwd"];
    for path in BAD {
        assert!(
            botapp::desktop::fs_scope::resolve(path).is_err(),
            "scope accepted traversal path {path}"
        );
        assert!(
            botapp::desktop::drive::delete_path(path).is_err(),
            "delete_path accepted traversal path {path}"
        );
        assert!(
            botapp::desktop::drive::list_files(path).is_err(),
            "list_files accepted traversal path {path}"
        );
    }
}

/// A real, local-only CSP must be shipped: no `null`, no inline/eval scripts,
/// no CDN hosts. Tauri adds hashes/nonces for the app's own inline scripts and
/// styles at build time, so `'self'` keeps the bundled pages working.
#[test]
fn tauri_config_sets_a_local_only_csp() {
    let config = config_json("tauri.conf.json");
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("app.security.csp must be a string, not null");
    assert!(csp.contains("default-src 'self'"), "CSP: {csp}");
    assert!(csp.contains("object-src 'none'"), "CSP: {csp}");
    assert!(!csp.contains("unsafe-inline"), "CSP: {csp}");
    assert!(!csp.contains("unsafe-eval"), "CSP: {csp}");

    let lowered = csp.to_ascii_lowercase();
    for cdn in ["cdn.", "unpkg", "jsdelivr", "cdnjs", "googleapis"] {
        assert!(!lowered.contains(cdn), "CSP references CDN {cdn}: {csp}");
    }
}

/// The capability grants IPC to the app origin plus the loopback sidecar that
/// actually serves the packaged suite. The dev server (`localhost:3000`) is
/// already a Tauri-local origin, so it needs no remote grant, and the broad
/// `http:default` plugin permission (wildcard `http://**`) is gone.
#[test]
fn capability_origins_are_scoped_to_shell_and_sidecar() {
    let caps = config_json("capabilities/default.json");
    assert_eq!(caps["local"].as_bool(), Some(true));

    let urls = caps["remote"]["urls"]
        .as_array()
        .expect("remote.urls must be a list");
    let urls: Vec<&str> = urls.iter().filter_map(|entry| entry.as_str()).collect();
    for url in &urls {
        assert!(!url.contains('*'), "wildcard remote origin {url}");
        assert!(!url.contains(":3000"), "dev-server remote origin {url}");
    }
    // The appliance window is navigated to the sidecar by wizard/window.rs.
    assert!(urls.contains(&"http://127.0.0.1:8080"));
    assert!(urls.contains(&"http://localhost:8080"));

    let raw = read_config("capabilities/default.json").to_ascii_lowercase();
    assert!(!raw.contains("http://**"), "broad HTTP plugin permission remains");
    assert!(!raw.contains("https://**"), "broad HTTP plugin permission remains");
}
