//! Row → domain conversion expectations.

use super::*;
use crate::models::{DbCanvas, DbCanvasElement, ElementProperties, ElementType};
use chrono::Utc;
use uuid::Uuid;

fn db_canvas(background_color: Option<&str>) -> DbCanvas {
    DbCanvas {
        id: Uuid::nil(),
        branch_id: Uuid::nil(),
        name: "Board".to_string(),
        description: Some("desc".to_string()),
        width: 640,
        height: 480,
        background_color: background_color.map(str::to_string),
        thumbnail_url: Some("/thumb.png".to_string()),
        is_public: true,
        is_template: false,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
    }
}

fn db_element(element_type: &str, properties: serde_json::Value) -> DbCanvasElement {
    DbCanvasElement {
        id: Uuid::nil(),
        canvas_id: Uuid::nil(),
        element_type: element_type.to_string(),
        x: 1.0,
        y: 2.0,
        width: 3.0,
        height: 4.0,
        rotation: 15.0,
        z_index: 9,
        locked: true,
        properties,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn missing_background_falls_back_to_white() {
    assert_eq!(
        db_to_canvas(db_canvas(None), vec![]).background_color,
        "#ffffff"
    );
    assert_eq!(
        db_to_canvas(db_canvas(Some("#123456")), vec![]).background_color,
        "#123456"
    );
}

#[test]
fn canvas_fields_and_element_order_are_preserved() {
    let elements = vec![
        db_to_canvas_element(db_element("text", serde_json::json!({}))),
        db_to_canvas_element(db_element("ellipse", serde_json::json!({}))),
    ];
    let canvas = db_to_canvas(db_canvas(Some("#000000")), elements);
    assert_eq!(canvas.width, 640);
    assert_eq!(canvas.height, 480);
    assert_eq!(canvas.description.as_deref(), Some("desc"));
    assert_eq!(canvas.thumbnail_url.as_deref(), Some("/thumb.png"));
    assert!(canvas.is_public);
    assert_eq!(
        canvas
            .elements
            .iter()
            .map(|e| e.element_type.as_str())
            .collect::<Vec<_>>(),
        ["text", "ellipse"]
    );
}

#[test]
fn element_rows_map_every_column() {
    let element = db_to_canvas_element(db_element(
        "sticky",
        serde_json::json!({ "fill_color": "#fde68a", "text": "note" }),
    ));
    assert_eq!(element.element_type, ElementType::Sticky);
    assert_eq!(element.x, 1.0);
    assert_eq!(element.y, 2.0);
    assert_eq!(element.width, 3.0);
    assert_eq!(element.height, 4.0);
    assert_eq!(element.rotation, 15.0);
    assert_eq!(element.z_index, 9);
    assert!(element.locked);
    assert_eq!(element.properties.fill_color.as_deref(), Some("#fde68a"));
    assert_eq!(element.properties.text.as_deref(), Some("note"));
}

#[test]
fn invalid_properties_and_unknown_labels_degrade_without_panicking() {
    let element = db_to_canvas_element(db_element("not-a-type", serde_json::json!("nope")));
    assert_eq!(element.element_type, ElementType::Rectangle);
    assert_eq!(element.properties.fill_color, None);
    assert_eq!(element.properties.text, None);
    let empty: ElementProperties = ElementProperties::default();
    assert_eq!(element.properties.corner_radius, empty.corner_radius);
}
