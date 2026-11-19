//! External-service requirements of a catalog app (#2479).
//!
//! The laptop image provisions `vault`, `tables`, `cache`, `drive`,
//! `vector_db`, `directory`, `llm` and `botmodels` — no Stalwart, no LiveKit,
//! no Incus. Before this declaration the registry could only advertise apps,
//! so Mail and Meet appeared as working icons that opened a pane which could
//! never load data, on a machine whose only UI is the shell.
//!
//! An app declares the provision component it needs; the catalog handler hides
//! it on an image that does not install that component (and, in every other
//! deployment, publishes the requirement so a client can explain the pane).

use serde::Serialize;
use std::collections::HashSet;

/// Provision component an app cannot work without.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AppRequirement {
    /// Component id, matching `component_plan()` in `wizard_provision` and the
    /// `[components.*]` table of `3rdparty.toml`.
    pub component: &'static str,
    /// What the user would have to install, phrased for an error line.
    pub label: &'static str,
}

impl AppRequirement {
    /// Requirement on the mail server (Stalwart).
    pub const MAIL: Self = Self {
        component: "mail",
        label: "the mail server (Stalwart)",
    };

    /// Requirement on the meeting server (LiveKit).
    pub const MEET: Self = Self {
        component: "meet",
        label: "the meeting server (LiveKit)",
    };
}

/// `true` when an app with this requirement can load on a host that provisioned
/// `provisioned`.
///
/// `provisioned` is `None` for deployments that are not a fixed image (a server,
/// a developer machine): there a component is installed on demand, so the app
/// stays offered and only the requirement is published.
pub fn is_available(requires: Option<AppRequirement>, provisioned: Option<&HashSet<&str>>) -> bool {
    match (requires, provisioned) {
        (Some(requirement), Some(components)) => components.contains(requirement.component),
        _ => true,
    }
}

/// Sentence for the catalog/UI when an app is advertised but cannot load.
pub fn unavailable_reason(requirement: AppRequirement) -> String {
    format!("Needs {}; this image does not install it", requirement.label)
}

#[cfg(test)]
mod tests {
    use super::{is_available, unavailable_reason, AppRequirement};
    use std::collections::HashSet;

    #[test]
    fn server_deployments_keep_every_app_available() {
        assert!(is_available(Some(AppRequirement::MAIL), None));
        assert!(is_available(None, None));
    }

    #[test]
    fn image_without_the_component_hides_the_app() {
        let laptop: HashSet<&str> = ["vault", "tables", "drive", "llm"].into_iter().collect();
        assert!(!is_available(Some(AppRequirement::MAIL), Some(&laptop)));
        assert!(is_available(None, Some(&laptop)));
    }

    #[test]
    fn image_with_the_component_offers_the_app() {
        let full: HashSet<&str> = ["mail", "meet"].into_iter().collect();
        assert!(is_available(Some(AppRequirement::MAIL), Some(&full)));
    }

    /// #2479 — the entries that cannot work on the laptop image must be the
    /// ones that declare it, so the catalog hides them instead of offering a
    /// pane that never fills.
    #[test]
    fn registry_declares_the_requirements_the_laptop_image_lacks() {
        use super::super::all_apps;

        let required = |id: &str| {
            all_apps()
                .into_iter()
                .find(|app| app.id == id)
                .and_then(|app| app.requires)
        };
        assert_eq!(required("mail").map(|r| r.component), Some("mail"));
        assert_eq!(required("meet").map(|r| r.component), Some("meet"));
        // Vibe loads without Incus; only its VM-backed kinds need the runtime.
        assert_eq!(required("vibe"), None);
    }

    #[test]
    fn reason_names_what_to_install() {
        assert_eq!(
            unavailable_reason(AppRequirement::MEET),
            "Needs the meeting server (LiveKit); this image does not install it"
        );
    }
}
