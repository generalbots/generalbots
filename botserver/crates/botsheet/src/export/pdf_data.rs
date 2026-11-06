//! PDF façade re-exported by the data exporters.

use botsheet_core::types::Spreadsheet;

pub fn export_to_pdf_data(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    crate::pdf::export_pdf(sheet)
}

pub fn export_to_pdf_data_with(
    sheet: &Spreadsheet,
    landscape: bool,
    scale: f32,
) -> Result<Vec<u8>, String> {
    crate::pdf::export_pdf_with(sheet, crate::pdf::PdfOptions { landscape, scale })
}
