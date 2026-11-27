//! Starts the assistant behind the splash and reveals the suite once it
//! answers, or shows the splash's failure state when it cannot.

use super::boot_guard::{boot_deadline, judge, BootFailure, BootObservation, BootVerdict};
use super::sidecar::port_holder::PortConflict;
use super::sidecar::probe::api_product_url;
use super::sidecar::{ChildState, Sidecar};
use super::splash_failure::{self, Choice};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::Manager;

const POLL: Duration = Duration::from_millis(500);

/// Starts the sidecar and reveals the suite once the API answers.
pub fn start_sidecar_and_show(app: &tauri::App) {
    let resource_dir = app.path().resource_dir().ok();
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        run(handle, resource_dir).await;
    });
}

async fn run(handle: tauri::AppHandle, resource_dir: Option<PathBuf>) {
    loop {
        super::splash_progress::watch(&handle);
        let failure = match boot_once(&handle, resource_dir.clone()).await {
            Ok(()) => {
                super::wizard::reveal_suite(&handle);
                return;
            }
            Err(failure) => failure,
        };
        log::error!("the assistant did not start: {failure:?}");
        if !splash_failure::splash_is_open(&handle) {
            // Nothing to show the failure on: open the suite, as before.
            super::wizard::reveal_suite(&handle);
            return;
        }
        splash_failure::show(&handle, &failure, &Sidecar::log_path()).await;
        match splash_failure::wait_choice(&handle).await {
            Choice::Retry => {
                log::info!("splash: trying the start again");
                let sidecar = handle.state::<Sidecar>().inner().clone();
                // The restart must not attach to the process being replaced.
                tauri::async_runtime::spawn_blocking(move || sidecar.stop())
                    .await
                    .unwrap_or_else(|e| log::warn!("could not stop the sidecar: {e}"));
                splash_failure::restart(&handle);
            }
            Choice::Close => {
                log::info!("splash: closing after a failed start");
                super::tray::quit(&handle);
                return;
            }
        }
    }
}

/// One attempt: attach to a running assistant or spawn one, then wait until
/// it answers, its process exits, or the deadline passes.
async fn boot_once(
    handle: &tauri::AppHandle,
    resource_dir: Option<PathBuf>,
) -> Result<(), BootFailure> {
    let url = api_product_url();
    let sidecar = handle.state::<Sidecar>().inner().clone();
    // A previous run may have left the sidecar alive (for example when the
    // window was closed without a graceful exit). Reuse it instead of
    // starting a second listener on the same port.
    if !Sidecar::wait_for_api(&url, 5).await {
        if let Err(e) = sidecar.start(resource_dir) {
            if let Some(conflict) = e.downcast_ref::<PortConflict>() {
                return Err(BootFailure::PortTaken {
                    port: conflict.port,
                    program: conflict.image.clone(),
                });
            }
            log::error!("Failed to start botserver sidecar: {e}");
            return Err(BootFailure::NotStarted);
        }
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| {
            log::error!("could not build the readiness client: {e}");
            BootFailure::NotStarted
        })?;
    let deadline = boot_deadline();
    let started = Instant::now();
    let mut spawned = false;
    loop {
        let child = sidecar.child_state();
        spawned |= child != ChildState::NotOwned;
        let api_ready = matches!(
            client.get(&url).send().await,
            Ok(response) if response.status().is_success()
        );
        let seen = BootObservation {
            api_ready,
            child,
            spawned,
            elapsed: started.elapsed(),
        };
        match judge(seen, deadline) {
            BootVerdict::Ready => {
                log::info!("API ready at {url} ({:.1}s)", started.elapsed().as_secs_f64());
                return Ok(());
            }
            BootVerdict::Failed(failure) => return Err(failure),
            BootVerdict::Waiting => tokio::time::sleep(POLL).await,
        }
    }
}
