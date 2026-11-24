//! Native PowerPoint chart writer (#2457).
//!
//! A chart element becomes a `p:graphicFrame` pointing at a real chart part
//! (`ppt/charts/chartN.xml`) with cached categories and values. The values are
//! written into OOXML caches, not flattened into a slide text box. Bounds keep
//! a hostile payload from producing unbounded XML.

use super::export::{hex_color, xfrm_xml, xml_escape};
use crate::types::{ChartData, SlideElement};

const MAX_SERIES: usize = 8;
const MAX_POINTS: usize = 100;
const MAX_TEXT_CHARS: usize = 200;
const MAX_VALUE: f64 = 1e12;
const CAT_AXIS_ID: i64 = 111_111_111;
const VAL_AXIS_ID: i64 = 222_222_222;

/// A chart part staged alongside the slide that references it. `file_name` is
/// assigned by the exporter once the global chart sequence is known.
#[derive(Debug, Clone)]
pub(super) struct ChartPart {
    pub(super) rel_id: String,
    pub(super) file_name: String,
    pub(super) xml: String,
}

pub(super) fn is_chart_element(element: &SlideElement) -> bool {
    element.element_type == "chart" && element.content.chart_data.is_some()
}

fn truncate(text: &str) -> String {
    text.chars().take(MAX_TEXT_CHARS).collect()
}

fn series_xml(data: &ChartData) -> String {
    let mut series = String::new();
    for (index, dataset) in data.datasets.iter().take(MAX_SERIES).enumerate() {
        let count = dataset
            .data
            .len()
            .min(data.labels.len())
            .min(MAX_POINTS);
        let color = hex_color(Some(&dataset.color), "4F81BD");
        let series_letter = char::from(b'A' + index as u8);
        let mut categories = String::new();
        let mut values = String::new();
        for point in 0..count {
            let label = data.labels.get(point).map(String::as_str).unwrap_or("");
            let value = dataset.data.get(point).copied().unwrap_or(0.0);
            let value = if value.is_finite() {
                value.clamp(-MAX_VALUE, MAX_VALUE)
            } else {
                0.0
            };
            categories.push_str(&format!(
                "<c:pt idx=\"{point}\"><c:v>{}</c:v></c:pt>",
                xml_escape(&truncate(label))
            ));
            values.push_str(&format!("<c:pt idx=\"{point}\"><c:v>{value}</c:v></c:pt>"));
        }
        series.push_str(&format!(
            "<c:ser><c:idx val=\"{index}\"/><c:order val=\"{index}\"/>\
             <c:tx><c:v>{}</c:v></c:tx>\
             <c:spPr><a:solidFill><a:srgbClr val=\"{color}\"/></a:solidFill><a:ln><a:noFill/></a:ln></c:spPr>\
             <c:cat><c:strRef><c:f>Sheet1!$A$2:$A${}</c:f><c:strCache><c:ptCount val=\"{count}\"/>{categories}</c:strCache></c:strRef></c:cat>\
             <c:val><c:numRef><c:f>Sheet1!${series_letter}$2:${series_letter}${}</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"{count}\"/>{values}</c:numCache></c:numRef></c:val>\
             </c:ser>",
            xml_escape(&truncate(&dataset.label)),
            count + 1,
            count + 1
        ));
    }
    series
}

fn axes_xml(kind: &str) -> String {
    if kind == "pieChart" || kind == "doughnutChart" {
        return String::new();
    }
    format!(
        "<c:catAx><c:axId val=\"{CAT_AXIS_ID}\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling>\
         <c:delete val=\"0\"/><c:axPos val=\"b\"/><c:numFmt formatCode=\"General\" sourceLinked=\"0\"/>\
         <c:majorTickMark val=\"none\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/>\
         <c:crossAx val=\"{VAL_AXIS_ID}\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/>\
         <c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/></c:catAx>\
         <c:valAx><c:axId val=\"{VAL_AXIS_ID}\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling>\
         <c:delete val=\"0\"/><c:axPos val=\"l\"/><c:majorGridlines/>\
         <c:numFmt formatCode=\"General\" sourceLinked=\"0\"/>\
         <c:majorTickMark val=\"none\"/><c:minorTickMark val=\"none\"/><c:tickLblPos val=\"nextTo\"/>\
         <c:crossAx val=\"{CAT_AXIS_ID}\"/><c:crosses val=\"autoZero\"/><c:crossBetween val=\"between\"/></c:valAx>"
    )
}

fn chart_part_xml(data: &ChartData, title: Option<&str>) -> String {
    let (kind, bar_dir) = match data.chart_type.as_str() {
        "line" => ("lineChart", "col"),
        "pie" => ("pieChart", "col"),
        "doughnut" => ("doughnutChart", "col"),
        "area" => ("areaChart", "col"),
        "horizontal" => ("barChart", "bar"),
        "bar" | "column" => ("barChart", "col"),
        other => {
            log::warn!("pptx export: unknown chart type '{other}' — using a clustered column chart");
            ("barChart", "col")
        }
    };
    let series = series_xml(data);
    let group = match kind {
        "lineChart" => format!(
            "<c:lineChart><c:grouping val=\"standard\"/>{series}<c:marker val=\"1\"/>\
             <c:axId val=\"{CAT_AXIS_ID}\"/><c:axId val=\"{VAL_AXIS_ID}\"/></c:lineChart>"
        ),
        "areaChart" => format!(
            "<c:areaChart><c:grouping val=\"standard\"/>{series}\
             <c:axId val=\"{CAT_AXIS_ID}\"/><c:axId val=\"{VAL_AXIS_ID}\"/></c:areaChart>"
        ),
        "pieChart" => format!("<c:pieChart><c:varyColors val=\"1\"/>{series}</c:pieChart>"),
        "doughnutChart" => format!(
            "<c:doughnutChart><c:varyColors val=\"1\"/><c:holeSize val=\"50\"/>{series}</c:doughnutChart>"
        ),
        _ => format!(
            "<c:barChart><c:barDir val=\"{bar_dir}\"/><c:grouping val=\"clustered\"/>{series}\
             <c:axId val=\"{CAT_AXIS_ID}\"/><c:axId val=\"{VAL_AXIS_ID}\"/></c:barChart>"
        ),
    };

    let title_xml = match title.map(truncate).filter(|text| !text.trim().is_empty()) {
        Some(text) => format!(
            "<c:title><c:tx><c:rich><a:bodyPr/><a:lstStyle/><a:p>\
             <a:r><a:rPr lang=\"en-US\" sz=\"1400\"/><a:t>{}</a:t></a:r>\
             </a:p></c:rich></c:tx><c:overlay val=\"0\"/></c:title>",
            xml_escape(&text)
        ),
        None => String::new(),
    };

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
         <c:chart>{title_xml}<c:autoTitleDeleted val=\"0\"/>\
         <c:plotArea><c:layout/>{group}{}</c:plotArea>\
         <c:legend><c:legendPos val=\"b\"/><c:overlay val=\"0\"/></c:legend>\
         <c:plotVisOnly val=\"1\"/><c:dispBlanksAs val=\"gap\"/></c:chart></c:chartSpace>",
        axes_xml(kind)
    )
}

/// Write one chart frame, staging its chart part for the exporter to package.
pub(super) fn chart_frame_xml(
    element: &SlideElement,
    id: usize,
    charts: &mut Vec<ChartPart>,
) -> String {
    let Some(data) = element.content.chart_data.as_ref() else {
        return String::new();
    };
    let rel_id = format!("rIdC{}", charts.len() + 1);
    charts.push(ChartPart {
        rel_id: rel_id.clone(),
        file_name: String::new(),
        xml: chart_part_xml(data, element.content.text.as_deref()),
    });
    let xfrm = xfrm_xml(element.x, element.y, element.width, element.height);
    format!(
        "<p:graphicFrame><p:nvGraphicFramePr>\
         <p:cNvPr id=\"{id}\" name=\"Chart {id}\"/>\
         <p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/>\
         </p:nvGraphicFramePr>{xfrm}<a:graphic>\
         <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
         <c:chart r:id=\"{rel_id}\"/></a:graphicData></a:graphic></p:graphicFrame>"
    )
}
