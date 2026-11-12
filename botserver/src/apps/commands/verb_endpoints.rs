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
    bind("sheet.import", "POST", "/api/sheet/import", &[], &[]),
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
#[path = "verb_endpoint_tests.rs"]
mod verb_endpoint_tests;
