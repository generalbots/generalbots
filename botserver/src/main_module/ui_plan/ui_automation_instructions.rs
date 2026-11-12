/// Formats the three prompt catalogues for the apps accepted by `available`.
/// Kept separate from the product-config lookup so the filter is testable.
fn catalog_lines<F>(available: F) -> (Vec<String>, Vec<String>, Vec<String>)
where
    F: Fn(&str) -> bool,
{
    let apps: Vec<String> = crate::apps::registry::all_apps()
        .iter()
        .filter(|a| available(&a.id))
        .map(|a| format!("- {} (id: {}) — {}", a.title, a.id, a.description))
        .collect();
    let deep_links: Vec<String> = crate::apps::commands::all_deep_links()
        .filter(|pair| available(pair.0))
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
        .filter(|(app_id, _)| available(app_id))
        .map(|(app_id, hint)| format!("- {app_id}: {hint}"))
        .collect();
    (apps, deep_links, seq_hints)
}

/// Builds the system-prompt fragment teaching the LLM the `__ui_plan__`
/// contract. Includes the app catalog so the model only references apps
/// that actually exist in this build.
pub fn ui_automation_instructions(client_caps: Option<&serde_json::Value>) -> String {
    // #2238 — explicit toggle: when app automation is off, omit UI hints
    // entirely so the model answers in prose/commands instead of plans.
    let ui_automation_on = client_caps
        .and_then(|caps| caps.get("ui_automation"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true);
    if !ui_automation_on {
        return String::new();
    }
    let preview_on = client_caps
        .and_then(|caps| caps.get("preview_mode"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    // #1691 — advertise only apps that are really available to this session.
    // #2097 — the registry declares the always/preview split; `.product` can
    // still enable an app explicitly (`apps=`) or withhold it (`hidden_apps`).
    let available = |id: &str| -> bool {
        let Some(config) = botcore::product::PRODUCT_CONFIG.read().ok() else {
            return crate::apps::registry::is_always_visible(id)
                || (preview_on && crate::apps::registry::is_preview_only(id));
        };
        if config.is_app_hidden(id) {
            return false;
        }
        if config.is_app_enabled(id) {
            return true;
        }
        match crate::apps::registry::launcher_visibility_of(id) {
            Some(crate::apps::registry::LauncherVisibility::Always) => true,
            Some(crate::apps::registry::LauncherVisibility::Preview) => preview_on,
            None => preview_on && config.is_app_preview(id),
        }
    };
    let (apps, deep_links, seq_hints) = catalog_lines(available);
    format!(
        r#"---
## Agentic UI Automation

You can drive the suite applications on behalf of the user. When the user
asks you to create, update, fill, or act on data inside an application,
respond with a single JSON object as the FIRST line of your reply, followed
by a short user-facing confirmation message.

Available applications:
{apps}

Only plan applications listed above. If the user asks for something that
needs an application outside this list, answer in prose (or explain that the
capability is not enabled) and NEVER emit a plan for an unlisted app id.

Contract — emit exactly:
{{"__ui_plan__": {{"app": "<app-id>", "steps": [
  {{"op": "open", "app": "<app-id>"}},
  {{"op": "click", "label": "<visible button/link text>"}},
  {{"op": "fill", "field": "<field label or name>", "value": "<value>"}},
  {{"op": "select", "field": "<field label>", "value": "<option>"}},
  {{"op": "cell", "cell": "<A1-style reference>", "value": "<value>"}},
  {{"op": "menu", "path": ["<top-level menu>", "<submenu>"]}},
  {{"op": "download", "label": "<export/download button text>"}},
  {{"op": "submit"}},
  {{"op": "wait", "ms": 500}}
]}}}}

Rules:
- Only use app ids from the list above; default to the app matching the user's request.
- App-backed intents (#1944): when the request names one of the applications above
  ("open the Database app and create a table", "use Weather for this city"), the
  answer MUST be a `__ui_plan__` for that app. Never route such a request to a
  VM, terminal, container or other agent execution path — those are for work the
  suite has no application for.
- Live document generation (#2184): when the user asks you to WRITE a document
  (résumé, letter, report, contract, article, memo, proposal), draw it in the
  Docs app instead of producing a file: `open` the app, then one `fill` step on
  the document body whose value is the whole document in markdown-lite
  (`# ` / `## ` headings, `- ` bullets, `1. ` numbered items, blank line between
  paragraphs, `**bold**`). Do not also generate a .docx/.pdf for the same
  request, and do not emit one fill per line — the single fill is what renders
  the document live.
- `fill` values must be plain text without HTML.
- Use `menu` with the exact visible labels for toolbar menus (e.g.
  ["Page", "Background", "Yellow"]) — one click cannot open a submenu.
- Use `download` (not `click`) when the step must produce a file, so the plan
  verifies an artefact appeared.
- Exports that live behind a toolbar menu need the menu opened first. In the
  sheet app the export menu is the "Export" toolbar button and its items are
  "CSV", "XLSX", "Markdown", "ODS", "PDF", "HTML", "JSON", so a CSV export
  is `{{"op":"menu","path":["Export","CSV"]}}` (or `["Export"]` followed by
  `{{"op":"download","label":"CSV"}}` when the file must be verified). A lone
  `download` step labelled "Export CSV" fails: no control carries that label.
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

    /// #1945 — the catalogue must be generated from the registry, so an app
    /// added there becomes discoverable without touching the prompt. A
    /// hand-maintained list silently hid newly enabled apps from the model.
    #[test]
    fn every_registered_app_reaches_the_catalogue() {
        let (apps, _, _) = catalog_lines(|_| true);
        for app in crate::apps::registry::all_apps() {
            assert!(
                apps.iter()
                    .any(|line| line.contains(&format!("(id: {})", app.id))),
                "{} is missing from the UI automation catalogue",
                app.id
            );
        }
    }

    #[test]
    fn only_enabled_apps_enter_the_prompt_catalogue() {
        let (apps, deep_links, hints) = catalog_lines(|id| id == "weather");
        assert!(
            apps.iter().any(|line| line.contains("(id: weather)")),
            "{apps:?}"
        );
        assert!(
            !apps.iter().any(|line| line.contains("(id: crm)")),
            "{apps:?}"
        );
        assert!(
            deep_links.iter().all(|line| line.starts_with("- weather")),
            "{deep_links:?}"
        );
        assert!(
            hints.iter().all(|line| line.starts_with("- weather")),
            "{hints:?}"
        );
    }
}
