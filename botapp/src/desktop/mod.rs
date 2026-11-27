/// Whether the app runs as the General Bots OS kiosk (#2450).
///
/// The image's session wrapper exports `GB_KIOSK=1`; every other desktop
/// build (Windows/macOS installer, dev run) leaves it unset and keeps its
/// normal window chrome. Kiosk mode removes the close affordance, goes
/// fullscreen, and turns a close request into the in-app power menu, because
/// under cage closing the only client is a black screen with a session
/// restart — not a window the user can recover.
pub fn shell_mode() -> bool {
    std::env::var("GB_KIOSK")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(feature = "desktop-local")]
pub mod activity;
pub mod app;
pub mod commands;
#[cfg(feature = "desktop-local")]
pub(crate) mod exit_decision;
#[cfg(feature = "desktop-local")]
pub mod boot_guard;
#[cfg(feature = "desktop-local")]
pub mod boot_steps;
#[cfg(feature = "desktop-local")]
pub mod boot_supervisor;
pub mod data_root;
pub mod download;

/// Port the sidecar listens on. Kept here so the download URL gate can allow
/// localhost sidecar traffic even when the `desktop-local` feature is off.
pub(crate) fn sidecar_port() -> u16 {
    std::env::var("GB_SIDECAR_PORT")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(8080)
}
pub mod drive;
#[cfg(target_os = "linux")]
pub mod dbus_isolation;
pub mod fs_scope;
pub mod i18n;
#[cfg(feature = "desktop-local")]
pub mod open_download;
pub mod payload;
pub(crate) mod no_window;
pub mod safe_command;
pub mod single_instance;
#[cfg(feature = "desktop-local")]
pub mod recovery;
#[cfg(feature = "desktop-local")]
pub mod sidecar;
#[cfg(feature = "gblite")]
pub mod stack_relocate;
#[cfg(feature = "desktop-local")]
pub mod stack_stop;
pub mod splash;
#[cfg(feature = "desktop-local")]
pub mod splash_failure;
#[cfg(feature = "desktop-local")]
pub mod splash_progress;
pub mod sync;
pub mod tray;
pub mod update;
#[cfg(target_os = "linux")]
pub mod webkit_permissions;

// The Tauri command list is shared by every platform, but WebKitGTK exists
// only on Linux. On other platforms the WebView handles getUserMedia itself;
// keep the command registered so the shared list compiles and report honestly
// that there is no Linux permission store to change.
#[cfg(not(target_os = "linux"))]
pub mod webkit_permissions {
    #[tauri::command]
    pub fn set_media_permission(kind: String, enabled: bool) -> Result<(), String> {
        Err(format!(
            "media permission for {kind} cannot be set on this platform (requested={enabled})"
        ))
    }
}
#[cfg(feature = "desktop-local")]
pub mod wizard;

/// Agent runtime detection, provisioning and removal (gbgov only).
/// Compiled under desktop-local so the commands register unconditionally;
/// every entry point returns "unsupported" when the gbgov feature is off.
#[cfg(feature = "desktop-local")]
pub mod agent_runtime;

/// Carries the WebKit profile over from the pre-rename identifier (#1870).
pub mod legacy_data;
