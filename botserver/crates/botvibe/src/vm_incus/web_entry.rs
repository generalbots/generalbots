//! `vm_incus::web_entry` — split per #1443 (AGENTS.md 450-line rule).

/// Web-entry resolution for `run_dev_app` (#1276 regression surface).
///
/// Pure function so the static/python/node precedence rules are unit-testable
/// without a container:
///
/// * a python entry (`app.py`/`server.py`/`main.py`/`api.py`/`run.py`) runs
///   with `python3` — unless a node entry exists too (python never shadows a
///   real node app);
/// * a `package.json` `main` or `node <file>` start script wins over
///   conventional names (an agent may build the real app in `server.js`
///   while a starter `index.js` template still exists);
/// * `server.js` is a REAL web entrypoint and must never be clobbered by the
///   generated static fallback (previous bug class);
/// * only a project with neither node nor python entries gets the generated
///   static `server.js` (pure static site).
pub(crate) struct WebEntry {
    pub(crate) entry: String,
    pub(crate) is_python: bool,
    /// True when the project ships no web entry at all and needs the
    /// generated static `server.js` pushed (node projects with index.js,
    /// python apps and node apps do NOT need it).
    pub(crate) needs_static_fallback: bool,
}

pub(crate) fn python_entry_of(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    match name {
        "app.py" | "server.py" | "main.py" | "api.py" | "run.py" => Some(name),
        _ => None,
    }
}

/// Check a project-declared web entry before it reaches a systemd unit (#2655).
/// The value is interpolated into `ExecStart`, so it must be a plain relative
/// path: no absolute paths, no `..` traversal, no whitespace/newlines, no NUL,
/// no shell metacharacters and no systemd specifiers (`%`).
pub(crate) fn is_safe_entry(entry: &str) -> bool {
    !entry.is_empty()
        && !entry.starts_with('/')
        && entry.split('/').all(|part| !part.is_empty() && part != "..")
        && entry
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
}

/// Parse every `package.json` in the file set, ignoring malformed manifests.
fn manifest_json(files: &[serde_json::Value]) -> impl Iterator<Item = serde_json::Value> + '_ {
    files
        .iter()
        .filter(|file| file.get("path").and_then(|v| v.as_str()) == Some("package.json"))
        .filter_map(|file| {
            let content = file.get("content").and_then(|v| v.as_array())?;
            let bytes: Vec<u8> = content
                .iter()
                .filter_map(|v| v.as_u64().filter(|n| *n <= u8::MAX as u64).map(|n| n as u8))
                .collect();
            serde_json::from_slice(&bytes).ok()
        })
}

/// All project-declared entry candidates (`main` first, then a `node <file>`
/// start script). Both are validated so a hostile value in either is refused.
fn manifest_entries(manifest: &serde_json::Value) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(main) = manifest.get("main").and_then(|v| v.as_str()) {
        entries.push(main.to_string());
    }
    if let Some(start) = manifest
        .get("scripts")
        .and_then(|s| s.get("start"))
        .and_then(|v| v.as_str())
    {
        if let Some(entry) = start.strip_prefix("node ") {
            entries.push(entry.to_string());
        }
    }
    entries
}

/// Fail-closed gate called before a VM is provisioned (#2655): a single unsafe
/// `main` / `node <file>` entry aborts the deployment instead of being written
/// into `vibe-app.service`.
pub(crate) fn validate_manifest_entries(files: &[serde_json::Value]) -> Result<(), String> {
    for manifest in manifest_json(files) {
        for entry in manifest_entries(&manifest) {
            if !is_safe_entry(entry.as_str()) {
                return Err(format!("unsafe package.json web entry rejected: {entry:?}"));
            }
        }
    }
    Ok(())
}

/// Build the systemd `ExecStart` line from an already-validated entry (#2655).
/// The allowlist excludes `%`, quotes, whitespace and shell metacharacters, so
/// no quoting is required; `%` is doubled defensively in case the allowlist is
/// widened later (systemd treats `%` as a specifier).
pub(crate) fn exec_start_line(interpreter: &str, entry: &str) -> Result<String, String> {
    if !is_safe_entry(entry) {
        return Err(format!("unsafe web entry cannot be used in ExecStart: {entry:?}"));
    }
    let interpreter = interpreter.replace('%', "%%");
    let path = format!("/opt/vibe/app/{entry}").replace('%', "%%");
    Ok(format!("ExecStart={interpreter} {path}"))
}

pub(crate) fn resolve_web_entry(files: &[serde_json::Value]) -> WebEntry {
    let has_index_js = files.iter().any(|f| {
        f.get("path")
            .and_then(|v| v.as_str())
            .map(|p| p == "index.js" || p.ends_with("/index.js"))
            .unwrap_or(false)
    });
    // A node-framework project may designate its web entry point as
    // `server.js` instead of `index.js` (or a package.json "start" script).
    let has_server_js = files.iter().any(|f| {
        f.get("path")
            .and_then(|v| v.as_str())
            .map(|p| p == "server.js" || p.ends_with("/server.js"))
            .unwrap_or(false)
    });
    // An explicit web entry declared by the project (package.json "main" or
    // a "start" script of the form `node <file>`) wins over conventional
    // file names.
    // #2655 — only a validated relative path may become an ExecStart
    // argument. A hostile declared entry is ignored here (defense in depth);
    // `validate_manifest_entries` refuses the deployment up front.
    let declared_entry: Option<String> = manifest_json(files)
        .find_map(|manifest| manifest_entries(&manifest).into_iter().next())
        .filter(|entry| is_safe_entry(entry.as_str()));
    let has_node_entry = has_index_js || has_server_js || declared_entry.is_some();
    let python_entry = files
        .iter()
        .filter_map(|f| f.get("path").and_then(|v| v.as_str()).and_then(python_entry_of))
        .next();
    let is_python = python_entry.is_some() && !has_node_entry;
    if is_python {
        return WebEntry {
            entry: python_entry.unwrap_or("app.py").to_string(),
            is_python: true,
            needs_static_fallback: false,
        };
    }
    if let Some(main) = declared_entry {
        return WebEntry {
            entry: main,
            is_python: false,
            needs_static_fallback: false,
        };
    }
    // `server.js` before `index.js`: the project scaffold ships a starter
    // `index.js`, so when an agent builds the app in `server.js` (the name
    // the node brief asks for) the untouched starter must not shadow it.
    if has_server_js {
        return WebEntry {
            entry: "server.js".to_string(),
            is_python: false,
            needs_static_fallback: false,
        };
    }
    if has_index_js {
        return WebEntry {
            entry: "index.js".to_string(),
            is_python: false,
            needs_static_fallback: false,
        };
    }
    WebEntry {
        entry: "server.js".to_string(),
        is_python: false,
        needs_static_fallback: true,
    }
}

/// Same TCP :3000 liveness probe in Python for runtime-python projects (the
/// base image may not have node installed, so the node probe would always
/// fail and wrongly trigger a static fallback).
pub(crate) const HEALTH_PROBE_PYTHON: &str = r#"import socket, sys, time
attempts = int(sys.argv[1] if len(sys.argv) > 1 else 20)
tried = 0
while tried < attempts:
    try:
        s = socket.create_connection(('127.0.0.1', 3000), timeout=1)
    except OSError:
        tried += 1
        time.sleep(1)
        continue
    s.close()
    sys.exit(0)
sys.exit(1)
"#;

#[cfg(test)]
pub(crate) mod entry_resolution_tests {
    use super::{exec_start_line, is_safe_entry, resolve_web_entry, validate_manifest_entries};

    fn f(path: &str) -> serde_json::Value {
        serde_json::json!({ "path": path, "content": [1, 2, 3] })
    }

    fn pkg_json(manifest: &str) -> serde_json::Value {
        let bytes: Vec<serde_json::Value> =
            manifest.as_bytes().iter().map(|b| serde_json::json!(b)).collect();
        serde_json::json!({ "path": "package.json", "content": bytes })
    }

    #[test]
    fn pure_static_gets_generated_fallback() {
        let r = resolve_web_entry(&[f("index.html"), f("style.css")]);
        assert!(r.needs_static_fallback, "static site must get generated server.js");
        assert_eq!(r.entry, "server.js");
        assert!(!r.is_python);
    }

    #[test]
    fn user_server_js_is_real_entry_never_clobbered() {
        // #1276 regression: an Express app's server.js must NOT degrade to the
        // "No web app yet" static fallback page.
        let r = resolve_web_entry(&[f("server.js"), f("index.html")]);
        assert_eq!(r.entry, "server.js");
        assert!(!r.needs_static_fallback, "real server.js clobbered by static fallback");
        assert!(!r.is_python);
    }

    #[test]
    fn declared_entry_wins_over_starter_index_js() {
        // The agent builds the app in src/main.js while a starter index.js
        // template still exists — index.js must not shadow the real app.
        let pkg = pkg_json(r#"{"main":"src/main.js","scripts":{"start":"node src/main.js"}}"#);
        let r = resolve_web_entry(&[f("index.js"), f("src/main.js"), pkg]);
        assert_eq!(r.entry, "src/main.js");
        assert!(!r.needs_static_fallback);
    }

    #[test]
    fn start_script_node_form_is_declared_entry() {
        let pkg = pkg_json(r#"{"scripts":{"start":"node app.js"}}"#);
        let r = resolve_web_entry(&[pkg]);
        assert_eq!(r.entry, "app.js");
        assert!(!r.needs_static_fallback);
    }

    #[test]
    fn server_js_wins_over_starter_index_js() {
        // Prod regression (#custom E2E): the scaffold seeds `index.js`, the
        // agent writes `server.js`; Run started the untouched starter and the
        // browser showed "Vibe Node" instead of the built app.
        let r = resolve_web_entry(&[f("index.js"), f("server.js"), f("README.md")]);
        assert_eq!(r.entry, "server.js");
        assert!(!r.needs_static_fallback);
    }

    #[test]
    fn python_entry_runs_with_python3() {
        let r = resolve_web_entry(&[f("app.py"), f("requirements.txt"), f("templates/x.html")]);
        assert!(r.is_python);
        assert_eq!(r.entry, "app.py");
        assert!(!r.needs_static_fallback, "python app must not be clobbered");
    }

    #[test]
    fn node_entry_shadows_python_entry() {
        // A real node app wins even when a stray .py exists.
        let r = resolve_web_entry(&[f("index.js"), f("util.py")]);
        assert!(!r.is_python);
        assert_eq!(r.entry, "index.js");
        assert!(!r.needs_static_fallback);
    }

    #[test]
    fn nested_python_entry_detected() {
        let r = resolve_web_entry(&[f("backend/main.py")]);
        assert!(r.is_python);
        assert_eq!(r.entry, "main.py");
    }

    #[test]
    fn hostile_manifest_entry_is_refused() {
        // #2655: a crafted `main` must not inject another systemd directive.
        let pkg = pkg_json(r#"{"main":"dist/index.js\n[Service]\nUser=root"}"#);
        assert!(validate_manifest_entries(&[pkg.clone()]).is_err());
        let r = resolve_web_entry(&[pkg]);
        assert!(!r.entry.contains('\n'), "newline reached ExecStart: {:?}", r.entry);
        assert!(!r.entry.contains("[Service]"));
        assert!(r.needs_static_fallback);
    }

    #[test]
    fn hostile_start_script_entry_is_refused() {
        let pkg = pkg_json(r#"{"scripts":{"start":"node dist/x.js; echo pwned"}}"#);
        assert!(validate_manifest_entries(&[pkg.clone()]).is_err());
        let r = resolve_web_entry(&[pkg]);
        assert!(!r.entry.contains(';'), "shell metacharacter reached ExecStart");
    }

    #[test]
    fn unsafe_entry_shapes_are_rejected() {
        for entry in [
            "/etc/passwd",
            "../evil.js",
            "dist/../../evil.js",
            "dist//index.js",
            "x\ny",
            "x\0y",
            "x%y",
            "x;y",
            "x$y",
            "x y",
            "",
        ] {
            assert!(!is_safe_entry(entry), "entry must be rejected: {entry:?}");
        }
        assert!(is_safe_entry("dist/index.js"));
        assert!(is_safe_entry("src/main.js"));
    }

    #[test]
    fn exec_start_line_never_emits_extra_directives() {
        assert!(exec_start_line("/usr/bin/node", "x\n[Service]\nUser=root").is_err());
        let line = exec_start_line("/usr/bin/node", "dist/index.js")
            .unwrap_or_else(|e| {
                log::warn!("exec_start_line failed: {e}");
                String::new()
            });
        assert_eq!(line, "ExecStart=/usr/bin/node /opt/vibe/app/dist/index.js");
        assert!(!line.contains('\n'));
    }
}
