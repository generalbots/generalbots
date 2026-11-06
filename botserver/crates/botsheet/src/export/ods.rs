//! ODS export (ODF package backed by a ZIP container).

use botsheet_core::types::Spreadsheet;

use super::util::cell_number;

fn build_ods_content(sheet: &Spreadsheet) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
office:version="1.2">
<office:body>
<office:spreadsheet>
"#);

    for ws in &sheet.worksheets {
        xml.push_str(&format!("<table:table table:name=\"{}\">\n", ws.name));

        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in ws.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        for _ in 0..=max_col {
            xml.push_str("<table:table-column/>\n");
        }

        for row in 0..=max_row {
            xml.push_str("<table:table-row>\n");
            for col in 0..=max_col {
                let key = format!("{row},{col}");
                let cell = ws.data.get(&key);
                let value = cell.and_then(|c| c.value.clone()).unwrap_or_default();
                let formula = cell.and_then(|c| c.formula.clone());

                if let Some(f) = formula {
                    xml.push_str(&format!(
                        "<table:table-cell table:formula=\"{}\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        f, value
                    ));
                } else if let Some(num) = cell.and_then(cell_number) {
                    xml.push_str(&format!(
                        "<table:table-cell office:value-type=\"float\" office:value=\"{}\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        num, value
                    ));
                } else {
                    xml.push_str(&format!(
                        "<table:table-cell office:value-type=\"string\">\n<text:p>{}</text:p>\n</table:table-cell>\n",
                        value
                    ));
                }
            }
            xml.push_str("</table:table-row>\n");
        }
        xml.push_str("</table:table>\n");
    }

    xml.push_str("</office:spreadsheet>\n</office:body>\n</office:document-content>");
    xml
}

/// ODS is an ODF package: a ZIP whose first entry is an uncompressed
/// `mimetype`, plus `content.xml` and the manifest. Returning the bare
/// content.xml (as this function used to) produced a 200 with a file no
/// office suite could open.
pub fn export_to_ods(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let content = build_ods_content(sheet);
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"#;

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let stored = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("mimetype", stored).map_err(|e| e.to_string())?;
        zip.write_all(b"application/vnd.oasis.opendocument.spreadsheet")
            .map_err(|e| e.to_string())?;
        zip.start_file("content.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("META-INF/manifest.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(cursor.into_inner())
}
