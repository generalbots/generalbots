use crate::types::{
    ElementContent, ElementStyle, PresentationTheme, Slide, SlideBackground,
    SlideElement, ThemeColors, ThemeFonts,
};
use uuid::Uuid;

struct TextElementParams {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    bold: bool,
}

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

fn create_text_element(
    text: &str,
    params: &TextElementParams,
    theme: &PresentationTheme,
) -> SlideElement {
    SlideElement {
        id: Uuid::new_v4().to_string(),
        element_type: "text".to_string(),
        x: params.x,
        y: params.y,
        width: params.width,
        height: params.height,
        rotation: 0.0,
        content: ElementContent {
            text: Some(text.to_string()),
            html: Some(format!("<p>{text}</p>")),
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
            font_size: Some(params.font_size),
            font_weight: if params.bold { Some("bold".to_string()) } else { None },
            font_style: None,
            text_align: Some("left".to_string()),
            vertical_align: Some("top".to_string()),
            color: Some(theme.colors.text.clone()),
            line_height: None,
            border_radius: None,
        },
        animations: vec![],
        z_index: 0,
        locked: false,
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
        transition_config: None,
        media: None,
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
        transition_config: None,
        media: None,
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
        transition_config: None,
        media: None,
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
        c
    } else {
        '_'
    }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

pub fn slides_from_markdown(md: &str) -> Vec<Slide> {
    let theme = create_default_theme();
    let mut slides = Vec::new();
    let sections: Vec<&str> = md.split("\n---\n").collect();

    for section in sections {
        let lines: Vec<&str> = section.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            continue;
        }

        let mut slide = create_blank_slide(&theme);
        let mut y_offset = 50.0;

        for line in lines {
            let trimmed = line.trim();
if let Some(stripped) = trimmed.strip_prefix("# ") {
    slide.elements.push(create_text_element(
        stripped, &TextElementParams { x: 50.0, y: y_offset, width: 860.0, height: 60.0, font_size: 44.0, bold: true }, &theme,
    ));
    y_offset += 80.0;
} else if let Some(stripped) = trimmed.strip_prefix("## ") {
    slide.elements.push(create_text_element(
        stripped, &TextElementParams { x: 50.0, y: y_offset, width: 860.0, height: 50.0, font_size: 32.0, bold: true }, &theme,
    ));
    y_offset += 60.0;
} else if let Some(stripped) = trimmed.strip_prefix("### ") {
    slide.elements.push(create_text_element(
        stripped, &TextElementParams { x: 50.0, y: y_offset, width: 860.0, height: 40.0, font_size: 24.0, bold: true }, &theme,
    ));
    y_offset += 50.0;
} else if trimmed.starts_with("![") {
                if let Some(start) = trimmed.find('(') {
                    if let Some(end) = trimmed.find(')') {
                        let src = &trimmed[start + 1..end];
                        slide.elements.push(SlideElement {
                            id: Uuid::new_v4().to_string(),
                            element_type: "image".to_string(),
                            x: 50.0,
                            y: y_offset,
                            width: 400.0,
                            height: 300.0,
                            rotation: 0.0,
                            content: ElementContent {
                                text: None,
                                html: None,
                                src: Some(src.to_string()),
                                shape_type: None,
                                chart_data: None,
                                table_data: None,
                            },
                            style: ElementStyle::default(),
                            animations: vec![],
                            z_index: slide.elements.len() as i32,
                            locked: false,
                        });
                        y_offset += 320.0;
                    }
                }
            } else if !trimmed.is_empty() {
slide.elements.push(create_text_element(
    trimmed, &TextElementParams { x: 50.0, y: y_offset, width: 860.0, height: 30.0, font_size: 18.0, bold: false }, &theme,
));
                y_offset += 40.0;
            }
        }

        slides.push(slide);
    }

    if slides.is_empty() {
        slides.push(create_title_slide(&theme));
    }

    slides
}
