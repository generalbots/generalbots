//! Native PowerPoint table writer (#2457).
//!
//! A table element becomes a `p:graphicFrame` containing an `a:tbl`, so grid
//! geometry, borders, cell fills and merged cells survive export. Cell text is
//! escaped, dimensions are bounded, and every indexed access is checked — the
//! exporter runs on untrusted presentation JSON.

use super::export::{emu, fill_xml, hex_color, xml_escape, xfrm_emu_xml};
use crate::types::SlideElement;

const MAX_ROWS: usize = 50;
const MAX_COLS: usize = 20;
const MAX_CELL_CHARS: usize = 1_000;
const MAX_PX: f64 = 20_000.0;
const MIN_EMU: i64 = 9_525; // 0.75pt, keeps zero-sized grid cells addressable

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Merge {
    None,
    Horizontal,
    Vertical,
}

const CELL_BORDERS: &str = concat!(
    "<a:lnL w=\"6350\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:lnL>",
    "<a:lnR w=\"6350\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:lnR>",
    "<a:lnT w=\"6350\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:lnT>",
    "<a:lnB w=\"6350\"><a:solidFill><a:srgbClr val=\"D9D9D9\"/></a:solidFill></a:lnB>",
);

pub(super) fn is_table_element(element: &SlideElement) -> bool {
    element.element_type == "table" && element.content.table_data.is_some()
}

fn truncate(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

fn dimension(px: f64, fallback_px: f64) -> i64 {
    if !px.is_finite() || px <= 0.0 {
        emu(fallback_px)
    } else {
        emu(px.min(MAX_PX))
    }
}

fn offset(px: f64) -> i64 {
    if px.is_finite() {
        emu(px.clamp(-MAX_PX, MAX_PX))
    } else {
        0
    }
}

fn fallback_px(total: f64, count: usize) -> f64 {
    let each = total / count as f64;
    if each.is_finite() && each > 0.0 {
        each.min(MAX_PX)
    } else {
        100.0
    }
}

fn cell_paragraphs(element: &SlideElement, cell: Option<&crate::types::TableCell>) -> String {
    let style = cell.and_then(|value| value.style.as_ref());
    let text = truncate(cell.map(|value| value.content.as_str()).unwrap_or(""), MAX_CELL_CHARS);
    let font_size = style
        .and_then(|value| value.font_size)
        .or(element.style.font_size)
        .unwrap_or(14.0)
        .clamp(6.0, 72.0);
    let sz = (font_size * 100.0).round() as i32;
    let color = hex_color(
        style
            .and_then(|value| value.color.as_ref())
            .or(element.style.color.as_ref()),
        "000000",
    );
    let font = truncate(
        style
            .and_then(|value| value.font_family.as_deref())
            .or(element.style.font_family.as_deref())
            .unwrap_or("Arial"),
        80,
    );
    let font = xml_escape(&font);
    let bold = if style
        .and_then(|value| value.font_weight.as_deref())
        .or(element.style.font_weight.as_deref())
        == Some("bold")
    {
        " b=\"1\""
    } else {
        ""
    };

    let mut paragraphs = String::new();
    for line in text.split('\n') {
        paragraphs.push_str(&format!(
            "<a:p><a:r><a:rPr lang=\"en-US\" sz=\"{sz}\"{bold}><a:solidFill><a:srgbClr val=\"{color}\"/></a:solidFill><a:latin typeface=\"{font}\"/></a:rPr><a:t>{}</a:t></a:r><a:endParaRPr lang=\"en-US\"/></a:p>",
            xml_escape(line)
        ));
    }
    paragraphs
}

/// Write one table as a native DrawingML table frame. Returns an empty string
/// only for a malformed element with no table data (nothing to export).
pub(super) fn table_shape_xml(element: &SlideElement, id: usize) -> String {
    let Some(data) = element.content.table_data.as_ref() else {
        return String::new();
    };

    // Trust the grid declared by the writer, but never let a hostile payload
    // grow it beyond the caps or disagree with the rows actually supplied.
    let from_cells = data.cells.len().min(MAX_ROWS);
    let widest = data.cells.iter().map(Vec::len).max().unwrap_or(0);
    let rows = data.rows.min(MAX_ROWS).max(from_cells).max(1);
    let cols = data.cols.min(MAX_COLS).max(widest.min(MAX_COLS)).max(1);

    let col_fallback = fallback_px(element.width, cols);
    let row_fallback = fallback_px(element.height, rows);
    let widths: Vec<i64> = (0..cols)
        .map(|col| {
            dimension(data.col_widths.get(col).copied().unwrap_or(col_fallback), col_fallback)
                .max(MIN_EMU)
        })
        .collect();
    let heights: Vec<i64> = (0..rows)
        .map(|row| {
            dimension(data.row_heights.get(row).copied().unwrap_or(row_fallback), row_fallback)
                .max(MIN_EMU)
        })
        .collect();
    let cx: i64 = widths.iter().copied().sum();
    let cy: i64 = heights.iter().copied().sum();

    let mut covered = vec![vec![Merge::None; cols]; rows];
    let mut body = String::new();
    for row in 0..rows {
        let mut cells = String::new();
        for col in 0..cols {
            let merge = covered
                .get(row)
                .and_then(|line| line.get(col))
                .copied()
                .unwrap_or(Merge::None);
            if merge != Merge::None {
                let tag = if merge == Merge::Horizontal { "hMerge" } else { "vMerge" };
                cells.push_str(&format!(
                    "<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p/></a:txBody><a:tcPr {tag}=\"1\"/></a:tc>"
                ));
                continue;
            }

            let cell = data.cells.get(row).and_then(|line| line.get(col));
            let colspan = cell.map(|value| value.colspan.max(1)).unwrap_or(1).min(cols - col);
            let rowspan = cell.map(|value| value.rowspan.max(1)).unwrap_or(1).min(rows - row);
            for next_col in col + 1..col + colspan {
                if let Some(slot) = covered.get_mut(row).and_then(|line| line.get_mut(next_col)) {
                    *slot = Merge::Horizontal;
                }
            }
            for next_row in row + 1..row + rowspan {
                for next_col in col..col + colspan {
                    if let Some(slot) = covered.get_mut(next_row).and_then(|line| line.get_mut(next_col)) {
                        *slot = Merge::Vertical;
                    }
                }
            }

            let mut attrs = String::new();
            if colspan > 1 {
                attrs.push_str(&format!(" gridSpan=\"{colspan}\""));
            }
            if rowspan > 1 {
                attrs.push_str(&format!(" rowSpan=\"{rowspan}\""));
            }
            let cell_style = cell.and_then(|value| value.style.as_ref());
            let fill = match cell_style {
                Some(style) => fill_xml(style, "FFFFFF"),
                None => fill_xml(&element.style, "FFFFFF"),
            };
            let paragraphs = cell_paragraphs(element, cell);
            cells.push_str(&format!(
                "<a:tc><a:txBody><a:bodyPr wrap=\"square\"/><a:lstStyle/>{paragraphs}</a:txBody>\
                 <a:tcPr{attrs}>{CELL_BORDERS}{fill}</a:tcPr></a:tc>"
            ));
        }
        let height = heights.get(row).copied().unwrap_or(MIN_EMU);
        body.push_str(&format!("<a:tr h=\"{height}\">{cells}</a:tr>"));
    }

    let grid = widths
        .iter()
        .map(|width| format!("<a:gridCol w=\"{width}\"/>"))
        .collect::<String>();
    let xfrm = xfrm_emu_xml(offset(element.x), offset(element.y), cx, cy);
    format!(
        "<p:graphicFrame><p:nvGraphicFramePr>\
         <p:cNvPr id=\"{id}\" name=\"Table {id}\"/>\
         <p:cNvGraphicFramePr><a:graphicFrameLocks noGrp=\"1\"/></p:cNvGraphicFramePr><p:nvPr/>\
         </p:nvGraphicFramePr>{xfrm}<a:graphic>\
         <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/table\">\
         <a:tbl><a:tblPr firstRow=\"1\" bandRow=\"1\"/><a:tblGrid>{grid}</a:tblGrid>{body}</a:tbl>\
         </a:graphicData></a:graphic></p:graphicFrame>"
    )
}
