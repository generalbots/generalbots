fn main() {
    // Only run tauri_build when the desktop feature is enabled
    #[cfg(feature = "desktop")]
    {
        tauri_build::build()
    }
}
