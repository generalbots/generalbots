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
pub(crate) mod commands;
pub mod download;
pub mod drive;
#[cfg(target_os = "linux")]
pub mod dbus_isolation;
pub mod fs_scope;
pub mod i18n;
pub mod payload;
pub(crate) mod no_window;
pub mod safe_command;
#[cfg(feature = "desktop-local")]
pub mod sidecar;
pub mod splash;
pub mod sync;
pub mod tray;
pub mod update;
#[cfg(target_os = "linux")]
pub mod webkit_permissions;
#[cfg(feature = "desktop-local")]
pub mod wizard;

/// Carries the WebKit profile over from the pre-rename identifier (#1870).
pub mod legacy_data;
