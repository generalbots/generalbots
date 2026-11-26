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
    switch_off_services(&mut plan, botcore::product::is_service_off);
    if wants_local {
        plan.push(Component {
            id: "llm",
            label: "Local model runtime",
            timeout_secs: 120,
            optional: true,
            // Windows: the assistant's boot starts the embedding engine and the
            // stored local model itself; a start here would wait for a chat
            // engine that nothing launches yet.
            start: !cfg!(target_os = "windows"),
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
    if wants_media_service(&with_botmodels, cfg!(feature = "gbgov")) {
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

/// Services the product file switches off stay in the plan so the wizard
/// shows them as done, but are never downloaded, started or awaited.
fn switch_off_services(plan: &mut [Component], is_off: impl Fn(&str) -> bool) {
    for component in plan.iter_mut().filter(|component| is_off(component.id)) {
        component.optional = true;
        component.start = false;
        component.unavailable = Some(
            if component.id == "cache" {
                "Built into the assistant"
            } else {
                "Not used in this edition"
            }
            .to_string(),
        );
    }
}

/// Whether setup installs the AI media service. The government desktop edition
/// reads text in images through Windows and has no other use for it, so it
/// never downloads the box; elsewhere `GB_WIZARD_WITH_BOTMODELS=0` opts out.
fn wants_media_service(opt_out: &str, edition_without_media: bool) -> bool {
    !edition_without_media && opt_out.trim() != "0"
}

#[cfg(test)]
mod media_service_tests {
    use super::wants_media_service;

    #[test]
    fn the_media_service_is_skipped_in_the_edition_without_it() {
        assert!(wants_media_service("", false));
        assert!(!wants_media_service("0", false));
        assert!(!wants_media_service("", true));
        assert!(!wants_media_service("1", true));
    }
}

#[cfg(test)]
mod switched_off_tests {
    use super::*;

    #[test]
    fn switched_off_services_are_reported_and_never_provisioned() {
        let mut plan = component_plan();
        switch_off_services(&mut plan, |id| matches!(id, "cache" | "vector_db" | "directory"));
        for component in &plan {
            if matches!(component.id, "cache" | "vector_db" | "directory") {
                assert!(component.unavailable.is_some(), "{}", component.id);
                assert!(component.optional && !component.start, "{}", component.id);
            }
            if matches!(component.id, "vault" | "tables" | "drive") {
                assert!(component.unavailable.is_none(), "{}", component.id);
            }
        }
        let tables = plan.iter().find(|component| component.id == "tables");
        assert!(tables.is_some_and(|component| component.start && !component.optional));
    }
}
