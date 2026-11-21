
#[cfg(feature = "mobile")]
pub mod mobile;
#[cfg(not(feature = "mobile"))]
pub mod build_info;
#[cfg(not(feature = "mobile"))]
pub mod desktop;

/// Native entry point used by Tauri Android (`MainActivity : TauriActivity`)
/// and the iOS AppDelegate. `tauri-build` sets `cfg(mobile)` for mobile targets
/// and the macro exports the `start_app` symbol the platform loaders call.
/// Desktop builds keep using `main.rs` and never reference this function.
#[cfg(feature = "mobile")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    mobile::run();
}
