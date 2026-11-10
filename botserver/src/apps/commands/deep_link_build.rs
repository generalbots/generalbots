//! #2143 (gap C) — the single place where an `app://` deep link is built.
//!
//! Command deep links are templates (`app://sheet?file_id={file_id}`). Nothing
//! filled those placeholders, so a rendered button carried the literal
//! `{file_id}` — or a bare `app://sheet` — and opened an empty window. Every
//! link is now resolved here, from the values the command actually has: the
//! params the caller passed and the result the bound endpoint returned (the
//! artifact id/sku/path it created or updated).
//!
//! Template grammar:
//!
//! - `{key}` — required: without a value the navigation FAILS with a clear
//!   error instead of opening an empty app.
//! - `{key?}` — optional: the query pair is dropped when no value is available
//!   (a list verb links the app view, not a record it cannot name).
//!
//! A template with no query (`app://docs`) names the app view; when the command
//! produced a record, the app's declared record key is appended so the link
//! opens that record.

use super::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Why a deep link could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepLinkError {
    /// App the link would have opened.
    pub app: String,
    /// Command that requested the navigation.
    pub command: String,
    /// Declared template, e.g. `app://sheet?file_id={file_id}`.
    pub template: &'static str,
    /// Placeholder that had no value.
    pub placeholder: String,
}

impl std::fmt::Display for DeepLinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot open the {} app from '{}': '{}' in the deep link '{}' has no value, \
             so nothing was opened",
            self.app, self.command, self.placeholder, self.template
        )
    }
}

impl std::error::Error for DeepLinkError {}

/// One `{...}` occurrence in a template.
struct Placeholder {
    /// Whole `{...}` text, exactly as it appears in the template.
    raw: String,
    /// Value key, e.g. `file_id`.
    key: String,
    /// True for `{key?}`.
    optional: bool,
}

/// Every placeholder of `text`, in order of appearance.
fn placeholders(text: &str) -> Vec<Placeholder> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else { break };
        let body = &after[..end];
        let (key, optional) = match body.strip_suffix('?') {
            Some(key) => (key.trim(), true),
            None => (body.trim(), false),
        };
        if !key.is_empty() && !key.contains('{') {
            out.push(Placeholder {
                raw: format!("{{{body}}}"),
                key: key.to_string(),
                optional,
            });
        }
        rest = &after[end + 1..];
    }
    out
}

/// Values a template can draw on, flattened out of the params and the result.
///
/// Objects are walked (a create endpoint may answer `{"sheet": {"id": ...}}`);
/// arrays are never walked: a list does not name one record, and taking the
/// first id would point the link at a record the user did not ask for.
#[derive(Default)]
struct ValueIndex {
    values: BTreeMap<String, String>,
}

impl ValueIndex {
    /// Params first, so an explicit reference wins over an echoed result.
    fn new(params: &Map<String, Value>, result: &Value) -> Self {
        let mut index = Self::default();
        index.walk_map(params, 0);
        if let Value::Object(map) = result {
            index.walk_map(map, 0);
        }
        index
    }

    fn walk_map(&mut self, map: &Map<String, Value>, depth: usize) {
        const MAX_DEPTH: usize = 4;
        if depth > MAX_DEPTH {
            return;
        }
        for (key, val) in map {
            match val {
                Value::String(text) if !text.trim().is_empty() => {
                    self.values
                        .entry(key.clone())
                        .or_insert_with(|| text.clone());
                }
                Value::Number(number) => {
                    self.values
                        .entry(key.clone())
                        .or_insert_with(|| number.to_string());
                }
                Value::Object(nested) => self.walk_map(nested, depth + 1),
                Value::Bool(_) | Value::Array(_) | Value::String(_) | Value::Null => {}
            }
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Value for `key`: exact, then the names other apps/verbs use for the same
    /// reference, then the record id a create endpoint answers with.
    fn resolve(&self, key: &str) -> Option<&str> {
        if let Some(value) = self.get(key) {
            return Some(value);
        }
        if let Some(value) = aliases_for(key).iter().find_map(|alias| self.get(alias)) {
            return Some(value);
        }
        if is_record_key(key) {
            return self.get("id");
        }
        None
    }
}

/// Names the apps and verbs use for the same reference: a create endpoint
/// answers with `id`, a deck link says `deck_id`, the sheet says `file_id`.
fn aliases_for(key: &str) -> &'static [&'static str] {
    match key {
        "file" => &["file_id", "doc_id", "path", "key"],
        "file_id" => &["file", "doc_id", "presentation_id", "sheet_id", "path"],
        "deck_id" => &["file_id", "presentation_id", "file"],
        "person_id" => &["contact_id", "person"],
        "contact_id" => &["person_id"],
        "product_id" => &["sku", "item_id"],
        "sku" => &["product_id"],
        "okr_id" => &["objective_id"],
        "objective_id" => &["okr_id"],
        "media_id" => &["path", "stream_id"],
        "path" => &["file", "key"],
        _ => &[],
    }
}

/// Keys that name a single record, so a result `id` can fill them.
fn is_record_key(key: &str) -> bool {
    key == "id" || key.ends_with("_id") || matches!(key, "file" | "path" | "sku")
}

/// Percent-encodes a value for a query string, keeping `/`, `:` and `%` so
/// paths stay readable and an already-encoded value is not encoded twice.
fn encode_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.as_bytes() {
        let ch = *byte as char;
        let keep = ch.is_ascii_alphanumeric()
            || matches!(ch, '-' | '_' | '.' | '~' | '/' | ':' | '@' | '%');
        if keep {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Builds `command`'s deep link. `Ok(None)` when the command declares none.
pub fn build_deep_link(
    command: &AppCommand,
    params: &Map<String, Value>,
    result: &Value,
) -> Result<Option<String>, DeepLinkError> {
    let Some(template) = command.deep_link else {
        return Ok(None);
    };
    let index = ValueIndex::new(params, result);
    let Some((base, query)) = template.split_once('?') else {
        return Ok(Some(record_or_app_view(command.app, template, &index)));
    };
    let mut pairs: Vec<String> = Vec::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let found = placeholders(pair);
        if found.is_empty() {
            pairs.push(pair.to_string());
            continue;
        }
        let mut filled = pair.to_string();
        let mut keep = true;
        for placeholder in &found {
            match index.resolve(&placeholder.key) {
                Some(value) => {
                    filled = filled.replace(&placeholder.raw, &encode_value(value));
                }
                None if placeholder.optional => {
                    keep = false;
                    break;
                }
                None => {
                    return Err(DeepLinkError {
                        app: command.app.to_string(),
                        command: command.name.to_string(),
                        template,
                        placeholder: placeholder.key.clone(),
                    })
                }
            }
        }
        if keep {
            pairs.push(filled);
        }
    }
    if pairs.is_empty() {
        return Ok(Some(base.to_string()));
    }
    Ok(Some(format!("{base}?{}", pairs.join("&"))))
}

/// A template without a query names the app view; a produced record turns it
/// into a record link so the window does not open empty.
fn record_or_app_view(app: &str, base: &str, index: &ValueIndex) -> String {
    let Some(record_key) = deep_link_params_for_app(app).first() else {
        return base.to_string();
    };
    match index.resolve(record_key.key) {
        Some(value) => format!("{base}?{}={}", record_key.key, encode_value(value)),
        None => base.to_string(),
    }
}

/// The navigation answer for a command that only opens an app. An unresolved
/// required placeholder is an error, never a link that opens an empty window.
pub fn navigation_result(
    command: &AppCommand,
    params: &Map<String, Value>,
) -> Result<Value, String> {
    match build_deep_link(command, params, &Value::Null) {
        Ok(Some(link)) => Ok(serde_json::json!({
            "navigate": true,
            "app": command.app,
            "label": command.label,
            "deep_link": link,
        })),
        Ok(None) => Err(format!(
            "command '{}' is not executable from chat; open the {} app instead",
            command.name,
            if command.app.is_empty() {
                "suite"
            } else {
                command.app
            }
        )),
        Err(error) => Err(error.to_string()),
    }
}

/// Attaches the resolved navigation to a command result. A bound create/edit
/// verb answers with the artifact id, so the deep link points at that artifact
/// instead of the bare app view. Nothing is attached when the link would only
/// be the app view: the result already describes what happened, and a create
/// verb that returned no id must not open an empty window.
pub fn with_navigation(command_name: &str, params: &Map<String, Value>, result: Value) -> Value {
    let Some(command) = command_by_name(command_name) else {
        return result;
    };
    let link = match build_deep_link(command, params, &result) {
        Ok(Some(link)) => link,
        Ok(None) => return result,
        Err(error) => {
            log::warn!("{error}");
            return result;
        }
    };
    if !link.contains('?') {
        return result;
    }
    match result {
        Value::Object(mut map) => {
            map.entry("navigate").or_insert(Value::Bool(true));
            map.entry("app")
                .or_insert(Value::String(command.app.to_string()));
            map.entry("label")
                .or_insert(Value::String(command.label.to_string()));
            map.entry("deep_link").or_insert(Value::String(link));
            Value::Object(map)
        }
        other => other,
    }
}

#[cfg(test)]
#[path = "deep_link_build_tests.rs"]
mod deep_link_build_tests;
