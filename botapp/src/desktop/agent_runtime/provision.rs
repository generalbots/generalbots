//! Provisioning step machine: installs WSL, downloads the rootfs and
//! workspace image, imports the distro, waits for first boot, and writes
//! the ready marker.

#[path = "provision_helpers.rs"]
mod helpers;

use std::path::Path;
use std::sync::atomic::AtomicBool;

use helpers::{
    cancelled, download_and_verify, emit_progress, fetch_manifest,
    record_error, save_state, wsl_exe,
};

use super::host::{self, RuntimeHost};
use super::state::{
    ErrorEvent, PersistedState, ProgressEvent, ProvisionStep, ReadyMarker,
    DISTRO_NAME, PINNED_RUNTIME_VERSION,
};

pub(crate) trait ProvisionEvents: Send + Sync {
    fn progress(&self, event: ProgressEvent);
    fn error(&self, event: ErrorEvent);
}

pub(crate) fn run(
    h: &dyn RuntimeHost,
    stack: &Path,
    events: &dyn ProvisionEvents,
    cancel: &AtomicBool,
) -> Result<(), &'static str> {
    let rt_dir = stack.join("agent-runtime");
    let sp = rt_dir.join("state.json");
    let dd = rt_dir.join("distro");

    if let Err(e) = h.create_dir_all(&rt_dir) {
        log::error!("agent_runtime: cannot create directory: {e}");
        return Err("unknown");
    }

    let mut st = match h.read_file(&sp) {
        Ok(data) => PersistedState::load(&data).unwrap_or_else(|_| PersistedState::new()),
        Err(_) => PersistedState::new(),
    };
    // Clear last error at the start of an install attempt.
    st.last_error = None;
    save_state(h, &sp, &st);

    // ── S1: WSL install ─────────────────────────────────────────────
    if st.completed_step.is_none() {
        if cancelled(cancel) { return Err("cancelled"); }
        emit_progress(events, "wsl", 0, 0, 0);
        if !h.wsl_installed() {
            match h.run_wsl_install_elevated() {
                Ok(()) => {
                    emit_progress(events, "wsl", 0, 0, 100);
                    events.error(ErrorEvent {
                        step: "wsl", code: "needs_reboot",
                        detail: "WSL installed; reboot required".into(),
                    });
                    st.completed_step = Some(ProvisionStep::WslInstall);
                    save_state(h, &sp, &st);
                    return Err("needs_reboot");
                }
                Err(e) => {
                    log::warn!("agent_runtime: WSL install failed: {e}");
                    record_error(&mut st, "wsl", "wsl_failed", "WSL installation failed or was cancelled");
                    save_state(h, &sp, &st);
                    events.error(ErrorEvent {
                        step: "wsl", code: "wsl_failed",
                        detail: "WSL installation failed or was cancelled".into(),
                    });
                    return Err("wsl_failed");
                }
            }
        }
        st.completed_step = Some(ProvisionStep::WslInstall);
        save_state(h, &sp, &st);
    }

    // ── S2: download rootfs ─────────────────────────────────────────
    if st.completed_step < Some(ProvisionStep::RootfsDownload) {
        if cancelled(cancel) { return Err("cancelled"); }
        emit_progress(events, "rootfs_download", 0, 0, 0);
        let manifest = fetch_manifest(h, events)?;
        let rootfs_path = rt_dir.join(&manifest.rootfs.file);
        let needed = manifest.rootfs.size + manifest.workspace_image.size + 512 * 1024 * 1024;
        if let Ok(free) = h.available_disk_space(&rt_dir) {
            if free < needed {
                let detail = format!("need {needed} bytes, only {free} available");
                record_error(&mut st, "rootfs_download", "disk_full", &detail);
                save_state(h, &sp, &st);
                events.error(ErrorEvent { step: "rootfs_download", code: "disk_full", detail });
                return Err("disk_full");
            }
        }
        download_and_verify(
            h, &super::state::asset_url(&manifest.rootfs.file), &rootfs_path,
            manifest.rootfs.size, &manifest.rootfs.sha256,
            "rootfs_download", events, cancel,
        )?;
        st.rootfs_file = Some(manifest.rootfs.file.clone());
        st.rootfs_sha256 = Some(manifest.rootfs.sha256.clone());
        st.image_file = Some(manifest.workspace_image.file.clone());
        st.image_sha256 = Some(manifest.workspace_image.sha256.clone());
        st.completed_step = Some(ProvisionStep::RootfsDownload);
        save_state(h, &sp, &st);
    }

    // ── S3: import distro ───────────────────────────────────────────
    if st.completed_step < Some(ProvisionStep::DistroImport) {
        if cancelled(cancel) { return Err("cancelled"); }
        emit_progress(events, "distro_import", 0, 0, 0);
        step_import_distro(h, events, &mut st, &sp, &rt_dir, &dd)?;
    }

    // ── S4: wait for first boot ─────────────────────────────────────
    if st.completed_step < Some(ProvisionStep::FirstBoot) {
        step_first_boot(h, events, cancel, &mut st, &sp)?;
    }

    // ── S5: download + import workspace image ───────────────────────
    if st.completed_step < Some(ProvisionStep::ImageImport) {
        step_image_import(h, events, cancel, &mut st, &rt_dir, &sp)?;
    }

    // ── S6: write ready.json + cleanup ──────────────────────────────
    if st.completed_step < Some(ProvisionStep::Finish) {
        if cancelled(cancel) { return Err("cancelled"); }
        emit_progress(events, "finish", 0, 0, 0);
        let marker = ReadyMarker {
            schema_version: 1,
            runtime_version: PINNED_RUNTIME_VERSION.to_string(),
            distro: DISTRO_NAME.to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            release_tag: format!("gbgov-agent-runtime-v{PINNED_RUNTIME_VERSION}"),
            image_fingerprint: st.image_sha256.clone().unwrap_or_default(),
        };
        let marker_json = serde_json::to_vec_pretty(&marker).map_err(|_| "unknown")?;
        h.write_file(&rt_dir.join("ready.json"), &marker_json).map_err(|_| "unknown")?;
        if let Some(ref f) = st.rootfs_file { let _ = h.remove_file(&rt_dir.join(f)); }
        if let Some(ref f) = st.image_file { let _ = h.remove_file(&rt_dir.join(f)); }
        emit_progress(events, "finish", 0, 0, 100);
        st.completed_step = Some(ProvisionStep::Finish);
        // Clear last_error on success.
        st.last_error = None;
        save_state(h, &sp, &st);
    }

    st.completed_step = Some(ProvisionStep::Done);
    st.last_error = None;
    save_state(h, &sp, &st);
    Ok(())
}

/// S3: Handle the case where the distro is already registered (crash during
/// a previous import).  If the name is registered with a BasePath inside our
/// folder and the disk file exists, treat the import as done.  If the name
/// is registered elsewhere, stop.  Otherwise unregister the half-made entry.
fn step_import_distro(
    h: &dyn RuntimeHost,
    events: &dyn ProvisionEvents,
    st: &mut PersistedState,
    sp: &Path,
    rt_dir: &Path,
    dd: &Path,
) -> Result<(), &'static str> {
    let rootfs_file = st.rootfs_file.as_deref().ok_or("unknown")?;
    let rootfs_path = rt_dir.join(rootfs_file);
    let wsl = wsl_exe();
    let ws = wsl.to_string_lossy();

    if let Err(e) = h.create_dir_all(dd) {
        log::error!("agent_runtime: cannot create distro dir: {e}");
        return Err("import_failed");
    }

    if h.distro_registered(DISTRO_NAME) {
        if let Some(base) = h.distro_base_path(DISTRO_NAME) {
            if host::path_is_inside(&base, dd) {
                // Our distro from a previous attempt.
                // Check if the disk file exists (import completed).
                let vhdx = base.join("ext4.vhdx");
                if h.file_exists(&vhdx) {
                    log::info!("agent_runtime: distro already imported (found {})", vhdx.display());
                    emit_progress(events, "distro_import", 0, 0, 100);
                    st.completed_step = Some(ProvisionStep::DistroImport);
                    save_state(h, sp, st);
                    return Ok(());
                }
                // Half-made entry: unregister and re-import.
                log::info!("agent_runtime: unregistering half-made distro entry");
                let _ = h.run_command(&ws, &["--unregister", DISTRO_NAME]);
            } else {
                // A distro with our name exists outside our folder.
                let detail = format!(
                    "a distro named {} already exists at {}; cannot import",
                    DISTRO_NAME, base.display()
                );
                record_error(st, "distro_import", "import_failed", &detail);
                save_state(h, sp, st);
                events.error(ErrorEvent {
                    step: "distro_import", code: "import_failed", detail,
                });
                return Err("import_failed");
            }
        }
    }

    let result = h.run_command(
        &ws,
        &["--import", DISTRO_NAME, &dd.to_string_lossy(),
          &rootfs_path.to_string_lossy(), "--version", "2"],
    );
    match result {
        Ok((0, _, _)) => log::info!("agent_runtime: distro imported"),
        Ok((code, _, stderr)) => {
            log::warn!("agent_runtime: wsl --import failed ({code}): {stderr}");
            record_error(st, "distro_import", "import_failed", "wsl --import failed");
            save_state(h, sp, st);
            events.error(ErrorEvent {
                step: "distro_import", code: "import_failed",
                detail: "wsl --import failed".into(),
            });
            return Err("import_failed");
        }
        Err(e) => {
            log::warn!("agent_runtime: wsl --import error: {e}");
            record_error(st, "distro_import", "import_failed", "could not run wsl --import");
            save_state(h, sp, st);
            events.error(ErrorEvent {
                step: "distro_import", code: "import_failed",
                detail: "could not run wsl --import".into(),
            });
            return Err("import_failed");
        }
    }

    // Best-effort sparse VHDX.
    let _ = h.run_command(&ws, &["--manage", DISTRO_NAME, "--set-sparse", "true"]);

    emit_progress(events, "distro_import", 0, 0, 100);
    st.completed_step = Some(ProvisionStep::DistroImport);
    save_state(h, sp, st);
    Ok(())
}

fn step_first_boot(
    h: &dyn RuntimeHost,
    events: &dyn ProvisionEvents,
    cancel: &AtomicBool,
    st: &mut PersistedState,
    sp: &Path,
) -> Result<(), &'static str> {
    if cancelled(cancel) { return Err("cancelled"); }
    emit_progress(events, "first_boot", 0, 0, 0);
    let status_file = "/var/lib/generalbots/incus-firstboot.status";
    let wsl = wsl_exe();
    let ws = wsl.to_string_lossy();
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300);
    loop {
        if cancelled(cancel) { return Err("cancelled"); }
        if start.elapsed() > timeout {
            record_error(st, "first_boot", "first_boot_failed", "timed out");
            save_state(h, sp, st);
            events.error(ErrorEvent {
                step: "first_boot", code: "first_boot_failed",
                detail: "timed out waiting for first boot".into(),
            });
            return Err("first_boot_failed");
        }
        if let Ok((0, stdout, _)) = h.run_command(
            &ws, &["-d", DISTRO_NAME, "--exec", "cat", status_file],
        ) {
            let status = stdout.trim();
            if status.contains("ok") { break; }
            if let Some(reason) = status.strip_prefix("failed:") {
                let detail = reason.trim().to_string();
                record_error(st, "first_boot", "first_boot_failed", &detail);
                save_state(h, sp, st);
                events.error(ErrorEvent {
                    step: "first_boot", code: "first_boot_failed", detail,
                });
                return Err("first_boot_failed");
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    emit_progress(events, "first_boot", 0, 0, 100);
    st.completed_step = Some(ProvisionStep::FirstBoot);
    save_state(h, sp, st);
    Ok(())
}

fn step_image_import(
    h: &dyn RuntimeHost,
    events: &dyn ProvisionEvents,
    cancel: &AtomicBool,
    st: &mut PersistedState,
    rt_dir: &Path,
    sp: &Path,
) -> Result<(), &'static str> {
    if cancelled(cancel) { return Err("cancelled"); }
    emit_progress(events, "image_download", 0, 0, 0);
    let image_file = st.image_file.as_deref().ok_or("unknown")?;
    let image_sha = st.image_sha256.as_deref().ok_or("unknown")?;
    let image_path = rt_dir.join(image_file);
    let manifest = fetch_manifest(h, events)?;
    download_and_verify(
        h, &super::state::asset_url(image_file), &image_path,
        manifest.workspace_image.size, image_sha,
        "image_download", events, cancel,
    )?;
    emit_progress(events, "image_import", 0, 0, 0);

    // Convert Windows path to WSL path.  After first boot the distro is
    // running, so use `wslpath -u` for an authoritative answer.
    let win_path = image_path.to_string_lossy();
    let wsl = wsl_exe();
    let ws = wsl.to_string_lossy();
    let wsl_path = match h.run_command(
        &ws, &["-d", DISTRO_NAME, "--exec", "wslpath", "-u", &win_path],
    ) {
        Ok((0, stdout, _)) if !stdout.trim().is_empty() => stdout.trim().to_string(),
        _ => {
            // Fallback to heuristic conversion.
            helpers::windows_to_wsl_path(&win_path).map_err(|detail| {
                record_error(st, "image_import", "import_failed", &detail);
                save_state(h, sp, st);
                events.error(ErrorEvent {
                    step: "image_import", code: "import_failed", detail,
                });
                "import_failed"
            })?
        }
    };

    // Each argument is a separate argv element -- never through a shell.
    let result = h.run_command(
        &ws,
        &["-d", DISTRO_NAME, "--exec", "incus", "image", "import",
          &wsl_path, "--alias", "gb-workspace-office"],
    );
    match result {
        Ok((0, _, _)) => {}
        Ok((code, _, stderr)) => {
            log::warn!("agent_runtime: incus image import failed ({code}): {stderr}");
            record_error(st, "image_import", "import_failed", "incus image import failed");
            save_state(h, sp, st);
            events.error(ErrorEvent {
                step: "image_import", code: "import_failed",
                detail: "incus image import failed".into(),
            });
            return Err("import_failed");
        }
        Err(e) => {
            log::warn!("agent_runtime: incus image import failed: {e}");
            record_error(st, "image_import", "import_failed", "could not run incus image import");
            save_state(h, sp, st);
            events.error(ErrorEvent {
                step: "image_import", code: "import_failed",
                detail: "could not run incus image import".into(),
            });
            return Err("import_failed");
        }
    }

    if let Some(fp) = manifest.workspace_image.incus_fingerprint.as_deref() {
        let result = h.run_command(
            &ws,
            &["-d", DISTRO_NAME, "--exec", "incus", "image", "info",
              "gb-workspace-office"],
        );
        match result {
            Ok((0, stdout, _)) if stdout.contains(fp) => {}
            _ => {
                record_error(st, "image_import", "checksum", "Incus fingerprint mismatch");
                save_state(h, sp, st);
                events.error(ErrorEvent {
                    step: "image_import", code: "checksum",
                    detail: "Incus fingerprint mismatch".into(),
                });
                return Err("checksum");
            }
        }
    }

    emit_progress(events, "image_import", 0, 0, 100);
    st.completed_step = Some(ProvisionStep::ImageImport);
    save_state(h, sp, st);
    Ok(())
}

#[cfg(test)]
#[path = "provision_tests.rs"]
mod tests;
