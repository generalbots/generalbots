//! One-shot, human-approved grants for destructive commands (#2497), bound to
//! the local window that requested them (#2692).
//!
//! Contract — the trusted local UI obtains a grant through
//! `request_destructive_confirmation` / `request_reset_stack_confirmation`:
//! 1. The command verifies the calling window is the local setup wizard (or,
//!    in non-sidecar builds, the suite window on the app document). A remote
//!    or sidecar window is refused even when a capability grants the command.
//! 2. The shell resolves the path through `path_scope::resolve_in_scopes` and
//!    shows a native Yes/No dialog naming the exact target, so no script can
//!    approve its own request.
//! 3. Only after the user clicks Yes does the shell mint an opaque token and
//!    store it as a single-use grant for that canonical target, operation and
//!    window label.
//! 4. The destructive command verifies the caller window *again*, then
//!    consumes a grant matching `(operation, target, label)`. A grant minted
//!    for another window is worthless, closing the cross-page replay window.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::Window;

/// A confirmation is only good for the action the user just approved.
pub const TTL: Duration = Duration::from_secs(60);
const MAX_GRANTS: usize = 32;

/// The only window label allowed to request or consume destructive grants.
pub const TRUSTED_WINDOW_LABEL: &str = "wizard";
/// Suite label. Trusted only in builds whose main window serves the local app
/// document; in desktop-local builds it is navigated to the sidecar origin.
const SUITE_WINDOW_LABEL: &str = "main";
const DEV_ORIGIN_PORT: u16 = 3000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Delete,
    CreateFolder,
    ResetStack,
}

impl Operation {
    /// Parses the operation name accepted from the webview.
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim() {
            "delete" => Ok(Self::Delete),
            "create_folder" => Ok(Self::CreateFolder),
            "reset_stack" => Ok(Self::ResetStack),
            _ => Err("Unknown destructive operation".to_string()),
        }
    }

    /// Stable name used in the native dialog and the audit log.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::CreateFolder => "create_folder",
            Self::ResetStack => "reset stack",
        }
    }
}

/// Pure description of a minted approval; testable without a native dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub token: String,
    pub operation: Operation,
    pub target: String,
    pub label: String,
    expires: Instant,
}

impl Grant {
    pub fn new(
        token: String,
        operation: Operation,
        target: &str,
        label: &str,
        now: Instant,
    ) -> Self {
        let expires = match now.checked_add(TTL) {
            Some(value) => value,
            None => now,
        };
        Self {
            token,
            operation,
            target: target.to_string(),
            label: label.to_string(),
            expires,
        }
    }

    /// Whether this approval has timed out.
    pub fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires
    }

    /// Whether this approval authorises one operation on one target.
    ///
    /// The label is checked separately by [`consume_for`]; every IPC command
    /// must use that function, never the label-less [`consume`] helper.
    pub fn matches(&self, operation: Operation, target: &str) -> bool {
        self.operation == operation && self.target == target
    }
}

fn grants() -> &'static Mutex<HashMap<String, Grant>> {
    static GRANTS: OnceLock<Mutex<HashMap<String, Grant>>> = OnceLock::new();
    GRANTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Unique receipt; not a secret, but never repeated for two confirmations.
fn next_token() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(_) => 0,
    };
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{nanos:016x}{sequence:08x}")
}

/// Mints a single-use approval after the native dialog was accepted.
pub fn mint(operation: Operation, target: &str, label: &str) -> Result<String, String> {
    let mut store = grants()
        .lock()
        .map_err(|e| format!("Confirmation store unavailable: {e}"))?;
    let now = Instant::now();
    store.retain(|_, grant| !grant.is_expired(now));
    // Re-asking for the same action in the same window returns the existing
    // receipt. A different window gets its own grant (and its own dialog), so
    // one page can never borrow another page's approval.
    if let Some(existing) = store
        .iter()
        .find(|(_, grant)| grant.matches(operation, target) && grant.label == label)
    {
        return Ok(String::clone(existing.0));
    }
    if store.len() >= MAX_GRANTS {
        return Err("Too many pending confirmations".to_string());
    }
    let token = next_token();
    store.insert(
        token.clone(),
        Grant::new(token.clone(), operation, target, label, now),
    );
    Ok(token)
}

/// Consumes an approval only when it was minted for the requesting window.
/// This is the only consume path allowed in a Tauri command (#2692).
pub fn consume_for(operation: Operation, target: &str, label: &str) -> Result<(), String> {
    consume_where(operation, target, Some(label))
}

/// Label-less compatibility helper for direct Rust callers (offline tests).
///
/// NEVER wire this into a `#[tauri::command]`: it deliberately ignores the
/// requesting window. IPC commands go through [`consume_for`] instead.
pub fn consume(operation: Operation, target: &str) -> Result<(), String> {
    consume_where(operation, target, None)
}

fn consume_where(operation: Operation, target: &str, label: Option<&str>) -> Result<(), String> {
    let mut store = grants()
        .lock()
        .map_err(|e| format!("Confirmation store unavailable: {e}"))?;
    let now = Instant::now();
    store.retain(|_, grant| !grant.is_expired(now));
    let key = store
        .iter()
        .find(|(_, grant)| {
            grant.matches(operation, target)
                && label.map_or(true, |expected| grant.label == expected)
        })
        .map(|(key, _)| (*key).clone());
    match key {
        Some(key) => {
            store.remove(&key);
            Ok(())
        }
        None => Err(match label {
            Some(_) => "This action was not approved in this window".to_string(),
            None => "This action requires an in-app confirmation".to_string(),
        }),
    }
}

/// Pure window-authorization predicate for destructive IPC commands.
///
/// `label` is the Tauri window label and `url` its current document URL. Only
/// the local wizard window is trusted; the suite window counts only in builds
/// that do not navigate it to the sidecar. Even a capability that grants the
/// command cannot bypass this check. The loopback sidecar origin is explicitly
/// untrusted because it serves model-generated, attacker-influenced content.
pub fn is_trusted_window(
    label: &str,
    url: Option<&str>,
    desktop_local: bool,
    dev_server: bool,
) -> bool {
    let label_allowed =
        label == TRUSTED_WINDOW_LABEL || (!desktop_local && label == SUITE_WINDOW_LABEL);
    if !label_allowed {
        return false;
    }
    let Some(raw) = url else {
        return false;
    };
    let Ok(parsed) = tauri::Url::parse(raw.trim()) else {
        return false;
    };
    // `user@host` does not change the origin but disguises it and can smuggle
    // credentials into privileged requests.
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    match (parsed.scheme(), parsed.host_str()) {
        ("tauri", Some("localhost")) => true,
        ("https", Some("tauri.localhost")) => true,
        ("http", Some("localhost" | "127.0.0.1")) => {
            dev_server && parsed.port_or_known_default() == Some(DEV_ORIGIN_PORT)
        }
        _ => false,
    }
}

/// Resolves and validates the caller of a destructive command, failing closed.
pub fn trusted_label(window: &Window) -> Result<String, String> {
    let label = window.label().to_string();
    let url = window.webviews().first().and_then(|w| w.url().ok());
    let dev_server = cfg!(debug_assertions) && !cfg!(feature = "custom-protocol");
    let trusted = is_trusted_window(
        &label,
        url.as_ref().map(|value| value.as_str()),
        cfg!(feature = "desktop-local"),
        dev_server,
    );
    if !trusted {
        log::warn!("destructive IPC denied: window={label}");
        return Err("This action is only available in the local setup window".to_string());
    }
    Ok(label)
}

#[cfg(test)]
mod tests {
    use super::{consume_for, is_trusted_window, mint, Grant, Operation, TTL};
    use std::time::{Duration, Instant};

    const SIDECAR: &str = "http://127.0.0.1:8080/suite/desktop.html";

    #[test]
    fn trusted_window_accepts_bundled_documents() {
        for document in [
            "tauri://localhost/wizard/index.html",
            "https://tauri.localhost/wizard/index.html",
        ] {
            assert!(
                is_trusted_window("wizard", Some(document), true, false),
                "rejected bundled wizard document: {document}"
            );
        }
        assert!(is_trusted_window(
            "main",
            Some("tauri://localhost/suite/desktop.html"),
            false,
            false
        ));
    }

    #[test]
    fn trusted_window_denies_sidecar_and_remote_origins() {
        // The sidecar window stays untrusted even if a capability grants the
        // command — #2692's defence-in-depth requirement.
        assert!(!is_trusted_window("main", Some(SIDECAR), true, false));
        assert!(!is_trusted_window("wizard", Some(SIDECAR), true, false));
        for hostile in [
            "https://evil.example/wizard/index.html",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "tauri://localhost.evil.example/wizard/index.html",
            "http://user@localhost:3000/wizard/index.html",
            "http://localhost:3001/wizard/index.html",
            "http://localhost/wizard/index.html",
            "not a url",
        ] {
            assert!(
                !is_trusted_window("wizard", Some(hostile), true, true),
                "accepted hostile window URL: {hostile}"
            );
        }
        assert!(!is_trusted_window("wizard", None, true, true));
        assert!(!is_trusted_window(
            "splash",
            Some("tauri://localhost/"),
            true,
            false
        ));
        assert!(!is_trusted_window(
            "",
            Some("tauri://localhost/"),
            true,
            false
        ));
    }

    #[test]
    fn trusted_window_dev_origin_is_explicit() {
        let dev = "http://localhost:3000/wizard/index.html";
        assert!(is_trusted_window("wizard", Some(dev), true, true));
        assert!(!is_trusted_window("wizard", Some(dev), true, false));
    }

    #[test]
    fn grant_from_page_a_cannot_be_consumed_by_page_b() {
        let target = "/tmp/issue-2692-cross-page";
        let token = mint(Operation::Delete, target, "wizard-a").expect("mint for page A");
        assert!(consume_for(Operation::Delete, target, "wizard-b").is_err());
        assert!(consume_for(Operation::Delete, target, "wizard-a").is_ok());
        // Single use: replay by either page fails.
        assert!(consume_for(Operation::Delete, target, "wizard-a").is_err());
        assert!(!token.is_empty());
    }

    #[test]
    fn mint_does_not_share_a_grant_across_windows() {
        let target = "/tmp/issue-2692-two-mints";
        let first = mint(Operation::ResetStack, target, "wizard-a").expect("first mint");
        let second = mint(Operation::ResetStack, target, "wizard-b").expect("second mint");
        assert_ne!(first, second);
        assert!(consume_for(Operation::ResetStack, target, "wizard-a").is_ok());
        assert!(consume_for(Operation::ResetStack, target, "wizard-b").is_ok());
    }

    #[test]
    fn expired_grant_is_not_authorisation() {
        let now = Instant::now();
        let grant = Grant::new(
            "token".to_string(),
            Operation::CreateFolder,
            "/tmp/issue-2692-expired",
            "wizard",
            now.checked_sub(TTL).unwrap_or(now),
        );
        assert!(grant.is_expired(now));
        assert!(grant.matches(Operation::CreateFolder, "/tmp/issue-2692-expired"));
        assert_ne!(grant.label, "another-window");
        assert!(grant.is_expired(now.checked_add(Duration::from_secs(1)).unwrap_or(now)));
    }
}
