//! PDF entry points.

use botsheet_core::types::Spreadsheet;

use super::options::PdfOptions;
use super::writer::PdfWriter;

/// Renders every worksheet of the spreadsheet as paginated PDF bytes.
pub fn export_pdf(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    export_pdf_with(sheet, PdfOptions::default())
}

pub fn export_pdf_with(sheet: &Spreadsheet, options: PdfOptions) -> Result<Vec<u8>, String> {
    let mut writer = PdfWriter::new(options);
    for worksheet in &sheet.worksheets {
        writer.add_worksheet(worksheet.name.as_str(), &worksheet.data)?;
    }
    if writer.page_count() == 0 {
        writer.add_blank_page(sheet.name.as_str());
    }
    Ok(writer.finish())
}
