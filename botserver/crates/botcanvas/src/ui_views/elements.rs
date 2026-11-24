use super::*;

pub async fn canvas_elements_svg(
    State(state): State<Arc<CanvasState>>,
    Path(canvas_id): Path<Uuid>,
) -> Html<String> {
    let Ok(mut conn) = state.pool.get() else {
        return Html(String::new());
    };

    let elements: Vec<DbCanvasElement> = canvas_elements::table
        .filter(canvas_elements::canvas_id.eq(canvas_id))
        .order(canvas_elements::z_index.asc())
        .load(&mut conn)
        .unwrap_or_else(|e| {
            log::warn!("canvas svg: failed to load elements: {e}");
            Vec::new()
        });

    let mut svg_elements = String::new();
    for el in &elements {
        let svg = render_element_svg(el);
        svg_elements.push_str(&svg);
    }

    Html(svg_elements)
}

pub(crate) fn render_element_svg(element: &DbCanvasElement) -> String {
    let id = element.id;
    let x = element.x;
    let y = element.y;
    let width = element.width;
    let height = element.height;
    let rotation = element.rotation;

    let transform = if rotation != 0.0 {
        format!(
            r##" transform="rotate({rotation} {} {})""##,
            x + width / 2.0,
            y + height / 2.0
        )
    } else {
        String::new()
    };

    let fill = element
        .properties
        .get("fill_color")
        .and_then(|v| v.as_str())
        .unwrap_or("transparent");
    let stroke = element
        .properties
        .get("stroke_color")
        .and_then(|v| v.as_str())
        .unwrap_or("#000000");
    let stroke_width = element
        .properties
        .get("stroke_width")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0);
    let opacity = element
        .properties
        .get("opacity")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    match element.element_type.as_str() {
        "rectangle" => {
            let corner_radius = element
                .properties
                .get("corner_radius")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" rx="{corner_radius}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "ellipse" => {
            let cx = x + width / 2.0;
            let cy = y + height / 2.0;
            let rx = width / 2.0;
            let ry = height / 2.0;
            format!(
                r##"<ellipse data-element-id="{id}" cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "line" | "arrow" => {
            let x2 = x + width;
            let y2 = y + height;
            let marker = if element.element_type == "arrow" {
                r##" marker-end="url(#arrowhead)""##
            } else {
                ""
            };
            format!(
                r##"<line data-element-id="{id}" x1="{x}" y1="{y}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{marker}{transform} class="canvas-element"/>"##
            )
        }
        "text" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let font_size = element
                .properties
                .get("font_size")
                .and_then(|v| v.as_f64())
                .unwrap_or(16.0);
            let font_family = element
                .properties
                .get("font_family")
                .and_then(|v| v.as_str())
                .unwrap_or("sans-serif");
            let text_y = y + font_size;
            format!(
                r##"<text data-element-id="{id}" x="{x}" y="{text_y}" font-size="{font_size}" font-family="{font_family}" fill="{fill}" opacity="{opacity}"{transform} class="canvas-element">{text}</text>"##,
                text = html_escape(text)
            )
        }
        "freehand_path" => {
            let path_data = element
                .properties
                .get("path_data")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!(
                r##"<path data-element-id="{id}" d="{path_data}" fill="none" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
            )
        }
        "sticky" => {
            let text = element
                .properties
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let bg = element
                .properties
                .get("fill_color")
                .and_then(|v| v.as_str())
                .unwrap_or("#ffeb3b");
            format!(
                r##"<g data-element-id="{id}" class="canvas-element sticky-note"{transform}>
<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{bg}" stroke="#000" stroke-width="1"/>
<text x="{text_x}" y="{text_y}" font-size="14" fill="#000">{text}</text>
</g>"##,
                text_x = x + 8.0,
                text_y = y + 24.0,
                text = html_escape(text)
            )
        }
        _ => format!(
            r##"<rect data-element-id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}" opacity="{opacity}"{transform} class="canvas-element"/>"##
        ),
    }
}
