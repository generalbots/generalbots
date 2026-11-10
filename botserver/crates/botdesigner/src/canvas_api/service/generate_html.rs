use super::*;

/// Escapes text for HTML text and quoted attribute positions.
///
/// Mirrors `botweba::escape_html` (#2124). It stays local because `botweba` is
/// an optional crate behind botserver's `weba` feature: making it mandatory
/// just for this pure function would pull the whole WebA stack into every
/// build.
fn escape_html(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for character in raw.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Wraps the rendered SVG in the standalone HTML export document.
///
/// `canvas.name` is user data: unescaped, a canvas named `</title><script>…`
/// closed the head and ran as the suite user — the stored-XSS class fixed in
/// WebA (#2124). `svg` stays markup because `generate_svg` produced it, not
/// the user.
fn html_document(canvas_name: &str, svg: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0"><title>{}</title>
<style>body{{margin:0;padding:0;display:flex;justify-content:center;align-items:center;min-height:100vh;background:#f0f0f0}}
.canvas-container{{background:white;box-shadow:0 4px 20px rgba(0,0,0,0.1)}}</style></head>
<body><div class="canvas-container">{}</div></body></html>"#,
        escape_html(canvas_name),
        svg
    )
}

impl CanvasService {
    pub(super) fn generate_html(
        &self,
        canvas: &Canvas,
        request: &ExportRequest,
    ) -> Result<String, CanvasError> {
        let svg = self.generate_svg(canvas, request)?;

        Ok(html_document(&canvas.name, &svg))
    }

    pub(crate) async fn save_canvas_elements(
        &self,
        canvas_id: Uuid,
        elements: &[CanvasElement],
    ) -> Result<(), CanvasError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|_| CanvasError::DatabaseConnection)?;

        let elements_json = serde_json::to_string(elements).unwrap_or_else(|_| "[]".to_string());

        diesel::sql_query(
            "UPDATE designer_canvases SET elements_json = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind::<Text, _>(&elements_json)
        .bind::<DieselUuid, _>(canvas_id)
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("Failed to save elements: {e}");
            CanvasError::UpdateFailed
        })?;

        Ok(())
    }

    pub(crate) async fn save_canvas_layers(
        &self,
        canvas_id: Uuid,
        layers: &[Layer],
    ) -> Result<(), CanvasError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|_| CanvasError::DatabaseConnection)?;

        let layers_json = serde_json::to_string(layers).unwrap_or_else(|_| "[]".to_string());

        diesel::sql_query(
            "UPDATE designer_canvases SET layers_json = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind::<Text, _>(&layers_json)
        .bind::<DieselUuid, _>(canvas_id)
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("Failed to save layers: {e}");
            CanvasError::UpdateFailed
        })?;

        Ok(())
    }

    pub(super) fn broadcast_event(
        &self,
        event_type: CanvasEventType,
        canvas_id: Uuid,
        user_id: Uuid,
        data: serde_json::Value,
    ) {
        let event = CanvasEvent {
            event_type,
            canvas_id,
            user_id,
            data,
            timestamp: Utc::now(),
        };
        let _ = self.event_sender.send(event);
    }

    pub async fn get_templates(
        &self,
        category: Option<String>,
    ) -> Result<Vec<CanvasTemplate>, CanvasError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|_| CanvasError::DatabaseConnection)?;

        let sql = match category {
            Some(ref cat) => format!(
                "SELECT id, name, description, category, thumbnail_url, canvas_data, is_system, created_by, created_at FROM designer_templates WHERE category = '{}' ORDER BY name",
                cat
            ),
            None => "SELECT id, name, description, category, thumbnail_url, canvas_data, is_system, created_by, created_at FROM designer_templates ORDER BY category, name".to_string(),
        };

        let rows: Vec<TemplateRow> = diesel::sql_query(&sql)
            .load(&mut conn)
            .unwrap_or_default();

        let templates = rows
            .into_iter()
            .map(|row| CanvasTemplate {
                id: row.id,
                name: row.name,
                description: row.description,
                category: row.category,
                thumbnail_url: row.thumbnail_url,
                canvas_data: serde_json::from_str(&row.canvas_data)
                    .unwrap_or(serde_json::json!({})),
                is_system: row.is_system,
                created_by: row.created_by,
                created_at: row.created_at,
            })
            .collect();

        Ok(templates)
    }

    pub async fn get_asset_library(
        &self,
        asset_type: Option<AssetType>,
    ) -> Result<Vec<AssetLibraryItem>, CanvasError> {
        let icons = vec![
            AssetLibraryItem { id: Uuid::new_v4(), name: "Bot".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["bot".to_string(), "assistant".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Analytics".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["analytics".to_string(), "chart".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Calendar".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["calendar".to_string(), "date".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Chat".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["chat".to_string(), "message".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Drive".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["drive".to_string(), "files".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Mail".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["mail".to_string(), "email".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Meet".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["meet".to_string(), "video".to_string()], is_system: true },
            AssetLibraryItem { id: Uuid::new_v4(), name: "Tasks".to_string(), asset_type: AssetType::Icon, url: None, svg_content: None, category: "General Bots".to_string(), tags: vec!["tasks".to_string(), "todo".to_string()], is_system: true },
        ];

        let filtered = match asset_type {
            Some(t) => icons.into_iter().filter(|i| i.asset_type == t).collect(),
            None => icons,
        };

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The payload from #2124: closing `<title>` is enough to run script.
    const TITLE_BREAKOUT: &str = "</title><script>alert(1)</script>";

    #[test]
    fn canvas_name_is_escaped_in_the_title() {
        let html = html_document(TITLE_BREAKOUT, "<svg></svg>");

        assert!(html.contains("&lt;/title&gt;&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
        // Exactly one real `</title>`: the escaped one cannot close the head.
        assert_eq!(html.matches("</title>").count(), 1);
    }

    #[test]
    fn plain_canvas_name_keeps_its_text_and_escapes_entities() {
        let html = html_document("Q3 & Q4 \"roadmap\"", "<svg></svg>");

        assert!(html.contains("<title>Q3 &amp; Q4 &quot;roadmap&quot;</title>"));
        assert!(html.contains(r#"<div class="canvas-container"><svg></svg></div>"#));
    }

    #[test]
    fn escape_html_covers_the_five_markup_characters() {
        assert_eq!(escape_html("<&>\"'"), "&lt;&amp;&gt;&quot;&#x27;");
        assert_eq!(escape_html("plain"), "plain");
    }
}
