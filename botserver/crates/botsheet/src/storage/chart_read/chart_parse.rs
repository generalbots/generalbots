//! Chart XML parsing (split from `chart_read` to respect the file-size ceiling).
//!
//! Walks a `xl/charts/chartN.xml` part and rebuilds the chart type, title,
//! legend position and series references into a [`ParsedChart`], which the
//! orchestrator then resolves against worksheet cell values.

use quick_xml::events::Event;

use super::{attr_value, local_name};

pub(crate) struct ParsedSeries {
    pub(crate) name: String,
    pub(crate) cat_ref: String,
    pub(crate) val_ref: String,
}

pub(crate) struct ParsedChart {
    pub(crate) chart_type: String,
    pub(crate) title: String,
    pub(crate) legend_pos: Option<String>,
    pub(crate) series: Vec<ParsedSeries>,
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
pub(crate) fn parse_chart(xml: &[u8]) -> ParsedChart {
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
