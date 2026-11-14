#[cfg(feature = "desktop-local")]
pub mod activity;
pub mod download;
pub mod drive;
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
