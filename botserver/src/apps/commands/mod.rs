//! Per-application command model — single source of truth for every action a
//! suite application exposes to the LLM (`__api_call__`), the command palette
//! and the unified Start Menu.
//!
//! Commands defined here are the ONLY declarative per-app action list. They are
//! consumed by:
//!   1. `GET /api/apps/catalog` (`apps/mod.rs`) — palette + frontend.
//!   2. `api_command_instructions()` (`core/bot/api_catalog.rs`) — chat/WhatsApp
//!      prompt (only `name: summary` is injected to keep the prompt compact).
//!   3. `api.find` / `apps.find` discovery — full params on demand.
//!   4. The frontend Start Menu / command palette.
//!
//! Deep-link parameters declare the canonical record key each app accepts via
//! `app://<appId>?<key>=<value>` so the frontend can open an app window already
//! contextualized to a record.

pub(crate) use serde::Serialize;

mod app_command;
mod app_commands_extra;
mod app_deep_links;
mod app_deep_links_extra;
mod deep_link_params_for_app;
mod verb_endpoints;

pub use app_command::*;
pub use app_deep_links::*;
pub use app_deep_links_extra::*;
pub use deep_link_params_for_app::*;
pub use verb_endpoints::*;

#[cfg(test)]
mod surface_guard_tests {
    use super::{all_commands, commands_for_app};
    use crate::apps::registry;

    /// #2036 — build-time guard: every registered app must expose at least one
    /// callable verb, so "add a note"/"insert a slide" can resolve to a command
    /// instead of a dead end. Adding a registry entry without a command fails
    /// here.
    #[test]
    fn every_registered_app_has_at_least_one_command() {
        let apps = registry::all_apps();
        assert!(!apps.is_empty(), "the app registry is empty");

        let missing: Vec<String> = apps
            .iter()
            .filter(|app| commands_for_app(&app.id).is_empty())
            .map(|app| app.id.clone())
            .collect();

        assert!(
            missing.is_empty(),
            "apps without any command: {}",
            missing.join(", ")
        );
    }

    /// A command that names an app the registry does not know can never be
    /// opened or resolved, so it is a typo, not a feature.
    #[test]
    fn every_command_targets_a_registered_app() {
        let known: std::collections::HashSet<String> =
            registry::all_apps().into_iter().map(|app| app.id).collect();

        let unknown: Vec<String> = all_commands()
            .iter()
            .filter(|command| !known.contains(command.app))
            .map(|command| format!("{} ({})", command.name, command.app))
            .collect();

        assert!(
            unknown.is_empty(),
            "commands for unknown apps: {}",
            unknown.join(", ")
        );
    }

    /// `api.find` matches by name, so duplicate names make discovery
    /// ambiguous and `command_by_name` would silently pick one.
    #[test]
    fn command_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        let duplicates: Vec<&str> = all_commands()
            .iter()
            .filter(|command| !seen.insert(command.name))
            .map(|command| command.name)
            .collect();

        assert!(
            duplicates.is_empty(),
            "duplicate command names: {}",
            duplicates.join(", ")
        );
    }
}
