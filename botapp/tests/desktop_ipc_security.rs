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

    // #2688 split the capabilities: the local-only wizard set lives in
    // default.json, while the sidecar-facing main WebView set lives in main.json.
    let main = config_json("capabilities/main.json");
    let urls = main["remote"]["urls"]
        .as_array()
        .expect("remote.urls must be a list in main.json");
    let urls: Vec<&str> = urls.iter().filter_map(|entry| entry.as_str()).collect();
    for url in &urls {
        assert!(!url.contains('*'), "wildcard remote origin {url}");
        assert!(!url.contains(":3000"), "dev-server remote origin {url}");
    }
    // The appliance window is navigated to the sidecar by wizard/window.rs.
    assert!(urls.contains(&"http://127.0.0.1:8080"));
    assert!(urls.contains(&"http://localhost:8080"));

    for file in ["capabilities/default.json", "capabilities/main.json"] {
        let raw = read_config(file).to_ascii_lowercase();
        assert!(!raw.contains("http://**"), "broad HTTP plugin permission remains in {file}");
        assert!(!raw.contains("https://**"), "broad HTTP plugin permission remains in {file}");
    }
}

/// #2688/#2698 — privilege separation. The sidecar origin (main window) gets
/// an explicit minimal permission list; every privileged command stays on the
/// local-only wizard capability. A script running on the sidecar must not be
/// able to invoke `wizard_reset_stack`, `delete_path`, `configure_remote`,
/// geolocation or notifications through IPC.
#[test]
fn remote_origin_capability_is_least_privilege() {
    let local = config_json("capabilities/default.json");
    assert_eq!(local["local"].as_bool(), Some(true));
    assert!(local["remote"].is_null(), "local wizard capability must have no remote origins");
    let windows = local["windows"].as_array().expect("wizard windows list");
    assert!(windows.iter().all(|w| w.as_str() != Some("main")));
    let local_perms = local["permissions"].as_array().expect("wizard permissions");
    assert!(local_perms.iter().any(|p| p.as_str() == Some("default")));

    let remote = config_json("capabilities/main.json");
    assert_eq!(
        remote["local"].as_bool(),
        Some(false),
        "sidecar capability must be remote-only"
    );
    let windows = remote["windows"].as_array().expect("main windows list");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].as_str(), Some("main"));
    let urls = remote["remote"]["urls"].as_array().expect("main remote origins");
    let urls: Vec<&str> = urls.iter().filter_map(|entry| entry.as_str()).collect();
    assert!(urls.contains(&"http://127.0.0.1:8080"));
    assert!(urls.contains(&"http://localhost:8080"));

    let permissions = remote["permissions"].as_array().expect("main permissions");
    let permissions: Vec<&str> = permissions.iter().filter_map(|p| p.as_str()).collect();
    assert!(!permissions.contains(&"default"), "remote origin got the full command set");
    let raw = read_config("capabilities/main.json").to_ascii_lowercase();
    for forbidden in [
        "allow-wizard-",
        "allow-delete-path",
        "allow-create-folder",
        "allow-set-sync-folder",
        "allow-configure-remote",
        "allow-request-destructive-confirmation",
        "geolocation:",
        "notification:",
    ] {
        assert!(!raw.contains(forbidden), "remote origin got privileged permission: {forbidden}");
    }

    // Applies to every shipped capability, not only the two named above.
    for file in std::fs::read_dir(Path::new(MANIFEST).join("capabilities"))
        .expect("capabilities directory")
        .flatten()
    {
        let path = file.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(cap) = serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(&path).unwrap_or_default(),
        ) else {
            continue;
        };
        let has_remote = cap["remote"]["urls"]
            .as_array()
            .is_some_and(|urls| !urls.is_empty());
        if !has_remote {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_default().to_ascii_lowercase();
        assert!(!source.contains("\"default\""), "remote capability has full ACL: {path:?}");
        assert!(!source.contains("allow-wizard-"), "remote capability has wizard IPC: {path:?}");
    }
}

/// Resolve a capability's permission identifiers to the app command names
/// Tauri actually checks at IPC time. The mapping is read from the build-time
/// generated ACL manifest, the same data the runtime permission table is built
/// from, so a renamed permission cannot hide a privileged command. `core:*`
/// grants are Tauri built-ins and are not app commands.
fn granted_app_commands<'a>(
    capability: &'a serde_json::Value,
    permissions: &'a serde_json::Map<String, serde_json::Value>,
    default_set: &'a serde_json::Value,
) -> std::collections::HashSet<&'a str> {
    use std::collections::HashSet;

    let mut commands = HashSet::new();
    let Some(grants) = capability["permissions"].as_array() else {
        return commands;
    };

    // `default` is a permission *set*: a list of identifiers, not a command
    // list. Expand it once before resolving identifiers to commands.
    let mut identifiers: Vec<&str> = Vec::new();
    for grant in grants {
        match grant.as_str() {
            Some("default") => {
                if let Some(nested) = default_set["permissions"].as_array() {
                    identifiers.extend(nested.iter().filter_map(|value| value.as_str()));
                }
            }
            Some(identifier) => identifiers.push(identifier),
            None => {}
        }
    }

    for identifier in identifiers {
        if identifier.starts_with("core:") {
            continue;
        }
        let Some(entry) = permissions.get(identifier) else {
            continue;
        };
        if let Some(allow) = entry["commands"]["allow"].as_array() {
            commands.extend(allow.iter().filter_map(|value| value.as_str()));
        }
    }
    commands
}

/// Acceptance test for #2688: a script running on the sidecar origin gets its
/// `invoke('wizard_reset_stack')` rejected because no permission granted to the
/// `main` window resolves to that command. The same command must remain callable
/// from the local-only wizard window so first-run provisioning still works.
#[test]
fn remote_origin_scripts_cannot_invoke_wizard_reset_stack() {
    use std::collections::HashSet;

    let acl = config_json("gen/schemas/acl-manifests.json");
    let app_acl = &acl["__app-acl__"];
    let permissions = app_acl["permissions"]
        .as_object()
        .expect("__app-acl__.permissions must be an object");
    let default_set = &app_acl["default_permission"];

    let local = config_json("capabilities/default.json");
    let remote = config_json("capabilities/main.json");

    let local_commands = granted_app_commands(&local, permissions, default_set);
    assert!(
        local_commands.contains("wizard_reset_stack"),
        "local wizard capability lost wizard_reset_stack"
    );

    // Every non-core grant on the sidecar must resolve in the app ACL; an
    // unknown identifier could otherwise silently under-report its commands.
    for grant in remote["permissions"]
        .as_array()
        .expect("main.json permissions must be a list")
    {
        let identifier = grant
            .as_str()
            .expect("permission identifier must be a string");
        if identifier.starts_with("core:") {
            continue;
        }
        assert!(
            permissions.contains_key(identifier),
            "unresolved permission {identifier} in main.json"
        );
    }

    let remote_commands = granted_app_commands(&remote, permissions, default_set);
    for command in [
        "wizard_reset_stack",
        "wizard_provision",
        "wizard_save_admin",
        "request_reset_stack_confirmation",
        "delete_path",
        "create_folder",
        "set_sync_folder",
        "configure_remote",
    ] {
        assert!(
            !remote_commands.contains(command),
            "sidecar origin can invoke {command} via IPC"
        );
    }

    // Positive control: the minimal read-only/download set the suite needs.
    let expected: HashSet<&str> = ["save_download", "check_release_update", "open_release_url"]
        .into_iter()
        .collect();
    assert_eq!(remote_commands, expected, "main.json command set drifted");
}
