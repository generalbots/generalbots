//! Container launch: image selection (#2062) and the cold-start bootstrap.
//!
//! Split out of `lifecycle` per the AGENTS.md file-size rule. The profile data
//! lives in [`super::profile`]; this module only decides which image to boot
//! and refuses a profile the host cannot deliver.

use axum::http::StatusCode;
use std::sync::Arc;

use crate::AgentService;

use super::bootstrap::bootstrap_script;
use super::incus::{
    cold_launch_wait_secs, exec_in_vm, instance_log_tail, run_incus, wait_for_running,
    wait_for_running_within,
};
use super::profile::{self, Availability};
use super::retry::{retry_incus, start_container};

/// Environment variable pinning the launch image.
const IMAGE_ENV: &str = "AGENT_VM_IMAGE";
/// Environment variable overriding the prefabricated image alias.
const PREFAB_IMAGE_ENV: &str = "AGENT_VM_PREFAB_IMAGE";

/// Provision the container the same way the Vibe runtime does: a Debian
/// system container with explicit CPU/memory limits attached to the managed
/// bridge (internet egress). Runtime packages (python3/node) are installed
/// in the background so the first user message is not blocked by apt.
pub(super) async fn launch_and_wait(
    state: &Arc<AgentService>,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    let (image, prefab_image) = resolve_launch_image(state).await;
    // #2062 — a profile this host cannot deliver is a capability error, not a
    // silent downgrade: a workspace without Chromium fails every PDF request.
    let profile = profile::active_profile();
    if let Err(error) = profile::ensure_available(profile, &Availability::from_env(prefab_image)) {
        tracing::error!("{error}");
        return Err((StatusCode::PRECONDITION_FAILED, error.to_string()));
    }
    let cpu = std::env::var("AGENT_VM_CPU").unwrap_or_else(|_| "1".to_string());
    let mem = std::env::var("AGENT_VM_MEMORY").unwrap_or_else(|_| "1GiB".to_string());
    let bridge = std::env::var("AGENT_VM_BRIDGE").unwrap_or_else(|_| "incusbr0".to_string());

    let launch_args = vec![
        "launch".to_string(),
        image,
        vm_name.to_string(),
        "--config".to_string(),
        format!("limits.cpu={cpu}"),
        "--config".to_string(),
        format!("limits.memory={mem}"),
    ];
    // A cold launch pulls and unpacks the image and can leave the daemon busy.
    // Retry with backoff instead of turning the first transient failure into a
    // terminal workspace error (issue #1798).
    if let Err(e) = retry_incus(state, &launch_args, "launch").await {
        // A host without Incus (or without socket permission) never recovers
        // by retrying `incus start`; return the actionable setup error now
        // instead of waiting out the cold-launch budget.
        if e.0 == StatusCode::SERVICE_UNAVAILABLE {
            return Err(e);
        }
        // A previous attempt can leave the container created but stopped; the
        // launch then fails with "already exists" and the workspace stayed
        // permanently unavailable. Start the existing container instead.
        tracing::warn!(
            "incus launch for {vm_name} failed after retries ({}); starting the existing container",
            e.1
        );
        if let Err(start_error) = start_container(state, vm_name).await {
            tracing::warn!(
                "container {vm_name} is not available to start yet ({}); waiting for the cold launch",
                start_error.1
            );
        }
    }

    // Attach the managed bridge. The default profile often ships no NIC;
    // ignore the error when a NIC already exists (idempotent provisioning).
    let _ = run_incus(
        state,
        vec![
            "config".into(),
            "device".into(),
            "add".into(),
            vm_name.into(),
            "eth0".into(),
            "nic".into(),
            format!("network={bridge}"),
            "name=eth0".into(),
        ],
    )
    .await;

    // The first launch of a fresh image can download/unpack for minutes; use
    // the cold budget before declaring the workspace unavailable.
    if wait_for_running_within(state, vm_name, cold_launch_wait_secs())
        .await
        .is_err()
    {
        // The container may still be created but stopped; start it explicitly
        // and wait again instead of failing the whole workspace.
        tracing::warn!("container {vm_name} did not start in time; retrying incus launch/start");
        // The instance may never have been created if the original launch kept
        // failing while the image was still downloading. Re-run launch (it is
        // idempotent when the instance now exists) before starting it.
        if let Err(launch_error) = retry_incus(state, &launch_args, "launch").await {
            if launch_error.0 == StatusCode::SERVICE_UNAVAILABLE {
                return Err(launch_error);
            }
            tracing::warn!(
                "container {vm_name} launch retry failed ({}); trying incus start",
                launch_error.1
            );
        }
        if let Err(start_error) = start_container(state, vm_name).await {
            tracing::warn!(
                "container {vm_name} still cannot start ({}); checking the instance log",
                start_error.1
            );
        }
        if let Err(wait_error) = wait_for_running(state, vm_name).await {
            // A created-but-never-running container keeps its reason inside
            // the instance log (bpf/mount/cgroup host-side failures); the
            // daemon can even return exit 0 from `start` while the instance
            // stays STOPPED. Attach that evidence so callers surface the
            // true cause instead of a bare timeout.
            let reason = match instance_log_tail(state, vm_name).await {
                Some(tail) => format!("{}; instance log: {tail}", wait_error.1),
                None => wait_error.1,
            };
            return Err((wait_error.0, reason));
        }
    }

    // Best-effort runtime bootstrap. Logs land inside the container; a
    // failure is reported by the first exec, never by provisioning.
    let state_for_bootstrap = Arc::clone(state);
    let name_for_bootstrap = vm_name.to_string();
    tokio::spawn(async move {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            exec_in_vm(
                &state_for_bootstrap,
                &name_for_bootstrap,
                &["sh".into(), "-c".into(), bootstrap_script()],
            ),
        )
        .await;
        match result {
            Ok(Ok((_, _, stderr))) if !stderr.trim().is_empty() => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap}: {stderr}");
            }
            Ok(Err((code, err))) => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap} ({code}): {err}");
            }
            Err(_) => tracing::warn!("agent runtime bootstrap {name_for_bootstrap} timed out"),
            _ => {}
        }
    });

    Ok(())
}

/// The image to launch and whether it is the pre-fabricated workspace image.
///
/// `AGENT_VM_IMAGE` wins when set, then the prefabricated alias when the host
/// has it, then today's base image. The base fallback is logged: it is the
/// only path that installs the profile at first boot instead of using the
/// baked image.
async fn resolve_launch_image(state: &Arc<AgentService>) -> (String, bool) {
    let prefab = std::env::var(PREFAB_IMAGE_ENV)
        .unwrap_or_else(|_| profile::DEFAULT_PREFAB_IMAGE.to_string());
    let requested = std::env::var(IMAGE_ENV).ok();
    let aliases = match requested {
        Some(_) => Vec::new(),
        None => installed_image_aliases(state).await,
    };
    let (image, prefab_image) = profile::resolve_image(requested.as_deref(), &prefab, &aliases);
    if !prefab_image && image == profile::BASE_IMAGE {
        tracing::info!(
            "prefabricated image '{prefab}' is not installed; launching '{}' and installing the \
             profile at first boot",
            profile::BASE_IMAGE
        );
    }
    (image, prefab_image)
}

/// Aliases of the images the host already has. A listing failure is not fatal:
/// the launch falls back to the base image, exactly like a host without the
/// prefabricated one.
async fn installed_image_aliases(state: &Arc<AgentService>) -> Vec<String> {
    let args = vec![
        "image".to_string(),
        "list".to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];
    let output = match run_incus(state, args).await {
        Ok(output) => output,
        Err((_, error)) => {
            tracing::warn!("cannot list incus images ({error}); using the base image");
            return Vec::new();
        }
    };
    let raw = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!("cannot parse incus image list ({error}); using the base image");
            return Vec::new();
        }
    };
    let mut aliases = Vec::new();
    if let Some(images) = parsed.as_array() {
        for image in images {
            if let Some(list) = image["aliases"].as_array() {
                aliases.extend(
                    list.iter()
                        .filter_map(|alias| alias["name"].as_str())
                        .map(str::to_string),
                );
            }
        }
    }
    aliases
}
