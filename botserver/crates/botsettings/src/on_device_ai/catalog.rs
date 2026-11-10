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
    },
    ModelSpec {
        id: "llama-3.3-70b-q4km",
        name: "Llama-3.3-70B",
        params: "70B",
        quant: "Q4_K_M",
        size_bytes: 42_949_672_960,
        min_vram_gb: 48,
        repo: "bartowski/Llama-3.3-70B-Instruct-GGUF",
        file: "Llama-3.3-70B-Instruct-Q4_K_M.gguf",
        sha256: "",
        server_only: false,
    },
    ModelSpec {
        id: "llama-3.3-70b-q5km",
        name: "Llama-3.3-70B",
        params: "70B",
        quant: "Q5_K_M",
        size_bytes: 55_834_574_848,
        min_vram_gb: 72,
        repo: "bartowski/Llama-3.3-70B-Instruct-GGUF",
        file: "Llama-3.3-70B-Instruct-Q5_K_M.gguf",
        sha256: "",
        server_only: false,
    },
    ModelSpec {
        id: "gpt-oss-120b-mxfp4",
        name: "gpt-oss-120b",
        params: "120B MoE",
        quant: "MXFP4",
        size_bytes: 64_424_509_440,
        min_vram_gb: 96,
        repo: "openai/gpt-oss-120b",
        file: "gpt-oss-120b-mxfp4-00001-of-00002.gguf",
        sha256: "",
        server_only: false,
    },
    ModelSpec {
        id: "gpt-oss-120b-q6",
        name: "gpt-oss-120b",
        params: "120B MoE",
        quant: "Q6_K",
        size_bytes: 91_268_907_008,
        min_vram_gb: 128,
        repo: "ggml-org/gpt-oss-120b-GGUF",
        file: "gpt-oss-120b-Q6_K.gguf",
        sha256: "",
        server_only: false,
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
