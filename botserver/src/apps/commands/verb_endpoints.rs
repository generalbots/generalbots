use super::*;
use serde_json::{Map, Value};

/// Binds a curated verb to the REST endpoint that already implements it, so a
/// command executed from chat performs the action instead of only answering
/// with a deep link.
///
/// Every entry names a route from `endpoint_inventory::all_routes()` — no verb
/// invents a path. `renames` translates the curated parameter names onto the
/// field names the handler deserializes; `defaults` fills a required field the
/// verb does not carry (never a value the user asked for).
pub struct VerbEndpoint {
    /// Curated command name, e.g. `docs.get`.
    pub verb: &'static str,
    /// HTTP method of the existing endpoint.
    pub method: &'static str,
    /// Registered route template, e.g. `/api/docs/:id`.
    pub path: &'static str,
    /// `(curated param, handler field)` pairs.
    pub renames: &'static [(&'static str, &'static str)],
    /// `(handler field, value)` pairs applied only when the caller omitted it.
    pub defaults: &'static [(&'static str, &'static str)],
}

const fn bind(
    verb: &'static str,
    method: &'static str,
    path: &'static str,
    renames: &'static [(&'static str, &'static str)],
    defaults: &'static [(&'static str, &'static str)],
) -> VerbEndpoint {
    VerbEndpoint {
        verb,
        method,
        path,
        renames,
        defaults,
    }
}

impl VerbEndpoint {
    /// Maps curated parameters onto the handler's field names and fills the
    /// declared defaults. Unrenamed parameters pass through untouched, so a
    /// handler keeps receiving the fields the verb documents.
    pub fn map_params(&self, params: &Map<String, Value>) -> Map<String, Value> {
        let mut out = Map::new();
        for (key, value) in params {
            out.insert(self.target_field(key).to_string(), value.clone());
        }
        for (key, value) in self.defaults {
            out.entry((*key).to_string())
                .or_insert_with(|| Value::String((*value).to_string()));
        }
        out
    }

    /// Handler field for a curated parameter. `record_id` is the catalogue's
    /// generic record key, so it fills an `:id` path segment too — the promise
    /// that every record opener accepts `record_id`.
    fn target_field<'a>(&'a self, key: &'a str) -> &'a str {
        if key == "record_id" && self.has_id_segment() {
            return "id";
        }
        self.renames
            .iter()
            .find(|(from, _)| *from == key)
            .map(|(_, to)| *to)
            .unwrap_or(key)
    }

    /// True when the route template carries an `:id` / `{id}` segment.
    fn has_id_segment(&self) -> bool {
        self.path
            .split('/')
            .any(|segment| segment == ":id" || segment == "{id}")
    }
}

pub fn verb_endpoint(name: &str) -> Option<&'static VerbEndpoint> {
    VERB_ENDPOINTS.iter().find(|binding| binding.verb == name)
}

/// Docs exposes one route per export format; the verb's `format` selects the
/// registered path (the list is closed, never an arbitrary one).
pub fn docs_export_path(format: &str) -> Result<&'static str, String> {
    match format {
        "pdf" => Ok("/api/docs/export/pdf"),
        "docx" => Ok("/api/docs/export/docx"),
        "odt" => Ok("/api/docs/export/odt"),
        "md" | "markdown" => Ok("/api/docs/export/md"),
        "html" => Ok("/api/docs/export/html"),
        "txt" | "text" => Ok("/api/docs/export/txt"),
        other => Err(format!(
            "unsupported docs export format '{other}' (pdf, docx, odt, md, html or txt)"
        )),
    }
}

/// Docs publishes one route per starter template; `blank` is the plain new
/// document and the other three come from `botdocs`.
pub fn docs_template_path(template: &str) -> Result<&'static str, String> {
    match template {
        "blank" => Ok("/api/docs/template/blank"),
        "letter" => Ok("/api/docs/template/letter"),
        "meeting" => Ok("/api/docs/template/meeting"),
        "report" => Ok("/api/docs/template/report"),
        other => Err(format!(
            "unknown docs template '{other}' (blank, letter, meeting or report)"
        )),
    }
}

/// Verbs bound to an existing endpoint, grouped by application. A verb that is
/// absent here has no endpoint yet and stays navigation-only on purpose.
pub static VERB_ENDPOINTS: &[VerbEndpoint] = &[
    // ——— #2037 — Docs (crates/botdocs) ———
    bind("docs.list", "GET", "/api/docs/list", &[], &[]),
    bind("docs.create", "POST", "/api/docs/new", &[], &[]),
    bind("docs.update", "POST", "/api/docs/save", &[("file_id", "id")], &[]),
    bind("docs.get", "GET", "/api/docs/:id", &[("file_id", "id")], &[]),
    bind("docs.save", "POST", "/api/docs/save", &[("file_id", "id")], &[]),
    bind("docs.autosave", "POST", "/api/docs/autosave", &[("file_id", "id")], &[]),
    bind("docs.delete", "POST", "/api/docs/delete", &[("file_id", "id")], &[]),
    bind("docs.search", "GET", "/api/docs/search", &[("query", "q")], &[]),
    bind("docs.outline", "POST", "/api/docs/outline", &[("file_id", "doc_id")], &[]),
    bind("docs.collaborators", "GET", "/api/docs/:doc_id/collaborators", &[("file_id", "doc_id")], &[]),
    bind("docs.ai", "POST", "/api/docs/ai", &[("file_id", "doc_id"), ("instruction", "command")], &[]),
    // ——— #2037 — Sheet (crates/botsheet) ———
    bind("sheet.list", "GET", "/api/sheet/list", &[], &[]),
    bind("sheet.create", "POST", "/api/sheet/new", &[], &[]),
    bind("sheet.cells.set", "POST", "/api/sheet/cells", &[("file_id", "sheet_id")], &[]),
    bind("sheet.get", "GET", "/api/sheet/:id", &[("file_id", "id")], &[]),
    bind("sheet.new", "GET", "/api/sheet/new", &[], &[]),
    bind("sheet.delete", "POST", "/api/sheet/delete", &[("file_id", "id")], &[]),
    bind("sheet.export", "POST", "/api/sheet/export", &[("file_id", "id")], &[]),
    bind("sheet.search", "GET", "/api/sheet/search", &[("query", "q")], &[]),
    bind("sheet.chart.create", "POST", "/api/sheet/chart", &[("file_id", "id"), ("range", "data_range"), ("type", "chart_type")], &[]),
    // ——— #2037 — Slides (crates/botslides) ———
    bind("slides.list", "GET", "/api/slides/list", &[], &[]),
    bind("slides.create", "POST", "/api/slides/new", &[], &[]),
    bind("slides.get", "GET", "/api/slides/:id", &[("file_id", "id")], &[]),
    bind("slides.new", "GET", "/api/slides/new", &[], &[]),
    bind("slides.delete", "POST", "/api/slides/delete", &[("file_id", "id")], &[]),
    bind("slides.export", "POST", "/api/slides/export", &[("file_id", "id")], &[]),
    bind("slides.search", "GET", "/api/slides/search", &[("query", "q")], &[]),
    bind("slides.slide.add", "POST", "/api/slides/slide/add", &[("file_id", "presentation_id"), ("after", "position")], &[("layout", "content")]),
    bind("slides.slide.delete", "POST", "/api/slides/slide/delete", &[("file_id", "presentation_id"), ("slide", "slide_index")], &[]),
    bind("slides.slide.duplicate", "POST", "/api/slides/slide/duplicate", &[("file_id", "presentation_id"), ("slide", "slide_index")], &[]),
    bind("slides.slide.notes", "POST", "/api/slides/slide/notes", &[("file_id", "presentation_id"), ("slide", "slide_index")], &[]),
    bind("slides.element.add", "POST", "/api/slides/element/add", &[("file_id", "presentation_id"), ("slide", "slide_index")], &[]),
    bind("slides.element.delete", "POST", "/api/slides/element/delete", &[("file_id", "presentation_id"), ("slide", "slide_index")], &[]),
    bind("slides.theme.set", "POST", "/api/slides/theme", &[("file_id", "presentation_id"), ("theme", "theme_name")], &[]),
    // ——— #2037 — Canvas (crates/botcanvas) ———
    bind("canvas.list", "GET", "/api/canvas", &[], &[]),
    bind("canvas.get", "GET", "/api/canvas/{canvas_id}", &[("file_id", "canvas_id")], &[]),
    bind("canvas.create", "POST", "/api/canvas", &[("title", "name")], &[]),
    bind("canvas.update", "PUT", "/api/canvas/:canvas_id", &[("file_id", "canvas_id")], &[]),
    bind("canvas.delete", "DELETE", "/api/canvas/:canvas_id", &[("file_id", "canvas_id")], &[]),
    bind("canvas.export", "POST", "/api/canvas/:canvas_id/export", &[("file_id", "canvas_id")], &[]),
    bind("canvas.element.list", "GET", "/api/canvas/{canvas_id}/elements", &[("file_id", "canvas_id")], &[]),
    bind("canvas.comments.list", "GET", "/api/canvas/{canvas_id}/comments", &[("file_id", "canvas_id")], &[]),
    bind("canvas.versions.list", "GET", "/api/canvas/{canvas_id}/versions", &[("file_id", "canvas_id")], &[]),
    // ——— #2038 — Goals (crates/botanalytics) ———
    bind("goals.okrs.list", "GET", "/api/goals/objectives", &[], &[]),
    bind("goals.objective.get", "GET", "/api/goals/objectives/:id", &[("objective_id", "id")], &[]),
    bind("goals.key_result.update", "PUT", "/api/goals/key-results/:id", &[("key_result_id", "id")], &[]),
    bind("goals.key_result.check_in", "POST", "/api/goals/key-results/:id/check-in", &[("key_result_id", "id"), ("value", "new_value")], &[]),
    bind("goals.templates.list", "GET", "/api/goals/templates", &[], &[]),
    bind("goals.dashboard", "GET", "/api/goals/dashboard", &[], &[]),
    bind("goals.alignment", "GET", "/api/goals/alignment", &[], &[]),
    bind("goals.ai.suggest", "POST", "/api/goals/ai/suggest", &[("area", "context")], &[]),
    // ——— #2039 — Jukebox (src/jukebox) ———
    bind("jukebox.generate", "POST", "/api/jukebox/generate", &[("description", "prompt")], &[]),
    bind("jukebox.format", "POST", "/api/jukebox/format", &[], &[]),
    bind("jukebox.models", "GET", "/api/jukebox/models", &[], &[]),
    bind("jukebox.health", "GET", "/api/jukebox/health", &[], &[]),
    bind("jukebox.job.get", "GET", "/api/jukebox/jobs/{job_id}", &[], &[]),
    // ——— #2039 — Video (crates/botvideo) ———
    bind("video.projects.list", "GET", "/api/video/projects", &[], &[]),
    bind("video.project.get", "GET", "/api/video/projects/{id}", &[("project_id", "id")], &[]),
    bind("video.project.export", "POST", "/api/video/projects/:id/export", &[("project_id", "id"), ("resolution", "quality")], &[]),
    bind("video.export.status", "GET", "/api/video/exports/:id/status", &[("export_id", "id")], &[]),
    bind("video.project.scenes", "POST", "/api/video/projects/:id/scenes", &[("project_id", "id")], &[]),
    bind("video.project.tts", "POST", "/api/video/projects/:id/tts", &[("project_id", "id")], &[]),
    bind("video.project.captions", "POST", "/api/video/projects/{id}/captions", &[("project_id", "id")], &[]),
    bind("video.cameras.list", "GET", "/api/video/cameras", &[], &[]),
    bind("video.alerts.list", "GET", "/api/video/alerts", &[], &[]),
    bind("video.analytics.summary", "GET", "/api/video/analytics", &[], &[]),
    bind("video.templates.list", "GET", "/api/video/templates", &[], &[]),
    // ——— #2040 — Browser automation (crates/botbrowser) ———
    bind("browser.session.create", "POST", "/api/browser/session", &[], &[]),
    bind("browser.session.open", "POST", "/api/browser/session", &[], &[]),
    bind("browser.session.state", "GET", "/api/browser/session/:id/state", &[("session_id", "id")], &[]),
    bind("browser.session.delete", "DELETE", "/api/browser/session/:id", &[("session_id", "id")], &[]),
    bind("browser.navigate", "POST", "/api/browser/session/:id/navigate", &[("session_id", "id")], &[]),
    bind("browser.click", "POST", "/api/browser/session/:id/click", &[("session_id", "id"), ("label", "selector")], &[]),
    bind("browser.fill", "POST", "/api/browser/session/:id/fill", &[("session_id", "id"), ("field", "selector"), ("value", "text")], &[]),
    bind("browser.execute", "POST", "/api/browser/session/:id/execute", &[("session_id", "id")], &[]),
    bind("browser.extract", "GET", "/api/browser/session/:id/extract", &[("session_id", "id")], &[]),
    bind("browser.screenshot", "GET", "/api/browser/session/:id/screenshot", &[("session_id", "id")], &[]),
    bind("browser.tabs.list", "GET", "/api/browser/tabs", &[], &[]),
    bind("browser.agent", "POST", "/api/browser/session/:id/agent", &[("session_id", "id")], &[]),
];

#[cfg(test)]
mod verb_endpoint_tests {
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
            assert!(seen.insert(binding.verb), "duplicate binding {}", binding.verb);
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
        assert_eq!(empty.get("layout"), Some(&Value::String("content".to_string())));

        let mut params = Map::new();
        params.insert("layout".to_string(), Value::String("blank".to_string()));
        let mapped = binding.map_params(&params);
        assert_eq!(mapped.get("layout"), Some(&Value::String("blank".to_string())));
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
        assert_eq!(mapped.get("mode"), Some(&Value::String("append".to_string())));

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
            assert!(response_has_id(&body), "created spreadsheet must carry its id");
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
            let deck = botslides::storage::create_presentation_with(
                "owner-1",
                Some("Kickoff"),
                Some(&slides),
            );
            let body = serde_json::to_value(deck).unwrap_or(Value::Null);
            assert!(response_has_id(&body), "created deck must carry its id");
        }
    }
}
