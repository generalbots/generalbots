//! Command-catalog tests. Kept in a sibling file so
//! `deep_link_params_for_app.rs` stays under the repository size limit.

use super::*;

#[cfg(test)]
mod database_verb_tests {
    use super::*;

    /// #2475 — the office `app://` contract emits `file` for every surface.
    /// The chat renderer's catalog gate drops links whose keys are all absent
    /// from the declared params, so a missing `file` key silently degrades a
    /// canonical Docs/Sheet/Slides link to plain text.
    #[test]
    fn office_apps_declare_the_contract_file_key() {
        for app in ["docs", "sheet", "slides"] {
            let params = deep_link_params_for_app(app);
            assert!(
                params.iter().any(|p| p.key == "file"),
                "{app} must declare the `file` deep-link key"
            );
        }
    }

    #[test]
    fn database_exposes_write_verbs() {
        let names: Vec<&str> = all_commands().iter().map(|command| command.name).collect();
        for name in [
            "database.rows.insert",
            "database.rows.update",
            "database.rows.delete",
            "database.rows.batch_delete",
            "database.rows.import",
            "database.table.create",
            "database.column.add",
        ] {
            assert!(names.contains(&name), "missing command {name}");
        }
    }

    /// #2752 — the catalogue flag is the only RBAC layer for chat writes.
    #[test]
    fn state_changing_database_verbs_are_admin_only() {
        for name in [
            "database.rows.insert",
            "database.rows.update",
            "database.rows.import",
            "database.rows.delete",
            "database.rows.batch_delete",
            "database.table.create",
            "database.column.add",
            "database.connections.create",
        ] {
            let command = all_commands()
                .iter()
                .find(|command| command.name == name)
                .expect("command is declared");
            assert!(command.admin_only, "{name} must be admin-only");
        }
    }
}

#[cfg(test)]
mod verb_coverage_tests {
    use super::*;

    /// #2035 — these apps had no curated verb at all, so a request to open or
    /// operate them could not be routed.
    #[test]
    fn every_app_has_at_least_one_verb() {
        for app in [
            "about",
            "automations",
            "calculator",
            "clock",
            "clock-widget",
            "concierge",
            "designer",
            "desktop",
            "memory",
            "notes",
            "o365",
            "photos",
            "plugins",
            "recycle",
            "store",
            "terminal",
            "timer",
            "weather",
        ] {
            assert!(
                !commands_for_app(app).is_empty(),
                "app {app} still has no verb"
            );
        }
    }

    #[test]
    fn office_apps_expose_create_and_export() {
        for name in [
            "docs.create",
            "docs.update",
            "docs.export",
            "sheet.create",
            "sheet.cells.set",
            "sheet.chart.create",
            "sheet.export",
            "slides.create",
            "slides.slide.add",
            "canvas.create",
            "canvas.export",
        ] {
            assert!(
                all_commands().iter().any(|command| command.name == name),
                "missing office verb {name}"
            );
        }
    }
}

#[cfg(test)]
mod surface_coverage_tests {
    use super::*;

    /// #2036 — every registered app must be reachable in both modes: a verb for
    /// api.find/api.exec, a deep link to point at a record, and a UI hint so a
    /// plan follows the app's real flow.
    #[test]
    fn every_registry_app_has_a_verb_a_deep_link_and_a_ui_hint() {
        for app in crate::apps::registry::all_apps() {
            assert!(
                !commands_for_app(&app.id).is_empty(),
                "{} has no curated verb",
                app.id
            );
            assert!(
                crate::apps::commands::all_deep_links().any(|(id, _)| *id == app.id),
                "{} has no deep link",
                app.id
            );
            assert!(
                !ui_sequence_hint_for(&app.id).is_empty(),
                "{} has no UI sequence hint",
                app.id
            );
        }
    }
}

#[cfg(test)]
mod command_name_tests {
    use super::*;
    use std::collections::HashSet;

    /// Two commands with the same name make api.find and the executor
    /// ambiguous; the joined table must stay unique.
    #[test]
    fn command_names_are_unique() {
        let mut seen = HashSet::new();
        for command in all_commands() {
            assert!(
                seen.insert(command.name),
                "duplicate command name {}",
                command.name
            );
        }
    }
}
