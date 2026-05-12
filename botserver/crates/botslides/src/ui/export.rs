use crate::types::{Presentation, Slide};

pub fn export_to_html(presentation: &Presentation) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>"#,
    );
    html.push_str(&presentation.name);
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

            let content = element
                .content
                .html
                .as_deref()
                .or(element.content.text.as_deref())
                .unwrap_or("");

            html.push_str(&format!(
                r#" <div class="element element-{}" style="{}">{}</div>
"#,
                element.element_type, style, content
            ));
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
                            r#" <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>
"#,
                            element.x, element.y, element.x + element.width, element.y + element.height, stroke, stroke_width
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
            "image" => {
                if let Some(ref src) = element.content.src {
                    svg.push_str(&format!(
                        r#" <image x="{}" y="{}" width="{}" height="{}" href="{}"/>
"#,
                        element.x, element.y, element.width, element.height, src
                    ));
                }
            }
            _ => {}
        }
    }

    svg.push_str("</svg>");
    svg
}

fn xml_escape(s: &str) -> String {
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

pub fn export_to_odp_content(presentation: &Presentation) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0"
xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
office:version="1.2">
<office:body>
<office:presentation>
"#);

    for (idx, slide) in presentation.slides.iter().enumerate() {
        xml.push_str(&format!(
            "<draw:page draw:name=\"Slide{}\" draw:style-name=\"dp1\">\n",
            idx + 1
        ));

        for element in &slide.elements {
            match element.element_type.as_str() {
                "text" => {
                    let text = element.content.text.as_deref().unwrap_or("");
                    xml.push_str(&format!(
                        r#" <draw:frame svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt">
<draw:text-box>
<text:p>{}</text:p>
</draw:text-box>
</draw:frame>
"#,
                        element.x, element.y, element.width, element.height, xml_escape(text)
                    ));
                }
                "shape" => {
                    let shape_type = element.content.shape_type.as_deref().unwrap_or("rectangle");
                    let fill = element.style.fill.as_deref().unwrap_or("#cccccc");

                    match shape_type {
                        "rectangle" | "rect" => {
                            xml.push_str(&format!(
                                r#" <draw:rect svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt" draw:fill-color="{}"/>
"#,
                                element.x, element.y, element.width, element.height, fill
                            ));
                        }
                        "circle" | "ellipse" => {
                            xml.push_str(&format!(
                                r#" <draw:ellipse svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt" draw:fill-color="{}"/>
"#,
                                element.x, element.y, element.width, element.height, fill
                            ));
                        }
                        _ => {}
                    }
                }
                "image" => {
                    if let Some(ref src) = element.content.src {
                        xml.push_str(&format!(
                            r#" <draw:frame svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt">
<draw:image xlink:href="{}"/>
</draw:frame>
"#,
                            element.x, element.y, element.width, element.height, src
                        ));
                    }
                }
                _ => {}
            }
        }

        xml.push_str("</draw:page>\n");
    }

    xml.push_str("</office:presentation>\n</office:body>\n</office:document-content>");
    xml
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
                    md.push_str(&format!("![Image]({})\n\n", src));
                }
            }
        }

        if let Some(ref notes) = slide.notes {
            md.push_str(&format!("**Speaker Notes:**\n{}\n\n", notes));
        }
    }

    md
}
