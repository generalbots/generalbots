//! The local engine on disk: where it is, which build it is, and replacing it
//! with the build this version of the product pins.
//!
//! An update never leaves the machine without an engine: the folder in use is
//! moved aside first and comes back when the download fails or when the new
//! engine cannot start the model that was running before.

use std::path::{Path, PathBuf};
use std::time::Duration;

use botcore::package_manager::facade::{pinned_engine_build, pinned_engine_url, ENGINE_BUILD_FILE};
use botcore::package_manager::{InstallMode, PackageManager};

use crate::on_device_ai::{runtime_ctl, runtime_spawn, runtime_stop};

/// The engine could not be stopped or its folder is held by another program.
pub(super) const CODE_ENGINE_IN_USE: &str = "engine_in_use";
/// The download or the unpacking failed; the engine in use was put back.
pub(super) const CODE_DOWNLOAD_FAILED: &str = "engine_download_failed";
/// The new engine could not start the model that was running; put back.
pub(super) const CODE_NEW_ENGINE_FAILED: &str = "engine_new_failed";

pub(super) const STAGE_STOPPING: &str = "stopping";
pub(super) const STAGE_DOWNLOADING: &str = "downloading";
pub(super) const STAGE_STARTING: &str = "starting";

pub(super) fn engine_dir() -> PathBuf {
    PathBuf::from(botcore::shared::utils::get_stack_path()).join("bin").join("llm")
}

fn previous_dir() -> PathBuf {
    PathBuf::from(botcore::shared::utils::get_stack_path()).join("bin").join("llm.previous")
}

/// `llama-server` inside `dir`: the Windows archives unpack flat, the Linux
/// ones into `bin/`.
pub(super) fn server_binary_in(dir: &Path) -> Option<PathBuf> {
    let name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
    [dir.join(name), dir.join("bin").join(name)]
        .into_iter()
        .find(|candidate| candidate.is_file())
}

pub(super) fn server_binary() -> Option<PathBuf> {
    server_binary_in(&engine_dir())
}

/// The build recorded when the engine was installed. `None` for an engine
/// installed before builds were recorded.
pub(super) fn installed_build() -> Option<String> {
    let text = std::fs::read_to_string(engine_dir().join(ENGINE_BUILD_FILE)).ok()?;
    let tag = text.trim();
    (!tag.is_empty()).then(|| tag.to_string())
}

pub(super) fn available_build() -> Option<String> {
    pinned_engine_build()
}

/// An engine is installed and it is not the pinned build.
pub(super) fn update_available() -> bool {
    needs_update(server_binary().is_some(), installed_build().as_deref(), available_build().as_deref())
}

pub(super) fn needs_update(installed: bool, build: Option<&str>, pinned: Option<&str>) -> bool {
    match (installed, pinned) {
        (true, Some(pinned)) => build != Some(pinned),
        _ => false,
    }
}

/// Approximate size of what an install downloads: the engine archive plus,
/// for a CUDA build, NVIDIA's runtime libraries.
pub(super) fn download_bytes() -> u64 {
    download_bytes_for(pinned_engine_url().as_deref().unwrap_or(""))
}

pub(super) fn download_bytes_for(url: &str) -> u64 {
    if url.contains("cuda-13") {
        630_000_000
    } else if url.contains("cuda-12") {
        715_000_000
    } else if url.contains("vulkan") {
        40_000_000
    } else {
        30_000_000
    }
}

/// What was running before the update and has to run again after it.
struct Running {
    chat_model: Option<String>,
    embedding: bool,
}

/// Replace the installed engine with the pinned build. `stage` is told what
/// is going on. On `Err` the engine that was installed is back in place and
/// what was running has been started again; the value is a failure code.
pub(super) async fn update(
    stage: &(dyn Fn(&'static str) + Send + Sync),
    gpu_layers: &str,
    ctx_size: &str,
) -> Result<(), &'static str> {
    stage(STAGE_STOPPING);
    let probe = runtime_ctl::probe().await;
    let stopped = runtime_stop::stop_all_stack_llama();
    let running = Running {
        chat_model: probe.reachable.then(|| bare(&probe.loaded_model)).filter(|m| !m.is_empty()),
        embedding: runtime_stop::served_port(&stopped, runtime_spawn::EMBEDDING_PORT),
    };

    let (current, previous) = (engine_dir(), previous_dir());
    if previous.exists() {
        if let Err(e) = std::fs::remove_dir_all(&previous) {
            log::warn!("[engine_update] cannot remove stale previous engine dir: {e}");
        }
    }
    if let Err(e) = rename_with_retry(&current, &previous).await {
        log::error!("[engine_update] cannot move the engine aside: {e}");
        restart(&running, gpu_layers, ctx_size).await;
        return Err(CODE_ENGINE_IN_USE);
    }

    stage(STAGE_DOWNLOADING);
    if let Err(e) = install().await {
        log::error!("[engine_update] install failed: {e}");
        put_back(&current, &previous).await;
        restart(&running, gpu_layers, ctx_size).await;
        return Err(CODE_DOWNLOAD_FAILED);
    }

    stage(STAGE_STARTING);
    if let Some(model) = &running.chat_model {
        if !start_chat(model, gpu_layers, ctx_size).await {
            log::error!("[engine_update] the new engine cannot start {model}; restoring the previous engine");
            runtime_stop::stop_all_stack_llama();
            put_back(&current, &previous).await;
            restart(&running, gpu_layers, ctx_size).await;
            return Err(CODE_NEW_ENGINE_FAILED);
        }
    }
    if running.embedding {
        start_embedding();
    }
    runtime_ctl::clear_last_failure();
    log::info!("[engine_update] engine updated to {:?}", installed_build());
    Ok(())
}

/// Install the pinned engine into an empty engine folder.
pub(super) async fn install() -> Result<(), String> {
    let pm = PackageManager::new(InstallMode::Local, None).map_err(|e| e.to_string())?;
    pm.install("llm").await.map_err(|e| e.to_string())?;
    if server_binary().is_none() {
        return Err("the archive was unpacked but llama-server is not in it".to_string());
    }
    Ok(())
}

fn bare(model: &str) -> String {
    model.rsplit(['/', '\\']).next().unwrap_or(model).to_string()
}

async fn start_chat(model: &str, gpu_layers: &str, ctx_size: &str) -> bool {
    match runtime_ctl::desired_from_config(model, gpu_layers, ctx_size) {
        Some(desired) => runtime_ctl::ensure(&desired).await.is_ok(),
        None => false,
    }
}

fn start_embedding() {
    if let Err(e) = runtime_spawn::start_embedding_runtime() {
        log::warn!("[engine_update] embedding runtime not restarted: {e}");
    }
}

async fn restart(running: &Running, gpu_layers: &str, ctx_size: &str) {
    if let Some(model) = &running.chat_model {
        if !start_chat(model, gpu_layers, ctx_size).await {
            log::warn!("[engine_update] {model} did not come back after the failed update");
        }
    }
    if running.embedding {
        start_embedding();
    }
}

/// Remove what the failed update left and bring the previous engine back.
async fn put_back(current: &Path, previous: &Path) {
    if current.exists() {
        if let Err(e) = std::fs::remove_dir_all(current) {
            log::warn!("[engine_update] cannot remove the partial engine: {e}");
        }
    }
    if let Err(e) = rename_with_retry(previous, current).await {
        log::error!("[engine_update] cannot restore the previous engine: {e}");
    }
}

/// Windows releases the files of a process a moment after it is gone.
async fn rename_with_retry(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut last = None;
    for _ in 0..20 {
        match std::fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("rename failed")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_is_offered_only_for_an_installed_engine_of_another_build() {
        assert!(needs_update(true, None, Some("b12483")));
        assert!(needs_update(true, Some("b7345"), Some("b12483")));
        assert!(!needs_update(true, Some("b12483"), Some("b12483")));
        assert!(!needs_update(false, None, Some("b12483")));
        assert!(!needs_update(true, Some("b7345"), None));
    }

    #[test]
    fn server_binary_is_found_flat_or_under_bin() {
        let name = if cfg!(windows) { "llama-server.exe" } else { "llama-server" };
        let root = std::env::temp_dir().join(format!("gb-engine-{}", uuid::Uuid::new_v4()));
        let (flat, nested) = (root.join("flat"), root.join("nested"));
        std::fs::create_dir_all(&flat).expect("flat dir");
        std::fs::create_dir_all(nested.join("bin")).expect("nested dir");
        assert_eq!(server_binary_in(&flat), None);
        std::fs::write(flat.join(name), b"x").expect("flat file");
        std::fs::write(nested.join("bin").join(name), b"x").expect("nested file");
        assert_eq!(server_binary_in(&flat), Some(flat.join(name)));
        assert_eq!(server_binary_in(&nested), Some(nested.join("bin").join(name)));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn a_failed_update_puts_the_previous_engine_back() {
        let root = std::env::temp_dir().join(format!("gb-engine-swap-{}", uuid::Uuid::new_v4()));
        let (current, previous) = (root.join("llm"), root.join("llm.previous"));
        std::fs::create_dir_all(&current).expect("engine dir");
        std::fs::write(current.join("engine.txt"), b"old").expect("old engine");

        // The update moves the engine in use aside ...
        rename_with_retry(&current, &previous).await.expect("move aside");
        assert!(!current.exists());
        // ... the install leaves a partial folder behind and fails ...
        std::fs::create_dir_all(current.join("bin")).expect("partial dir");
        std::fs::write(current.join("bin").join("half.txt"), b"new").expect("partial file");
        // ... and the previous engine comes back whole.
        put_back(&current, &previous).await;
        assert_eq!(std::fs::read(current.join("engine.txt")).expect("restored"), b"old");
        assert!(!current.join("bin").exists());
        assert!(!previous.exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_cuda_build_downloads_its_runtime_libraries_too() {
        assert!(download_bytes_for("llama-b12483-bin-win-cuda-13.5-x64.zip") > 600_000_000);
        assert!(download_bytes_for("llama-b12483-bin-win-cpu-x64.zip") < 100_000_000);
    }
}
