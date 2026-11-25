//! Offline tests for native PPTX table/chart export (#2457).
//!
//! These exercise the real package writer and read the resulting zip back,
//! so a regression to flattened text fails here without a running server.

use super::export_to_pptx;
use crate::types::*;
use chrono::Utc;
use std::io::{Cursor, Read};
use zip::ZipArchive;

fn element(element_type: &str, content: ElementContent) -> SlideElement {
    SlideElement {
        id: "e1".to_string(),
        element_type: element_type.to_string(),
        x: 10.0,
        y: 20.0,
        width: 400.0,
        height: 200.0,
        rotation: 0.0,
        content,
        style: ElementStyle::default(),
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    }
}

fn deck(elements: Vec<SlideElement>) -> Presentation {
    Presentation {
        id: "p1".to_string(),
        name: "Fidelity".to_string(),
        owner_id: "o1".to_string(),
        slides: vec![Slide {
            id: "s1".to_string(),
            layout: "blank".to_string(),
            elements,
            background: SlideBackground::default(),
            notes: None,
            transition: None,
            transition_config: None,
            media: None,
        }],
        theme: crate::utils::create_default_theme(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_bucket: None,
        source_path: None,
    }
}

fn part(bytes: &[u8], name: &str) -> Option<String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut text = String::new();
    file.read_to_string(&mut text).ok()?;
    Some(text)
}

/// An export error is unexpected in these tests: log it and hand back an
/// empty stream so the assertions below still fail with a clear message.
fn exported(elements: Vec<SlideElement>) -> Vec<u8> {
    match export_to_pptx(&deck(elements)) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!("slides export test: export failed: {e}");
            Vec::new()
        }
    }
}

/// A named part must exist in a generated package; absence is a test failure.
fn part_text(bytes: &[u8], name: &str) -> String {
    match part(bytes, name) {
        Some(text) => text,
        None => {
            log::warn!("slides export test: missing part {name}");
            String::new()
        }
    }
}

fn chart_data() -> ChartData {
    ChartData {
        chart_type: "bar".to_string(),
        labels: vec!["Q1".to_string(), "Q2".to_string()],
        datasets: vec![ChartDataset {
            label: "2026".to_string(),
            data: vec![10.0, 20.0],
            color: "#3b82f6".to_string(),
        }],
    }
}

#[test]
fn table_exports_as_native_graphic_frame_without_flattened_text() {
    let table = element(
        "table",
        ElementContent {
            table_data: Some(TableData {
                rows: 2,
                cols: 2,
                cells: vec![
                    vec![
                        TableCell {
                            content: "Region".to_string(),
                            ..Default::default()
                        },
                        TableCell {
                            content: "Sales".to_string(),
                            ..Default::default()
                        },
                    ],
                    vec![
                        TableCell {
                            content: "<script>&\nnext".to_string(),
                            ..Default::default()
                        },
                        TableCell {
                            content: "42".to_string(),
                            ..Default::default()
                        },
                    ],
                ],
                col_widths: Vec::new(),
                row_heights: Vec::new(),
            }),
            ..Default::default()
        },
    );

    let bytes = exported(vec![table]);
    assert!(!bytes.is_empty(), "export must produce a pptx stream");
    let slide = part_text(&bytes, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("<p:graphicFrame>"),
        "table must be a graphic frame"
    );
    assert!(slide.contains("http://schemas.openxmlformats.org/drawingml/2006/table"));
    assert!(slide.contains("<a:tbl>"), "native a:tbl must be present");
    assert!(slide.contains("<a:tblGrid>"), "table grid must be present");
    assert_eq!(slide.matches("<a:gridCol ").count(), 2, "two columns");
    assert_eq!(slide.matches("<a:tr ").count(), 2, "two rows");
    assert!(
        slide.contains("<a:tcPr"),
        "cell borders/fills must be written"
    );
    assert!(
        slide.contains("&lt;script&gt;&amp;"),
        "cell text must be escaped"
    );
    assert!(
        !slide.contains("Region | Sales"),
        "no pipe-joined text fallback"
    );
    assert!(
        !slide.contains("<p:sp>"),
        "no text-box fallback shape for a table"
    );
}

#[test]
fn chart_exports_as_native_chart_part_without_flattened_text() {
    let chart = element(
        "chart",
        ElementContent {
            text: Some("Revenue".to_string()),
            chart_data: Some(chart_data()),
            ..Default::default()
        },
    );

    let bytes = exported(vec![chart]);
    let slide = part_text(&bytes, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("<p:graphicFrame>"),
        "chart must be a graphic frame"
    );
    assert!(
        slide.contains("<c:chart r:id=\"rIdC1\"/>"),
        "frame must reference chart rel"
    );
    assert!(!slide.contains("Q1: 10"), "no label/value text fallback");
    assert!(
        !slide.contains("<p:sp>"),
        "no text-box fallback shape for a chart"
    );

    let chart_part = part_text(&bytes, "ppt/charts/chart1.xml");
    assert!(chart_part.contains("<c:barChart>"), "native bar chart part");
    assert!(chart_part.contains("<c:strCache>"), "category cache");
    assert!(chart_part.contains("<c:numCache>"), "value cache");
    assert!(
        chart_part.contains("<c:v>Q1</c:v>"),
        "cached category survives"
    );
    assert!(
        chart_part.contains("<c:v>10</c:v>"),
        "cached value survives"
    );
    assert!(chart_part.contains("Revenue"), "chart title survives");

    let rels = part_text(&bytes, "ppt/slides/_rels/slide1.xml.rels");
    assert!(rels.contains("Id=\"rIdC1\""), "chart relationship id");
    assert!(
        rels.contains("Target=\"../charts/chart1.xml\""),
        "chart relationship target"
    );
    let types = part_text(&bytes, "[Content_Types].xml");
    assert!(
        types.contains("/ppt/charts/chart1.xml"),
        "chart content type override"
    );
    assert!(types.contains("drawingml.chart+xml"), "chart content type");
}

#[test]
fn multiple_charts_get_distinct_parts_and_relationship_ids() {
    let first = element(
        "chart",
        ElementContent {
            chart_data: Some(chart_data()),
            ..Default::default()
        },
    );
    let second = element(
        "chart",
        ElementContent {
            chart_data: Some(ChartData {
                chart_type: "pie".to_string(),
                ..chart_data()
            }),
            ..Default::default()
        },
    );
    let bytes = exported(vec![first, second]);
    let slide = part_text(&bytes, "ppt/slides/slide1.xml");
    assert!(
        slide.contains("rIdC1") && slide.contains("rIdC2"),
        "distinct chart rel ids"
    );
    assert!(
        part(&bytes, "ppt/charts/chart1.xml").is_some(),
        "first chart part"
    );
    assert!(
        part(&bytes, "ppt/charts/chart2.xml").is_some(),
        "second chart part"
    );
    let rels = part_text(&bytes, "ppt/slides/_rels/slide1.xml.rels");
    assert!(
        rels.contains("Target=\"../charts/chart2.xml\""),
        "second chart target"
    );
}

#[test]
fn table_merges_and_hostile_sizes_are_bounded() {
    let long_text = "x".repeat(5_000);
    let table = element(
        "table",
        ElementContent {
            table_data: Some(TableData {
                rows: 5_000,
                cols: 5_000,
                cells: vec![vec![
                    TableCell {
                        content: long_text,
                        colspan: 3,
                        rowspan: 3,
                        ..Default::default()
                    },
                    TableCell {
                        content: "next".to_string(),
                        ..Default::default()
                    },
                ]],
                col_widths: vec![f64::NAN; 5_000],
                row_heights: vec![f64::INFINITY; 5_000],
            }),
            ..Default::default()
        },
    );
    let bytes = exported(vec![table]);
    let slide = part_text(&bytes, "ppt/slides/slide1.xml");
    assert_eq!(slide.matches("<a:tr ").count(), 50, "rows are capped");
    assert_eq!(
        slide.matches("<a:gridCol ").count(),
        20,
        "columns are capped"
    );
    assert!(slide.contains("gridSpan=\"3\""), "horizontal span survives");
    assert!(slide.contains("rowSpan=\"3\""), "vertical span survives");
    assert!(slide.contains("hMerge=\"1\""), "merged continuation cell");
    assert!(slide.contains("vMerge=\"1\""), "merged continuation row");
    assert!(
        slide.len() < 2_000_000,
        "hostile dimensions cannot inflate the part"
    );
    assert!(
        !slide.contains("NaN") && !slide.contains("inf"),
        "no invalid XML numbers"
    );
}

#[test]
fn non_finite_chart_values_are_written_as_zero() {
    let chart = element(
        "chart",
        ElementContent {
            chart_data: Some(ChartData {
                labels: vec!["A".to_string(), "B".to_string()],
                datasets: vec![ChartDataset {
                    label: "bad".to_string(),
                    data: vec![f64::NAN, f64::INFINITY],
                    color: "not-a-color".to_string(),
                }],
                ..chart_data()
            }),
            ..Default::default()
        },
    );
    let bytes = exported(vec![chart]);
    let chart_part = part_text(&bytes, "ppt/charts/chart1.xml");
    assert!(chart_part.contains("<c:v>0</c:v>"), "NaN/inf become 0");
    assert!(
        !chart_part.contains("NaN") && !chart_part.contains("inf"),
        "no invalid numbers"
    );
    assert!(
        chart_part.contains("val=\"4F81BD\""),
        "invalid colour falls back"
    );
}
