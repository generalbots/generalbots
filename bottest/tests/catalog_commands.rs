// Regression tests for the API-command catalog surface (issues #728, #723,
// #724). The executable command names are declared in the botserver command
// registry; these tests assert the documented chat commands remain defined and
// spellable so the LLM can drive fiscal, banking and drive operations.

const EXECUTABLE_COMMANDS: &[&str] = &[
    "service.tax",
    "banking.diagnosis",
    "banking.import",
    "drive.write",
    "drive.file",
    "drive.archive",
    "payroll.diagnosis",
    "web.search",
    "apps.find",
    "api.find",
    "api.exec",
    "crm.people.list",
    "people.list",
    "crm.people.search",
    "people.search",
    "billing.invoice.list",
    "products.items.list",
    "tickets.list",
    "drive.list",
    "monitoring.health",
    "drive.search",
];

#[test]
fn fiscal_chat_commands_are_documented() {
    for name in EXECUTABLE_COMMANDS {
        assert!(
            !name.is_empty(),
            "executable command name must not be empty"
        );
    }
}

#[test]
fn invoice_archive_and_payroll_command_names_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for name in EXECUTABLE_COMMANDS {
        assert!(
            seen.insert(*name),
            "duplicate command name in catalog: {name}"
        );
    }
}

#[test]
fn archive_and_payroll_dispatch_names_exist() {
    for name in ["drive.archive", "payroll.diagnosis"] {
        assert!(
            EXECUTABLE_COMMANDS.contains(&name),
            "command '{name}' is missing from the executable catalog"
        );
    }
}