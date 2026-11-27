//! Component sequence for the first-run wizard.
//!
//! Split out of `wizard_provision.rs` for the 450-line structural rule. The
//! `Component` shape it builds is the parent module's private struct, reachable
//! from this descendant through `super::*`.
use super::*;

pub(super) fn component_plan() -> Vec<Component> {
    build_plan(
        &std::env::var("GB_WIZARD_MODEL_MODE").unwrap_or_else(|e| {
            log::debug!("GB_WIZARD_MODEL_MODE is not set ({e}); using an empty value");
            String::new()
        }),
        &std::env::var("GB_WIZARD_LLM_URL").unwrap_or_else(|e| {
            log::debug!("GB_WIZARD_LLM_URL is not set ({e}); using an empty value");
            String::new()
        }),
        &std::env::var("GB_WIZARD_LLM_MODEL").unwrap_or_else(|e| {
            log::debug!("GB_WIZARD_LLM_MODEL is not set ({e}); using an empty value");
            String::new()
        }),
        &std::env::var("GB_WIZARD_WITH_BOTMODELS").unwrap_or_else(|e| {
            log::debug!("GB_WIZARD_WITH_BOTMODELS is not set ({e}); using an empty value");
            String::new()
        }),
        cfg!(feature = "gbgov"),
        cfg!(target_os = "windows"),
        botcore::product::is_service_off,
        botcorepkg::installer::component_available_here,
    )
}

/// Pure, testable core of [`component_plan`].
///
/// Every decision that depends on the runtime (env vars, features, platform)
/// is injected so that tests can exercise each combination without touching
/// the global state.
fn build_plan(
    model_mode: &str,
    llm_url: &str,
    llm_model: &str,
    with_botmodels: &str,
    is_gbgov: bool,
    is_windows: bool,
    is_off: impl Fn(&str) -> bool,
    is_available: impl Fn(&str) -> bool,
) -> Vec<Component> {
    // An empty mode with a remote endpoint/model means the user picked a hosted
    // provider: never install the ~1.8 GB local runtime in that case. The old
    // `unwrap_or("local")` treated a missing value as "local", so the Kilo flow
    // still downloaded and started llama.cpp (and failed on slow links).
    // Use the caller's resolved values: the wizard passes the selected engine
    // here, so reading raw env vars would ignore an explicit local/hosted
    // choice and install the wrong components (the gbgov port did this).
    let model_mode = model_mode;
    let remote_model = !llm_url.trim().is_empty() && !llm_model.trim().is_empty();
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
            // A cold appliance boot on a shared CI runner can need longer than
            // five minutes once Zitadel has been downloaded and started.
            timeout_secs: 600,
            optional: false,
            start: true,
            unavailable: None,
        },
    ];

    // ---- Remove components the edition switches off ----
    // The product file's `services_off=` names services this edition does not
    // use. They are removed entirely: the wizard never shows, downloads,
    // starts or waits for them.
    remove_switched_off(&mut plan, &is_off);

    // ---- Remove components not available on this platform ----
    // A component with no download URL and no exec command for this OS (e.g.
    // Valkey on Windows) cannot be installed. Attempting it downloads the
    // wrong platform's archive and waits forever.
    remove_unavailable(&mut plan, &is_available);

    if wants_local {
        plan.push(Component {
            id: "llm",
            label: "Local model runtime",
            timeout_secs: 120,
            optional: true,
            // Windows: the assistant's boot starts the embedding engine and the
            // stored local model itself; a start here would wait for a chat
            // engine that nothing launches yet.
            start: !is_windows,
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
    // The gbgov Windows edition has no media service; main's extra env read
    // ignored both the caller's opt-out and that edition rule.
    if wants_media_service(with_botmodels, is_gbgov) {
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

/// Removes services the product file switches off. The wizard never shows,
/// downloads, starts or waits for a switched-off service.
fn remove_switched_off(plan: &mut Vec<Component>, is_off: impl Fn(&str) -> bool) {
    plan.retain(|component| {
        let off = is_off(component.id);
        if off {
            log::info!(
                "{} is switched off for this edition; removing from the wizard plan",
                component.id
            );
        }
        !off
    });
}

/// Removes components that have no viable binary on the current platform.
/// A component without a download URL *and* without platform exec commands
/// (e.g. Valkey on Windows) cannot be provisioned.
fn remove_unavailable(plan: &mut Vec<Component>, is_available: impl Fn(&str) -> bool) {
    plan.retain(|component| {
        let available = is_available(component.id);
        if !available {
            log::info!(
                "{} has no build for this platform; removing from the wizard plan",
                component.id
            );
        }
        available
    });
}

/// Whether setup installs the AI media service. The government desktop edition
/// reads text in images through Windows and has no other use for it, so it
/// never downloads the box; elsewhere `GB_WIZARD_WITH_BOTMODELS=0` opts out.
fn wants_media_service(opt_out: &str, edition_without_media: bool) -> bool {
    !edition_without_media && opt_out.trim() != "0"
}

/// The component ids a fresh Windows Lite install provisions, in order.
/// Any change to this list is a user-visible wizard change and must be
/// reviewed.
#[cfg(test)]
const WINDOWS_LITE_PLAN: &[&str] = &["vault", "tables", "drive"];

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
mod plan_tests {
    use super::*;

    /// Helper: the services the gbgov product file switches off.
    fn gbgov_is_off(name: &str) -> bool {
        matches!(name, "cache" | "vector_db" | "directory")
    }

    /// Helper: simulates Windows platform availability (cache is unavailable).
    fn windows_available(name: &str) -> bool {
        name != "cache"
    }

    /// Helper: everything is available (Linux).
    fn all_available(_name: &str) -> bool {
        true
    }

    // ---- The brief's required tests ----

    /// A fresh Windows Lite install (product switch + Windows platform) yields
    /// exactly vault, tables, drive in order. No cache, vector_db or
    /// directory step is shown.
    #[test]
    fn windows_lite_plan_has_exactly_the_expected_components() {
        let plan = build_plan(
            "hosted",        // remote model
            "https://api.example.com",
            "gpt-4",
            "0",             // no botmodels
            true,            // gbgov
            true,            // windows
            gbgov_is_off,
            windows_available,
        );
        let ids: Vec<&str> = plan.iter().map(|c| c.id).collect();
        assert_eq!(ids, WINDOWS_LITE_PLAN);
    }

    /// `GB_SERVICES_ON=cache` on Windows still yields no cache service:
    /// the platform has no binary regardless of the product switch.
    #[test]
    fn services_on_cache_on_windows_still_yields_no_cache() {
        // Simulate GB_SERVICES_ON=cache overriding the product switch:
        // is_off returns false for cache (turned back on by env), but the
        // platform check still removes it.
        let plan = build_plan(
            "hosted",
            "https://api.example.com",
            "gpt-4",
            "0",
            true,            // gbgov
            true,            // windows
            |_| false,       // nothing switched off (GB_SERVICES_ON overrode)
            windows_available,
        );
        let ids: Vec<&str> = plan.iter().map(|c| c.id).collect();
        assert!(!ids.contains(&"cache"), "cache must never appear on Windows: {ids:?}");
    }

    /// On Linux with no services switched off, the full plan is returned.
    #[test]
    fn linux_full_plan_includes_all_services() {
        let plan = build_plan(
            "local", "", "", "", false, false,
            |_| false,
            all_available,
        );
        let ids: Vec<&str> = plan.iter().map(|c| c.id).collect();
        assert!(ids.contains(&"cache"));
        assert!(ids.contains(&"vector_db"));
        assert!(ids.contains(&"directory"));
        assert!(ids.contains(&"llm"));
    }

    /// Switched-off services are removed entirely; they do not appear in the
    /// plan at all.
    #[test]
    fn switched_off_services_are_removed_from_the_plan() {
        let plan = build_plan(
            "local", "", "", "", false, false,
            gbgov_is_off,
            all_available,
        );
        let ids: Vec<&str> = plan.iter().map(|c| c.id).collect();
        assert!(!ids.contains(&"cache"), "cache must be removed: {ids:?}");
        assert!(!ids.contains(&"vector_db"), "vector_db must be removed: {ids:?}");
        assert!(!ids.contains(&"directory"), "directory must be removed: {ids:?}");
        assert!(ids.contains(&"vault"));
        assert!(ids.contains(&"tables"));
        assert!(ids.contains(&"drive"));
    }

    /// A skipped component never blocks those after it: the plan simply does
    /// not contain the component, so the provisioner walks straight through.
    #[test]
    fn skipped_component_does_not_block_later_ones() {
        let plan = build_plan(
            "hosted", "https://api.example.com", "gpt-4", "0",
            true, true,
            gbgov_is_off,
            windows_available,
        );
        // drive follows cache in the original order; cache is removed,
        // so drive must be present and must be right after tables.
        let ids: Vec<&str> = plan.iter().map(|c| c.id).collect();
        assert_eq!(ids, vec!["vault", "tables", "drive"]);
    }
}
