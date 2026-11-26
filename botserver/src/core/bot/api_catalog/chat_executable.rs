//! Which catalog commands a chat turn can actually run. A command that only
//! exists as an app action (no in-process handler, no endpoint binding and no
//! deep link) fails with "not executable from chat", so it is never offered
//! to the model: it called `settings.read` to answer "what is my sector?" and
//! the turn ended on that error.

/// Names `execute_command` answers in process, in every build.
const IN_PROCESS: &[&str] = &[
    "service.tax",
    "tax.calculate",
    "drive.write",
    "drive.file",
    "apps.find",
    "api.find",
    "api.exec",
    "skills.list",
    "skills.find",
    "integrations.catalog.search",
    "integrations.catalog.provider",
    "integrations.connect.start",
    "integrations.actions.list",
    "integrations.invoke",
    "database.connections.create",
    "vibe.project.change",
    "crm.people.list",
    "people.list",
    "crm.people.search",
    "people.search",
    "crm.pipeline.forecast",
    "crm.leads.create",
    "crm.leads.report",
    "crm.deal.create",
    "crm.deal.move-stage",
    "crm.contact.create",
    "billing.invoice.list",
    "billing.payment.list",
    "billing.payments.list",
    "billing.quote.list",
    "billing.quotes.list",
    "products.items.list",
    "store.catalog.list",
    "products.items.create",
    "products.items.update",
    "tickets.list",
    "drive.list",
    "drive.search",
    "drive.archive",
    "payroll.diagnosis",
    "monitoring.health",
];

/// Names answered in process only when their feature is compiled.
fn in_process_with_feature(name: &str) -> bool {
    match name {
        "banking.diagnosis" | "banking.import" => cfg!(feature = "banking"),
        "web.search" => cfg!(any(feature = "research", feature = "web-search")),
        "integrations.decline" => cfg!(feature = "integrations"),
        "tasks.autotask.create" | "tasks.autotask.run" | "automations.create" | "automations.run" => {
            cfg!(feature = "automations")
        }
        _ => false,
    }
}

fn verb_family(name: &str) -> bool {
    #[cfg(feature = "database")]
    if super::database_verbs::handles(name) {
        return true;
    }
    super::docs_verbs::handles(name) || super::sheet_verbs::handles(name)
}

/// True when a chat turn can run `name` without failing for lack of a handler.
pub(super) fn is_chat_executable(name: &str) -> bool {
    IN_PROCESS.contains(&name)
        || in_process_with_feature(name)
        || verb_family(name)
        || crate::apps::commands::verb_endpoint(name).is_some()
        || super::command_by_name(name).is_some_and(|cmd| cmd.deep_link.is_some())
}

#[cfg(test)]
mod tests {
    use super::is_chat_executable;

    #[test]
    fn an_app_only_command_is_not_offered_to_chat() {
        assert!(!is_chat_executable("settings.read"));
        assert!(!is_chat_executable("no.such.command"));
    }

    #[test]
    fn in_process_commands_are_offered() {
        for name in ["api.find", "api.exec", "apps.find", "drive.list", "drive.search"] {
            assert!(is_chat_executable(name), "{name}");
        }
    }
}
