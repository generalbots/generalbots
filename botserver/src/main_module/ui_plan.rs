//! Agentic UI plan protocol.
//!
//! The LLM emits a `{"__ui_plan__": {...}}` JSON block (mirroring the existing
//! `__tool_call__` mechanism) when the user requests a UI-level operation such
//! as creating a record. The backend validates every plan before it reaches
//! the frontend driver — the client never trusts the LLM directly.

use serde::{Deserialize, Serialize};

pub const UI_PLAN_TRIGGER: &str = "\"__ui_plan__\":";

/// Maximum number of steps accepted in a single plan. Spreadsheet data entry
/// can legitimately exceed 32 steps (e.g. filling a 10-row table), so keep
/// this generous while still bounding runaway LLM output.
pub const MAX_UI_STEPS: usize = 200;

/// Maximum length of any string value produced by the LLM.
pub const MAX_VALUE_LEN: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiOp {
    Open,
    Click,
    Fill,
    Select,
    Submit,
    Wait,
    Cell,
}

impl UiOp {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "open" => Some(Self::Open),
            "click" => Some(Self::Click),
            "fill" => Some(Self::Fill),
            "select" => Some(Self::Select),
            "submit" => Some(Self::Submit),
            "wait" => Some(Self::Wait),
            "cell" => Some(Self::Cell),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStep {
    pub op: UiOp,
    pub app: Option<String>,
    pub label: Option<String>,
    pub field: Option<String>,
    pub value: Option<String>,
    pub ms: Option<u64>,
    pub cell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPlan {
    pub app: Option<String>,
    pub steps: Vec<UiStep>,
}

/// Strips HTML/control characters and truncates a raw LLM value.
pub fn sanitize_value(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .filter(|c| !matches!(c, '<' | '>' | '\0'))
        .collect();
    cleaned.chars().take(MAX_VALUE_LEN).collect()
}

fn valid_app_id(app: &str) -> bool {
    let trimmed = app.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.len() > 64 {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
}

/// Validates a decoded plan. Returns the sanitized plan on success.
pub fn validate_ui_plan(raw: &UiPlan) -> Result<UiPlan, String> {
    if raw.steps.is_empty() {
        return Err("UI plan has no steps".to_string());
    }
    if raw.steps.len() > MAX_UI_STEPS {
        return Err(format!(
            "UI plan exceeds step limit of {MAX_UI_STEPS}"
        ));
    }

    if let Some(app) = &raw.app {
        if !valid_app_id(app) {
            return Err(format!("Invalid app id '{app}'"));
        }
    }

    let mut sanitized_steps = Vec::with_capacity(raw.steps.len());
    for (idx, step) in raw.steps.iter().enumerate() {
        let sanitized = sanitize_step(step)?;
        if step.op == UiOp::Open {
            let app = step
                .app
                .as_deref()
                .or(raw.app.as_deref())
                .ok_or_else(|| "open step requires an app".to_string())?;
            if !valid_app_id(app) {
                return Err(format!("Invalid app id '{app}' at step {idx}"));
            }
        }
        sanitized_steps.push(sanitized);
    }

    Ok(UiPlan {
        app: raw.app.as_deref().map(sanitize_value),
        steps: sanitized_steps,
    })
}

fn sanitize_step(step: &UiStep) -> Result<UiStep, String> {
    let mut out = UiStep {
        op: step.op.clone(),
        app: None,
        label: None,
        field: None,
        value: None,
        ms: None,
        cell: None,
    };
    if let Some(app) = &step.app {
        out.app = Some(sanitize_value(app));
    }
    if let Some(label) = &step.label {
        out.label = Some(sanitize_value(label));
    }
    if let Some(field) = &step.field {
        out.field = Some(sanitize_value(field));
    }
    if let Some(value) = &step.value {
        out.value = Some(sanitize_value(value));
    }
    if let Some(ms) = step.ms {
        out.ms = Some(ms.min(30_000));
    }
    if let Some(cell) = &step.cell {
        let upper = cell.trim().to_uppercase();
        if !is_cell_ref(&upper) {
            return Err(format!("Invalid cell reference '{cell}' at step"));
        }
        out.cell = Some(upper);
    }
    if step.op == UiOp::Cell && out.cell.is_none() {
        return Err("cell step requires a cell reference".to_string());
    }
    if step.op == UiOp::Cell && out.value.is_none() {
        return Err("cell step requires a value".to_string());
    }
    Ok(out)
}

/// Validates a spreadsheet cell reference like "A1" or "B5".
fn is_cell_ref(reference: &str) -> bool {
    let bytes = reference.as_bytes();
    if bytes.is_empty() || bytes.len() > 5 {
        return false;
    }
    let letters: Vec<u8> = bytes.iter().copied().take_while(u8::is_ascii_uppercase).collect();
    if letters.is_empty() || letters.len() > 3 {
        return false;
    }
    let rest = &bytes[letters.len()..];
    if rest.is_empty() || rest.len() > 3 {
        return false;
    }
    rest.iter().all(u8::is_ascii_digit)
}

/// Attempts to decode a raw `__ui_plan__` payload into a validated plan.
pub fn parse_ui_plan(payload: &serde_json::Value) -> Result<UiPlan, String> {
    let raw: UiPlan = serde_json::from_value(payload.clone())
        .map_err(|e| format!("UI plan parse error: {e}"))?;
    validate_ui_plan(&raw)
}

/// Extracts and validates the first `{"__ui_plan__": {...}}` object from a
/// raw LLM response string. Returns `None` when no plan is present.
pub fn extract_and_validate_plan(raw_response: &str) -> Option<Result<UiPlan, String>> {
    let start = raw_response.find(UI_PLAN_TRIGGER)?;
    let obj_start = raw_response[..start].rfind('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = obj_start;
    for (idx, c) in raw_response[obj_start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = obj_start + idx + c.len_utf8();
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let candidate = raw_response[obj_start..end].trim();
    let payload: serde_json::Value = match serde_json::from_str(candidate) {
        Ok(v) => v,
        Err(e) => return Some(Err(format!("UI plan JSON error: {e}"))),
    };
    let plan_value = payload.get("__ui_plan__").cloned().unwrap_or(payload);
    Some(parse_ui_plan(&plan_value))
}

/// Removes the `{"__ui_plan__": {...}}` JSON block from a response so the
/// plan contract never reaches the user-visible chat content. Handles the
/// trigger spanning stream chunk boundaries.
pub fn strip_plan_json(raw_response: &str) -> String {
    let Some(start) = raw_response.find(UI_PLAN_TRIGGER) else {
        return raw_response.to_string();
    };
    let Some(obj_start) = raw_response[..start].rfind('{') else {
        return raw_response.to_string();
    };
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = obj_start;
    for (idx, c) in raw_response[obj_start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = obj_start + idx + c.len_utf8();
                    break;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return raw_response.to_string();
    }
    let mut stripped = String::with_capacity(raw_response.len());
    stripped.push_str(raw_response[..obj_start].trim_start_matches(|c: char| c.is_whitespace()));
    stripped.push_str(raw_response[end..].trim_start_matches(|c: char| c == '\n'));
    stripped.trim().to_string()
}

/// Builds the system-prompt fragment teaching the LLM the `__ui_plan__`
/// contract. Includes the app catalog so the model only references apps
/// that actually exist in this build.
pub fn ui_automation_instructions() -> String {
    let apps: Vec<String> = crate::apps::registry::all_apps()
        .iter()
        .map(|a| format!("- {} (id: {}) — {}", a.title, a.id, a.description))
        .collect();
    let deep_links: Vec<String> = crate::apps::commands::APP_DEEP_LINKS
        .iter()
        .map(|(app_id, params)| {
            let keys = params
                .iter()
                .map(|p| format!("{} ({})", p.key, p.description))
                .collect::<Vec<_>>()
                .join(", ");
            format!("- {app_id}: {keys}")
        })
        .collect();
    let seq_hints: Vec<String> = crate::apps::commands::UI_SEQUENCE_HINTS
        .iter()
        .map(|(app_id, hint)| format!("- {app_id}: {hint}"))
        .collect();
    format!(
        r#"---
## Agentic UI Automation

You can drive the suite applications on behalf of the user. When the user
asks you to create, update, fill, or act on data inside an application,
respond with a single JSON object as the FIRST line of your reply, followed
by a short user-facing confirmation message.

Available applications:
{apps}

Contract — emit exactly:
{{"__ui_plan__": {{"app": "<app-id>", "steps": [
  {{"op": "open", "app": "<app-id>"}},
  {{"op": "click", "label": "<visible button/link text>"}},
  {{"op": "fill", "field": "<field label or name>", "value": "<value>"}},
  {{"op": "select", "field": "<field label>", "value": "<option>"}},
  {{"op": "cell", "cell": "<A1-style reference>", "value": "<value>"}},
  {{"op": "submit"}},
  {{"op": "wait", "ms": 500}}
]}}}}

Rules:
- Only use app ids from the list above; default to the app matching the user's request.
- `fill` values must be plain text without HTML.
- When unsure of a field name, use the label text shown in the form.
- Keep plans minimal: open, fill the requested fields, submit.
- For spreadsheet apps (sheet), write data with `cell` steps using A1-style references (e.g. "A1", "B2"). One `cell` step per cell.
- Plans may contain up to 200 steps; emit one step per cell/field, never truncate data.
- If the user asks to FIND something, use the search endpoint or apps.find instead: do not emit a plan, just answer normally.

### Contextual deep links to applications
When you reference a specific record that lives in an application (a person in
CRM, an invoice in billing, a file in drive, a product in products), include a
clickable deep link in your web reply so the user can open the app already
contextualized to that record. Use the exact markdown form:

  [<short action label>](app://<app-id>?<key>=<value>&<key2>=<value2>)

Examples:
- A person: `[Abrir ficha de Maria Silva](app://crm?person_id=123e4567-e89b-12d3-a456-426614174000)`
- An invoice: `[Ver fatura](app://billing?invoice_id=<billing-user-id>)`
- A product: `[Ver produto](app://products?product_id=<id>)`
- A file: `[Abrir no Drive](app://drive?path=faturas%2F2026-08)`

Per-app deep-link keys (use the key that matches the record you reference):

{deep_links}

Rules for deep links:
- Only emit links to apps from the list above and only for ids you actually
  returned in your answer (never invent an id).
- Put ONE deep link per referenced record, directly after it in the reply.
- This deep-link syntax is only rendered by the web desktop shell; on other
  channels the syntax is invisible to the user, so it is safe to include.

### UI navigation hints (web desktop only)
When planning `__ui_plan__` steps, follow the natural flow of the target app:

{seq_hints}

These hints describe how each app loads and where its actions live, so your
plan should navigate the app realistically instead of guessing buttons.
"#,
        apps = apps.join("\n"),
        deep_links = deep_links.join("\n"),
        seq_hints = seq_hints.join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_validate() {
        let plan = serde_json::json!({
            "app": "crm",
            "steps": [
                {"op": "open", "app": "crm"},
                {"op": "fill", "field": "First Name", "value": "Jeff"},
                {"op": "submit"}
            ]
        });
        let parsed = parse_ui_plan(&plan).unwrap();
        assert_eq!(parsed.steps.len(), 3);
        assert_eq!(parsed.app.as_deref(), Some("crm"));
    }

    #[test]
    fn test_rejects_unknown_op() {
        let plan = serde_json::json!({
            "steps": [{"op": "rm -rf"}]
        });
        let raw: Result<UiPlan, _> = serde_json::from_value(plan);
        assert!(raw.is_err());
    }

    #[test]
    fn test_rejects_invalid_app() {
        let plan = serde_json::json!({
            "steps": [{"op": "open", "app": "../etc"}]
        });
        let raw: UiPlan = serde_json::from_value(plan).unwrap();
        assert!(validate_ui_plan(&raw).is_err());
    }

    #[test]
    fn test_sanitizes_values() {
        let plan = serde_json::json!({
            "steps": [{"op": "fill", "field": "Name", "value": "<script>alert(1)</script>"}]
        });
        let parsed = parse_ui_plan(&plan).unwrap();
        assert_eq!(parsed.steps[0].value.as_deref(), Some("scriptalert(1)/script"));
    }

    #[test]
    fn test_rejects_too_many_steps() {
        let steps: Vec<serde_json::Value> = (0..300)
            .map(|i| serde_json::json!({"op": "wait", "ms": i + 1}))
            .collect();
        let plan = serde_json::json!({"steps": steps});
        let raw: UiPlan = serde_json::from_value(plan).unwrap();
        assert!(validate_ui_plan(&raw).is_err());
    }

    #[test]
    fn test_open_step_requires_app() {
        let plan = serde_json::json!({
            "steps": [{"op": "open"}]
        });
        let raw: UiPlan = serde_json::from_value(plan).unwrap();
        assert!(validate_ui_plan(&raw).is_err());
    }

    #[test]
    fn test_extract_embedded_plan() {
        let raw = r#"Sure! Creating the customer now.
{"__ui_plan__": {"app": "crm", "steps": [
  {"op": "open", "app": "crm"},
  {"op": "click", "label": "New Lead"},
  {"op": "fill", "field": "First Name", "value": "Jeff"}
]}}
Done."#;
        let result = extract_and_validate_plan(raw);
        let plan = result.expect("plan present").expect("plan valid");
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.steps[2].op, UiOp::Fill);
        assert_eq!(
            plan.steps[2].value.as_deref(),
            Some("Jeff")
        );
    }

    #[test]
    fn test_extract_missing_plan_returns_none() {
        assert!(extract_and_validate_plan("just a greeting").is_none());
    }

    #[test]
    fn test_extract_invalid_json_returns_err() {
        let raw = r#"{"__ui_plan__": {"app": "crm", "steps": [{ "op": "bogus_op" }]}}"#;
        let result = extract_and_validate_plan(raw);
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_extract_unbalanced_json_returns_none() {
        let raw = r#"{"__ui_plan__": {"app": "crm", "steps": [broken"#;
        assert!(extract_and_validate_plan(raw).is_none());
    }

    #[test]
    fn test_strip_plan_json_removes_block() {
        let raw = r#"{"__ui_plan__": {"app": "mail", "steps": [{"op": "open", "app": "mail"}]}}
Compose a message to Rodrigo."#;
        let stripped = strip_plan_json(raw);
        assert!(!stripped.contains("__ui_plan__"));
        assert!(stripped.contains("Compose a message to Rodrigo."));
    }

    #[test]
    fn test_strip_plan_json_keeps_normal_content() {
        let raw = "Just a plain greeting.";
        assert_eq!(strip_plan_json(raw), raw);
    }
}
