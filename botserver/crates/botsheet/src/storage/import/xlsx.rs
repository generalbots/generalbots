use botcore::{ImportSize, WorkClass};
use botsheet_core::engine::from_xlsx_formula;
use botsheet_core::engine::value::CellValue;
use std::collections::HashMap;
use std::io::Read;

/// #2530 — hard cap on the number of cells materialised per worksheet during
/// import. Each cell allocates a heap `String` key plus a typed `CellData`;
/// uncapped imports of dense large workbooks grew botserver RSS past the
/// 1200 MB ceiling and timed out. Exceeding the cap fails closed with an
/// actionable message instead of exhausting process memory.
const MAX_IMPORT_CELLS_PER_SHEET: usize = 250_000;

/// Fail closed once a worksheet reaches the materialisation cap (#2530).
/// Pure helper so the boundary is unit-testable without building a huge workbook.
fn ensure_within_cell_cap(retained: usize, sheet_name: &str) -> Result<(), String> {
    if retained >= MAX_IMPORT_CELLS_PER_SHEET {
        return Err(format!(
            "worksheet '{sheet_name}' exceeds the {MAX_IMPORT_CELLS_PER_SHEET} cell import limit; \
             split the workbook or export a smaller range"
        ));
    }
    Ok(())
}

/// Counts `marker` occurrences across a chunked byte stream using only a
/// `marker.len() - 1` byte carry. A marker split across two reads is still
/// counted; memory is constant regardless of the part size (#2530 streaming).
fn count_marker_in_chunk(chunk: &[u8], marker: &[u8], carry: &mut Vec<u8>) -> u64 {
    let mut combined = Vec::with_capacity(carry.len().saturating_add(chunk.len()));
    combined.extend_from_slice(carry);
    combined.extend_from_slice(chunk);

    let mut count = 0u64;
    let mut index = 0usize;
    while index + marker.len() <= combined.len() {
        if &combined[index..index + marker.len()] == marker {
            count = count.saturating_add(1);
            index += marker.len();
        } else {
            index += 1;
        }
    }

    carry.clear();
    let keep = marker.len().saturating_sub(1).min(combined.len());
    let start = combined.len() - keep;
    carry.extend_from_slice(&combined[start..]);
    count
}

/// Streaming pre-parse row count for an OOXML package: reads only the
/// `xl/worksheets/*.xml` parts in 8 KiB chunks and stops as soon as the budget
/// is exceeded. Returns `0` for packages the zip reader cannot open; the real
/// parser below then reports the format error as before.
fn count_rows_streaming(bytes: &[u8], limit: u64) -> u64 {
    let Ok(mut archive) = zip::ZipArchive::new(std::io::Cursor::new(bytes)) else {
        return 0;
    };
    let mut total = 0u64;
    for index in 0..archive.len() {
        let Ok(mut entry) = archive.by_index(index) else {
            continue;
        };
        let name = match entry.name() {
            Ok(name) => name.into_owned(),
            Err(_) => continue,
        };
        if !name.starts_with("xl/worksheets/") || !name.ends_with(".xml") {
            continue;
        }
        let mut buffer = [0u8; 8192];
        let mut carry = Vec::new();
        loop {
            let read = match entry.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => size,
                Err(_) => break,
            };
            total =
                total.saturating_add(count_marker_in_chunk(&buffer[..read], b"<row ", &mut carry));
            if total > limit {
                return total;
            }
        }
    }
    total
}

pub fn parse_xlsx_to_worksheets(
    bytes: &[u8],
    ext: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    if ext == "ods" {
        // #2463 — route the drive-open path through the hardened ODF parser
        // too. The legacy `storage/import_ods.rs` ignores repeated
        // columns/rows, has no mimetype check and no embedded-object/XXE
        // refusal; the UI import path already uses this hardened parser.
        return super::odf::parse_ods_to_worksheets(bytes);
    }

    if ext == "xlsx" || ext == "xlsm" || ext == "xls" {
        // #2530 — reject oversized workbooks *before* umya builds the whole
        // workbook DOM. The row count comes from a bounded streaming scan of
        // the worksheet parts, so the check costs kilobytes, not the hundreds
        // of megabytes the parse would allocate.
        let budget = crate::handlers::admission::import_budget(WorkClass::SheetsImport);
        let rows = count_rows_streaming(bytes, budget.max_rows);
        if let Err(error) = budget.check(ImportSize {
            bytes: bytes.len() as u64,
            rows,
            ..ImportSize::default()
        }) {
            return Err(error.message());
        }

        // Repair the OOXML forms the reader cannot parse (self-closing
        // `<mergeCell/>`, emitted by our own export) and never let a library
        // panic escape into a request handler (#2152).
        let package = match super::super::xlsx_normalize::normalise_merge_cells(bytes) {
            Ok(package) => package,
            Err(error) => {
                log::warn!("xlsx normalisation skipped: {error}");
                bytes.to_vec()
            }
        };
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            umya_spreadsheet::reader::xlsx::read_reader(std::io::Cursor::new(package), true)
        }))
        .map_err(|_| log::error!("xlsx reader panicked on this package; treating it as unreadable"))
        .ok();
        if let Some(Ok(workbook)) = parsed {
            let mut worksheets = Vec::new();

            for sheet in workbook.sheet_collection() {
                let mut data: HashMap<String, crate::types::CellData> = HashMap::new();

                // Iterate actual cells (umya's map), not the 1..=max bounding box.
                // umya keys the map `(row, col)` (1-based) — destructuring it as
                // `(col, row)` transposed every imported workbook, so formulas
                // referenced mirrored cells and pivots summed the wrong columns
                // (#2152).
                for ((row, col), cell) in sheet.collection_to_hashmap() {
                    // #2530 — fail closed before the per-cell allocations push
                    // RSS past the admission ceiling. Count only cells that
                    // would be retained (the empty/no-formula check below).
                    ensure_within_cell_cap(data.len(), sheet.get_name())?;
                    let cell = cell.as_ref();
                    let value = cell.value().to_string();
                    let formula = if cell.formula().is_empty() {
                        None
                    } else {
                        Some(from_xlsx_formula(&format!("={}", cell.formula())))
                    };

                    if value.is_empty() && formula.is_none() {
                        continue;
                    }

                    let key = format!("{},{}", row - 1, col - 1);
                    let style = super::super::xlsx_write::extract_cell_style(cell);

                    // Format code (e.g. "$#,##0.00") so the type survives round-trip.
                    let number_format = cell
                        .style()
                        .number_format()
                        .map(|nf| nf.format_code().to_string())
                        .filter(|c| c != "General" && !c.is_empty());

                    // Typed value (#781/#785): keeps the real cell type for
                    // format rendering, sort and filter. A text cell whose
                    // content is numeric-looking (`0123`, a phone number, a
                    // zip code) must stay `Text` — parsing it back from the
                    // display string would turn it into a number and drop a
                    // leading zero.
                    let typed = match cell.raw_value() {
                        umya_spreadsheet::structs::CellRawValue::Numeric(n) => {
                            CellValue::Number(*n)
                        }
                        umya_spreadsheet::structs::CellRawValue::Bool(b) => CellValue::Bool(*b),
                        umya_spreadsheet::structs::CellRawValue::Error(_) => {
                            CellValue::Error(value.clone())
                        }
                        umya_spreadsheet::structs::CellRawValue::String(s) => {
                            CellValue::Text(s.to_string())
                        }
                        umya_spreadsheet::structs::CellRawValue::RichText(_) => {
                            CellValue::Text(value.clone())
                        }
                        umya_spreadsheet::structs::CellRawValue::Empty => CellValue::Empty,
                        umya_spreadsheet::structs::CellRawValue::Lazy(_) => {
                            CellValue::parse(&value)
                        }
                    };

                    data.insert(
                        key,
                        crate::types::CellData {
                            value: if value.is_empty() { None } else { Some(value) },
                            typed: Some(typed),
                            formula,
                            style,
                            format: number_format,
                            note: None,
                            locked: None,
                            has_comment: None,
                            array_formula_id: None,
                        },
                    );
                }
                let layout = super::super::xlsx_layout::extract_layout(sheet);
                let hyperlinks = super::super::xlsx_layout::extract_hyperlinks(sheet);
                let structures = super::super::xlsx_layout::extract_structures(sheet);
                worksheets.push(crate::types::Worksheet {
                    name: sheet.name().to_string(),
                    data,
                    column_widths: layout.column_widths,
                    row_heights: layout.row_heights,
                    frozen_rows: layout.frozen_rows,
                    frozen_cols: layout.frozen_cols,
                    merged_cells: layout.merged_cells,
                    filters: None,
                    hidden_rows: layout.hidden_rows,
                    validations: super::super::xlsx_rules::extract_validations(sheet),
                    conditional_formats: super::super::xlsx_rules::extract_conditional_formats(
                        sheet,
                    ),
                    charts: None,
                    comments: None,
                    protection: None,
                    array_formulas: None,
                    tables: structures.tables,
                    hidden_columns: layout.hidden_columns,
                    sheet_state: super::sheet_visibility(sheet),
                    hyperlinks,
                    print_setup: super::super::xlsx_layout::extract_print_setup(sheet),
                    autofilter: structures.autofilter,
                    row_page_breaks: structures.row_page_breaks,
                    column_page_breaks: structures.column_page_breaks,
                    images: super::super::xlsx_layout::extract_images(sheet),
                    print_areas: None,
                    rich_text: None,
                });
            }

            if !worksheets.is_empty() {
                // Generated packages store formulas without cached results, so
                // the workbook must be calculated before charts resolve their
                // ranges — otherwise every series comes back empty and the
                // dashboard shows blank boxes (#2152).
                let recalculated = botsheet_core::recalc::recalculate_if_needed(&mut worksheets);
                if recalculated > 0 {
                    log::info!("xlsx import recalculated {recalculated} formula cells");
                }

                #[cfg(feature = "xlsx")]
                if let Ok(format_map) = super::super::format_codes::extract_cell_format_codes(bytes)
                {
                    super::super::format_codes::apply_format_codes(&mut worksheets, &format_map);
                }

                // umya drops charts on read; extract from the raw package.
                #[cfg(feature = "xlsx")]
                match super::super::chart_read::extract_charts(bytes, &worksheets) {
                    Ok(charts_by_ws) => {
                        for (ws_index, charts) in charts_by_ws.into_iter().enumerate() {
                            if !charts.is_empty() {
                                if let Some(ws) = worksheets.get_mut(ws_index) {
                                    ws.charts = Some(charts);
                                }
                            }
                        }
                    }
                    Err(e) => log::warn!("chart extraction skipped: {e}"),
                }

                // Print areas + titles live in workbook defined names (#788 E11).
                let print_areas = super::super::xlsx_layout::extract_print_areas(&workbook);
                for (sheet_index, areas) in print_areas {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        ws.print_areas = Some(areas);
                    }
                }
                let print_titles = super::super::xlsx_layout::extract_print_titles(&workbook);
                for (sheet_index, titles) in print_titles {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        let setup = ws.print_setup.get_or_insert_with(Default::default);
                        setup.print_titles = Some(titles);
                    }
                }

                // Rich-text runs (E6): umya flattens them, so recover from the
                // raw sharedStrings.xml + sheet parts.
                let rich_text_by_sheet = super::super::xlsx_rich_text::extract_rich_text_all(bytes);
                for (sheet_index, rich_text) in rich_text_by_sheet {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        ws.rich_text = Some(rich_text);
                    }
                }

                // Cell comments / notes (gap 27): recover from xl/commentsN.xml.
                let comments_by_sheet = super::super::xlsx_comments::extract_comments(bytes);
                for (sheet_index, comments) in comments_by_sheet {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        for key in comments.keys() {
                            if let Some(cell) = ws.data.get_mut(key) {
                                cell.has_comment = Some(true);
                            }
                        }
                        ws.comments = Some(comments);
                    }
                }

                return Ok(worksheets);
            }
        }
    }

    Err("Failed to parse spreadsheet".to_string())
}

#[cfg(test)]
#[cfg(feature = "xlsx")]
mod orientation_tests {
    use super::parse_xlsx_to_worksheets;

    /// Builds a three-cell workbook whose shape is asymmetric, so a transposed
    /// import cannot pass: A1 = "Year", B1 = "GDP", A2 = 2024, B2 = 10.5.
    fn sample_workbook() -> Vec<u8> {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Data").expect("sheet name");
        worksheet.write_string(0, 0, "Year").expect("A1");
        worksheet.write_string(0, 1, "GDP").expect("B1");
        worksheet.write_number(1, 0, 2024.0).expect("A2");
        worksheet.write_number(1, 1, 10.5).expect("B2");
        // A merged pair: rust_xlsxwriter emits `<mergeCell .../>`, the form the
        // reader used to panic on.
        worksheet
            .merge_range(3, 0, 3, 1, "Header", &rust_xlsxwriter::Format::new())
            .expect("merge");
        workbook.save_to_buffer().expect("xlsx bytes")
    }

    #[test]
    fn keeps_row_column_orientation() {
        let worksheets = parse_xlsx_to_worksheets(&sample_workbook(), "xlsx").expect("parse");
        assert_eq!(worksheets.len(), 1);
        let data = &worksheets[0].data;
        assert_eq!(
            data.get("0,0").and_then(|cell| cell.value.clone()),
            Some("Year".to_string()),
            "A1 must stay at row 0 col 0"
        );
        assert_eq!(
            data.get("0,1").and_then(|cell| cell.value.clone()),
            Some("GDP".to_string()),
            "B1 must stay at row 0 col 1"
        );
        assert_eq!(
            data.get("1,0").and_then(|cell| cell.value.clone()),
            Some("2024".to_string()),
            "A2 must stay at row 1 col 0"
        );
        assert_eq!(
            data.get("1,1").and_then(|cell| cell.value.clone()),
            Some("10.5".to_string()),
            "B2 must stay at row 1 col 1"
        );
    }

    #[test]
    fn import_cell_cap_fails_closed_at_the_boundary() {
        use super::{ensure_within_cell_cap, MAX_IMPORT_CELLS_PER_SHEET};
        // Just under the cap is admitted; at and above it are refused with an
        // actionable message naming the sheet and the limit (#2530).
        assert!(ensure_within_cell_cap(MAX_IMPORT_CELLS_PER_SHEET - 1, "Data").is_ok());
        let at = ensure_within_cell_cap(MAX_IMPORT_CELLS_PER_SHEET, "Data").expect_err("at cap");
        assert!(at.contains("Data"));
        assert!(at.contains("250000"));
        assert!(at.contains("split the workbook"));
        assert!(ensure_within_cell_cap(MAX_IMPORT_CELLS_PER_SHEET + 1, "Data").is_err());
    }

    #[test]
    fn reads_merged_ranges_without_panicking() {
        // rust_xlsxwriter emits self-closing `<mergeCell/>`; the reader used to
        // panic on it before the package normalisation step (#2152).
        let worksheets = parse_xlsx_to_worksheets(&sample_workbook(), "xlsx").expect("parse");
        assert!(!worksheets.is_empty());
        assert!(worksheets[0].data.contains_key("0,0"));
    }
}

#[cfg(test)]
mod budget_streaming_tests {
    use super::count_marker_in_chunk;

    #[test]
    fn counts_markers_within_a_single_chunk() {
        let mut carry = Vec::new();
        let count = count_marker_in_chunk(b"<row r=\"1\"><row r=\"2\">", b"<row ", &mut carry);
        assert_eq!(count, 2);
    }

    #[test]
    fn counts_a_marker_split_across_two_chunks() {
        let mut carry = Vec::new();
        assert_eq!(count_marker_in_chunk(b"<ro", b"<row ", &mut carry), 0);
        assert_eq!(carry.as_slice(), b"<ro".as_slice());
        assert_eq!(
            count_marker_in_chunk(b"w r=\"1\">", b"<row ", &mut carry),
            1
        );
    }

    #[test]
    fn counts_a_run_of_markers_across_chunk_boundaries() {
        let mut carry = Vec::new();
        let mut total = 0u64;
        for chunk in [b"x<ro".as_slice(), b"w >y<row ".as_slice(), b"z".as_slice()] {
            total += count_marker_in_chunk(chunk, b"<row ", &mut carry);
        }
        assert_eq!(total, 2);
    }
}
