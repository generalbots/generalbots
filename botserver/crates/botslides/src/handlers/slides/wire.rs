//! Canvas payload → typed element conversion.
//!
//! The canvas posts flat element fields (`slide_id`, `element_id`,
//! `element_type`, x/y/width/height, rotation, fill, shape_type) while this
//! API was typed for a nested SlideElement — every canvas edit 422'd. The
//! helpers below accept both shapes and the bare `elements: [ids]` order
//! payload sent when elements move between slides.

use crate::types::{
    ChartData, ChartDataset, ElementContent, ElementStyle, SlideElement, TableCell, TableData,
};
use uuid::Uuid;

pub(super) fn presentation_id_of(payload: &serde_json::Value) -> String {
    payload
        .get("presentation_id")
        .and_then(|value| value.as_str())
        .unwrap_or("current")
        .to_string()
}

pub(super) fn slide_index_of(payload: &serde_json::Value) -> usize {
    payload
        .get("slide_index")
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_str().and_then(|text| text.trim().parse::<u64>().ok()))
        })
        .or_else(|| {
            payload
                .get("slide_id")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0) as usize
}

pub(super) fn wire_element(payload: &serde_json::Value) -> Option<SlideElement> {
    if let Some(element) = payload.get("element") {
        if let Ok(element) = serde_json::from_value::<SlideElement>(element.clone()) {
            return Some(element);
        }
    }
    // Toolbar modals add elements without an id; generate one instead of
    // rejecting the request (the id only has to be unique inside the slide).
    let id = payload
        .get("element_id")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("el_{}", Uuid::new_v4()));
    let number = |key: &str| payload.get(key).and_then(|value| value.as_f64()).unwrap_or(0.0);
    let number_opt = |key: &str| payload.get(key).and_then(|value| value.as_f64());
    let text = |key: &str| {
        payload
            .get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    };
    let element_type = text("element_type").unwrap_or_else(|| "shape".to_string());
    // Toolbar modals send only text/colour, so a 0×0 element at (0,0) was
    // saved and stayed invisible. Give the standard insert actions a visible
    // default box.
    let width = number("width");
    let width = if width > 0.0 {
        width
    } else if element_type == "text" {
        360.0
    } else {
        200.0
    };
    let height = number("height");
    let height = if height > 0.0 {
        height
    } else if element_type == "text" {
        80.0
    } else {
        120.0
    };
    let x = number("x");
    let x = if x > 0.0 { x } else { 80.0 };
    let y = number("y");
    let y = if y > 0.0 { y } else { 80.0 };
    let table_data = if element_type == "table" {
        let unsigned = |key: &str, default: u64| {
            payload
                .get(key)
                .and_then(|value| {
                    value
                        .as_u64()
                        .or_else(|| value.as_str().and_then(|text| text.trim().parse::<u64>().ok()))
                })
                .map(|value| value.clamp(1, 50))
                .unwrap_or(default)
        };
        let rows = unsigned("rows", 3) as usize;
        let cols = unsigned("cols", 3) as usize;
        let headers: Vec<String> = payload
            .get("headers")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .split(',')
            .map(|header| header.trim().to_string())
            .collect();
        let mut cells = vec![vec![TableCell::default(); cols]; rows];
        for (column, header) in headers.iter().enumerate().take(cols) {
            if !header.is_empty() {
                cells[0][column].content = header.clone();
            }
        }
        Some(TableData {
            rows,
            cols,
            cells,
            col_widths: Vec::new(),
            row_heights: Vec::new(),
        })
    } else {
        None
    };
    let chart_data = if element_type == "chart" {
        let parsed = payload.get("data").and_then(|value| {
            if value.is_string() {
                value.as_str().and_then(|text| serde_json::from_str(text).ok())
            } else {
                Some(value.clone())
            }
        });
        parsed.map(|value| {
            let mut labels: Vec<String> = value
                .get("labels")
                .and_then(|items| items.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let mut datasets: Vec<ChartDataset> = value
                .get("datasets")
                .and_then(|items| items.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| serde_json::from_value::<ChartDataset>(item.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();
            // The form starts with `{}`; a demo series keeps the inserted
            // chart visible instead of saving an empty box.
            if datasets.is_empty() {
                datasets.push(ChartDataset {
                    label: "Series 1".to_string(),
                    data: vec![1.0, 2.0, 3.0],
                    color: "#3b82f6".to_string(),
                });
            }
            if labels.is_empty() {
                labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
            }
            ChartData {
                chart_type: text("chart_type").unwrap_or_else(|| "bar".to_string()),
                labels,
                datasets,
            }
        })
    } else {
        None
    };
    Some(SlideElement {
        id,
        element_type,
        x,
        y,
        width,
        height,
        rotation: number("rotation"),
        content: ElementContent {
            text: text("chart_title").or_else(|| text("text")),
            html: None,
            src: text("src"),
            shape_type: text("shape_type"),
            chart_data,
            table_data,
        },
        style: ElementStyle {
            fill: text("fill"),
            stroke: text("stroke"),
            stroke_width: number_opt("stroke_width").or_else(|| number_opt("strokeWidth")),
            opacity: number_opt("opacity"),
            font_size: number_opt("font_size").or_else(|| number_opt("fontSize")),
            color: text("color"),
            ..Default::default()
        },
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    })
}
