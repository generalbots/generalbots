//! SVG export expectations.

use super::*;
use crate::models::{Canvas, CanvasElement, ElementProperties, ElementType};
use chrono::Utc;
use uuid::Uuid;

fn element(kind: ElementType, properties: ElementProperties) -> CanvasElement {
    CanvasElement {
        id: Uuid::nil(),
        canvas_id: Uuid::nil(),
        element_type: kind,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        rotation: 0.0,
        z_index: 0,
        locked: false,
        properties,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn canvas(elements: Vec<CanvasElement>) -> Canvas {
    Canvas {
        id: Uuid::nil(),
        org_id: Uuid::nil(),
        name: "Board".to_string(),
        description: None,
        width: 800,
        height: 600,
        background_color: "#ffffff".to_string(),
        thumbnail_url: None,
        is_public: false,
        is_template: false,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        elements,
    }
}

fn with(kind: ElementType, properties: ElementProperties) -> String {
    generate_svg(&canvas(vec![element(kind, properties)]), true)
}

#[test]
fn document_root_carries_the_canvas_size() {
    let svg = generate_svg(&canvas(vec![]), true);
    assert!(svg.starts_with(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600" viewBox="0 0 800 600">"##
    ));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##));
    assert!(!generate_svg(&canvas(vec![]), false).contains("width=\"100%\""));
}

#[test]
fn rectangle_uses_corner_radius_and_defaults() {
    let svg = with(ElementType::Rectangle, ElementProperties::default());
    assert!(svg.contains(
        r##"<rect x="10" y="20" width="100" height="50" rx="0" fill="transparent" stroke="none" stroke-width="1" opacity="1"/>"##
    ));

    let rounded = with(
        ElementType::Rectangle,
        ElementProperties {
            corner_radius: Some(8.0),
            fill_color: Some("#ff0000".to_string()),
            stroke_color: Some("#000000".to_string()),
            stroke_width: Some(2.0),
            opacity: Some(0.5),
            ..ElementProperties::default()
        },
    );
    assert!(rounded.contains(
        r##"<rect x="10" y="20" width="100" height="50" rx="8" fill="#ff0000" stroke="#000000" stroke-width="2" opacity="0.5"/>"##
    ));
}

#[test]
fn ellipse_is_centred_on_the_element_box() {
    let svg = with(ElementType::Ellipse, ElementProperties::default());
    assert!(svg.contains(
        r##"<ellipse cx="60" cy="45" rx="50" ry="25" fill="transparent" stroke="none" stroke-width="1" opacity="1"/>"##
    ));
}

#[test]
fn text_sits_below_the_element_origin_and_uses_font_attributes() {
    let svg = with(
        ElementType::Text,
        ElementProperties {
            text: Some("Hello".to_string()),
            font_size: Some(24.0),
            font_family: Some("Georgia".to_string()),
            fill_color: Some("#333333".to_string()),
            ..ElementProperties::default()
        },
    );
    assert!(svg.contains(
        r##"<text x="10" y="44" font-size="24" font-family="Georgia" fill="#333333" opacity="1">Hello</text>"##
    ));
}

#[test]
fn rotation_adds_a_transform_around_the_centre() {
    let mut rotated = element(ElementType::Rectangle, ElementProperties::default());
    rotated.rotation = 90.0;
    let svg = generate_svg(&canvas(vec![rotated]), true);
    assert!(svg.contains(r##" transform="rotate(90 60 45)""##));
}

#[test]
fn lines_and_arrows_join_the_box_corners() {
    let line = with(ElementType::Line, ElementProperties::default());
    assert!(line.contains(r##"<line x1="10" y1="20" x2="110" y2="70""##));
    assert!(!line.contains("marker-end"));

    let arrow = with(ElementType::Arrow, ElementProperties::default());
    assert!(arrow.contains(r##" marker-end="url(#arrowhead)""##));
}

#[test]
fn freehand_paths_need_path_data() {
    assert!(!with(ElementType::FreehandPath, ElementProperties::default()).contains("<path"));
    let svg = with(
        ElementType::FreehandPath,
        ElementProperties {
            path_data: Some("M 0 0 L 10 10".to_string()),
            ..ElementProperties::default()
        },
    );
    assert!(svg.contains(r##"<path d="M 0 0 L 10 10" fill="none""##));
}

#[test]
fn images_stickies_frames_and_connectors_all_render() {
    let image = with(
        ElementType::Image,
        ElementProperties {
            image_url: Some("/drive/a.png".to_string()),
            ..ElementProperties::default()
        },
    );
    assert!(image.contains(r##"<image x="10" y="20" width="100" height="50" href="/drive/a.png""##));
    assert!(!with(ElementType::Image, ElementProperties::default()).contains("<image"));

    let sticky = with(
        ElementType::Sticky,
        ElementProperties {
            text: Some("note".to_string()),
            ..ElementProperties::default()
        },
    );
    assert!(sticky.contains(r##"rx="2""##));
    assert!(sticky.contains(">note</text>"));

    for kind in [ElementType::Frame, ElementType::Connector] {
        let svg = with(kind.clone(), ElementProperties::default());
        assert!(
            svg.contains(r##"fill="none" stroke="none""##),
            "{kind:?} must render an outlined box: {svg}"
        );
    }
}

#[test]
fn markup_in_values_is_escaped() {
    let svg = with(
        ElementType::Text,
        ElementProperties {
            text: Some("a & b <c> \"d\"".to_string()),
            font_family: Some("A&B".to_string()),
            ..ElementProperties::default()
        },
    );
    assert!(svg.contains("a &amp; b &lt;c&gt; &quot;d&quot;"));
    assert!(svg.contains(r##"font-family="A&amp;B""##));
    assert!(!svg.contains("<c>"));

    let mut board = canvas(vec![]);
    board.background_color = "#fff&co".to_string();
    assert!(generate_svg(&board, true).contains(r##"fill="#fff&amp;co""##));
}
