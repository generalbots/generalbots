use super::*;

/// File inside the engine folder that records which llama.cpp build is
/// installed there, so Settings can tell when a newer one is pinned.
pub const ENGINE_BUILD_FILE: &str = ".engine-build";

/// The engine archive this version of the product installs on this machine.
/// Resolved once: choosing the build asks the CUDA toolkit for its version.
pub fn pinned_engine_url() -> Option<String> {
    static URL: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    URL.get_or_init(botcorepkg::installer::get_llama_cpp_url).clone()
}

/// The llama.cpp build of [`pinned_engine_url`], such as `b12483`.
pub fn pinned_engine_build() -> Option<String> {
    pinned_engine_url().as_deref().and_then(botcorepkg::installer::llama_build_tag)
}

impl PackageManager {
    /// What the engine archive alone does not bring: the NVIDIA runtime
    /// libraries of a CUDA build, and the record of the build installed.
    pub(super) async fn finish_llm_install(&self, engine_url: &str) -> Result<()> {
        if let Some(runtime_url) = botcorepkg::installer::get_llama_cpp_runtime_url() {
            info!("Installing the CUDA runtime libraries next to the engine");
            self.download_and_install(&runtime_url, "llm", None).await?;
        }
        if let Some(tag) = botcorepkg::installer::llama_build_tag(engine_url) {
            let marker = self.base_path.join("bin").join("llm").join(ENGINE_BUILD_FILE);
            if let Err(e) = std::fs::write(&marker, &tag) {
                warn!("Could not record the engine build at {}: {e}", marker.display());
            }
        }
        Ok(())
    }
}
