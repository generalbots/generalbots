//! #2693 — offline regression tests for webview-controlled workspace paths.
//!
//! The webview can type or inject the workspace string, so every hostile form
//! must be refused before `create_dir_all` or the `.gb-write-test` write.

use super::{check_workspace, resolve_workspace};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// `GB_FS_SCOPES` is process-global; serialise the tests that change it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn text(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

/// Private roots under the system temp dir, so tests never touch user data.
struct ScopeFixture {
    root: PathBuf,
    outside: PathBuf,
    previous_scopes: Option<OsString>,
    previous_allow_any: Option<OsString>,
}

impl ScopeFixture {
    fn new() -> Self {
        let stamp = std::process::id();
        let root = std::env::temp_dir().join(format!("gb-wizard-scope-{stamp}"));
        let outside = std::env::temp_dir().join(format!("gb-wizard-outside-{stamp}"));
        let _ = std::fs::create_dir_all(&root);
        let _ = std::fs::create_dir_all(&outside);
        let previous_scopes = std::env::var_os("GB_FS_SCOPES");
        let previous_allow_any = std::env::var_os("GB_FS_ALLOW_ANY");
        std::env::set_var("GB_FS_SCOPES", &root);
        std::env::remove_var("GB_FS_ALLOW_ANY");
        Self {
            root,
            outside,
            previous_scopes,
            previous_allow_any,
        }
    }
}

impl Drop for ScopeFixture {
    fn drop(&mut self) {
        match &self.previous_scopes {
            Some(value) => std::env::set_var("GB_FS_SCOPES", value),
            None => std::env::remove_var("GB_FS_SCOPES"),
        }
        match &self.previous_allow_any {
            Some(value) => std::env::set_var("GB_FS_ALLOW_ANY", value),
            None => std::env::remove_var("GB_FS_ALLOW_ANY"),
        }
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.outside);
    }
}

#[test]
fn hostile_paths_are_refused_before_any_disk_write() {
    let _guard = env_guard();
    let fixture = ScopeFixture::new();
    let hostile = [
        String::new(),
        "   ".to_string(),
        "workspace".to_string(),
        format!("{}/../outside/escape", text(&fixture.root)),
        format!("{}/.env", text(&fixture.root)),
        "/".to_string(),
        text(&fixture.outside.join("workspace")),
        format!("{}\0/child", text(&fixture.root)),
        format!("{}/workspace\0", text(&fixture.root)),
    ];
    for path in hostile {
        assert!(
            resolve_workspace(&path).is_err(),
            "accepted hostile path: {path:?}"
        );
        let check = check_workspace(&path);
        assert!(!check.writable, "workspace check writable for {path:?}");
        assert!(
            !fixture.outside.join(".gb-write-test").exists(),
            "probe written outside the scope for {path:?}"
        );
    }
}

#[test]
fn a_legitimate_workspace_still_passes_and_the_probe_is_cleaned_up() {
    let _guard = env_guard();
    let fixture = ScopeFixture::new();
    let workspace = fixture.root.join("workspace");

    let resolved = resolve_workspace(&text(&workspace)).expect("workspace inside the scope");
    let canonical_root = std::fs::canonicalize(&fixture.root).expect("fixture root exists");
    assert_eq!(resolved, canonical_root.join("workspace"));

    let check = check_workspace(&text(&workspace));
    assert!(
        check.writable,
        "legitimate workspace refused: {}",
        check.reason
    );
    assert!(workspace.is_dir());
    assert!(!workspace.join(".gb-write-test").exists());
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_refused() {
    use std::os::unix::fs::symlink;

    let _guard = env_guard();
    let fixture = ScopeFixture::new();
    let link = fixture.root.join("escape");
    symlink(&fixture.outside, &link).expect("create the test symlink");

    assert!(
        resolve_workspace(&text(&link)).is_err(),
        "a symlink to an out-of-scope directory was accepted"
    );
    let candidate = link.join("workspace");
    assert!(
        resolve_workspace(&text(&candidate)).is_err(),
        "a path traversing an out-of-scope symlink was accepted"
    );
    assert!(!check_workspace(&text(&candidate)).writable);
    assert!(
        !fixture.outside.join("workspace").exists(),
        "created a directory outside the scope"
    );
}

#[cfg(unix)]
#[test]
fn a_protected_name_exposed_by_a_symlink_is_refused() {
    use std::os::unix::fs::symlink;

    let _guard = env_guard();
    let fixture = ScopeFixture::new();
    let secret = fixture.root.join(".env");
    std::fs::write(&secret, b"token").expect("write the fixture secret");
    let alias = fixture.root.join("benign");
    symlink(&secret, &alias).expect("create the test symlink");

    assert!(
        resolve_workspace(&text(&alias)).is_err(),
        "a protected name reached through a symlink was accepted"
    );
}
