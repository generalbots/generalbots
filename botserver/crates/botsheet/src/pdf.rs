//! Minimal PDF 1.4 exporter (#788).
//!
//! Produces a dependency-free PDF: one A4 page set per worksheet, each grid
//! rendered as a table of Helvetica text with a repeated header row. TrueType
//! embedding, colors and vector graphics are intentionally out of scope; the
//! goal is a faithful, printable snapshot of the data — not a designer layout.

use std::collections::HashMap;

use crate::types::{CellData, Spreadsheet};

/// A4 page size in PostScript points (1 pt = 1/72 inch).
const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;

/// Margins around the grid.
const MARGIN: f32 = 48.0;

/// Height of one grid row, including the header.
const ROW_HEIGHT: f32 = 15.0;

/// Width of the row-address gutter.
const ADDRESS_WIDTH: f32 = 42.0;

/// Width of one data column.
const COL_WIDTH: f32 = 96.0;

/// Rows per page; the remaining cells flow onto continuation pages.
const ROWS_PER_PAGE: usize = 48;

/// Longest cell text rendered; longer values are truncated with an ellipsis.
const MAX_CELL_CHARS: usize = 16;

const FONT_SIZE: f32 = 8.5;

/// Object numbers of the three structural objects emitted before the pages.
const HEADER_OBJECTS: u32 = 3;

/// Renders every worksheet of the spreadsheet as paginated PDF bytes.
pub fn export_pdf(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    let mut writer = PdfWriter::new();
    for worksheet in &sheet.worksheets {
        writer.add_worksheet(worksheet.name.as_str(), &worksheet.data)?;
    }
    if writer.page_count() == 0 {
        writer.add_blank_page(sheet.name.as_str());
    }
    Ok(writer.finish())
}

/// A cell extracted for rendering, sorted into row-major order.
struct GridCell {
    row: u32,
    col: u32,
    value: String,
}

/// Accumulates PDF objects and emits the final byte stream with a valid
/// xref table and trailer.
struct PdfWriter {
    objects: Vec<Vec<u8>>,
    page_refs: Vec<u32>,
}

impl PdfWriter {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            page_refs: Vec::new(),
        }
    }

    fn page_count(&self) -> usize {
        self.page_refs.len()
    }

    /// Pushes an object body and returns its absolute object number.
    fn push_object(&mut self, body: &[u8]) -> u32 {
        self.objects.push(body.to_vec());
        (self.objects.len() as u32) + HEADER_OBJECTS
    }

    /// Renders one worksheet onto as many pages as its grid needs. Rows with
    /// no data are skipped, so a sparse sheet prints compactly.
    fn add_worksheet(
        &mut self,
        name: &str,
        data: &HashMap<String, CellData>,
    ) -> Result<(), String> {
        let mut cells: Vec<GridCell> = Vec::new();
        let mut max_col: u32 = 0;
        for (key, cell) in data {
            let Some((row, col)) = parse_key(key) else {
                continue;
            };
            let Some(value) = cell.value.as_deref() else {
                continue;
            };
            if value.trim().is_empty() {
                continue;
            }
            max_col = max_col.max(col);
            cells.push(GridCell {
                row,
                col,
                value: cell_to_text(value),
            });
        }
        cells.sort_by(|a, b| (a.row, a.col).cmp(&(b.row, b.col)));

        if cells.is_empty() {
            return Ok(());
        }

        // Group cells into rows, keeping the row-major order.
        let mut rows: Vec<(u32, Vec<&GridCell>)> = Vec::new();
        for cell in &cells {
            match rows.last_mut() {
                Some((row, group)) if *row == cell.row => group.push(cell),
                _ => rows.push((cell.row, vec![cell])),
            }
        }

        for chunk in rows.chunks(ROWS_PER_PAGE) {
            self.add_page(name, max_col, chunk)?;
        }
        Ok(())
    }

    /// Emits one page: title, column-letter header, then data rows.
    fn add_page(
        &mut self,
        sheet_name: &str,
        max_col: u32,
        rows: &[(u32, Vec<&GridCell>)],
    ) -> Result<(), String> {
        let mut stream = String::new();
        stream.push_str("BT\n/F1 ");
        stream.push_str(&format!("{FONT_SIZE} Tf\n"));

        let top = PAGE_HEIGHT - MARGIN;
        let first = rows.first().map(|(r, _)| *r).unwrap_or(0);
        let last = rows.last().map(|(r, _)| *r).unwrap_or(first);
        stream.push_str(&text_cmd(MARGIN, top - 4.0, sheet_name));
        if last != first {
            stream.push_str(&text_cmd(
                MARGIN + 260.0,
                top - 4.0,
                &format!("rows {}–{}", first + 1, last + 1),
            ));
        }

        let header_y = top - 18.0;
        stream.push_str(&text_cmd(MARGIN, header_y, "#"));
        for col in 0..=max_col {
            let x = MARGIN + ADDRESS_WIDTH + col as f32 * COL_WIDTH;
            stream.push_str(&text_cmd(x, header_y, &column_letter(col)));
        }

        for (slot, (row, cells_of_row)) in rows.iter().enumerate() {
            let y = top - 18.0 - (slot as u32 + 1) as f32 * ROW_HEIGHT - FONT_SIZE;
            stream.push_str(&text_cmd(MARGIN, y, &(row + 1).to_string()));
            for cell in cells_of_row {
                let x = MARGIN + ADDRESS_WIDTH + cell.col as f32 * COL_WIDTH;
                stream.push_str(&text_cmd(x, y, &cell.value));
            }
        }

        stream.push_str("ET");
        self.add_page_object(stream.as_bytes())
    }

    fn add_page_object(&mut self, stream: &[u8]) -> Result<(), String> {
        let stream_obj = self.push_object(stream);
        let page_body = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] \
             /Resources << /Font << /F1 3 0 R >> >> /Contents {stream} 0 R >>",
            w = PAGE_WIDTH as u32,
            h = PAGE_HEIGHT as u32,
            stream = stream_obj,
        );
        let page_ref = self.push_object(page_body.as_bytes());
        self.page_refs.push(page_ref);
        Ok(())
    }

    fn add_blank_page(&mut self, sheet_name: &str) {
        let stream = format!(
            "BT\n/F1 {FONT_SIZE} Tf\n{} ET",
            text_cmd(MARGIN, PAGE_HEIGHT - MARGIN - 4.0, sheet_name)
        );
        if let Err(e) = self.add_page_object(stream.as_bytes()) {
            log::warn!("blank PDF page emission failed: {e}");
        }
    }

    /// Serializes the object list with the catalog, pages tree, font object,
    /// xref table and trailer.
    fn finish(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(4096);
        out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
        let mut offsets: Vec<usize> = Vec::with_capacity(self.objects.len() + HEADER_OBJECTS as usize);

        let mut emit = |out: &mut Vec<u8>, body: &[u8]| -> u32 {
            let obj_num = (offsets.len() + 1) as u32;
            offsets.push(out.len());
            out.extend_from_slice(format!("{obj_num} 0 obj\n").as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
            obj_num
        };

        emit(&mut out, b"<< /Type /Catalog /Pages 2 0 R >>");
        let pages_kids: Vec<String> = self
            .page_refs
            .iter()
            .map(|p| format!("{p} 0 R"))
            .collect();
        let pages_body = format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            pages_kids.join(" "),
            self.page_refs.len()
        );
        emit(&mut out, pages_body.as_bytes());
        emit(
            &mut out,
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
        );
        for body in &self.objects {
            emit(&mut out, body);
        }

        let xref_offset = out.len();
        out.extend_from_slice(b"xref\n");
        out.extend_from_slice(format!("0 {}\n", offsets.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &offsets {
            out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                offsets.len() + 1
            )
            .as_bytes(),
        );
        out
    }
}

/// One positioned text run: `x y Tm (escaped) Tj`.
fn text_cmd(x: f32, y: f32, text: &str) -> String {
    format!("1 0 0 1 {x:.1} {y:.1} Tm ({}) Tj\n", escape_text(text))
}

/// Escapes PDF string specials and replaces characters outside WinAnsi.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            ch if (' '..='~').contains(&ch) => out.push(ch),
            _ => out.push('?'),
        }
    }
    out
}

/// Zero-based column index to spreadsheet letters (`0 -> A`, `27 -> AB`).
fn column_letter(col: u32) -> String {
    if col > 16_383 {
        return "?".to_string();
    }
    let mut name = String::new();
    let mut n = col + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        name.insert(0, (b'A' + rem as u8) as char);
        n = (n - 1) / 26;
    }
    name
}

/// Truncates long values to keep a page legible.
fn cell_to_text(value: &str) -> String {
    if value.chars().count() <= MAX_CELL_CHARS {
        return value.to_string();
    }
    let truncated: String = value.chars().take(MAX_CELL_CHARS - 1).collect();
    format!("{truncated}…")
}

fn parse_key(key: &str) -> Option<(u32, u32)> {
    let (row, col) = key.split_once(',')?;
    Some((row.parse().ok()?, col.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

fn ws() -> Spreadsheet {
    let mut data = HashMap::new();
    data.insert(
        "0,0".to_string(),
        CellData {
            value: Some("Name".to_string()),
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
    data.insert(
        "1,0".to_string(),
        CellData {
            value: Some("Alice".to_string()),
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
    data.insert(
        "1,1".to_string(),
        CellData {
            value: Some("42".to_string()),
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
    data.insert(
        "5,2".to_string(),
        CellData {
            value: Some("Deep (parens) value\\with\\slashes".to_string()),
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
    sheet_with(data, "Report", "Sheet1")
}

fn sheet_with(data: HashMap<String, CellData>, name: &str, ws_name: &str) -> Spreadsheet {
    Spreadsheet {
        id: "s".to_string(),
        name: name.to_string(),
        owner_id: "o".to_string(),
        worksheets: vec![crate::types::Worksheet {
            name: ws_name.to_string(),
            data,
            ..crate::types::Worksheet::default()
        }],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: HashMap::new(),
    }
}

    #[test]
    fn produces_well_formed_pdf() {
        let pdf = export_pdf(&ws()).expect("pdf export should succeed");
        let header = &pdf[..8];
        assert_eq!(header, b"%PDF-1.4");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("endobj"));
        assert!(text.contains("startxref"));
        assert!(text.contains("%%EOF"));
        assert!(text.contains("/Helvetica"));
        assert!(text.contains("(Alice)"));
        // The parens and backslashes must be escaped in the content stream.
        assert!(text.contains("\\("));
        assert!(text.contains("\\"));
    }

    #[test]
    fn empty_spreadsheet_still_renders() {
        let empty = sheet_with(HashMap::new(), "Blank", "Sheet1");
        let pdf = export_pdf(&empty).expect("blank export should succeed");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("%%EOF"));
    }
}