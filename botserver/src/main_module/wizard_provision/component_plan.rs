//! Component sequence for the first-run wizard.
//!
//! Split out of `wizard_provision.rs` for the 450-line structural rule. The
//! `Component` shape it builds is the parent module's private struct, reachable
//! from this descendant through `super::*`.
use super::*;

pub(super) fn component_plan() -> Vec<Component> {
    // An empty mode with a remote endpoint/model means the user picked a hosted
    // provider: never install the ~1.8 GB local runtime in that case. The old
    // `unwrap_or("local")` treated a missing value as "local", so the Kilo flow
    // still downloaded and started llama.cpp (and failed on slow links).
    let model_mode = std::env::var("GB_WIZARD_MODEL_MODE").unwrap_or_default();
    let remote_model = !std::env::var("GB_WIZARD_LLM_URL")
        .unwrap_or_default()
        .trim()
        .is_empty()
        && !std::env::var("GB_WIZARD_LLM_MODEL")
            .unwrap_or_default()
            .trim()
            .is_empty();
    let wants_local = if model_mode.trim().is_empty() {
        !remote_model
    } else {
        model_mode.eq_ignore_ascii_case("local")
    };
    let mut plan = vec![
        Component {
            id: "vault",
            label: "Local vault",
            timeout_secs: 90,
            optional: false,
            start: true,
            unavailable: None,
        },
        // 90 s was not enough on a cold CI runner (first-run smoke test timed
        // out while initdb was still writing); the wait is bounded either way.
        Component {
            id: "tables",
            label: "Database",
            timeout_secs: 180,
            optional: false,
            start: true,
            unavailable: None,
        },
        Component {
            id: "cache",
            label: "Cache",
            timeout_secs: 45,
            optional: false,
            start: true,
            unavailable: None,
        },
        Component {
            id: "drive",
            label: "Object store",
            timeout_secs: 60,
            optional: false,
            start: true,
            unavailable: None,
        },
        Component {
            id: "vector_db",
            label: "Vector store",
            timeout_secs: 60,
            optional: false,
            start: true,
            unavailable: None,
        },
        Component {
            id: "directory",
            label: "Identity provider",
            timeout_secs: 300,
            optional: false,
            start: true,
            unavailable: None,
        },
    ];
    if wants_local {
        plan.push(Component {
            id: "llm",
            label: "Local model runtime",
            timeout_secs: 120,
            optional: true,
            start: true,
            unavailable: None,
        });
    }
    // BotModels ships as a pythonbox: the installer carries the archive and a
    // CPU-tier wheelhouse, so the wizard only has to unpack it. The virtual
    // environment is created offline on first use (no PyPI round-trip), which
    // must never block setup — hence `start: false`.
    // GB_WIZARD_WITH_BOTMODELS=0 opts out entirely. Linux x86_64 and Windows
    // x86_64 have published boxes; Linux arm64 and Windows arm64 get an
    // explicit reason instead of a download that cannot succeed.
    let with_botmodels = std::env::var("GB_WIZARD_WITH_BOTMODELS").unwrap_or_default();
    if with_botmodels.trim() != "0" {
        plan.push(Component {
            id: "botmodels",
            label: "AI media service",
            timeout_secs: 120,
            optional: true,
            start: false,
            unavailable: crate::main_module::media_capabilities::botmodels_unavailable_reason(
                std::env::consts::OS,
                std::env::consts::ARCH,
            ),
        });
    }
    plan
}

/// Returns the first failure reason without including credentials or paths that
/// the wizard cannot display.
