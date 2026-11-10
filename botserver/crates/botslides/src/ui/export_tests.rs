//! Offline unit tests for the slide exporters (#2075).
//!
//! `export_to_html`, `export_to_svg`, `export_to_json` and `export_to_markdown`
//! turn the stored presentation model into files users download and open. They
//! are pure functions over the model, so every branch is pinned here: escaping,
//! heading levels, empty decks and the media/shape rendering paths.

use super::*;
use crate::types::*;
use chrono::Utc;

fn theme() -> PresentationTheme {
    PresentationTheme {
        name: "Test theme".to_string(),
        colors: ThemeColors {
            primary: "#111111".to_string(),
            secondary: "#222222".to_string(),
            accent: "#333333".to_string(),
            background: "#ffffff".to_string(),
            text: "#000000".to_string(),
            text_light: "#666666".to_string(),
        },
        fonts: ThemeFonts {
            heading: "Arial".to_string(),
            body: "Arial".to_string(),
        },
    }
}

fn presentation(name: &str, slides: Vec<Slide>) -> Presentation {
    Presentation {
        id: "p1".to_string(),
        name: name.to_string(),
        owner_id: "o1".to_string(),
        slides,
        theme: theme(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_bucket: None,
        source_path: None,
    }
}

fn slide(elements: Vec<SlideElement>) -> Slide {
    Slide {
        id: "s1".to_string(),
        layout: "blank".to_string(),
        elements,
        background: SlideBackground::default(),
        notes: None,
        transition: None,
        transition_config: None,
        media: None,
    }
}

fn element(element_type: &str, content: ElementContent) -> SlideElement {
    SlideElement {
        id: "e1".to_string(),
        element_type: element_type.to_string(),
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        rotation: 0.0,
        content,
        style: ElementStyle::default(),
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    }
}

fn text(text: &str, font_size: Option<f64>) -> SlideElement {
    let mut element = element(
        "text",
        ElementContent {
            text: Some(text.to_string()),
            ..Default::default()
        },
    );
    element.style.font_size = font_size;
    element
}

fn image(src: &str) -> SlideElement {
    element(
        "image",
        ElementContent {
            src: Some(src.to_string()),
            ..Default::default()
        },
    )
}

#[test]
fn markdown_uses_heading_levels_from_the_font_size() {
    let deck = presentation(
        "Quarterly review",
        vec![slide(vec![
            text("Big title", Some(40.0)),
            text("Body copy", Some(18.0)),
            text("Default size", None),
        ])],
    );

    let md = export_to_markdown(&deck);
    assert!(md.starts_with("# Quarterly review\n"));
    assert!(md.contains("## Slide 1"));
    assert!(md.contains("### Big title"));
    assert!(md.contains("\nBody copy\n"));
    assert!(md.contains("\nDefault size\n"));
    // A 32pt title is still a heading; 31.9 is body text.
    let deck = presentation("t", vec![slide(vec![text("edge", Some(32.0))])]);
    assert!(export_to_markdown(&deck).contains("### edge"));
    let deck = presentation("t", vec![slide(vec![text("edge", Some(31.9))])]);
    assert!(!export_to_markdown(&deck).contains("### edge"));
}

#[test]
fn markdown_renders_images_notes_and_slide_numbers() {
    let mut first = slide(vec![image("https://example.test/a.png")]);
    first.notes = Some("remember the demo".to_string());
    let deck = presentation("Deck", vec![first, slide(Vec::new())]);

    let md = export_to_markdown(&deck);
    assert!(md.contains("## Slide 1"));
    assert!(md.contains("## Slide 2"));
    assert!(md.contains("![Image](https://example.test/a.png)"));
    assert!(md.contains("**Speaker Notes:**\nremember the demo"));
    // An image element without a src renders nothing rather than an empty tag.
    let deck = presentation("Deck", vec![slide(vec![image("")])]);
    assert!(!export_to_markdown(&deck).contains("![Image]"));
}

#[test]
fn markdown_of_an_empty_deck_is_just_the_title() {
    assert_eq!(export_to_markdown(&presentation("Empty", Vec::new())), "# Empty\n\n");
}

#[test]
fn json_export_round_trips_the_model() {
    let deck = presentation("Deck", vec![slide(vec![text("hello", None)])]);
    let json = export_to_json(&deck);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(parsed["name"], "Deck");
    assert_eq!(parsed["slides"].as_array().map(Vec::len), Some(1));
    assert_eq!(parsed["slides"][0]["elements"][0]["content"]["text"], "hello");
    // Pretty-printed, so a human can diff two decks.
    assert!(json.contains("\n  \"name\""));
}

#[test]
fn svg_has_the_requested_canvas_and_default_background() {
    let svg = export_to_svg(&slide(Vec::new()), 960, 540);
    assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(svg.contains(r#"width="960" height="540" viewBox="0 0 960 540""#));
    assert!(svg.contains(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##));
    assert!(svg.trim_end().ends_with("</svg>"));
}

#[test]
fn svg_uses_the_slide_background_and_escapes_text() {
    let mut s = slide(vec![text("a < b & \"c\"", Some(24.0))]);
    s.background.color = Some("#123456".to_string());

    let svg = export_to_svg(&s, 100, 100);
    assert!(svg.contains(r#"fill="#123456""#));
    assert!(svg.contains("a &lt; b &amp; &quot;c&quot;"));
    // The text baseline is offset by the font size, not drawn on the top edge.
    assert!(svg.contains(r#"<text x="10" y="44""#));
}

#[test]
fn svg_escapes_image_sources_so_an_ampersand_query_is_valid_xml() {
    // `?a=1&b=2` is a normal image URL; unescaped it makes the SVG unparsable.
    let svg = export_to_svg(&slide(vec![image("https://x.test/a.png?a=1&b=2")]), 10, 10);
    assert!(svg.contains("href=\"https://x.test/a.png?a=1&amp;b=2\""));

    let mut with_media = slide(Vec::new());
    with_media.media = Some(vec![MediaElement {
        id: "m1".to_string(),
        media_type: "image".to_string(),
        src: "https://x.test/b.png?c=3&d=4".to_string(),
        poster: None,
        autoplay: false,
        loop_playback: false,
        muted: false,
        controls: false,
        start_time: None,
        end_time: None,
        volume: None,
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }]);
    let svg = export_to_svg(&with_media, 10, 10);
    assert!(svg.contains("href=\"https://x.test/b.png?c=3&amp;d=4\""));
}

#[test]
fn svg_renders_every_shape_branch() {
    for shape in ["rectangle", "circle", "line", "triangle"] {
        let mut element = element(
            "shape",
            ElementContent {
                shape_type: Some(shape.to_string()),
                ..Default::default()
            },
        );
        element.style.fill = Some("#abcdef".to_string());
        let svg = export_to_svg(&slide(vec![element]), 10, 10);
        let expected_tag = match shape {
            "rectangle" => "<rect",
            "circle" => "<ellipse",
            "line" => "<line",
            _ => "<polygon",
        };
        assert!(svg.contains(expected_tag), "{shape} must render {expected_tag}");
        assert!(svg.contains("#abcdef"), "{shape} must keep its fill");
    }

    // An unknown shape type draws nothing but still closes the document.
    let unknown = element(
        "shape",
        ElementContent {
            shape_type: Some("hexagon".to_string()),
            ..Default::default()
        },
    );
    let svg = export_to_svg(&slide(vec![unknown]), 10, 10);
    assert!(!svg.contains("<polygon"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn svg_renders_table_rows_and_chart_values() {
    let table = element(
        "table",
        ElementContent {
            table_data: Some(TableData {
                rows: 1,
                cols: 2,
                cells: vec![vec![
                    TableCell {
                        content: "Region".to_string(),
                        ..Default::default()
                    },
                    TableCell {
                        content: "Sales".to_string(),
                        ..Default::default()
                    },
                ]],
                col_widths: Vec::new(),
                row_heights: Vec::new(),
            }),
            ..Default::default()
        },
    );
    let chart = element(
        "chart",
        ElementContent {
            text: Some("Revenue".to_string()),
            chart_data: Some(ChartData {
                chart_type: "bar".to_string(),
                labels: vec!["Q1".to_string(), "Q2".to_string()],
                datasets: vec![ChartDataset {
                    label: "2026".to_string(),
                    data: vec![10.0, 20.0],
                    color: "#3b82f6".to_string(),
                }],
            }),
            ..Default::default()
        },
    );

    let svg = export_to_svg(&slide(vec![table, chart]), 10, 10);
    assert!(svg.contains("Region | Sales"));
    assert!(svg.contains("Revenue"));
    assert!(svg.contains("Q1: 10"));
    assert!(svg.contains("Q2: 20"));
}

#[test]
fn html_export_escapes_the_deck_name_and_table_cells() {
    let table = element(
        "table",
        ElementContent {
            table_data: Some(TableData {
                rows: 2,
                cols: 1,
                cells: vec![
                    vec![TableCell {
                        content: "Head".to_string(),
                        ..Default::default()
                    }],
                    vec![TableCell {
                        content: "<script>".to_string(),
                        ..Default::default()
                    }],
                ],
                col_widths: Vec::new(),
                row_heights: Vec::new(),
            }),
            ..Default::default()
        },
    );
    let deck = presentation("Q3 <Draft> & Co", vec![slide(vec![table])]);

    let html = export_to_html(&deck);
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Q3 &lt;Draft&gt; &amp; Co</title>"));
    assert!(!html.contains("<title>Q3 <Draft>"));
    // Header row uses <th>, body rows use <td>, and the payload is escaped.
    assert!(html.contains("<th style="));
    assert!(html.contains("<td style="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn html_export_renders_slide_backgrounds_elements_and_media() {
    let mut s = slide(vec![text("hello", Some(20.0))]);
    s.background.color = Some("#101010".to_string());
    s.media = Some(vec![MediaElement {
        id: "m1".to_string(),
        media_type: "image".to_string(),
        src: "https://x.test/a.png?a=1&b=2".to_string(),
        poster: None,
        autoplay: false,
        loop_playback: false,
        muted: false,
        controls: false,
        start_time: None,
        end_time: None,
        volume: None,
        x: 1.0,
        y: 2.0,
        width: 3.0,
        height: 4.0,
    }]);

    let html = export_to_html(&presentation("Deck", vec![s]));
    assert!(html.contains("background-color: #101010"));
    assert!(html.contains("class=\"element element-text\""));
    assert!(html.contains("left: 10px; top: 20px; width: 100px; height: 50px;"));
    assert!(html.contains("src=\"https://x.test/a.png?a=1&amp;b=2\""));
    assert!(html.trim_end().ends_with("</html>"));
}

#[test]
fn html_export_of_an_empty_deck_is_still_a_document() {
    let html = export_to_html(&presentation("Empty", Vec::new()));
    assert!(html.contains("<body>"));
    assert!(html.contains("</html>"));
    assert!(!html.contains("class=\"slide\""));
}

#[test]
fn the_png_placeholder_is_the_svg_bytes() {
    let s = slide(vec![text("x", None)]);
    let png = export_slide_to_png_placeholder(&s, 32, 32);
    assert_eq!(png, export_to_svg(&s, 32, 32).into_bytes());
}
