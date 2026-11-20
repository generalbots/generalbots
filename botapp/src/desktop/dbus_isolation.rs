//! #2502 — keep the desktop WebView off the host's D-Bus session bus.
//!
//! The appliance session wrapper (`gb-session` in `desktop/os/mkosi.finalize.chroot`)
//! exports a *full* session bus address before `cage` starts the launcher. The
//! Tauri process is that launcher, so its WebKit subprocesses inherit the same
//! environment. Page scripts can then reach logind, the portals, the Secret
//! Service and AT-SPI through WebKit's bus connection, without any bug in the
//! suite itself.
//!
//! We cannot change the session unit from this crate, but we can stop the
//! inheritance at the last point we own: [`isolate_webview_session_bus`] runs
//! from the Linux webview plugin (registered in `main.rs` before the first
//! WebView exists) and replaces the session bus variables with a sentinel no
//! D-Bus daemon can answer. WebKit still starts; D-Bus-only features simply
//! degrade instead of exposing the host session.
//!
//! The original address is retained in [`shell_bus_address`] so shell-owned
//! helpers that genuinely need it (desktop notifications) can opt back in
//! explicitly instead of silently handing the whole bus to every child.

use std::collections::BTreeMap;
use std::sync::{Once, OnceLock};

/// Log prefix; keep it searchable in the desktop log.
const LOG_TAG: &str = "generalbots-dbus-isolation";

/// Name of the variable WebKit/GLib use to find the session bus.
pub const SESSION_BUS_ADDRESS: &str = "DBUS_SESSION_BUS_ADDRESS";

/// Everything that can point a child process at a session/activation bus.
///
/// `DBUS_STARTER_*` is included because a D-Bus activated service would use it
/// even when `DBUS_SESSION_BUS_ADDRESS` has been replaced.
const SESSION_BUS_VARS: [&str; 3] = [
    SESSION_BUS_ADDRESS,
    "DBUS_STARTER_ADDRESS",
    "DBUS_STARTER_BUS_TYPE",
];

/// Isolated address handed to the WebView and all of its children.
///
/// `/dev/null` is a character device, never a listening Unix socket, so
/// `connect(2)` fails deterministically. An empty value would let libdbus fall
/// back to autolaunch or to `$XDG_RUNTIME_DIR/bus`; an explicit bad address
/// does not. This is the "empty/isolated bus" option of #2502: the allow-list
/// is empty, not "the real bus with a filter", because no filter lives here.
pub const WEBVIEW_BUS_ADDRESS: &str = "unix:path=/dev/null";

/// Address the session wrapper gave us, kept only for explicit shell use.
static ORIGINAL_BUS: OnceLock<Option<String>> = OnceLock::new();

/// Guards against a second plugin instance undoing the shell's saved address.
static ISOLATED: Once = Once::new();

/// Pure computation of the WebView environment from an inherited environment.
///
/// Kept separate from [`isolate_webview_session_bus`] so the isolation can be
/// unit-tested without touching the real process environment or a live bus.
#[must_use]
pub fn webview_env(inherited: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut env = inherited.clone();
    for key in SESSION_BUS_VARS {
        env.remove(key);
    }
    env.insert(SESSION_BUS_ADDRESS.to_string(), WEBVIEW_BUS_ADDRESS.to_string());
    env
}

/// Installs the isolated session-bus environment for the webview process tree.
///
/// Called once, from the Linux webview plugin, before Tauri creates a WebView.
/// Every WebKit subprocess inherits the sanitized environment; the shell can
/// still recover the original address through [`shell_bus_address`].
pub fn isolate_webview_session_bus() {
    ISOLATED.call_once(|| {
        let inherited: BTreeMap<String, String> = std::env::vars_os()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
            .collect();
        let original = inherited.get(SESSION_BUS_ADDRESS).cloned();

        // A second call cannot reach this point (Once), so the value is set at
        // most once. Never unwrap: a poisoned/occupied cell must not abort the app.
        let _ = ORIGINAL_BUS.set(original);

        let isolated = webview_env(&inherited);
        for key in SESSION_BUS_VARS {
            match isolated.get(key) {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }

        log::info!(
            "{LOG_TAG}: webview session bus isolated (address replaced, activation vars removed)"
        );
    });
}

/// The session bus address the shell started with, if the wrapper provided one.
///
/// Shell-owned code that needs D-Bus passes this explicitly to its child
/// process (see the notification path in `desktop/tray/mod.rs`) instead of
/// relying on inheritance.
#[must_use]
pub fn shell_bus_address() -> Option<&'static str> {
    ORIGINAL_BUS.get().and_then(|stored| stored.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{webview_env, SESSION_BUS_ADDRESS, SESSION_BUS_VARS, WEBVIEW_BUS_ADDRESS};
    use std::collections::BTreeMap;

    fn appliance_environment() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                SESSION_BUS_ADDRESS.to_string(),
                "unix:path=/run/user/1000/bus".to_string(),
            ),
            (
                "DBUS_STARTER_ADDRESS".to_string(),
                "unix:path=/tmp/dbus-private-session".to_string(),
            ),
            (
                "DBUS_STARTER_BUS_TYPE".to_string(),
                "session".to_string(),
            ),
            (
                "DBUS_SYSTEM_BUS_ADDRESS".to_string(),
                "unix:path=/run/dbus/system_bus_socket".to_string(),
            ),
            ("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string()),
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ])
    }

    #[test]
    fn webview_never_inherits_the_full_session_bus_address() {
        let inherited = appliance_environment();
        let isolated = webview_env(&inherited);

        assert_eq!(
            isolated.get(SESSION_BUS_ADDRESS),
            Some(&WEBVIEW_BUS_ADDRESS.to_string())
        );
        assert!(
            !isolated
                .values()
                .any(|value| value.contains("/run/user/1000/bus")),
            "the user session bus must not survive into the webview"
        );
        assert!(
            !isolated
                .values()
                .any(|value| value.contains("dbus-private-session")),
            "a dbus-run-session bus must not survive into the webview"
        );
        for key in SESSION_BUS_VARS {
            if key != SESSION_BUS_ADDRESS {
                assert!(
                    !isolated.contains_key(key),
                    "activation variable {key} must be removed"
                );
            }
        }
    }

    #[test]
    fn unrelated_environment_is_untouched() {
        let inherited = appliance_environment();
        let isolated = webview_env(&inherited);

        for key in ["DBUS_SYSTEM_BUS_ADDRESS", "XDG_RUNTIME_DIR", "PATH"] {
            assert_eq!(
                isolated.get(key),
                inherited.get(key),
                "{key} is not a session-bus variable"
            );
        }
    }

    #[test]
    fn isolation_is_idempotent() {
        let inherited = appliance_environment();
        assert_eq!(webview_env(&webview_env(&inherited)), webview_env(&inherited));
    }

    #[test]
    fn a_missing_session_bus_still_gets_an_isolated_address() {
        let inherited = BTreeMap::from([("PATH".to_string(), "/usr/bin".to_string())]);
        let isolated = webview_env(&inherited);
        assert_eq!(
            isolated.get(SESSION_BUS_ADDRESS),
            Some(&WEBVIEW_BUS_ADDRESS.to_string())
        );
    }
}
