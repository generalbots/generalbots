//! Slide layouts offered by the insert dialog (UAT SLIDE-02, #2031).
//!
//! `create_slide_with_layout` answered every layout it did not know with a
//! plain content slide, so choosing "Duas Colunas" in `+ → New Slide` produced
//! a single-column body: the two editable placeholders the dialog promised
//! never existed and the deck looked unchanged. The layouts that the toolbar
//! advertises live here so the handler and the dialog cannot drift apart.

use crate::types::{
    ElementContent, ElementStyle, PresentationTheme, Slide, SlideBackground, SlideElement,
};
use uuid::Uuid;

/// Layout names the insert dialog and the API accept, in dialog order.
pub const LAYOUTS: [&str; 6] = [
    "title",
    "content",
    "two_column",
    "section",
    "image",
    "blank",
];

fn solid_background(theme: &PresentationTheme) -> SlideBackground {
    SlideBackground {
        bg_type: "solid".to_string(),
        color: Some(theme.colors.background.clone()),
        gradient: None,
        image_url: None,
        image_fit: None,
    }
}

struct TextBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    font_size: f64,
    bold: bool,
    align: &'static str,
    /// `None` keeps the theme text colour.
    color: Option<String>,
}

fn text_element(
    text: &str,
    index: usize,
    font: &str,
    box_spec: TextBox,
    theme: &PresentationTheme,
) -> SlideElement {
    SlideElement {
        id: Uuid::new_v4().to_string(),
        element_type: "text".to_string(),
        x: box_spec.x,
        y: box_spec.y,
        width: box_spec.width,
        height: box_spec.height,
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
            font_family: Some(font.to_string()),
            font_size: Some(box_spec.font_size),
            font_weight: if box_spec.bold {
                Some("bold".to_string())
            } else {
                None
            },
            font_style: None,
            text_align: Some(box_spec.align.to_string()),
            vertical_align: Some("top".to_string()),
            color: Some(box_spec.color.unwrap_or_else(|| theme.colors.text.clone())),
            line_height: Some(1.4),
            border_radius: None,
        },
        animations: vec![],
        z_index: index as i32 + 1,
        locked: false,
    }
}

/// Two-column slide: a title plus a left and a right placeholder body, both
/// editable. This is the layout UAT SLIDE-02 asks for ("insert a new slide
/// after slide 3 with a two-column layout").
pub fn create_two_column_slide(theme: &PresentationTheme) -> Slide {
    let heading = theme.fonts.heading.clone();
    let body = theme.fonts.body.clone();
    let elements = vec![
        text_element(
            "Slide Title",
            0,
            &heading,
            TextBox {
                x: 50.0,
                y: 40.0,
                width: 860.0,
                height: 60.0,
                font_size: 32.0,
                bold: true,
                align: "left",
                color: None,
            },
            theme,
        ),
        text_element(
            "Left column",
            1,
            &body,
            TextBox {
                x: 50.0,
                y: 130.0,
                width: 410.0,
                height: 380.0,
                font_size: 18.0,
                bold: false,
                align: "left",
                color: None,
            },
            theme,
        ),
        text_element(
            "Right column",
            2,
            &body,
            TextBox {
                x: 500.0,
                y: 130.0,
                width: 410.0,
                height: 380.0,
                font_size: 18.0,
                bold: false,
                align: "left",
                color: None,
            },
            theme,
        ),
    ];
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "two_column".to_string(),
        elements,
        background: solid_background(theme),
        notes: None,
        transition: None,
        transition_config: None,
        media: None,
    }
}

/// Section divider: a centred heading and a subtitle, no body.
pub fn create_section_slide(theme: &PresentationTheme) -> Slide {
    let heading = theme.fonts.heading.clone();
    let body = theme.fonts.body.clone();
    let elements = vec![
        text_element(
            "Section title",
            0,
            &heading,
            TextBox {
                x: 100.0,
                y: 240.0,
                width: 760.0,
                height: 90.0,
                font_size: 40.0,
                bold: true,
                align: "center",
                color: None,
            },
            theme,
        ),
        text_element(
            "Section subtitle",
            1,
            &body,
            TextBox {
                x: 100.0,
                y: 340.0,
                width: 760.0,
                height: 60.0,
                font_size: 20.0,
                bold: false,
                align: "center",
                color: Some(theme.colors.text_light.clone()),
            },
            theme,
        ),
    ];
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "section".to_string(),
        elements,
        background: solid_background(theme),
        notes: None,
        transition: None,
        transition_config: None,
        media: None,
    }
}

/// Image-forward slide: a title plus a framed placeholder the user replaces
/// with a picture (the media insert form fills `slide.media`).
pub fn create_image_slide(theme: &PresentationTheme) -> Slide {
    let heading = theme.fonts.heading.clone();
    let body = theme.fonts.body.clone();
    let mut placeholder = text_element(
        "Drop an image here",
        1,
        &body,
        TextBox {
            x: 130.0,
            y: 130.0,
            width: 700.0,
            height: 380.0,
            font_size: 18.0,
            bold: false,
            align: "center",
            color: Some(theme.colors.text_light.clone()),
        },
        theme,
    );
    placeholder.style.fill = Some("#f1f5f9".to_string());
    placeholder.style.stroke = Some(theme.colors.primary.clone());
    placeholder.style.stroke_width = Some(2.0);
    placeholder.style.border_radius = Some(8.0);
    let elements = vec![
        text_element(
            "Slide Title",
            0,
            &heading,
            TextBox {
                x: 50.0,
                y: 40.0,
                width: 860.0,
                height: 60.0,
                font_size: 32.0,
                bold: true,
                align: "left",
                color: None,
            },
            theme,
        ),
        placeholder,
    ];
    Slide {
        id: Uuid::new_v4().to_string(),
        layout: "image".to_string(),
        elements,
        background: solid_background(theme),
        notes: None,
        transition: None,
        transition_config: None,
        media: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_default_theme;

    #[test]
    fn test_two_column_has_two_body_placeholders() {
        let slide = create_two_column_slide(&create_default_theme());
        assert_eq!(slide.layout, "two_column");
        assert_eq!(slide.elements.len(), 3);
        let bodies: Vec<&SlideElement> = slide.elements.iter().skip(1).collect();
        assert!(bodies[0].x < bodies[1].x, "left placeholder must be left");
        assert_eq!(bodies[0].width, bodies[1].width);
        for element in bodies {
            assert_eq!(element.element_type, "text");
            assert!(!element.content.text.as_deref().unwrap_or("").is_empty());
        }
    }

    #[test]
    fn test_section_and_image_layouts_are_named() {
        let theme = create_default_theme();
        assert_eq!(create_section_slide(&theme).layout, "section");
        assert_eq!(create_image_slide(&theme).layout, "image");
    }

    #[test]
    fn test_every_advertised_layout_is_built() {
        let theme = create_default_theme();
        for layout in LAYOUTS {
            let slide = crate::storage::create_slide_with_layout(layout, &theme);
            assert_eq!(
                slide.layout, layout,
                "layout '{layout}' silently fell back to another layout"
            );
        }
    }
}
