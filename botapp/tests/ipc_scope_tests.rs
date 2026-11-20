//! Scope and confirmation regression tests for #2497's tractable IPC fixes.
//!
//! The native dialog itself needs a running shell; what is testable offline is
//! the pure decision behind it: paths stay inside the owned roots, and a
//! destructive command only runs after a matching, single-use grant exists.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use botapp::desktop::drive::confirmation::{consume, mint, Grant, Operation, TTL};
use botapp::desktop::drive::{sort_files, upload_destination, upload_percent, FileItem};
use botapp::desktop::{drive, fs_scope};

/// `GB_FS_SCOPES` is process-global; serialise the tests that change it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    match ENV_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Private roots under the system temp dir, so tests never touch user data.
struct ScopeFixture {
    root: PathBuf,
    outside: PathBuf,
}

impl ScopeFixture {
    fn new() -> Self {
        let stamp = std::process::id();
        let root = std::env::temp_dir().join(format!("gb-ipc-scope-{stamp}"));
        let outside = std::env::temp_dir().join(format!("gb-ipc-outside-{stamp}"));
        let _ = std::fs::create_dir_all(&root);
        let _ = std::fs::create_dir_all(&outside);
        std::env::set_var("GB_FS_SCOPES", &root);
        std::env::remove_var("GB_FS_ALLOW_ANY");
        Self { root, outside }
    }

    fn text(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    /// The canonical form a command would check, as a string.
    fn scoped_text(path: &Path) -> String {
        fs_scope::resolve(&Self::text(path))
            .map(|resolved| Self::text(&resolved))
            .unwrap_or_default()
    }
}

impl Drop for ScopeFixture {
    fn drop(&mut self) {
        std::env::remove_var("GB_FS_SCOPES");
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.outside);
    }
}

#[test]
fn scope_accepts_in_root_and_rejects_relative_parent_and_outside() {
    let guard = env_guard();
    let fixture = ScopeFixture::new();
    let inside = fixture.root.join("report.txt");

    assert!(
        fs_scope::resolve(&ScopeFixture::text(&inside)).is_ok(),
        "a new file inside the root must be accepted"
    );
    assert!(fs_scope::resolve("report.txt").is_err(), "relative path accepted");
    assert!(
        fs_scope::resolve(&format!("{}/../escape.txt", fixture.root.display())).is_err(),
        "parent traversal accepted"
    );
    assert!(
        fs_scope::resolve(&ScopeFixture::text(&fixture.outside.join("secret.txt"))).is_err(),
        "absolute path outside the root accepted"
    );
    assert!(
        fs_scope::resolve(&ScopeFixture::text(&fixture.root.join(".env"))).is_err(),
        "protected secret name accepted"
    );
    drop(guard);
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_refused() {
    use std::os::unix::fs::symlink;

    let guard = env_guard();
    let fixture = ScopeFixture::new();
    let link = fixture.root.join("escape");
    assert!(
        symlink(&fixture.outside, &link).is_ok(),
        "could not create the test symlink"
    );
    let resolved = fs_scope::resolve(&ScopeFixture::text(&link));
    assert!(resolved.is_err(), "a symlink out of the root was accepted");
    drop(guard);
}

#[test]
fn grants_are_operation_and_target_specific_and_expire() {
    let now = std::time::Instant::now();
    let grant = Grant::new(
        "token-1".to_string(),
        Operation::Delete,
        "/root/a.txt",
        "main",
        now,
    );

    assert_eq!(grant.operation, Operation::Delete);
    assert_eq!(grant.target, "/root/a.txt");
    assert_eq!(grant.label, "main");
    assert_eq!(grant.token, "token-1");
    assert!(grant.matches(Operation::Delete, "/root/a.txt"));
    assert!(!grant.matches(Operation::CreateFolder, "/root/a.txt"));
    assert!(!grant.matches(Operation::Delete, "/root/b.txt"));
    assert!(!grant.is_expired(now));
    assert!(!grant.is_expired(now + TTL / 2));
    assert!(grant.is_expired(now + TTL));

    assert_eq!(Operation::parse("delete"), Ok(Operation::Delete));
    assert_eq!(Operation::parse("create_folder"), Ok(Operation::CreateFolder));
    assert!(Operation::parse("rm -rf").is_err());
    assert_eq!(Operation::Delete.as_str(), "delete");
    assert_eq!(Operation::CreateFolder.as_str(), "create_folder");
}

#[test]
fn confirmation_is_single_use_and_scoped_to_its_target() {
    let target = format!(
        "{}/granted.txt",
        std::env::temp_dir()
            .join(format!("gb-ipc-token-{}", std::process::id()))
            .display()
    );

    assert!(mint(Operation::Delete, &target, "main").is_ok());
    assert!(consume(Operation::Delete, &target).is_ok());
    assert!(
        consume(Operation::Delete, &target).is_err(),
        "a grant must not be reusable"
    );
    assert!(consume(Operation::CreateFolder, &target).is_err());
    assert!(consume(Operation::Delete, "/somewhere/else").is_err());
}

#[test]
fn destructive_commands_require_confirmation() {
    let guard = env_guard();
    let fixture = ScopeFixture::new();
    let file = fixture.root.join("doc.txt");
    assert!(std::fs::write(&file, b"keep me").is_ok());
    let file_target = ScopeFixture::scoped_text(&file);

    let unconfirmed = drive::delete_path(&file_target);
    assert!(unconfirmed.is_err(), "unconfirmed delete was accepted");
    assert!(file.exists(), "unconfirmed delete removed the file");

    assert!(mint(Operation::Delete, &file_target, "main").is_ok());
    assert!(drive::delete_path(&file_target).is_ok());
    assert!(!file.exists(), "confirmed delete did not remove the file");

    let folder = fixture.root.join("new-folder");
    let folder_target = ScopeFixture::scoped_text(&folder);
    let parent = ScopeFixture::text(&fixture.root);
    assert!(drive::create_folder(&parent, "new-folder").is_err());
    assert!(!folder.exists(), "unconfirmed create was accepted");

    assert!(mint(Operation::CreateFolder, &folder_target, "main").is_ok());
    assert!(drive::create_folder(&parent, "new-folder").is_ok());
    assert!(folder.is_dir(), "confirmed create did not make the folder");
    drop(guard);
}

#[test]
fn the_confirmation_command_is_declared_in_permissions() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(manifest.join("permissions/default.toml"))
        .unwrap_or_default();
    assert!(raw.contains("allow-request-destructive-confirmation"));
    assert!(raw.contains("commands.allow = [\"request_destructive_confirmation\"]"));
}

// Drive helper tests moved here so `drive.rs` stays under the 450-line limit.

fn item(name: &str, is_dir: bool) -> FileItem {
    FileItem {
        name: name.to_string(),
        path: format!("/drive/{name}"),
        is_dir,
        size: None,
    }
}

#[test]
fn folders_come_first_then_names_ignore_case() {
    let mut files = vec![
        item("zeta.txt", false),
        item("Beta.txt", false),
        item("alpha", true),
        item("Gamma.txt", false),
        item("delta", true),
    ];
    sort_files(&mut files);
    let names: Vec<&str> = files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "delta", "Beta.txt", "Gamma.txt", "zeta.txt"]);
}

#[test]
fn the_upload_keeps_the_source_name_inside_the_target_folder() {
    let dest = upload_destination(Path::new("/home/u/report.pdf"), Path::new("/drive/inbox"))
        .unwrap_or_default();
    assert_eq!(dest, PathBuf::from("/drive/inbox/report.pdf"));
}

#[test]
fn an_upload_onto_itself_is_refused() {
    // Creating this destination would truncate the source before copying.
    let refused = upload_destination(Path::new("/drive/a.txt"), Path::new("/drive"));
    assert!(refused.is_err());
    // A source with no file name cannot be placed anywhere.
    assert!(upload_destination(Path::new("/"), Path::new("/drive")).is_err());
}

#[test]
fn progress_is_a_percentage_and_never_overflows() {
    assert_eq!(upload_percent(0, 0), 100);
    assert_eq!(upload_percent(0, 10), 0);
    assert_eq!(upload_percent(5, 10), 50);
    assert_eq!(upload_percent(10, 10), 100);
    // A file that grew mid-copy is clamped instead of reporting 120%.
    assert_eq!(upload_percent(2_000, 1_000), 100);
    // Saturating multiplication: a huge byte count cannot panic in debug.
    assert!(upload_percent(u64::MAX, u64::MAX) <= 100);
}
