//! On-device model catalogue (#2109).
//!
//! One quantised GGUF per VRAM tier. The table is data only: tier selection,
//! recommendation and status derivation are pure functions so they can be
//! unit-tested without a GPU, a model file or a running engine.
//!
//! Sizes are the published file sizes; `sha256` is pinned per entry before a
//! build ships. An empty digest means the downloader verifies the byte size
//! only and the row reports `checksum_pending` in the API.

use serde::Serialize;

/// One downloadable local model.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModelSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub params: &'static str,
    pub quant: &'static str,
    pub size_bytes: u64,
    pub min_vram_gb: u16,
    pub repo: &'static str,
    pub file: &'static str,
    /// Empty until the published digest is pinned at build time.
    pub sha256: &'static str,
    /// The bundled server-class model: never recommended for a workstation.
    pub server_only: bool,
    /// SPDX licence identifier shown in the UI.
    pub licence: &'static str,
    /// Host where the file is downloaded from (shown in the UI).
    pub source_host: &'static str,
}

/// `Active`, `Downloaded`, `Downloading` or `Available` — exactly one per row.
pub(crate) const STATUS_ACTIVE: &str = "active";
pub(crate) const STATUS_DOWNLOADED: &str = "downloaded";
pub(crate) const STATUS_DOWNLOADING: &str = "downloading";
pub(crate) const STATUS_AVAILABLE: &str = "available";

/// The tiers from the issue, ascending by VRAM. The server profile keeps the
/// bundled model and is listed last so a workstation never sees it first.
pub(crate) const CATALOG: &[ModelSpec] = &[
    ModelSpec {
        id: "deepseek-r1-distill-qwen-7b-q4km",
        name: "DeepSeek-R1-Distill-Qwen-7B",
        params: "7B",
        quant: "Q4_K_M",
        size_bytes: 4_683_073_504,
        min_vram_gb: 6,
        repo: "bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF",
        file: "DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
        sha256: "731ece8d06dc7eda6f6572997feb9ee1258db0784827e642909d9b565641937b",
        server_only: false,
        licence: "MIT",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "qwen3.5-9b-q4km",
        name: "Qwen3.5-9B",
        params: "9B",
        quant: "Q4_K_M",
        size_bytes: 5_905_580_032,
        min_vram_gb: 8,
        repo: "unsloth/Qwen3.5-9B-GGUF",
        file: "Qwen3.5-9B-Q4_K_M.gguf",
        sha256: "",
        server_only: false,
        licence: "Apache-2.0",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "glm-4-9b-chat-q4km",
        name: "GLM-4-9B",
        params: "9B",
        quant: "Q4_K_M",
        size_bytes: 6_250_926_848,
        min_vram_gb: 8,
        repo: "bartowski/glm-4-9b-chat-GGUF",
        file: "glm-4-9b-chat-Q4_K_M.gguf",
        sha256: "aa6cb8f5ef0a70399bdbf92eef566c26c6017ee0b131a424b65b5c757f1c81a2",
        server_only: false,
        licence: "LicenseRef-GLM-4",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "qwen3.5-14b-q4km",
        name: "Qwen3.5-14B",
        params: "14B",
        quant: "Q4_K_M",
        size_bytes: 9_663_676_416,
        min_vram_gb: 12,
        repo: "unsloth/Qwen3.5-14B-GGUF",
        file: "Qwen3.5-14B-Q4_K_M.gguf",
        sha256: "",
        server_only: false,
        licence: "Apache-2.0",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "qwen3.8-27b-q3kxl",
        name: "Qwen3.8-27B",
        params: "27B",
        quant: "Q3_K_XL",
        size_bytes: 11_769_856_000,
        min_vram_gb: 16,
        repo: "unsloth/Qwen3.8-27B-GGUF",
        file: "Qwen3.8-27B-Q3_K_XL.gguf",
        sha256: "",
        server_only: false,
        licence: "Apache-2.0",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "qwen3.8-27b-udq4kxl",
        name: "Qwen3.8-27B",
        params: "27B",
        quant: "UD-Q4_K_XL",
        size_bytes: 19_240_000_000,
        min_vram_gb: 24,
        repo: "unsloth/Qwen3.8-27B-GGUF",
        file: "Qwen3.8-27B-UD-Q4_K_XL.gguf",
        sha256: "",
        server_only: false,
        licence: "Apache-2.0",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "qwen3.8-27b-q6",
        name: "Qwen3.8-27B",
        params: "27B",
        quant: "Q6_K",
        size_bytes: 25_300_000_000,
        min_vram_gb: 32,
        repo: "unsloth/Qwen3.8-27B-GGUF",
        file: "Qwen3.8-27B-Q6_K.gguf",
        sha256: "",
        server_only: false,
        licence: "Apache-2.0",
        source_host: "huggingface.co",
    },
    ModelSpec {
        id: "deepseek-v3-bundled",
        name: "DeepSeek V3 (bundled)",
        params: "671B MoE",
        quant: "Q4",
        size_bytes: 400_000_000_000,
        min_vram_gb: 320,
        repo: "",
        file: "deepseek-v3-q4.gguf",
        sha256: "",
        server_only: true,
        licence: "MIT",
        source_host: "",
    },
];

/// Named bundles written together with the model on activation.
#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct Profile {
    pub id: &'static str,
    pub label: &'static str,
    pub ctx_size: u32,
    pub gpu_layers: &'static str,
}

pub(crate) const PROFILES: &[Profile] = &[
    Profile { id: "fast", label: "Fast", ctx_size: 8_192, gpu_layers: "auto" },
    Profile { id: "balanced", label: "Balanced", ctx_size: 32_768, gpu_layers: "auto" },
    Profile { id: "max-context", label: "Max context", ctx_size: 131_072, gpu_layers: "auto" },
];

pub(crate) fn profiles() -> &'static [Profile] {
    PROFILES
}

pub(crate) fn find_profile(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|profile| profile.id == id)
}

pub(crate) fn find(id: &str) -> Option<&'static ModelSpec> {
    CATALOG.iter().find(|spec| spec.id == id)
}

/// Memory the engine can use: a detected GPU wins, otherwise the whole
/// system RAM backs CPU inference. Pure so the recommendation is testable.
pub(crate) fn effective_budget_gb(vram_gb: u16, ram_gb: u64) -> u16 {
    if vram_gb > 0 {
        return vram_gb;
    }
    ram_gb.min(u64::from(u16::MAX)) as u16
}

/// Largest non-server model whose minimum VRAM fits the budget. When nothing
/// fits (a very small machine) the smallest entry is returned so the page
/// still offers one obvious button, flagged `fits=false`.
pub(crate) fn recommended(vram_gb: u16, ram_gb: u64) -> &'static ModelSpec {
    let budget = effective_budget_gb(vram_gb, ram_gb);
    let mut best: Option<&'static ModelSpec> = None;
    let mut smallest: Option<&'static ModelSpec> = None;
    for spec in CATALOG.iter().filter(|spec| !spec.server_only) {
        if smallest.is_none_or(|current| spec.min_vram_gb < current.min_vram_gb) {
            smallest = Some(spec);
        }
        if spec.min_vram_gb <= budget
            && best.is_none_or(|current| spec.min_vram_gb > current.min_vram_gb)
        {
            best = Some(spec);
        }
    }
    match (best, smallest) {
        (Some(spec), _) => spec,
        (None, Some(spec)) => spec,
        (None, None) => &CATALOG[0],
    }
}

pub(crate) fn fits(spec: &ModelSpec, vram_gb: u16, ram_gb: u64) -> bool {
    spec.min_vram_gb <= effective_budget_gb(vram_gb, ram_gb)
}

/// Three-tier fit verdict. Rule: a model's file size plus ~2 GB context overhead
/// against available memory. "comfortable" = budget >= min_vram * 1.25,
/// "tight" = budget >= min_vram, "wont_run" = budget < min_vram.
pub(crate) fn fit_verdict(spec: &ModelSpec, vram_gb: u16, ram_gb: u64) -> &'static str {
    let budget = effective_budget_gb(vram_gb, ram_gb);
    let comfortable_threshold = spec.min_vram_gb.saturating_add(spec.min_vram_gb / 4);
    if budget >= comfortable_threshold {
        "comfortable"
    } else if budget >= spec.min_vram_gb {
        "tight"
    } else {
        "wont_run"
    }
}

/// Status precedence: active beats downloaded, downloaded beats a running
/// download, and an idle entry is simply available.
pub(crate) fn row_status(active: bool, downloaded: bool, downloading: bool) -> &'static str {
    if active {
        return STATUS_ACTIVE;
    }
    if downloaded {
        return STATUS_DOWNLOADED;
    }
    if downloading {
        return STATUS_DOWNLOADING;
    }
    STATUS_AVAILABLE
}

/// Every catalogue entry, in declaration order.
#[cfg(test)]
pub(crate) fn catalog() -> &'static [ModelSpec] {
    CATALOG
}

/// One VRAM tier. `tier_gb` is `None` for the server profile group. The HTTP
/// layer builds the JSON, so this stays a plain in-crate view.
#[derive(Debug, Clone)]
pub(crate) struct TierGroup {
    pub tier_gb: Option<u16>,
    pub label: String,
    pub models: Vec<&'static ModelSpec>,
}

/// Rows grouped by tier, ascending, server profile last.
pub(crate) fn tier_groups() -> Vec<TierGroup> {
    let mut groups: Vec<TierGroup> = Vec::new();
    let mut tiers: Vec<u16> = CATALOG
        .iter()
        .filter(|spec| !spec.server_only)
        .map(|spec| spec.min_vram_gb)
        .collect();
    tiers.sort_unstable();
    tiers.dedup();
    for tier in tiers {
        groups.push(TierGroup {
            tier_gb: Some(tier),
            label: format!("{tier} GB VRAM"),
            models: CATALOG
                .iter()
                .filter(|spec| !spec.server_only && spec.min_vram_gb == tier)
                .collect(),
        });
    }
    let server: Vec<&'static ModelSpec> = CATALOG.iter().filter(|spec| spec.server_only).collect();
    if !server.is_empty() {
        groups.push(TierGroup {
            tier_gb: None,
            label: "Server profile".to_string(),
            models: server,
        });
    }
    groups
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod catalog_tests;
