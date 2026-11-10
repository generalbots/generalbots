//! Worksheet layout + print-setup extraction (#788, E3/E6/E11).
//!
//! Reads column widths, row heights, merged cells, frozen panes, hidden
//! rows/columns, hyperlinks, tables, autofilter and page setup from a umya
//! worksheet into the `Worksheet` model. Write-back lives in `xlsx_layout_patch`.

use crate::types::{Hyperlink, MergedCell, PrintSetup, PrintTitles, SheetImage, TableConfig};
use std::collections::HashMap;
use uuid::Uuid;

/// The layout dimensions extracted from a single worksheet.
pub struct ExtractedLayout {
    pub column_widths: Option<HashMap<u32, u32>>,
    pub row_heights: Option<HashMap<u32, u32>>,
    pub frozen_rows: Option<u32>,
    pub frozen_cols: Option<u32>,
    pub merged_cells: Option<Vec<MergedCell>>,
    pub hidden_rows: Option<Vec<u32>>,
    pub hidden_columns: Option<Vec<u32>>,
}

/// Reads cell hyperlinks (internal/external + tooltip) keyed by `"row,col"`
/// (0-based, matching `data`). Iterates the actual cells, not the bounding box.
pub fn extract_hyperlinks(
    sheet: &umya_spreadsheet::Worksheet,
) -> Option<HashMap<String, Hyperlink>> {
    let mut hyperlinks = HashMap::new();
    for ((col, row), cell) in sheet.collection_to_hashmap() {
        if let Some(hyperlink) = cell.as_ref().hyperlink() {
            let url = hyperlink.url();
            if url.is_empty() {
                continue;
            }
            let tooltip = hyperlink.tooltip();
            hyperlinks.insert(
                format!("{},{}", row - 1, col - 1),
                Hyperlink {
                    url: url.to_string(),
                    tooltip: if tooltip.is_empty() {
                        None
                    } else {
                        Some(tooltip.to_string())
                    },
                    is_internal: hyperlink.location(),
                },
            );
        }
    }
    if hyperlinks.is_empty() {
        None
    } else {
        Some(hyperlinks)
    }
}

/// Structured features read from a worksheet: tables, the autofilter range and
/// manual page breaks (E6/E11). Tables use the model's 0-based coordinates;
/// page breaks stay 1-based to mirror the OOXML `<brk id=...>` attribute.
pub struct ExtractedStructures {
    pub autofilter: Option<String>,
    pub tables: Option<Vec<TableConfig>>,
    pub row_page_breaks: Option<Vec<u32>>,
    pub column_page_breaks: Option<Vec<u32>>,
}

/// Reads tables, the autofilter range and manual page breaks from `sheet`.
pub fn extract_structures(sheet: &umya_spreadsheet::Worksheet) -> ExtractedStructures {
    let autofilter = sheet
        .auto_filter()
        .map(|af| af.range().range().to_string())
        .filter(|r| !r.is_empty());

    let mut tables = Vec::new();
    for table in sheet.tables() {
        let area = table.area();
        let start_row = area.0.row_num();
        let start_col = area.0.col_num();
        let end_row = area.1.row_num();
        let end_col = area.1.col_num();
        let name = table.name().to_string();
        tables.push(TableConfig {
            id: name.clone(),
            name,
            start_row: start_row.saturating_sub(1),
            start_col: start_col.saturating_sub(1),
            end_row: end_row.saturating_sub(1),
            end_col: end_col.saturating_sub(1),
            has_header_row: true,
        });
    }

    let mut row_page_breaks = Vec::new();
    for brk in sheet.row_breaks().break_list() {
        row_page_breaks.push(brk.id());
    }
    let mut column_page_breaks = Vec::new();
    for brk in sheet.column_breaks().break_list() {
        column_page_breaks.push(brk.id());
    }

    ExtractedStructures {
        autofilter,
        tables: if tables.is_empty() {
            None
        } else {
            Some(tables)
        },
        row_page_breaks: if row_page_breaks.is_empty() {
            None
        } else {
            Some(row_page_breaks)
        },
        column_page_breaks: if column_page_breaks.is_empty() {
            None
        } else {
            Some(column_page_breaks)
        },
    }
}

/// Reads images anchored to the worksheet (E6). Row/col are 0-based, matching
/// the `data` keys. Guarded against umya's internal panic when an image has
/// neither a two-cell nor a one-cell anchor.
pub fn extract_images(sheet: &umya_spreadsheet::Worksheet) -> Option<Vec<SheetImage>> {
    let mut images = Vec::new();
    for image in sheet.image_collection() {
        let anchored = image.two_cell_anchor().is_some()
            || image.one_cell_anchor().is_some();
        if !anchored {
            continue;
        }
        let name = image.image_name();
        if name.is_empty() {
            continue;
        }
        images.push(SheetImage {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            row: image.row(),
            col: image.col(),
        });
    }
    if images.is_empty() {
        None
    } else {
        Some(images)
    }
}

/// Reads the `_xlnm.Print_Titles` defined names (E11), keyed by the 0-based
/// worksheet index. The address is normalised from `'Sheet1'!$1:$3,'Sheet1'!$A:$B`
/// to `rows = "1:3"`, `columns = "A:B"`.
pub fn extract_print_titles(
    workbook: &umya_spreadsheet::Workbook,
) -> HashMap<usize, PrintTitles> {
    let mut map: HashMap<usize, PrintTitles> = HashMap::new();
    for name in workbook.defined_names() {
        if name.name() != "_xlnm.Print_Titles" {
            continue;
        }
        let sheet_index = name.local_sheet_id() as usize;
        let titles = map.entry(sheet_index).or_default();
        for part in name.address().split(',') {
            let range = part.rsplit('!').next().unwrap_or(part).replace('$', "");
            if range.is_empty() {
                continue;
            }
            // Row ranges are all digits ("1:3"); column ranges contain letters ("A:B").
            if range.chars().any(|c| c.is_ascii_alphabetic()) {
                titles.columns = Some(range);
            } else {
                titles.rows = Some(range);
            }
        }
    }
    map
}

/// Reads the `_xlnm.Print_Area` defined names (E11), keyed by the 0-based
/// worksheet index. The address is normalised from `Sheet1!$A$1:$D$10` to the
/// bare A1 range `A1:D10`; comma-separated multi-areas are split.
pub fn extract_print_areas(
    workbook: &umya_spreadsheet::Workbook,
) -> HashMap<usize, Vec<String>> {
    let mut map: HashMap<usize, Vec<String>> = HashMap::new();
    for name in workbook.defined_names() {
        if name.name() != "_xlnm.Print_Area" {
            continue;
        }
        let address = name.address();
        let sheet_index = name.local_sheet_id() as usize;
        for part in address.split(',') {
            let range = part.rsplit('!').next().unwrap_or(part).replace('$', "");
            if !range.is_empty() {
                map.entry(sheet_index).or_default().push(range);
            }
        }
    }
    map
}

/// Reads page setup, margins and header/footer (E11) from `sheet`. Returns
/// `None` when umya reports only its all-defaults, so the model is not
/// polluted with meaningless zeroes for untouched sheets.
pub fn extract_print_setup(sheet: &umya_spreadsheet::Worksheet) -> Option<PrintSetup> {
    use umya_spreadsheet::structs::OrientationValues;

    let page_setup = sheet.page_setup();
    let margins = sheet.page_margins();
    let header_footer = sheet.header_footer();

    let orientation = match page_setup.orientation() {
        OrientationValues::Landscape => Some("landscape".to_string()),
        OrientationValues::Portrait => Some("portrait".to_string()),
        OrientationValues::Default => None,
    };

    let odd_header = {
        let value = header_footer.odd_header().value();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    };
    let odd_footer = {
        let value = header_footer.odd_footer().value();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    };

    let paper_size = page_setup.paper_size();
    let scale = page_setup.scale();
    let fit_to_width = page_setup.fit_to_width();
    let fit_to_height = page_setup.fit_to_height();

    // Skip untouched sheets: umya defaults are paper_size/scale/fit = 0 and
    // empty header/footer text.
    if paper_size == 0
        && orientation.is_none()
        && scale == 0
        && fit_to_width == 0
        && fit_to_height == 0
        && odd_header.is_none()
        && odd_footer.is_none()
    {
        return None;
    }

    Some(PrintSetup {
        paper_size: Some(paper_size),
        orientation,
        scale: Some(scale),
        fit_to_width: Some(fit_to_width),
        fit_to_height: Some(fit_to_height),
        margin_left: Some(margins.left()),
        margin_right: Some(margins.right()),
        margin_top: Some(margins.top()),
        margin_bottom: Some(margins.bottom()),
        margin_header: Some(margins.header()),
        margin_footer: Some(margins.footer()),
        // Print titles are extracted separately (extract_print_titles) and
        // merged on the import path; the layout writer has no titles source.
        print_titles: None,
        odd_header,
        odd_footer,
    })
}

/// Reads column widths, row heights, frozen panes, merged cells and hidden rows
/// from `sheet`, so an opened workbook renders with its real geometry.
pub fn extract_layout(sheet: &umya_spreadsheet::Worksheet) -> ExtractedLayout {
    // Sparse dimension collections (one entry per explicit `<col>`/`<row>`), so
    // a sheet with data at row 1,000,000 does not scan a million empty rows.
    // umya's `col_num`/`row_num` are 1-based (OOXML `<col min>`/`<row r>`); the
    // model is 0-based everywhere else (cell keys, resize, filter, export), so
    // subtract one at the boundary — otherwise imported geometry renders one
    // column/row to the right of where it belongs.
    let mut column_widths = HashMap::new();
    let mut hidden_columns = Vec::new();
    for dim in sheet.column_dimensions() {
        let col = dim.col_num().saturating_sub(1);
        let width = dim.width();
        if width > 0.0 {
            column_widths.insert(col, width.round() as u32);
        }
        if dim.hidden() {
            hidden_columns.push(col);
        }
    }

    let mut row_heights = HashMap::new();
    let mut hidden_rows = Vec::new();
    for dim in sheet.row_dimensions() {
        let row = dim.row_num().saturating_sub(1);
        let height = dim.height();
        if height > 0.0 {
            row_heights.insert(row, height.round() as u32);
        }
        if dim.hidden() {
            hidden_rows.push(row);
        }
    }

    let merged_cells: Vec<MergedCell> = sheet
        .merge_cells()
        .iter()
        .filter_map(|mc| parse_merge_range(&mc.range().to_string()))
        .collect();

    let view = sheet.sheets_views().sheet_view_list().first();
    let pane = view.and_then(|v| v.pane());
    let frozen_rows = pane
        .map(|p| p.vertical_split() as u32)
        .filter(|&v| v > 0);
    let frozen_cols = pane
        .map(|p| p.horizontal_split() as u32)
        .filter(|&v| v > 0);

    ExtractedLayout {
        column_widths: if column_widths.is_empty() {
            None
        } else {
            Some(column_widths)
        },
        row_heights: if row_heights.is_empty() {
            None
        } else {
            Some(row_heights)
        },
        frozen_rows,
        frozen_cols,
        merged_cells: if merged_cells.is_empty() {
            None
        } else {
            Some(merged_cells)
        },
        hidden_rows: if hidden_rows.is_empty() {
            None
        } else {
            Some(hidden_rows)
        },
        hidden_columns: if hidden_columns.is_empty() {
            None
        } else {
            Some(hidden_columns)
        },
    }
}

/// Parses an A1 merge range (`A1:B2`) into zero-based row/col bounds.
fn parse_merge_range(range: &str) -> Option<MergedCell> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_cell_ref(parts[0])?;
    let end = parse_cell_ref(parts[1])?;
    Some(MergedCell {
        start_row: start.0,
        start_col: start.1,
        end_row: end.0,
        end_col: end.1,
    })
}

/// Parses a single A1 reference (`B2`) into zero-based (row, col).
pub(crate) fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let mut col_str = String::new();
    let mut row_str = String::new();
    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            col_str.push(c.to_ascii_uppercase());
        } else if c.is_ascii_digit() {
            row_str.push(c);
        }
    }
    let col = col_str
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));
    let row: u32 = row_str.parse().ok()?;
    Some((row.saturating_sub(1), col.saturating_sub(1)))
}
