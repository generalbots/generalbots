//! Deep-link resolution: templates filled from params and command results
//! (#2143 gap C). The tests live apart so the builder stays under the size limit.

use super::*;

fn command(app: &'static str, name: &'static str, deep_link: &'static str) -> AppCommand {
    AppCommand {
        app,
        name,
        label: "Test",
        summary: "test verb",
        params: &[],
        deep_link: Some(deep_link),
        admin_only: false,
    }
}

fn params(pairs: &[(&str, &str)]) -> Map<String, Value> {
    let mut map = Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), Value::String((*value).to_string()));
    }
    map
}

/// The record a bound endpoint returns fills the template: the button opens
/// the artifact, not the literal `{file_id}`.
#[test]
fn a_result_with_an_id_fills_the_template() {
    let verb = command("sheet", "sheet.cells.set", "app://sheet?file={file_id}");
    let result = serde_json::json!({ "id": "sheet-42", "name": "Fluxo" });
    let link = build_deep_link(&verb, &Map::new(), &result)
        .expect("template resolves")
        .expect("template exists");
    assert_eq!(link, "app://sheet?file=sheet-42");
}

/// A missing id is an error, not a link that opens an empty window.
#[test]
fn a_missing_id_fails_instead_of_a_broken_link() {
    let verb = command("sheet", "sheet.open", "app://sheet?file_id={file_id}");
    let error = build_deep_link(&verb, &Map::new(), &serde_json::json!({ "count": 0 }))
        .expect_err("no id means no link");
    assert_eq!(error.placeholder, "file_id");
    assert_eq!(error.app, "sheet");
    let text = error.to_string();
    assert!(text.contains("file_id"), "{text}");
    assert!(text.contains("nothing was opened"), "{text}");
    // The template is quoted for the developer, but no link is produced.
    assert!(text.contains("app://sheet?file_id={file_id}"), "{text}");
}

/// Navigation through the catalog answer fails with that error instead of
/// rendering a broken button.
#[test]
fn navigation_without_the_record_fails() {
    let verb = command("docs", "docs.update", "app://docs?file={file_id}");
    let outcome = navigation_result(&verb, &Map::new());
    assert!(outcome.is_err(), "expected an error, got {outcome:?}");
}

/// An optional placeholder is dropped so a list verb links the app view.
#[test]
fn an_optional_placeholder_is_dropped() {
    let verb = command("tasks", "tasks.list", "app://tasks?task_id={task_id?}");
    let link = build_deep_link(&verb, &Map::new(), &serde_json::json!({ "tasks": [] }))
        .expect("optional placeholder never fails")
        .expect("template exists");
    assert_eq!(link, "app://tasks");
}

/// A list result must never supply an id: "list my docs" opens the list.
#[test]
fn a_list_result_never_supplies_an_id() {
    let verb = command("docs", "docs.list", "app://docs?file={file?}");
    let result = serde_json::json!({ "documents": [{ "id": "doc-1" }] });
    let link = build_deep_link(&verb, &Map::new(), &result)
        .expect("optional placeholder never fails")
        .expect("template exists");
    assert_eq!(link, "app://docs");
}

/// A create verb whose template is the bare app view gets the new record id
/// appended, so the window opens the artifact instead of a blank app.
#[test]
fn a_plain_link_gains_the_created_record_id() {
    let verb = command("docs", "docs.create", "app://docs");
    let result = serde_json::json!({ "id": "doc-7", "title": "Proposta" });
    let link = build_deep_link(&verb, &Map::new(), &result)
        .expect("template resolves")
        .expect("template exists");
    assert_eq!(link, "app://docs?file=doc-7");
}

/// Without a record the bare app view stays the bare app view.
#[test]
fn a_plain_link_without_a_record_stays_the_app_view() {
    let verb = command("tasks", "tasks.autotask.stats", "app://tasks");
    let link = build_deep_link(&verb, &Map::new(), &serde_json::json!({ "runs": 3 }))
        .expect("template resolves")
        .expect("template exists");
    assert_eq!(link, "app://tasks");
}

/// The caller's own reference wins, and a nested record object is read.
#[test]
fn params_win_and_nested_records_are_read() {
    let verb = command("slides", "slides.present", "app://slides?file={file_id}");
    let explicit = params(&[("file_id", "deck-1")]);
    let nested = serde_json::json!({ "presentation": { "id": "deck-2" } });
    assert_eq!(
        build_deep_link(&verb, &explicit, &nested)
            .expect("template resolves")
            .expect("template exists"),
        "app://slides?file=deck-1"
    );
    assert_eq!(
        build_deep_link(&verb, &Map::new(), &nested)
            .expect("template resolves")
            .expect("template exists"),
        "app://slides?file=deck-2"
    );
}

/// Values that would break the query are encoded, paths stay readable.
#[test]
fn values_are_encoded_for_the_query_string() {
    let verb = command("drive", "drive.search", "app://drive?path={path}");
    let given = params(&[("path", "faturas/2026 & notas")]);
    let link = build_deep_link(&verb, &given, &Value::Null)
        .expect("template resolves")
        .expect("template exists");
    assert_eq!(link, "app://drive?path=faturas/2026%20%26%20notas");
}

/// The attachment keeps the command result and adds the navigation, and it
/// never adds a link it could not resolve.
#[test]
fn the_result_carries_the_resolved_navigation() {
    let result = serde_json::json!({ "id": "doc-9", "title": "Contrato" });
    let with_link = with_navigation("docs.create", &Map::new(), result);
    assert_eq!(with_link["id"], "doc-9");
    assert_eq!(with_link["deep_link"], "app://docs?file=doc-9");
    assert_eq!(with_link["navigate"], true);

    let without_link = with_navigation(
        "docs.create",
        &Map::new(),
        serde_json::json!({ "message": "created" }),
    );
    assert!(without_link.get("deep_link").is_none(), "{without_link:?}");
}

/// A real catalogue verb fills its record link from the params the model
/// passes, so the button opens the spreadsheet instead of a blank window.
#[test]
fn a_catalogue_verb_fills_its_record_link() {
    let verb = command_by_name("sheet.open").expect("sheet.open is declared");
    let given = params(&[("file_id", "fluxo-2026-08"), ("cell", "B2")]);
    let link = build_deep_link(verb, &given, &Value::Null)
        .expect("template resolves")
        .expect("template exists");
    assert_eq!(link, "app://sheet?file_id=fluxo-2026-08&cell=B2");
}

/// A list verb has no single record, so it links the app view — not the first
/// row of its result and not a literal placeholder.
#[test]
fn a_list_verb_links_the_app_view() {
    let verb = command_by_name("crm.people.list").expect("crm.people.list is declared");
    let result = serde_json::json!({ "count": 2, "people": [{ "person_id": "p-1" }] });
    let link = build_deep_link(verb, &Map::new(), &result)
        .expect("optional placeholder never fails")
        .expect("template exists");
    assert_eq!(link, "app://crm");
}

/// Guard: a required placeholder must be reachable — either a param the
/// verb declares or the id the result returns. A template that demands a
/// record the verb can never name fails every navigation, which is the bug
/// this module fixes; mark such a placeholder `{key?}` instead.
#[test]
fn required_placeholders_are_reachable() {
    for verb in all_commands() {
        let Some(template) = verb.deep_link else {
            continue;
        };
        for placeholder in placeholders(template) {
            if placeholder.optional {
                continue;
            }
            let declared = verb.params.iter().any(|(name, _)| {
                *name == placeholder.key || aliases_for(&placeholder.key).contains(name)
            });
            assert!(
                declared || is_record_key(&placeholder.key),
                "{}: '{}' in '{template}' is neither a declared param nor a record id",
                verb.name,
                placeholder.key
            );
        }
    }
}
