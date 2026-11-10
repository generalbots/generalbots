//! PDF object and page writer.

use std::collections::HashMap;

use botsheet_core::types::CellData;

use super::options::PdfOptions;
use super::text::{cell_to_text, column_letter, parse_key, text_cmd};

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

const FONT_SIZE: f32 = 8.5;

/// Object numbers of the three structural objects emitted before the pages.
const HEADER_OBJECTS: u32 = 3;

/// A cell extracted for rendering, sorted into row-major order.
struct GridCell {
    row: u32,
    col: u32,
    value: String,
}

/// Accumulates PDF objects and emits the final byte stream with a valid
/// xref table and trailer.
pub(super) struct PdfWriter {
    objects: Vec<Vec<u8>>,
    page_refs: Vec<u32>,
    page_w: f32,
    page_h: f32,
    col_width: f32,
    row_height: f32,
    font_size: f32,
    rows_per_page: usize,
}

impl PdfWriter {
    pub(super) fn new(options: PdfOptions) -> Self {
        let (page_w, page_h) = if options.landscape {
            (PAGE_HEIGHT, PAGE_WIDTH)
        } else {
            (PAGE_WIDTH, PAGE_HEIGHT)
        };
        let scale = if options.scale.is_finite() {
            options.scale.clamp(0.5, 2.0)
        } else {
            1.0
        };
        let row_height = ROW_HEIGHT * scale;
        let rows_per_page = (((page_h - 2.0 * MARGIN - 18.0) / row_height).floor() as usize)
            .clamp(1, ROWS_PER_PAGE * 2);
        Self {
            objects: Vec::new(),
            page_refs: Vec::new(),
            page_w,
            page_h,
            col_width: COL_WIDTH * scale,
            row_height,
            font_size: FONT_SIZE * scale,
            rows_per_page,
        }
    }

    pub(super) fn page_count(&self) -> usize {
        self.page_refs.len()
    }

    /// Pushes an object body and returns its absolute object number.
    fn push_object(&mut self, body: &[u8]) -> u32 {
        self.objects.push(body.to_vec());
        (self.objects.len() as u32) + HEADER_OBJECTS
    }

    /// Renders one worksheet onto as many pages as its grid needs. Rows with
    /// no data are skipped, so a sparse sheet prints compactly.
    pub(super) fn add_worksheet(
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
        cells.sort_by_key(|a| (a.row, a.col));

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

        for chunk in rows.chunks(self.rows_per_page) {
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
        stream.push_str(&format!("{} Tf\n", self.font_size));

        let top = self.page_h - MARGIN;
        let first = rows.first().map(|(r, _)| *r).unwrap_or(0);
        let last = rows.last().map(|(r, _)| *r).unwrap_or(first);
        stream.push_str(&text_cmd(MARGIN, top - 4.0, sheet_name));
        if last != first {
            stream.push_str(&text_cmd(
                MARGIN + 260.0,
                top - 4.0,
                &format!("rows {}-{}", first + 1, last + 1),
            ));
        }

        let header_y = top - 18.0;
        stream.push_str(&text_cmd(MARGIN, header_y, "#"));
        for col in 0..=max_col {
            let x = MARGIN + ADDRESS_WIDTH + col as f32 * self.col_width;
            stream.push_str(&text_cmd(x, header_y, &column_letter(col)));
        }

        for (slot, (row, cells_of_row)) in rows.iter().enumerate() {
            let y = top - 18.0 - (slot as u32 + 1) as f32 * self.row_height - self.font_size;
            stream.push_str(&text_cmd(MARGIN, y, &(row + 1).to_string()));
            for cell in cells_of_row {
                let x = MARGIN + ADDRESS_WIDTH + cell.col as f32 * self.col_width;
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
            w = self.page_w as u32,
            h = self.page_h as u32,
            stream = stream_obj,
        );
        let page_ref = self.push_object(page_body.as_bytes());
        self.page_refs.push(page_ref);
        Ok(())
    }

    pub(super) fn add_blank_page(&mut self, sheet_name: &str) {
        let stream = format!(
            "BT\n/F1 {} Tf\n{} ET",
            self.font_size,
            text_cmd(MARGIN, self.page_h - MARGIN - 4.0, sheet_name)
        );
        if let Err(e) = self.add_page_object(stream.as_bytes()) {
            log::warn!("blank PDF page emission failed: {e}");
        }
    }

    /// Serializes the object list with the catalog, pages tree, font object,
    /// xref table and trailer.
    pub(super) fn finish(self) -> Vec<u8> {
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

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
