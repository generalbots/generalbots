use crate::slides::types::{
    ElementContent, ElementStyle, PresentationTheme, Slide, SlideBackground, SlideElement,
    ThemeColors, ThemeFonts,
};
use uuid::Uuid;

pub fn create_default_theme() -> PresentationTheme {
    PresentationTheme {
        name: "Default".to_string(),
        colors: ThemeColors {
            primary: "#1a73e8".to_string(),
            secondary: "#34a853".to_string(),
            accent: "#ea4335".to_string(),
            background: "#ffffff".to_string(),
            text: "#202124".to_string(),
            text_light: "#5f6368".to_string(),
        },
        fonts: ThemeFonts {
            heading: "Arial".to_string(),
            body: "Arial".to_string(),
        },
    }
}

pub fn create_title_slide(theme: &PresentationTheme) -> Slide {
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "title".to_string(),
        elements: vec![
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 100.0,
                y: 200.0,
                width: 760.0,
                height: 100.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Presentation Title".to_string()),
                    html: Some("<h1>Presentation Title</h1>".to_string()),
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    fill: None,
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                    shadow: None,
                    font_family: Some(theme.fonts.heading.clone()),
                    font_size: Some(44.0),
                    font_weight: Some("bold".to_string()),
                    font_style: None,
                    text_align: Some("center".to_string()),
                    vertical_align: Some("middle".to_string()),
                    color: Some(theme.colors.text.clone()),
                    line_height: None,
                    border_radius: None,
                },
                animations: vec![],
                z_index: 1,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 100.0,
                y: 320.0,
                width: 760.0,
                height: 60.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Subtitle".to_string()),
                    html: Some("<p>Subtitle</p>".to_string()),
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    fill: None,
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                    shadow: None,
                    font_family: Some(theme.fonts.body.clone()),
                    font_size: Some(24.0),
                    font_weight: None,
                    font_style: None,
                    text_align: Some("center".to_string()),
                    vertical_align: Some("middle".to_string()),
                    color: Some(theme.colors.text_light.clone()),
                    line_height: None,
                    border_radius: None,
                },
                animations: vec![],
                z_index: 2,
                locked: false,
            },
        ],
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some(theme.colors.background.clone()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: None,
    }
}

pub fn create_content_slide(theme: &PresentationTheme) -> Slide {
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "content".to_string(),
        elements: vec![
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 40.0,
                width: 860.0,
                height: 60.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Slide Title".to_string()),
                    html: Some("<h2>Slide Title</h2>".to_string()),
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    fill: None,
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                    shadow: None,
                    font_family: Some(theme.fonts.heading.clone()),
                    font_size: Some(32.0),
                    font_weight: Some("bold".to_string()),
                    font_style: None,
                    text_align: Some("left".to_string()),
                    vertical_align: Some("middle".to_string()),
                    color: Some(theme.colors.text.clone()),
                    line_height: None,
                    border_radius: None,
                },
                animations: vec![],
                z_index: 1,
                locked: false,
            },
            SlideElement {
                id: Uuid::new_v4().to_string(),
                element_type: "text".to_string(),
                x: 50.0,
                y: 120.0,
                width: 860.0,
                height: 400.0,
                rotation: 0.0,
                content: ElementContent {
                    text: Some("Content goes here...".to_string()),
                    html: Some("<p>Content goes here...</p>".to_string()),
                    src: None,
                    shape_type: None,
                    chart_data: None,
                    table_data: None,
                },
                style: ElementStyle {
                    fill: None,
                    stroke: None,
                    stroke_width: None,
                    opacity: None,
                    shadow: None,
                    font_family: Some(theme.fonts.body.clone()),
                    font_size: Some(18.0),
                    font_weight: None,
                    font_style: None,
                    text_align: Some("left".to_string()),
                    vertical_align: Some("top".to_string()),
                    color: Some(theme.colors.text.clone()),
                    line_height: Some(1.5),
                    border_radius: None,
                },
                animations: vec![],
                z_index: 2,
                locked: false,
            },
        ],
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some(theme.colors.background.clone()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: None,
    }
}

pub fn create_blank_slide(theme: &PresentationTheme) -> Slide {
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "blank".to_string(),
        elements: vec![],
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some(theme.colors.background.clone()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: None,
    }
}

pub fn get_user_presentations_path(user_id: &str) -> String {
    format!("users/{}/presentations", user_id)
}

pub fn generate_presentation_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn export_to_html(presentation: &crate::slides::types::Presentation) -> String {
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
        let bg_color = slide
            .background
            .color
            .as_deref()
            .unwrap_or("#ffffff");
        html.push_str(&format!(
            r#"    <div class="slide" style="background-color: {};">
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
                r#"        <div class="element element-{}" style="{}">{}</div>
"#,
                element.element_type, style, content
            ));
        }

        html.push_str("    </div>\n");
    }

    html.push_str("</body>\n</html>");
    html
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else if c == ' ' {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
