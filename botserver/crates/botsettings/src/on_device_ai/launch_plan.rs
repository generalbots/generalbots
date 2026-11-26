//! How the chat engine is started on this machine.
//!
//! The engine places as many model layers on the graphics card as fit in the
//! memory that is free when it starts, keeping a safety margin, and runs the
//! rest on the processor, which answers many times slower. To keep every
//! layer on the card when the model allows it, the context is sized from the
//! FREE video memory (measured after the previous engine stopped), one slot
//! is opened (model calls are already serialised by the server), and the
//! margin is lowered from the engine's 1 GiB to [`FIT_TARGET_MIB`].

use std::sync::Mutex;

/// Slots the chat engine opens. The server sends one model call at a time.
pub const PARALLEL_SLOTS: u32 = 1;
/// Video memory the engine leaves free when it chooses how many layers fit.
pub const FIT_TARGET_MIB: u32 = 512;
/// Context used when the card's room is unknown or small.
pub const BASE_CTX: u32 = 8192;
/// Room kept for the engine's working buffers beside weights and context.
const BUFFER_RESERVE_MIB: u64 = 1024;
/// Context memory per token of a dense 8B-class model (f16 keys and values).
const KV_KIB_PER_TOKEN: u64 = 128;
/// Free memory left after a load that still means the card was not filled.
const FULL_CARD_SLACK_MIB: u64 = 256;
const MIB: u64 = 1024 * 1024;

/// Context to start with when none is configured.
pub fn ctx_for_free_vram(model_bytes: u64, vram_free_mib: Option<u64>) -> u32 {
    let (Some(free), true) = (vram_free_mib, model_bytes > 0) else { return BASE_CTX };
    let spare = free.saturating_sub(model_bytes / MIB + BUFFER_RESERVE_MIB);
    [32_768_u32, 16_384]
        .into_iter()
        .find(|ctx| kv_mib(*ctx) <= spare)
        .unwrap_or(BASE_CTX)
}

fn kv_mib(ctx: u32) -> u64 {
    u64::from(ctx) * KV_KIB_PER_TOKEN / 1024
}

/// Whether the model, its context and buffers are expected to fit on the card.
pub fn expect_full_offload(model_bytes: u64, ctx: u32, vram_free_mib: u64) -> bool {
    let need = model_bytes / MIB + kv_mib(ctx) + BUFFER_RESERVE_MIB / 2 + u64::from(FIT_TARGET_MIB);
    vram_free_mib >= need
}

/// Where the model ended up, judged from the free video memory after the load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// The card kept room beyond the margin: every layer is on it.
    Gpu,
    /// The engine filled the card to its margin: some layers run on the processor.
    Split,
    /// The engine took (almost) no video memory.
    Cpu,
    /// No graphics card, or memory could not be read.
    Unknown,
}

/// Judge the placement from free video memory before and after the load.
pub fn placement(before_mib: Option<u64>, after_mib: Option<u64>, fit_target_mib: u64) -> Placement {
    let (Some(before), Some(after)) = (before_mib, after_mib) else { return Placement::Unknown };
    if before.saturating_sub(after) < FULL_CARD_SLACK_MIB {
        Placement::Cpu
    } else if after <= fit_target_mib + FULL_CARD_SLACK_MIB {
        Placement::Split
    } else {
        Placement::Gpu
    }
}

/// The last engine start, for the settings page and the log.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LaunchRecord {
    pub model_file: String,
    pub model_mib: u64,
    pub ctx_size: String,
    pub gpu_layers: String,
    pub vram_free_before_mib: Option<u64>,
    pub vram_free_after_mib: Option<u64>,
    pub placement: Placement,
}

static LAST_LAUNCH: Mutex<Option<LaunchRecord>> = Mutex::new(None);

/// What the last start measured; `None` before the first start.
pub fn last_launch() -> Option<LaunchRecord> {
    LAST_LAUNCH.lock().ok().and_then(|slot| slot.clone())
}

fn free_vram_mib() -> Option<u64> {
    let hardware = super::hardware::probe();
    hardware.cuda_usable.then_some(u64::from(hardware.vram_free_mib))
}

/// Resolve an empty context from the free video memory and log the plan.
/// Called right before the engine is spawned, after the old one stopped.
pub fn resolve(desired: &super::runtime_ctl::DesiredRuntime) -> super::runtime_ctl::DesiredRuntime {
    let model_bytes = std::fs::metadata(&desired.model_path).map(|m| m.len()).unwrap_or(0);
    let free = free_vram_mib();
    let mut resolved = desired.clone();
    if resolved.ctx_size.trim().is_empty() {
        resolved.ctx_size = ctx_for_free_vram(model_bytes, free).to_string();
    }
    let ctx: u32 = resolved.ctx_size.parse().unwrap_or(BASE_CTX);
    let layers = if resolved.gpu_layers.is_empty() { "auto" } else { resolved.gpu_layers.as_str() };
    let free_text = free.map_or_else(|| "unknown".to_string(), |mib| format!("{mib} MiB"));
    let expectation = match free {
        Some(mib) if expect_full_offload(model_bytes, ctx, mib) => "all layers on the graphics card",
        Some(_) => "some layers may run on the processor",
        None => "no graphics card detected",
    };
    log::info!(
        "[launch_plan] model={} ({} MiB) ctx={} gpu_layers={} parallel={} fit_target={} MiB free_vram={} -> {}",
        resolved.model_file, model_bytes / MIB, ctx, layers, PARALLEL_SLOTS, FIT_TARGET_MIB, free_text, expectation,
    );
    if let Ok(mut slot) = LAST_LAUNCH.lock() {
        *slot = Some(LaunchRecord {
            model_file: resolved.model_file.clone(),
            model_mib: model_bytes / MIB,
            ctx_size: resolved.ctx_size.clone(),
            gpu_layers: layers.to_string(),
            vram_free_before_mib: free,
            vram_free_after_mib: None,
            placement: Placement::Unknown,
        });
    }
    resolved
}

/// Measure the card after the engine answered and log where the model went.
pub fn record_after_load(model_file: &str) {
    let after = free_vram_mib();
    let Ok(mut slot) = LAST_LAUNCH.lock() else { return };
    let Some(record) = slot.as_mut().filter(|r| model_file.ends_with(r.model_file.as_str())) else { return };
    record.vram_free_after_mib = after;
    record.placement = placement(record.vram_free_before_mib, after, u64::from(FIT_TARGET_MIB));
    let used = record.vram_free_before_mib.zip(after).map(|(b, a)| b.saturating_sub(a));
    log::info!(
        "[launch_plan] loaded model={} engine_vram={} MiB free_vram_after={} MiB placement={:?}",
        record.model_file,
        used.map_or_else(|| "unknown".to_string(), |mib| mib.to_string()),
        after.map_or_else(|| "unknown".to_string(), |mib| mib.to_string()),
        record.placement,
    );
    if record.placement == Placement::Split {
        log::warn!(
            "[launch_plan] {} does not fit fully on the graphics card ({} MiB free before the load); answers will be slow. Close other programs that use the card or choose a smaller model.",
            record.model_file,
            record.vram_free_before_mib.unwrap_or(0),
        );
    }
}

#[cfg(test)]
#[path = "launch_plan_tests.rs"]
mod launch_plan_tests;
