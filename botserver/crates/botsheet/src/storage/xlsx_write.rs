use crate::types::{CellStyle, Spreadsheet};
use std::io::Cursor;

pub fn convert_to_xlsx(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    let mut workbook = umya_spreadsheet::new_file();

    for (ws_idx, worksheet) in sheet.worksheets.iter().enumerate() {
        let umya_sheet = if ws_idx == 0 {
            workbook
                .get_sheet_mut(&0)
                .ok_or("Failed to get first sheet")?
        } else {
            workbook
                .new_sheet(&worksheet.name)
                .map_err(|e| format!("Failed to create sheet: {e}"))?
        };

        umya_sheet.set_name(&worksheet.name);

        for (key, cell_data) in &worksheet.data {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 2 {
                continue;
            }

            let row: u32 = parts[0].parse().unwrap_or(0) + 1;
            let col: u32 = parts[1].parse().unwrap_or(0) + 1;

            if row == 0 || col == 0 {
                continue;
            }

            let cell = umya_sheet.get_cell_mut((col, row));

            if let Some(ref formula) = cell_data.formula {
                let formula_str = if formula.starts_with('=') {
                    &formula[1..]
                } else {
                    formula.as_str()
                };
                cell.set_formula(formula_str);
            } else if let Some(ref value) = cell_data.value {
                if let Ok(num) = value.parse::<f64>() {
                    cell.set_value_number(num);
                } else if value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("false")
                {
                    cell.set_value_bool(value.eq_ignore_ascii_case("true"));
                } else {
                    cell.set_value_string(value);
                }
            }

            if let Some(ref style) = cell_data.style {
                apply_umya_style(cell, style);
            }
        }

        if let Some(ref widths) = worksheet.column_widths {
            for (col_idx, width) in widths {
                let col_letter = get_col_letter(*col_idx);
                let col_dim = umya_sheet.get_column_dimension_mut(&col_letter);
                col_dim.set_width(*width as f64);
            }
        }

        if let Some(ref heights) = worksheet.row_heights {
            for (row_idx, height) in heights {
                let row_dim = umya_sheet.get_row_dimension_mut(row_idx);
                row_dim.set_height(*height as f64);
            }
        }

        if let Some(ref merged) = worksheet.merged_cells {
            for merge in merged {
                let start_col = get_col_letter(merge.start_col + 1);
                let end_col = get_col_letter(merge.end_col + 1);
                let range = format!(
                    "{}{}:{}{}",
                    start_col, merge.start_row + 1, end_col, merge.end_row + 1
                );
                let _ = umya_sheet.add_merge_cells(&range);
            }
        }

        if let Some(frozen_rows) = worksheet.frozen_rows {
            if frozen_rows > 0 {
                let sheet_views = umya_sheet.get_sheet_views_mut();
                if let Some(view) = sheet_views.get_sheet_view_list_mut().first_mut() {
                    if let Some(pane) = view.get_pane_mut() {
                        pane.set_vertical_split(frozen_rows as f64);
                        pane.set_state(umya_spreadsheet::structs::PaneStateValues::Frozen);
                    }
                }
            }
        }

        if let Some(frozen_cols) = worksheet.frozen_cols {
            if frozen_cols > 0 {
                let sheet_views = umya_sheet.get_sheet_views_mut();
                if let Some(view) = sheet_views.get_sheet_view_list_mut().first_mut() {
                    if let Some(pane) = view.get_pane_mut() {
                        pane.set_horizontal_split(frozen_cols as f64);
                        pane.set_state(umya_spreadsheet::structs::PaneStateValues::Frozen);
                    }
                }
            }
        }
    }

    let mut buf = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(&workbook, &mut buf)
        .map_err(|e| format!("Failed to write xlsx: {e}"))?;

    Ok(buf.into_inner())
}

pub fn apply_umya_style(cell: &mut umya_spreadsheet::Cell, style: &CellStyle) {
    let cell_style = cell.get_style_mut();

    if let Some(ref weight) = style.font_weight {
        if weight == "bold" {
            cell_style.get_font_mut().set_bold(true);
        }
    }

    if let Some(ref font_style) = style.font_style {
        if font_style == "italic" {
            cell_style.get_font_mut().set_italic(true);
        }
    }

    if let Some(ref decoration) = style.text_decoration {
        if decoration.contains("underline") {
            cell_style.get_font_mut().set_underline("single");
        }
        if decoration.contains("line-through") {
            cell_style.get_font_mut().set_strikethrough(true);
        }
    }

    if let Some(size) = style.font_size {
        cell_style.get_font_mut().set_size(size as f64);
    }

    if let Some(ref font) = style.font_family {
        cell_style.get_font_mut().set_name(font);
    }

    if let Some(ref color) = style.color {
        let color_str = color.trim_start_matches('#');
        cell_style
            .get_font_mut()
            .get_color_mut()
            .set_argb(&format!("FF{color_str}"));
    }

    if let Some(ref bg) = style.background {
        let bg_str = bg.trim_start_matches('#');
        cell_style
            .get_fill_mut()
            .get_pattern_fill_mut()
            .get_foreground_color_mut()
            .set_argb(&format!("FF{bg_str}"));
        cell_style
            .get_fill_mut()
            .get_pattern_fill_mut()
            .set_pattern_type(umya_spreadsheet::structs::PatternValues::Solid);
    }

    if let Some(ref align) = style.text_align {
        let h_align = match align.as_str() {
            "left" => umya_spreadsheet::structs::HorizontalAlignmentValues::Left,
            "center" => umya_spreadsheet::structs::HorizontalAlignmentValues::Center,
            "right" => umya_spreadsheet::structs::HorizontalAlignmentValues::Right,
            _ => umya_spreadsheet::structs::HorizontalAlignmentValues::Left,
        };
        cell_style.get_alignment_mut().set_horizontal(h_align);
    }

    if let Some(ref v_align) = style.vertical_align {
        let v = match v_align.as_str() {
            "top" => umya_spreadsheet::structs::VerticalAlignmentValues::Top,
            "middle" => umya_spreadsheet::structs::VerticalAlignmentValues::Center,
            "bottom" => umya_spreadsheet::structs::VerticalAlignmentValues::Bottom,
            _ => umya_spreadsheet::structs::VerticalAlignmentValues::Center,
        };
        cell_style.get_alignment_mut().set_vertical(v);
    }
}

pub fn extract_cell_style(cell: &umya_spreadsheet::Cell) -> Option<CellStyle> {
    let style = cell.get_style();
    let font = style.get_font();
    let fill = style.get_fill();
    let alignment = style.get_alignment();

    let font_weight = font
        .as_ref()
        .and_then(|f| if *f.get_bold() { Some("bold".to_string()) } else { None });
    let font_style = font
        .as_ref()
        .and_then(|f| if *f.get_italic() { Some("italic".to_string()) } else { None });

    let underline_str = font
        .as_ref()
        .map(|f| f.get_underline().to_string())
        .unwrap_or_default();
    let has_strikethrough = font
        .as_ref()
        .map(|f| *f.get_strikethrough())
        .unwrap_or(false);
    let text_decoration = {
        let mut dec = Vec::new();
        if underline_str != "none" && !underline_str.is_empty() {
            dec.push("underline");
        }
        if has_strikethrough {
            dec.push("line-through");
        }
        if dec.is_empty() {
            None
        } else {
            Some(dec.join(" "))
        }
    };

    let font_size = font.as_ref().map(|f| f.get_size().round() as u32);
    let font_family = font.as_ref().map(|f| f.get_name().to_string());

    let color = font
        .as_ref()
        .map(|f| {
            let argb = f.get_color().get_argb();
            if argb.len() == 8 {
                format!("#{}", &argb[2..])
            } else if argb.is_empty() {
                "#000000".to_string()
            } else {
                format!("#{argb}")
            }
        })
        .filter(|c| c != "#000000");

    let background = fill
        .and_then(|f| f.get_pattern_fill())
        .and_then(|pf| {
            pf.get_foreground_color().map(|color| {
                let argb = color.get_argb();
                if argb.len() >= 8 {
                    format!("#{}", &argb[2..])
                } else if argb.is_empty() {
                    "#FFFFFF".to_string()
                } else {
                    format!("#{argb}")
                }
            })
        })
        .filter(|c| c != "#FFFFFF");

    let text_align = alignment
        .map(|a| {
            use umya_spreadsheet::structs::HorizontalAlignmentValues;
            match a.get_horizontal() {
                HorizontalAlignmentValues::Left => Some("left".to_string()),
                HorizontalAlignmentValues::Center => Some("center".to_string()),
                HorizontalAlignmentValues::Right => Some("right".to_string()),
                _ => None,
            }
        })
        .flatten();

    let vertical_align = alignment
        .map(|a| {
            use umya_spreadsheet::structs::VerticalAlignmentValues;
            match a.get_vertical() {
                VerticalAlignmentValues::Top => Some("top".to_string()),
                VerticalAlignmentValues::Center => Some("middle".to_string()),
                VerticalAlignmentValues::Bottom => Some("bottom".to_string()),
                _ => None,
            }
        })
        .flatten();

    if font_weight.is_some()
        || font_style.is_some()
        || text_decoration.is_some()
        || color.is_some()
        || background.is_some()
        || text_align.is_some()
    {
        Some(CellStyle {
            font_family,
            font_size,
            font_weight,
            font_style,
            text_decoration,
            color,
            background,
            text_align,
            vertical_align,
            border: None,
        })
    } else {
        None
    }
}

pub fn get_col_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col;
    while n > 0 {
        n -= 1;
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    result
}
