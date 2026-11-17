use crate::types::{
    ElementContent, ElementStyle, PresentationTheme, Slide, SlideBackground, SlideElement,
    ThemeColors, ThemeFonts,
};
use uuid::Uuid;

pub(crate) struct TextElementParams {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) font_size: f64,
    pub(crate) bold: bool,
}

/// Resolves a theme-picker preset id to a full theme. The picker posts only
/// the preset name, while the API expects a theme object.
pub fn theme_from_name(name: &str) -> PresentationTheme {
    let (primary, secondary, background, text) = match name {
        "corporate" => ("#1e40af", "#0891b2", "#f1f5f9", "#0f172a"),
        "creative" => ("#ec4899", "#8b5cf6", "#fff7ed", "#1f2937"),
        "minimal" => ("#475569", "#94a3b8", "#fafafa", "#111827"),
        "dark" => ("#0f172a", "#3b82f6", "#0f172a", "#f8fafc"),
        "warm" => ("#dc2626", "#f59e0b", "#fef3c7", "#7c2d12"),
        "nature" => ("#16a34a", "#65a30d", "#f0fdf4", "#14532d"),
        "ocean" => ("#0369a1", "#0ea5e9", "#e0f2fe", "#0c4a6e"),
        "sunset" => ("#ea580c", "#facc15", "#fffbeb", "#7c2d12"),
        "midnight" => ("#1e1b4b", "#6366f1", "#e0e7ff", "#1e1b4b"),
        _ => ("#1a73e8", "#34a853", "#ffffff", "#202124"),
    };
    PresentationTheme {
        name: if name.is_empty() {
            "Default".to_string()
        } else {
            name.to_string()
        },
        colors: ThemeColors {
            primary: primary.to_string(),
            secondary: secondary.to_string(),
            accent: "#ea4335".to_string(),
            background: background.to_string(),
            text: text.to_string(),
            text_light: "#5f6368".to_string(),
        },
        fonts: ThemeFonts {
            heading: "Arial".to_string(),
            body: "Arial".to_string(),
        },
    }
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

pub(crate) fn create_text_element(
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
            font_weight: if params.bold {
                Some("bold".to_string())
            } else {
                None
            },
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

mod builders;
mod markdown;

pub use builders::{create_blank_slide, create_content_slide, create_title_slide};
pub use markdown::slides_from_markdown;

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

#[cfg(test)]
mod tests;
