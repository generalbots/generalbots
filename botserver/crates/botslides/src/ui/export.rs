use crate::types::{Presentation, Slide, TableData};


fn table_html(data: &TableData) -> String {
    let mut html = String::from(
        r#"<table style="border-collapse:collapse;width:100%;height:100%;font-size:12px;color:#0f172a;background:#fff;">"#,
    );
    for (row_index, row) in data.cells.iter().enumerate() {
        html.push_str("<tr>");
        for cell in row {
            let tag = if row_index == 0 { "th" } else { "td" };
            html.push_str(&format!(
                r#"<{tag} style="border:1px solid #cbd5e1;padding:3px 6px;text-align:left;">{}</{tag}>"#,
                xml_escape(&cell.content)
            ));
        }
        html.push_str("</tr>");
    }
    html.push_str("</table>");
    html
}

pub fn export_to_html(presentation: &Presentation) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>"#,
    );
    html.push_str(&xml_escape(&presentation.name));
    html.push_str(
        r#"</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: Arial, sans-serif; background: #000; }
.slide {
width: 960px;
height: 540px;
margin: 20px auto;
position: relative;
overflow: hidden;
box-shadow: 0 4px 20px rgba(0,0,0,0.3);
}
.element { position: absolute; }
.element-text { white-space: pre-wrap; }
</style>
</head>
<body>
"#,
    );

    for slide in &presentation.slides {
        let bg_color = slide.background.color.as_deref().unwrap_or("#ffffff");
        html.push_str(&format!(
            r#" <div class="slide" style="background-color: {};">
"#,
            bg_color
        ));

        for element in &slide.elements {
            let style = format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                element.x, element.y, element.width, element.height
            );

            let content = match element.element_type.as_str() {
                "table" => element
                    .content
                    .table_data
                    .as_ref()
                    .map(table_html)
                    .or_else(|| element.content.html.clone())
                    .or_else(|| element.content.text.clone())
                    .unwrap_or_default(),
                "chart" => element
                    .content
                    .chart_data
                    .as_ref()
                    .map(|data| crate::ui_fragments::sidebars::render_chart_html(element.content.text.as_deref().unwrap_or(""), data))
                    .or_else(|| element.content.html.clone())
                    .or_else(|| element.content.text.clone())
                    .unwrap_or_default(),
                _ => element
                    .content
                    .html
                    .as_deref()
                    .or(element.content.text.as_deref())
                    .unwrap_or("")
                    .to_string(),
            };

            html.push_str(&format!(
                r#" <div class="element element-{}" style="{}">{}</div>
"#,
                element.element_type, style, content
            ));
        }

        if let Some(media) = &slide.media {
            for item in media {
                if item.media_type != "image" {
                    continue;
                }
                html.push_str(&format!(
                    r#" <img class="element" src="{}" alt="" style="left:{}px;top:{}px;width:{}px;height:{}px;object-fit:contain;"/>
"#,
                    xml_escape(&item.src), item.x, item.y, item.width, item.height
                ));
            }
        }

        html.push_str(" </div>\n");
    }

    html.push_str("</body>\n</html>");
    html
}

pub fn export_to_svg(slide: &Slide, width: u32, height: u32) -> String {
    let mut svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        width, height, width, height
    );

    let bg_color = slide.background.color.as_deref().unwrap_or("#ffffff");
    svg.push_str(&format!(
        r#" <rect width="100%" height="100%" fill="{}"/>
"#,
        bg_color
    ));

    for element in &slide.elements {
        match element.element_type.as_str() {
            "text" => {
                let text = element.content.text.as_deref().unwrap_or("");
                let font_size = element.style.font_size.unwrap_or(18.0);
                let color = element.style.color.as_deref().unwrap_or("#000000");
                let font_family = element.style.font_family.as_deref().unwrap_or("Arial");
                let font_weight = element.style.font_weight.as_deref().unwrap_or("normal");

                svg.push_str(&format!(
                    r#" <text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" fill="{}">{}</text>
"#,
                    element.x,
                    element.y + font_size,
                    font_family,
                    font_size,
                    font_weight,
                    color,
                    xml_escape(text)
                ));
            }
            "shape" => {
                let shape_type = element.content.shape_type.as_deref().unwrap_or("rectangle");
                let fill = element.style.fill.as_deref().unwrap_or("#cccccc");
                let stroke = element.style.stroke.as_deref().unwrap_or("none");
                let stroke_width = element.style.stroke_width.unwrap_or(1.0);

                match shape_type {
                    "rectangle" | "rect" => {
                        let rx = element.style.border_radius.unwrap_or(0.0);
                        svg.push_str(&format!(
                            r#" <rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
                            element.x, element.y, element.width, element.height, rx, fill, stroke, stroke_width
                        ));
                    }
                    "circle" | "ellipse" => {
                        let cx = element.x + element.width / 2.0;
                        let cy = element.y + element.height / 2.0;
                        let rx = element.width / 2.0;
                        let ry = element.height / 2.0;
                        svg.push_str(&format!(
                            r#" <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
                            cx, cy, rx, ry, fill, stroke, stroke_width
                        ));
                    }
                    "line" => {
                        svg.push_str(&format!(
                            r#" <line x1="{}" y1="{}" x2="{}" y2="{}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
                            element.x, element.y, element.x + element.width, element.y + element.height, fill, stroke, stroke_width
                        ));
                    }
                    "triangle" => {
                        let x1 = element.x + element.width / 2.0;
                        let y1 = element.y;
                        let x2 = element.x;
                        let y2 = element.y + element.height;
                        let x3 = element.x + element.width;
                        let y3 = element.y + element.height;
                        svg.push_str(&format!(
                            r#" <polygon points="{},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
                            x1, y1, x2, y2, x3, y3, fill, stroke, stroke_width
                        ));
                    }
                    _ => {}
                }
            }
            "table" => {
                if let Some(data) = &element.content.table_data {
                    let mut line_y = element.y + 16.0;
                    for row in data.cells.iter().take(24) {
                        let line = row
                            .iter()
                            .map(|cell| cell.content.clone())
                            .collect::<Vec<_>>()
                            .join(" | ");
                        svg.push_str(&format!(
                            r##" <text x="{}" y="{}" font-family="Arial" font-size="12" fill="#0f172a">{}</text>
"##,
                            element.x,
                            line_y,
                            xml_escape(&line)
                        ));
                        line_y += 16.0;
                    }
                }
            }
            "chart" => {
                if let Some(data) = &element.content.chart_data {
                    let title = element.content.text.as_deref().unwrap_or("");
                    svg.push_str(&format!(
                        r##" <text x="{}" y="{}" font-family="Arial" font-size="14" font-weight="bold" fill="#0f172a">{}</text>
"##,
                        element.x,
                        element.y + 16.0,
                        xml_escape(title)
                    ));
                    let values = data
                        .datasets
                        .first()
                        .map(|dataset| dataset.data.clone())
                        .unwrap_or_default();
                    let mut line_y = element.y + 36.0;
                    for (index, value) in values.iter().enumerate().take(12) {
                        let label = data.labels.get(index).cloned().unwrap_or_default();
                        svg.push_str(&format!(
                            r##" <text x="{}" y="{}" font-family="Arial" font-size="12" fill="#334155">{}: {}</text>
"##,
                            element.x,
                            line_y,
                            xml_escape(&label),
                            value
                        ));
                        line_y += 16.0;
                    }
                }
            }
            "image" => {
                if let Some(ref src) = element.content.src {
                    svg.push_str(&format!(
                        r#" <image x="{}" y="{}" width="{}" height="{}" href="{}"/>
"#,
                        element.x, element.y, element.width, element.height, xml_escape(src)
                    ));
                }
            }
            _ => {}
        }
    }

    if let Some(media) = &slide.media {
        for item in media {
            if item.media_type == "image" {
                svg.push_str(&format!(
                    r#" <image x="{}" y="{}" width="{}" height="{}" href="{}"/>
"#,
                    item.x, item.y, item.width, item.height, xml_escape(&item.src)
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

pub(super) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn export_slide_to_png_placeholder(slide: &Slide, width: u32, height: u32) -> Vec<u8> {
    let svg = export_to_svg(slide, width, height);
    svg.into_bytes()
}

pub fn export_to_json(presentation: &Presentation) -> String {
    serde_json::to_string_pretty(presentation).unwrap_or_default()
}

pub fn export_to_markdown(presentation: &Presentation) -> String {
    let mut md = format!("# {}\n\n", presentation.name);

    for (idx, slide) in presentation.slides.iter().enumerate() {
        md.push_str(&format!("---\n\n## Slide {}\n\n", idx + 1));

        for element in &slide.elements {
            if element.element_type == "text" {
                if let Some(ref text) = element.content.text {
                    let font_size = element.style.font_size.unwrap_or(18.0);
                    if font_size >= 32.0 {
                        md.push_str(&format!("### {}\n\n", text));
                    } else {
                        md.push_str(&format!("{}\n\n", text));
                    }
                }
            } else if element.element_type == "image" {
                if let Some(ref src) = element.content.src {
                    if !src.is_empty() {
                        md.push_str(&format!("![Image]({})\n\n", src));
                    }
                }
            }
        }

        if let Some(ref notes) = slide.notes {
            md.push_str(&format!("**Speaker Notes:**\n{}\n\n", notes));
        }
    }

    md
}


#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;
