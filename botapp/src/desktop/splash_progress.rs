//! Drives the splash's six steps from the server's real start.
//!
//! The suite handoff (`splash::watch_boot_ready`) only begins once the API
//! answers, when the server has already finished every step, so the steps are
//! followed from the sidecar log instead, starting before the sidecar spawns.

use super::boot_steps::{LogTail, StepTracker, LAST_STEP};
use super::sidecar::Sidecar;
use log::info;
use std::time::Duration;
use tauri::{Manager, WebviewWindow};

const POLL: Duration = Duration::from_millis(400);

/// Starts following the sidecar log. Call it before the sidecar is spawned:
/// the tail opens at the current end of the log, so earlier runs are ignored.
/// The thread ends with the splash window, or once the last step is shown.
pub fn watch(app: &tauri::AppHandle) {
    let mut tail = LogTail::at_end(Sidecar::log_path());
    let handle = app.clone();
    std::thread::spawn(move || {
        let mut tracker = StepTracker::default();
        let mut shown: Option<u8> = None;
        loop {
            let Some(splash) = handle.get_webview_window("splash") else {
                return;
            };
            for line in tail.read_lines() {
                tracker.observe(&line);
            }
            let step = tracker.current();
            // A script evaluated before the page has loaded is lost, so the
            // first step waits for the page to have painted.
            if super::splash::painted() && shown != Some(step) {
                info!("splash: boot step {step}");
                push_step(&splash, step);
                shown = Some(step);
                if step >= LAST_STEP {
                    return;
                }
            }
            std::thread::sleep(POLL);
        }
    });
}

fn push_step(splash: &WebviewWindow, step: u8) {
    let script = format!(
        "if(typeof window.gbSplashStep==='function')window.gbSplashStep({step});"
    );
    let _ = splash.eval(&script);
}
