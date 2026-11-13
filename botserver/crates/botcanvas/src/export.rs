use super::*;

pub(crate) async fn export_canvas(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    let db_canvas: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Canvas not found".to_string()))?;

    let db_elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_default();

    let elements: Vec<CanvasElement> = db_elements.into_iter().map(db_to_canvas_element).collect();
    let canvas = db_to_canvas(db_canvas, elements);

    match req.format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&canvas).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("JSON error: {e}"),
                )
            })?;
            Ok(Json(ExportResponse {
                format: ExportFormat::Json,
                url: None,
                data: Some(json),
            }))
        }
        ExportFormat::Svg => {
            let svg = generate_svg(&canvas, req.background.unwrap_or(true));
            Ok(Json(ExportResponse {
                format: ExportFormat::Svg,
                url: None,
                data: Some(svg),
            }))
        }
        _ => Ok(Json(ExportResponse {
            format: req.format,
            url: Some(format!("/api/canvas/{canvas_id}/export/file")),
            data: None,
        })),
    }
}

pub(crate) fn generate_svg(canvas: &Canvas, include_background: bool) -> String {
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"##,
        canvas.width, canvas.height, canvas.width, canvas.height
    );

    if include_background {
        svg.push_str(&format!(
            r##"<rect width="100%" height="100%" fill="{}"/>"##,
            svg_escape(&canvas.background_color)
        ));
    }

    for element in &canvas.elements {
        let transform = if element.rotation != 0.0 {
            format!(
                r##" transform="rotate({} {} {})""##,
                element.rotation,
                element.x + element.width / 2.0,
                element.y + element.height / 2.0
            )
        } else {
            String::new()
        };

        let fill = svg_escape(
            element
                .properties
                .fill_color
                .as_deref()
                .unwrap_or("transparent"),
        );
        let stroke = svg_escape(element.properties.stroke_color.as_deref().unwrap_or("none"));
        let stroke_width = element.properties.stroke_width.unwrap_or(1.0);
        let opacity = element.properties.opacity.unwrap_or(1.0);

        match element.element_type {
            ElementType::Rectangle => {
                let radius = element.properties.corner_radius.unwrap_or(0.0);
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                    element.x, element.y, element.width, element.height,
                    radius, fill, stroke, stroke_width, opacity, transform
                ));
            }
            ElementType::Ellipse => {
                svg.push_str(&format!(
                    r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                    element.x + element.width / 2.0,
                    element.y + element.height / 2.0,
                    element.width / 2.0,
                    element.height / 2.0,
                    fill, stroke, stroke_width, opacity, transform
                ));
            }
            ElementType::Text => {
                let text = svg_escape(element.properties.text.as_deref().unwrap_or(""));
                let font_size = element.properties.font_size.unwrap_or(16.0);
                let font_family = svg_escape(
                    element
                        .properties
                        .font_family
                        .as_deref()
                        .unwrap_or("sans-serif"),
                );
                svg.push_str(&format!(
                    r##"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{}" opacity="{}"{}>{}</text>"##,
                    element.x, element.y + font_size, font_size, font_family,
                    fill, opacity, transform, text
                ));
            }
            ElementType::FreehandPath => {
                if let Some(path_data) = &element.properties.path_data {
                    svg.push_str(&format!(
                        r##"<path d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                        svg_escape(path_data), stroke, stroke_width, opacity, transform
                    ));
                }
            }
            ElementType::Line | ElementType::Arrow => {
                let marker = if element.element_type == ElementType::Arrow {
                    r##" marker-end="url(#arrowhead)""##
                } else {
                    ""
                };
                svg.push_str(&format!(
                    r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}" opacity="{}"{}{}/>"##,
                    element.x, element.y,
                    element.x + element.width, element.y + element.height,
                    stroke, stroke_width, opacity, marker, transform
                ));
            }
            ElementType::Image => {
                if let Some(url) = &element.properties.image_url {
                    svg.push_str(&format!(
                        r##"<image x="{}" y="{}" width="{}" height="{}" href="{}" opacity="{}"{}/>"##,
                        element.x,
                        element.y,
                        element.width,
                        element.height,
                        svg_escape(url),
                        opacity,
                        transform
                    ));
                }
            }
            ElementType::Sticky => {
                let radius = element.properties.corner_radius.unwrap_or(2.0);
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                    element.x, element.y, element.width, element.height,
                    radius, fill, stroke, stroke_width, opacity, transform
                ));
                let text = svg_escape(element.properties.text.as_deref().unwrap_or(""));
                if !text.is_empty() {
                    let font_size = element.properties.font_size.unwrap_or(14.0);
                    svg.push_str(&format!(
                        r##"<text x="{}" y="{}" font-size="{}" fill="#1f2937">{}</text>"##,
                        element.x + 8.0,
                        element.y + 8.0 + font_size,
                        font_size,
                        text
                    ));
                }
            }
            ElementType::Frame | ElementType::Connector => {
                // A frame is an outlined region; a connector joins two points.
                svg.push_str(&format!(
                    r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}/>"##,
                    element.x, element.y, element.width, element.height,
                    stroke, stroke_width, opacity, transform
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Escapes the five XML markup characters so an `&` in a value cannot break
/// the document.
fn svg_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
