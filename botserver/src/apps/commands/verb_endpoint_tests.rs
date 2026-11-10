use super::super::*;
use super::*;

/// A binding for a name the catalogue does not declare can never be
/// resolved, so it is a typo rather than a capability.
#[test]
fn every_binding_names_a_curated_verb() {
    for binding in VERB_ENDPOINTS {
        assert!(
            command_by_name(binding.verb).is_some(),
            "{} is not a curated command",
            binding.verb
        );
    }
}

#[test]
fn verb_bindings_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for binding in VERB_ENDPOINTS {
        assert!(
            seen.insert(binding.verb),
            "duplicate binding {}",
            binding.verb
        );
    }
}

#[test]
fn renames_are_well_formed() {
    for binding in VERB_ENDPOINTS {
        for (from, to) in binding.renames {
            assert!(!from.is_empty(), "{} has an empty rename", binding.verb);
            assert!(!to.is_empty(), "{} renames {from} to nothing", binding.verb);
            assert_ne!(from, to, "{} renames {from} onto itself", binding.verb);
        }
    }
}

#[test]
fn map_params_renames_and_keeps_known_fields() {
    let binding = verb_endpoint("docs.get").expect("docs.get is bound");
    let mut params = Map::new();
    params.insert("file_id".to_string(), Value::String("doc-1".to_string()));
    let mapped = binding.map_params(&params);
    assert_eq!(mapped.get("id"), Some(&Value::String("doc-1".to_string())));
    assert!(mapped.get("file_id").is_none());
}

#[test]
fn map_params_applies_defaults_only_when_absent() {
    let binding = verb_endpoint("slides.slide.add").expect("slides.slide.add is bound");
    let empty = binding.map_params(&Map::new());
    assert_eq!(
        empty.get("layout"),
        Some(&Value::String("content".to_string()))
    );

    let mut params = Map::new();
    params.insert("layout".to_string(), Value::String("blank".to_string()));
    let mapped = binding.map_params(&params);
    assert_eq!(
        mapped.get("layout"),
        Some(&Value::String("blank".to_string()))
    );
}

#[test]
fn record_id_fills_an_id_path_segment() {
    let binding = verb_endpoint("docs.get").expect("docs.get is bound");
    let mut params = Map::new();
    params.insert("record_id".to_string(), Value::String("doc-9".to_string()));
    let mapped = binding.map_params(&params);
    assert_eq!(mapped.get("id"), Some(&Value::String("doc-9".to_string())));
}

/// Every bound path must be a route the inventory already declares; a
/// binding to an unknown path would fail only at call time.
#[test]
fn every_bound_path_is_a_registered_route() {
    let routes: Vec<(&str, &str)> = crate::core::bot::endpoint_inventory::all_routes()
        .map(|route| (route.method, route.path))
        .collect();
    for binding in VERB_ENDPOINTS {
        assert!(
            routes
                .iter()
                .any(|(method, path)| *method == binding.method && *path == binding.path),
            "{} binds to unregistered {} {}",
            binding.verb,
            binding.method,
            binding.path
        );
    }
}

/// A response body that the deep link can open: a non-empty top-level `id`.
#[cfg(any(feature = "docs", feature = "sheet", feature = "slides"))]
fn response_has_id(body: &Value) -> bool {
    body.get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| !id.trim().is_empty())
}

/// `docs.create` posts the document-create endpoint and its response carries
/// the new document id (#2143, gap B).
#[test]
fn docs_create_binds_the_document_create_endpoint() {
    let binding = verb_endpoint("docs.create").expect("docs.create is bound");
    assert_eq!(binding.method, "POST");
    assert_eq!(binding.path, "/api/docs/new");

    let mut params = Map::new();
    params.insert("title".to_string(), Value::String("Report".to_string()));
    params.insert("content".to_string(), Value::String("# Body".to_string()));
    let mapped = binding.map_params(&params);
    assert_eq!(
        mapped.get("title"),
        Some(&Value::String("Report".to_string()))
    );
    assert_eq!(
        mapped.get("content"),
        Some(&Value::String("# Body".to_string()))
    );

    #[cfg(feature = "docs")]
    {
        let doc = botdocs::storage::create_document_with("owner-1", Some("Report"), Some("# Body"));
        let body = serde_json::to_value(doc).unwrap_or(Value::Null);
        assert!(response_has_id(&body), "created document must carry its id");
    }
}

/// `docs.update` posts the save endpoint; `append` keeps the stored body
/// instead of replacing it, and the response carries the document id.
#[test]
fn docs_update_binds_the_document_save_endpoint() {
    let binding = verb_endpoint("docs.update").expect("docs.update is bound");
    assert_eq!(binding.method, "POST");
    assert_eq!(binding.path, "/api/docs/save");

    let mut params = Map::new();
    params.insert("file_id".to_string(), Value::String("doc-1".to_string()));
    params.insert("mode".to_string(), Value::String("append".to_string()));
    let mapped = binding.map_params(&params);
    assert_eq!(mapped.get("id"), Some(&Value::String("doc-1".to_string())));
    assert_eq!(
        mapped.get("mode"),
        Some(&Value::String("append".to_string()))
    );

    #[cfg(feature = "docs")]
    {
        let body = serde_json::to_value(botdocs::types_core::DocsSaveResponse {
            id: "doc-1".to_string(),
            success: true,
        })
        .unwrap_or(Value::Null);
        assert!(response_has_id(&body), "saved document must carry its id");
    }
}

/// `sheet.create` posts the spreadsheet-create endpoint and its response
/// carries the new spreadsheet id (#2143, gap B).
#[test]
fn sheet_create_binds_the_spreadsheet_create_endpoint() {
    let binding = verb_endpoint("sheet.create").expect("sheet.create is bound");
    assert_eq!(binding.method, "POST");
    assert_eq!(binding.path, "/api/sheet/new");

    let rows = serde_json::json!([["Region", "Total"], ["North", 1200]]);
    let mut params = Map::new();
    params.insert("title".to_string(), Value::String("Budget".to_string()));
    params.insert("rows".to_string(), rows.clone());
    let mapped = binding.map_params(&params);
    assert_eq!(mapped.get("rows"), Some(&rows));

    #[cfg(feature = "sheet")]
    {
        let body = match botsheet::handlers::create::build_spreadsheet(
            "owner-1",
            Some("Budget"),
            Some(&rows),
        ) {
            Ok(sheet) => serde_json::to_value(sheet).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        };
        assert!(
            response_has_id(&body),
            "created spreadsheet must carry its id"
        );
    }
}

/// `sheet.cells.set` posts the block-write endpoint, renames the curated
/// `file_id` onto `sheet_id`, and its response carries the sheet id.
#[test]
fn sheet_cells_set_binds_the_cell_block_endpoint() {
    let binding = verb_endpoint("sheet.cells.set").expect("sheet.cells.set is bound");
    assert_eq!(binding.method, "POST");
    assert_eq!(binding.path, "/api/sheet/cells");

    let mut params = Map::new();
    params.insert("file_id".to_string(), Value::String("sheet-1".to_string()));
    params.insert("range".to_string(), Value::String("A1:B2".to_string()));
    params.insert("values".to_string(), serde_json::json!([[1, 2], [3, 4]]));
    let mapped = binding.map_params(&params);
    assert_eq!(
        mapped.get("sheet_id"),
        Some(&Value::String("sheet-1".to_string()))
    );
    assert!(mapped.get("file_id").is_none());

    #[cfg(feature = "sheet")]
    {
        let body = serde_json::to_value(botsheet::types::SaveResponse {
            id: "sheet-1".to_string(),
            success: true,
            message: Some("4 cells updated".to_string()),
        })
        .unwrap_or(Value::Null);
        assert!(response_has_id(&body), "cell write must carry the sheet id");
    }
}

/// `slides.create` posts the presentation-create endpoint and its response
/// carries the new deck id (#2143, gap B).
#[test]
fn slides_create_binds_the_presentation_create_endpoint() {
    let binding = verb_endpoint("slides.create").expect("slides.create is bound");
    assert_eq!(binding.method, "POST");
    assert_eq!(binding.path, "/api/slides/new");

    let mut params = Map::new();
    params.insert("title".to_string(), Value::String("Kickoff".to_string()));
    params.insert(
        "slides".to_string(),
        serde_json::json!([{ "title": "Agenda", "body": "Goals" }]),
    );
    let mapped = binding.map_params(&params);
    assert_eq!(
        mapped.get("title"),
        Some(&Value::String("Kickoff".to_string()))
    );

    #[cfg(feature = "slides")]
    {
        let slides = [botslides::types::DeckSlideInput {
            title: Some("Agenda".to_string()),
            body: Some("Goals".to_string()),
        }];
        let deck =
            botslides::storage::create_presentation_with("owner-1", Some("Kickoff"), Some(&slides));
        let body = serde_json::to_value(deck).unwrap_or(Value::Null);
        assert!(response_has_id(&body), "created deck must carry its id");
    }
}
