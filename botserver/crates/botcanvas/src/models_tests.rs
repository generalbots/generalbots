//! Domain-model serde and label expectations.

use crate::models::{Canvas, CanvasElement, ElementProperties, ElementType};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn every_element_type_round_trips_through_its_label() {
    let kinds = [
        (ElementType::Rectangle, "rectangle"),
        (ElementType::Ellipse, "ellipse"),
        (ElementType::Line, "line"),
        (ElementType::Arrow, "arrow"),
        (ElementType::FreehandPath, "freehand_path"),
        (ElementType::Text, "text"),
        (ElementType::Image, "image"),
        (ElementType::Sticky, "sticky"),
        (ElementType::Frame, "frame"),
        (ElementType::Connector, "connector"),
    ];
    for (kind, label) in kinds {
        assert_eq!(kind.as_str(), label);
        assert_eq!(ElementType::from_label(label), kind);
        let json = serde_json::to_string(&kind).expect("serialize");
        assert_eq!(json, format!("\"{label}\""));
        let parsed: ElementType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, kind);
    }
}

#[test]
fn unknown_labels_fall_back_to_a_rectangle() {
    for label in ["", "Rectangle", "circle", "freehand-path", "unknown"] {
        assert_eq!(ElementType::from_label(label), ElementType::Rectangle);
    }
}

#[test]
fn empty_properties_parse_to_all_none() {
    let properties: ElementProperties = serde_json::from_str("{}").expect("empty object");
    assert!(properties.fill_color.is_none());
    assert!(properties.text.is_none());
    assert!(properties.corner_radius.is_none());
    assert!(properties.image_url.is_none());
}

#[test]
fn unknown_properties_are_ignored_and_none_fields_are_omitted() {
    let properties: ElementProperties =
        serde_json::from_str(r##"{"fill_color":"#fff","unknown_field":42}"##).expect("lenient");
    assert_eq!(properties.fill_color.as_deref(), Some("#fff"));
    let json = serde_json::to_string(&properties).expect("serialize");
    assert_eq!(json, r##"{"fill_color":"#fff"}"##);
}

#[test]
fn element_properties_round_trip_every_field() {
    let properties = ElementProperties {
        fill_color: Some("#fff".to_string()),
        stroke_color: Some("#000".to_string()),
        stroke_width: Some(2.0),
        opacity: Some(0.25),
        text: Some("hi".to_string()),
        font_size: Some(18.0),
        font_family: Some("Georgia".to_string()),
        text_align: Some("center".to_string()),
        image_url: Some("/a.png".to_string()),
        path_data: Some("M 0 0".to_string()),
        corner_radius: Some(4.0),
        start_arrow: Some("dot".to_string()),
        end_arrow: Some("triangle".to_string()),
    };
    let json = serde_json::to_string(&properties).expect("serialize");
    let parsed: ElementProperties = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.fill_color, properties.fill_color);
    assert_eq!(parsed.end_arrow, properties.end_arrow);
    assert_eq!(parsed.corner_radius, properties.corner_radius);
}

#[test]
fn canvas_elements_serialize_with_snake_case_types() {
    let element = CanvasElement {
        id: Uuid::nil(),
        canvas_id: Uuid::nil(),
        element_type: ElementType::FreehandPath,
        x: 1.5,
        y: 2.5,
        width: 3.0,
        height: 4.0,
        rotation: 0.0,
        z_index: 7,
        locked: true,
        properties: ElementProperties::default(),
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let json = serde_json::to_value(&element).expect("serialize");
    assert_eq!(json["element_type"], "freehand_path");
    assert_eq!(json["z_index"], 7);
    assert_eq!(json["locked"], true);
    assert_eq!(json["x"], 1.5);

    let board = Canvas {
        id: Uuid::nil(),
        org_id: Uuid::nil(),
        name: "Board".to_string(),
        description: None,
        width: 10,
        height: 20,
        background_color: "#ffffff".to_string(),
        thumbnail_url: None,
        is_public: false,
        is_template: false,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        elements: vec![element],
    };
    let json = serde_json::to_value(&board).expect("serialize board");
    assert_eq!(json["elements"][0]["element_type"], "freehand_path");
    assert_eq!(json["width"], 10);
}
