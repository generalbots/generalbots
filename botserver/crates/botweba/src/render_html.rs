use super::*;
use crate::escape::{escape_html, safe_resource_url};

/// Renders the shell for one published app page.
///
/// Everything except `content` is user data and is escaped: the app name (which
/// alone was enough for `</title><script>…`), the description, and the
/// meta-tag names/values. `config.scripts` and `config.styles` are filtered
/// through [`safe_resource_url`], so `javascript:`/`data:` entries are dropped
/// instead of becoming executable markup.
///
/// `content` stays raw on purpose — it is the app's HTML — but it is served
/// with [`crate::csp::WEBA_CSP`], which gives the document an opaque origin and
/// refuses inline script, so raw markup can no longer act as the suite (#2124).
pub fn render_html(app: &WebApp, content: &str) -> String {
    let description = app
        .description
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .filter(|_| !app.config.meta_tags.contains_key("description"))
        .map(|text| {
            format!(
                r#"<meta name="description" content="{}">"#,
                escape_html(text)
            )
        })
        .unwrap_or_default();

    let meta_tags: String = app
        .config
        .meta_tags
        .iter()
        .map(|(name, value)| {
            format!(
                r#"<meta name="{}" content="{}">"#,
                escape_html(name),
                escape_html(value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let scripts: String = app
        .config
        .scripts
        .iter()
        .filter_map(|script| safe_resource_url(script))
        .map(|script| format!(r#"<script src="{}"></script>"#, escape_html(&script)))
        .collect::<Vec<_>>()
        .join("\n    ");

    let styles: String = app
        .config
        .styles
        .iter()
        .filter_map(|style| safe_resource_url(style))
        .map(|style| {
            format!(
                r#"<link rel="stylesheet" href="{}">"#,
                escape_html(&style)
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {description}
    {meta_tags}
    {styles}
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }}
    </style>
</head>
<body>
    {content}
    {scripts}
</body>
</html>"#,
        title = escape_html(&app.name),
    )
}
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
pub fn init() {
    log::info!("WEBA module initialized");
}
