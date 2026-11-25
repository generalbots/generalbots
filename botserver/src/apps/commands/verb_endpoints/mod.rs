use serde_json::{Map, Value};

/// Binds a curated verb to the REST endpoint that already implements it, so a
/// command executed from chat performs the action instead of only answering
/// with a deep link.
///
/// Every entry names a route from `endpoint_inventory::all_routes()` — no verb
/// invents a path. `renames` translates the curated parameter names onto the
/// field names the handler deserializes; `defaults` fills a required field the
/// verb does not carry (never a value the user asked for).
#[derive(Debug, Clone, Copy)]
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

pub(super) const fn bind(
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
    table::all().iter().find(|binding| binding.verb == name)
}

/// True when a catalog verb marked `admin_only` binds to this concrete
/// `(method, path)`. The trusted `api.exec` path uses it so a prompt injection
/// cannot reach the admin verb's REST route by naming the path instead of the
/// verb (#2448 criterion 5).
pub fn admin_only_verb_binding(method: &str, path: &str) -> bool {
    table::all().iter().any(|binding| {
        binding.method.eq_ignore_ascii_case(method)
            && path_matches_template(path, binding.path)
            && crate::apps::commands::command_by_name(binding.verb)
                .is_some_and(|command| command.admin_only)
    })
}

/// Compares a filled path with a bind template. `:name` and `{name}` template
/// segments are wildcards; all other segments must match case-insensitively.
fn path_matches_template(path: &str, template: &str) -> bool {
    let actual: Vec<&str> = path.split('/').collect();
    let expected: Vec<&str> = template.split('/').collect();
    actual.len() == expected.len()
        && actual.iter().zip(expected).all(|(segment, pattern)| {
            pattern.starts_with(':')
                || (pattern.starts_with('{') && pattern.ends_with('}'))
                || segment.eq_ignore_ascii_case(pattern)
        })
}

#[cfg(test)]
mod admin_binding_tests {
    use super::*;

    #[test]
    fn admin_only_verbs_match_their_bound_routes() {
        assert!(admin_only_verb_binding(
            "POST",
            "/api/docs/track-changes/accept-reject-all"
        ));
        assert!(admin_only_verb_binding(
            "POST",
            "/api/sheet/worksheets/delete"
        ));
    }

    #[test]
    fn other_routes_and_methods_are_not_admin_only() {
        assert!(!admin_only_verb_binding(
            "POST",
            "/api/docs/comment/reply"
        ));
        assert!(!admin_only_verb_binding(
            "GET",
            "/api/docs/track-changes/accept-reject-all"
        ));
        assert!(!admin_only_verb_binding(
            "POST",
            "/api/docs/unknown-route"
        ));
    }

    /// Non-vacuous guard: an admin-only verb that gains a REST binding must be
    /// denied through `api.exec` on that very route.
    #[test]
    fn every_admin_only_binding_is_denied_on_its_own_route() {
        let mut gated = 0usize;
        for binding in table::all() {
            if crate::apps::commands::command_by_name(binding.verb)
                .is_some_and(|command| command.admin_only)
            {
                assert!(
                    admin_only_verb_binding(binding.method, binding.path),
                    "admin-only verb {} is not gated on its bound route",
                    binding.verb
                );
                gated += 1;
            }
        }
        assert!(gated > 0, "no admin-only verb bindings found");
    }
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

mod table;

pub fn verb_endpoints() -> &'static [VerbEndpoint] {
    table::all()
}

#[cfg(test)]
#[path = "../verb_endpoint_tests.rs"]
mod verb_endpoint_tests;
