//! #2448 — parity checks for the single chat-native undo verb only.

use crate::apps::commands::{all_commands, command_by_name, verb_endpoint};

const VERB: &str = "docs.undo";

#[test]
fn chat_undo_verb_is_cataloged_and_surfaced_to_the_llm() -> Result<(), String> {
    let command = command_by_name(VERB).ok_or("docs.undo is missing from the catalog")?;
    assert_eq!(command.app, "docs");
    assert!(
        all_commands().iter().any(|entry| entry.name == VERB),
        "docs.undo must be part of the compact catalog"
    );
    assert!(
        command.summary.contains("undo"),
        "api.find matches summaries; the verb must be discoverable by 'undo'"
    );
    Ok(())
}

#[test]
fn chat_undo_verb_takes_no_prompt_supplied_ids_and_is_not_admin_only() -> Result<(), String> {
    let command = command_by_name(VERB).ok_or("docs.undo is missing from the catalog")?;
    assert!(
        command.params.is_empty(),
        "no param may carry a user, bot, session or batch id from the prompt"
    );
    assert!(
        !command.admin_only,
        "ordinary users must be able to undo their own batch"
    );
    Ok(())
}

/// The undo route is absent from the harvested endpoint inventory, so the verb
/// is dispatched in-process instead of through the `api.exec` loopback. A REST
/// binding would fail the global "bound path is registered" guard at call time.
#[test]
fn chat_undo_verb_is_an_in_process_verb_with_no_rest_binding() {
    assert!(
        verb_endpoint(VERB).is_none(),
        "docs.undo must not use the HTTP loopback: it would drop the verified scope"
    );
}

#[test]
fn chat_undo_rest_route_is_not_an_admin_only_binding() {
    assert!(
        !crate::apps::commands::admin_only_verb_binding("POST", "/api/collab/undo"),
        "the undo route must not be reachable only by admins"
    );
}
