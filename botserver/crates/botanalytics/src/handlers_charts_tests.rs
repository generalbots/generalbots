//! Offline tests for the SVG line/area chart renderer (issue #2075).
//! `render_line_chart` is a pure formatter: values in, markup out.

use super::*;

const POLYLINE: &str = "stroke-width=\"2\" points=\"";
const POLYGON: &str = "fill-opacity=\"0.15\" stroke=\"none\" points=\"";

fn coordinates(html: &str, marker: &str) -> Vec<(f64, f64)> {
    let start = html.find(marker).expect("marker present");
    let rest = &html[start + marker.len()..];
    let end = rest.find('"').expect("closing quote");
    rest[..end]
        .split_whitespace()
        .map(|pair| {
            let (x, y) = pair.split_once(',').expect("x,y pair");
            (x.parse().expect("x"), y.parse().expect("y"))
        })
        .collect()
}

fn labels(count: usize) -> Vec<String> {
    (0..count).map(|h| format!("{h}:00")).collect()
}

#[test]
fn empty_series_still_produces_a_well_formed_svg() {
    let html = render_line_chart(&[], 1.0, &[], false);

    assert!(html.starts_with("<div class=\"chart-container line-chart\">"));
    assert!(html.contains("<svg viewBox=\"0 0 480 200\" preserveAspectRatio=\"none\">"));
    assert!(html.contains("</svg>"));
    assert!(html.trim_end().ends_with("</div></div>"));
    assert!(coordinates(&html, POLYLINE).is_empty());
    assert!(!html.contains("<polygon"));
}

#[test]
fn the_first_point_sits_on_the_baseline_and_the_last_on_the_top() {
    let html = render_line_chart(&[0.0, 5.0, 10.0], 10.0, &labels(3), false);
    let points = coordinates(&html, POLYLINE);

    assert_eq!(points.len(), 3);
    assert_eq!(points[0], (0.0, 200.0));
    assert!((points[1].1 - 110.0).abs() < 1e-9, "midpoint y {}", points[1].1);
    assert!((points[2].1 - 20.0).abs() < 1e-9, "top y {}", points[2].1);
}

#[test]
fn x_coordinates_spread_across_the_full_width() {
    let values: Vec<f64> = (0..24).map(|i| i as f64).collect();
    let html = render_line_chart(&values, 23.0, &labels(24), false);
    let points = coordinates(&html, POLYLINE);

    assert_eq!(points.len(), 24);
    assert_eq!(points[0].0, 0.0);
    assert_eq!(points[23].0, 480.0);
    assert!((points[12].0 - 250.434_782_608_695_66).abs() < 1e-9);
}

#[test]
fn fill_adds_a_closed_polygon_under_the_line() {
    let html = render_line_chart(&[0.0, 10.0], 10.0, &labels(2), true);

    assert!(html.contains("<polygon"));
    let polygon = coordinates(&html, POLYGON);
    // One point per value plus the two baseline corners that close the area.
    assert_eq!(polygon.len(), 4);
    assert_eq!(&polygon[2..], &[(480.0, 200.0), (0.0, 200.0)]);
    assert!(html.contains("480,200 0,200\"/>"));
}

#[test]
fn labels_are_printed_every_four_hours() {
    let html = render_line_chart(&[1.0], 1.0, &labels(24), false);

    assert_eq!(html.matches("<span>").count(), 6);
    assert!(html.contains("<span>0:00</span>"));
    assert!(html.contains("<span>4:00</span>"));
    assert!(!html.contains("<span>1:00</span>"));
}

#[test]
fn a_non_positive_scale_does_not_emit_infinite_coordinates() {
    let html = render_line_chart(&[0.0, 0.0], 0.0, &labels(2), false);

    assert!(!html.contains("inf"));
    assert!(!html.contains("NaN"));
    let points = coordinates(&html, POLYLINE);
    assert_eq!(points.len(), 2);
    assert_eq!(points[0], (0.0, 200.0));
    assert_eq!(points[1].1, 200.0);
    assert!((points[1].0 - 20.869_565_217_391_305).abs() < 1e-9);
}
