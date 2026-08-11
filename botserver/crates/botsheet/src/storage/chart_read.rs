//! XLSX chart extraction: the structured reader (umya-spreadsheet) drops
//! charts when loading a workbook, so this module walks the raw .xlsx
//! package (drawings + chart parts) and rebuilds ChartConfig objects so the
//! Sheets UI can render charts loaded from Drive files.

use crate::types::{ChartConfig, ChartDataset, ChartOptions, ChartPosition, Worksheet};
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

const PALETTE: [&str; 8] = [
    "#3b82f6", "#ef4444", "#22c55e", "#eab308", "#a855f7", "#06b6d4", "#f97316", "#ec4899",
];

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 180;
const CELL_WIDTH_PX: u32 = 64;
const CELL_HEIGHT_PX: u32 = 20;

struct ParsedSeries {
    name: String,
    cat_ref: String,
    val_ref: String,
}

struct ParsedChart {
    chart_type: String,
    title: String,
    legend_pos: Option<String>,
    series: Vec<ParsedSeries>,
}

/// Extracts charts from an .xlsx package. Returns one entry per worksheet
/// (index-aligned with `worksheets`); entries are empty when the sheet has
/// no charts.
pub fn extract_charts(
    bytes: &[u8],
    worksheets: &[Worksheet],
) -> Result<Vec<Vec<ChartConfig>>, String> {
    let files = read_zip_entries(bytes)?;
    let mut result: Vec<Vec<ChartConfig>> = vec![Vec::new(); worksheets.len()];

    for (ws_index, _ws) in worksheets.iter().enumerate() {
        let sheet_num = ws_index + 1;
        let (Some(sheet_xml), Some(rels_xml)) = (
            files.get(&format!("xl/worksheets/sheet{sheet_num}.xml")),
            files.get(&format!("xl/worksheets/_rels/sheet{sheet_num}.xml.rels")),
        ) else {
            continue;
        };

        let sheet_rels = parse_rels(rels_xml);
        let Some(drawing_rid) = find_drawing_rid(sheet_xml) else {
            continue;
        };
        let Some(drawing_target) = sheet_rels.get(&drawing_rid) else {
            continue;
        };
        let Some(drawing_path) = normalize_path("xl/worksheets", drawing_target) else {
            continue;
        };
        let Some(drawing_xml) = files.get(&drawing_path) else {
            continue;
        };

        let drawing_rels_path = format!(
            "xl/drawings/_rels/{}.rels",
            drawing_path.rsplit('/').next().unwrap_or("")
        );
        let drawing_rels = files
            .get(&drawing_rels_path)
            .map(|r| parse_rels(r))
            .unwrap_or_default();

        let anchored = parse_drawing(drawing_xml, &drawing_rels);
        if anchored.is_empty() {
            continue;
        }

        let mut charts = Vec::new();
        for (chart_path, position) in anchored {
            let Some(chart_xml) = files.get(&chart_path) else {
                continue;
            };
            let parsed = parse_chart(chart_xml);
            charts.push(to_chart_config(&parsed, ws_index, worksheets, position));
        }
        if !charts.is_empty() {
            result[ws_index] = charts;
        }
    }

    Ok(result)
}

fn to_chart_config(
    parsed: &ParsedChart,
    ws_index: usize,
    worksheets: &[Worksheet],
    position: ChartPosition,
) -> ChartConfig {
    let mut datasets = Vec::new();
    for (i, series) in parsed.series.iter().enumerate() {
        let label = if series.name.contains('!') {
            resolve_single_cell(&series.name, ws_index, worksheets)
                .unwrap_or_default()
        } else {
            series.name.clone()
        };
        let data = resolve_range_values(&series.val_ref, ws_index, worksheets);
        datasets.push(ChartDataset {
            label,
            data,
            color: PALETTE[i % PALETTE.len()].to_string(),
            background_color: None,
        });
    }

    let mut labels = Vec::new();
    if let Some(first) = parsed.series.first() {
        labels = resolve_range_labels(&first.cat_ref, ws_index, worksheets);
    }

    ChartConfig {
        id: Uuid::new_v4().to_string(),
        chart_type: parsed.chart_type.clone(),
        title: parsed.title.clone(),
        data_range: parsed.series.first().map(|s| s.val_ref.clone()).unwrap_or_default(),
        label_range: parsed.series.first().map(|s| s.cat_ref.clone()).unwrap_or_default(),
        position,
        options: ChartOptions {
            show_legend: parsed.legend_pos.is_some(),
            show_grid: true,
            stacked: false,
            legend_position: parsed.legend_pos.clone(),
            x_axis_title: None,
            y_axis_title: None,
        },
        datasets,
        labels,
    }
}

fn read_zip_entries(bytes: &[u8]) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Failed to open xlsx zip: {e}"))?;
    let mut files = HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data)
            .map_err(|e| format!("Failed to read zip entry {name}: {e}"))?;
        files.insert(name, data);
    }
    Ok(files)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|&c| c == b':').next().unwrap_or(name)
}

fn attr_value(attr: &quick_xml::events::attributes::Attribute) -> String {
    attr.unescape_value()
        .map(|v| v.into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).into_owned())
}

/// Parses an OOXML `.rels` document into a map of relationship id -> target.
fn parse_rels(xml: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"Relationship" =>
            {
                let mut id = String::new();
                let mut target = String::new();
                for attr in e.attributes().flatten() {
                    match local_name(attr.key.as_ref()) {
                        b"Id" => id = attr_value(&attr),
                        b"Target" => target = attr_value(&attr),
                        _ => {}
                    }
                }
                if !id.is_empty() && !target.is_empty() {
                    map.insert(id, target);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

/// Finds the `r:id` of the `<drawing>` element in a sheet part.
fn find_drawing_rid(sheet_xml: &[u8]) -> Option<String> {
    let mut reader = quick_xml::Reader::from_reader(sheet_xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"drawing" =>
            {
                for attr in e.attributes().flatten() {
                    let value = attr_value(&attr);
                    if value.starts_with("rId") {
                        return Some(value);
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Normalizes a package-relative target against a base directory into a zip
/// entry path. Targets starting with `/` are absolute package paths and are
/// used as-is (openpyxl writes such targets, e.g. `/xl/charts/chart1.xml`).
fn normalize_path(base_dir: &str, target: &str) -> Option<String> {
    let mut parts: Vec<&str> = if let Some(rest) = target.strip_prefix('/') {
        rest.split('/').filter(|s| !s.is_empty()).collect()
    } else {
        base_dir.split('/').filter(|s| !s.is_empty()).collect()
    };
    if target.starts_with('/') {
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("/"));
    }
    for comp in target.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

/// Extracts (chart file path, anchor position) pairs from a drawing part.
fn parse_drawing(xml: &[u8], rels: &HashMap<String, String>) -> Vec<(String, ChartPosition)> {
    struct Anchor {
        from_col: u32,
        from_row: u32,
        to_col: Option<u32>,
        to_row: Option<u32>,
        rid: Option<String>,
    }

    let mut out = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut anchor: Option<Anchor> = None;
    let mut section: Option<bool> = None; // true = "from", false = "to"
    let mut in_coord = false; // currently inside <col>/<row> (not colOff/rowOff)
    let mut coord_is_col = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                match local_name(e.name().as_ref()) {
                    b"twoCellAnchor" | b"oneCellAnchor" => {
                        anchor = Some(Anchor {
                            from_col: 0,
                            from_row: 0,
                            to_col: None,
                            to_row: None,
                            rid: None,
                        });
                    }
                    b"from" => section = Some(true),
                    b"to" => section = Some(false),
                    b"col" if anchor.is_some() && section.is_some() => {
                        in_coord = true;
                        coord_is_col = true;
                    }
                    b"row" if anchor.is_some() && section.is_some() => {
                        in_coord = true;
                        coord_is_col = false;
                    }
                    b"colOff" | b"rowOff" => in_coord = false,
                    b"chart" => {
                        if let Some(ref mut a) = anchor {
                            for attr in e.attributes().flatten() {
                                let value = attr_value(&attr);
                                if value.starts_with("rId") {
                                    a.rid = Some(value);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if let (Some(ref mut a), Some(is_from)) = (anchor.as_mut(), section) {
                    if in_coord {
                        if let Ok(text) = t.unescape() {
                            let value = text.trim().parse::<u32>().unwrap_or(0);
                            if coord_is_col {
                                if is_from {
                                    a.from_col = value;
                                } else {
                                    a.to_col = Some(value);
                                }
                            } else if is_from {
                                a.from_row = value;
                            } else {
                                a.to_row = Some(value);
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match local_name(e.name().as_ref()) {
                    b"from" => section = None,
                    b"to" => section = None,
                    b"col" | b"row" => in_coord = false,
                    b"twoCellAnchor" | b"oneCellAnchor" => {
                        if let Some(a) = anchor.take() {
                            if let Some(chart_target) = a.rid.as_ref().and_then(|r| rels.get(r)) {
                                if let Some(path) = normalize_path("xl/drawings", chart_target) {
                                    let width = match a.to_col {
                                        Some(to_col) if to_col > a.from_col => {
                                            (to_col - a.from_col) * CELL_WIDTH_PX
                                        }
                                        _ => DEFAULT_WIDTH,
                                    };
                                    let height = match a.to_row {
                                        Some(to_row) if to_row > a.from_row => {
                                            (to_row - a.from_row) * CELL_HEIGHT_PX
                                        }
                                        _ => DEFAULT_HEIGHT,
                                    };
                                    out.push((
                                        path,
                                        ChartPosition {
                                            row: a.from_row,
                                            col: a.from_col,
                                            width: width.max(120),
                                            height: height.max(90),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

fn map_chart_type(name: &[u8]) -> &'static str {
    match name {
        b"pieChart" | b"pie3DChart" | b"doughnutChart" | b"doughnut3DChart" => "pie",
        b"lineChart" | b"line3DChart" | b"scatterChart" | b"areaChart" | b"area3DChart" => "line",
        b"barChart" | b"bar3DChart" | b"colChart" | b"col3DChart" | b"radarChart" => "bar",
        _ => "bar",
    }
}

/// Parses a chart part into type/title/legend/series references.
fn parse_chart(xml: &[u8]) -> ParsedChart {
    let mut chart = ParsedChart {
        chart_type: "bar".to_string(),
        title: String::new(),
        legend_pos: None,
        series: Vec::new(),
    };

    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut in_title = false;
    let mut plot_area_seen = false;
    let mut title_text = String::new();

    let stack_ends_with = |stack: &Vec<Vec<u8>>, expected: &[&[u8]]| -> bool {
        if stack.len() < expected.len() {
            return false;
        }
        let start = stack.len() - expected.len();
        expected
            .iter()
            .enumerate()
            .all(|(i, exp)| stack[start + i].as_slice() == *exp)
    };

    let ser_index = |stack: &Vec<Vec<u8>>| -> usize {
        stack.iter().filter(|s| s.as_slice() == b"ser").count()
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_q = e.name();
                let name = local_name(name_q.as_ref());
                match name {
                    b"plotArea" => plot_area_seen = true,
                    b"title" => in_title = true,
                    b"ser" => chart.series.push(ParsedSeries {
                        name: String::new(),
                        cat_ref: String::new(),
                        val_ref: String::new(),
                    }),
                    _ => {}
                }
                if !plot_area_seen && matches!(name, b"barChart" | b"lineChart" | b"pieChart"
                    | b"scatterChart" | b"doughnutChart" | b"areaChart" | b"radarChart"
                    | b"bar3DChart" | b"line3DChart" | b"pie3DChart" | b"colChart"
                    | b"col3DChart" | b"area3DChart" | b"doughnut3DChart")
                {
                    // Chart type element appears inside plotArea; remember the
                    // first one encountered. The plotArea_started flag is set
                    // when the chart-type element itself is seen.
                    plot_area_seen = true;
                    chart.chart_type = map_chart_type(name).to_string();
                }
                stack.push(name.to_vec());
            }
            Ok(Event::Empty(ref e)) => {
                let name_q = e.name();
                let name = local_name(name_q.as_ref());
                if name == b"legendPos" {
                    for attr in e.attributes().flatten() {
                        if local_name(attr.key.as_ref()) == b"val" {
                            chart.legend_pos = Some(attr_value(&attr));
                        }
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                let Ok(text) = t.unescape() else {
                    continue;
                };
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                if in_title
                    && !stack.iter().any(|s| s.as_slice() == b"plotArea")
                    && stack_ends_with(&stack, &[b"title", b"tx", b"rich", b"p", b"r", b"t"])
                {
                    title_text.push_str(text);
                    continue;
                }
                if stack_ends_with(&stack, &[b"ser", b"tx", b"strRef", b"f"])
                    || stack_ends_with(&stack, &[b"ser", b"tx", b"v"])
                {
                    let idx = ser_index(&stack).saturating_sub(1);
                    if let Some(s) = chart.series.get_mut(idx) {
                        s.name.push_str(text);
                    }
                } else if stack_ends_with(&stack, &[b"ser", b"cat", b"strRef", b"f"])
                    || stack_ends_with(&stack, &[b"ser", b"cat", b"numRef", b"f"])
                    || stack_ends_with(&stack, &[b"ser", b"cat", b"strLit", b"pt", b"v"])
                    || stack_ends_with(&stack, &[b"ser", b"cat", b"numLit", b"pt", b"v"])
                {
                    let idx = ser_index(&stack).saturating_sub(1);
                    if let Some(s) = chart.series.get_mut(idx) {
                        s.cat_ref.push_str(text);
                    }
                } else if stack_ends_with(&stack, &[b"ser", b"val", b"numRef", b"f"])
                    || stack_ends_with(&stack, &[b"ser", b"val", b"strRef", b"f"])
                    || stack_ends_with(&stack, &[b"ser", b"val", b"numLit", b"pt", b"v"])
                    || stack_ends_with(&stack, &[b"ser", b"val", b"strLit", b"pt", b"v"])
                {
                    let idx = ser_index(&stack).saturating_sub(1);
                    if let Some(s) = chart.series.get_mut(idx) {
                        s.val_ref.push_str(text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name_q = e.name();
                let name = local_name(name_q.as_ref());
                if name == b"title" {
                    in_title = false;
                }
                stack.pop();
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    chart.title = title_text.trim().to_string();
    chart
}

fn parse_range_bounds(range: &str) -> Option<(u32, u32, u32, u32)> {
    let cell_part = range.split('!').next_back().unwrap_or(range);
    let cell_part = cell_part.replace('$', "");
    let parts: Vec<&str> = cell_part.split(':').collect();
    if parts.len() > 2 {
        return None;
    }
    let (r1, c1) = parse_cell_ref(parts[0])?;
    let (r2, c2) = parts
        .get(1)
        .and_then(|p| parse_cell_ref(p))
        .unwrap_or((r1, c1));
    Some((r1.min(r2), c1.min(c2), r1.max(r2), c1.max(c2)))
}

/// Resolves a chart range reference against the matching worksheet and
/// returns the cell strings in row-major order.
fn resolve_range_values(
    range: &str,
    default_ws: usize,
    worksheets: &[Worksheet],
) -> Vec<f64> {
    let Some((ws_idx, r0, _c0, r1, c1)) = resolve_ref(range, default_ws, worksheets) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for r in r0..=r1 {
        for c in _c0..=c1 {
            let key = format!("{r},{c}");
            if let Some(cell) = worksheets[ws_idx].data.get(&key) {
                if let Some(num) = parse_number(cell) {
                    values.push(num);
                }
            }
        }
    }
    values
}

fn resolve_range_labels(
    range: &str,
    default_ws: usize,
    worksheets: &[Worksheet],
) -> Vec<String> {
    let Some((ws_idx, r0, c0, r1, c1)) = resolve_ref(range, default_ws, worksheets) else {
        return Vec::new();
    };
    let mut labels = Vec::new();
    for r in r0..=r1 {
        for c in c0..=c1 {
            let key = format!("{r},{c}");
            if let Some(cell) = worksheets[ws_idx].data.get(&key) {
                labels.push(cell.value.clone().unwrap_or_default());
            }
        }
    }
    labels
}

fn resolve_single_cell(
    range: &str,
    default_ws: usize,
    worksheets: &[Worksheet],
) -> Option<String> {
    let (ws_idx, r0, _c0, _r1, _c1) = resolve_ref(range, default_ws, worksheets)?;
    let key = format!("{r0},{_c0}");
    worksheets[ws_idx].data.get(&key).and_then(|c| c.value.clone())
}

fn resolve_ref(
    range: &str,
    default_ws: usize,
    worksheets: &[Worksheet],
) -> Option<(usize, u32, u32, u32, u32)> {
    let (sheet_part, cell_part) = match range.split_once('!') {
        Some((s, c)) => (Some(s), c),
        None => (None, range),
    };
    let ws_idx = match sheet_part {
        Some(name) => {
            let name = name.trim_matches('\'');
            worksheets
                .iter()
                .position(|w| w.name.eq_ignore_ascii_case(name))
                .unwrap_or(default_ws)
        }
        None => default_ws,
    };
    let (r0, c0, r1, c1) = parse_range_bounds(cell_part)?;
    Some((ws_idx, r0, c0, r1, c1))
}

fn parse_number(cell: &crate::types::CellData) -> Option<f64> {
    if let Some(typed) = &cell.typed {
        if let botsheet_core::engine::value::CellValue::Number(n) = typed {
            return Some(*n);
        }
    }
    cell.value
        .as_deref()
        .and_then(|v| v.trim().parse::<f64>().ok())
}

fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let mut col_str = String::new();
    let mut row_str = String::new();

    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            col_str.push(c.to_ascii_uppercase());
        } else if c.is_ascii_digit() {
            row_str.push(c);
        }
    }

    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }

    let col = col_str
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    let row: u32 = row_str.parse().ok()?;

    Some((row.saturating_sub(1), col.saturating_sub(1)))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_real_openpyxl_chart() {
        let bytes = std::fs::read("/tmp/test_chart.xlsx").expect("test file");
        let workbook = umya_spreadsheet::reader::xlsx::read_reader(
            std::io::Cursor::new(&bytes),
            true,
        )
        .expect("parse");
        let mut worksheets = Vec::new();
        for sheet in workbook.get_sheet_collection() {
            let mut data = HashMap::new();
            let (max_col, max_row) = sheet.get_highest_column_and_row();
            for row in 1..=max_row {
                for col in 1..=max_col {
                    if let Some(cell) = sheet.get_cell((col, row)) {
                        let value = cell.get_value().to_string();
                        if !value.is_empty() {
                            data.insert(
                                format!("{},{}", row - 1, col - 1),
                                crate::types::CellData {
                                    value: Some(value),
                                    typed: None,
                                    formula: None,
                                    style: None,
                                    format: None,
                                    note: None,
                                    locked: None,
                                    has_comment: None,
                                    array_formula_id: None,
                                },
                            );
                        }
                    }
                }
            }
            worksheets.push(Worksheet {
                name: sheet.get_name().to_string(),
                data,
                ..Worksheet::default()
            });
        }
        let charts = extract_charts(&bytes, &worksheets).expect("extract");
        assert!(!charts.is_empty(), "no charts extracted at all");
        let ws0 = &charts[0];
        assert!(!ws0.is_empty(), "worksheet 0 has no charts");
        let c = &ws0[0];
        assert_eq!(c.chart_type, "bar");
        assert_eq!(c.title, "Vendas por Mês");
        assert!(!c.labels.is_empty());
        assert_eq!(c.labels, vec!["Jan", "Fev", "Mar", "Abr"]);
        assert_eq!(c.datasets.len(), 1);
        assert_eq!(c.datasets[0].data, vec![110.0, 150.0, 130.0, 170.0]);
    }
}
