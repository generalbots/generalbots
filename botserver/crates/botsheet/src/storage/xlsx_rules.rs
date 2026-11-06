//! Data validation + conditional formatting import (#790, gap 19).
//!
//! These rules are preserved byte-for-byte by the passthrough save; importing
//! them into the model makes the existing grid rendering (validation dots,
//! in-cell dropdowns, conditional highlights) work on Drive-opened workbooks.

use crate::types::{CellStyle, ConditionalFormatRule, ValidationRule};
use std::collections::HashMap;
use uuid::Uuid;

/// Reads data validations, expanding each range into per-cell rules keyed
/// `"row,col"` (matching the model's `validations` map).
pub fn extract_validations(
    sheet: &umya_spreadsheet::Worksheet,
) -> Option<HashMap<String, ValidationRule>> {
    use umya_spreadsheet::structs::EnumTrait;

    let data_validations = sheet.data_validations()?;
    let mut map: HashMap<String, ValidationRule> = HashMap::new();

    for dv in data_validations.data_validation_list() {
        let validation_type = dv.get_type().value_string().to_string();
        let operator = dv.operator().value_string().to_string();
        let formula1 = dv.formula1();
        let formula2 = dv.formula2();

        let rule = ValidationRule {
            validation_type: validation_type.clone(),
            operator: if operator.is_empty() {
                None
            } else {
                Some(operator)
            },
            value1: non_empty(formula1),
            value2: non_empty(formula2),
            allowed_values: list_values(&validation_type, formula1),
            error_title: non_empty(dv.error_title()),
            error_message: non_empty(dv.error_message()),
            input_title: non_empty(dv.prompt_title()),
            input_message: non_empty(dv.prompt()),
        };

        for range in dv.sequence_of_references().range_collection() {
            if let Some((sr, sc, er, ec)) = parse_a1_range(&range.range()) {
                for row in sr..=er {
                    for col in sc..=ec {
                        map.insert(format!("{row},{col}"), rule.clone());
                    }
                }
            }
        }
    }

    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Reads conditional formatting rules. The rule type, operator and range are
/// mapped faithfully; the differential (dxf) style is reduced to background
/// fill + font colour (the common highlight case). Formula-backed conditions
/// (`expression`) keep the rule type but not the exact expression.
pub fn extract_conditional_formats(
    sheet: &umya_spreadsheet::Worksheet,
) -> Option<Vec<ConditionalFormatRule>> {
    use umya_spreadsheet::structs::EnumTrait;

    let mut rules = Vec::new();
    for cf in sheet.conditional_formatting_collection() {
        let ranges: Vec<(u32, u32, u32, u32)> = cf
            .sequence_of_references()
            .range_collection()
            .iter()
            .filter_map(|r| parse_a1_range(&r.range()))
            .collect();

        for rule in cf.conditional_collection() {
            let rule_type = rule.get_type().value_string().to_string();
            let operator = rule.operator().value_string().to_string();
            let text = rule.text();
            // cellIs/top10/duplicate/unique: the operator is the condition.
            // Text rules: the text is the condition. Both empty (expression):
            // fall back to the rule type itself.
            let condition = if !operator.is_empty() && operator != "lessThan" {
                operator
            } else if !text.is_empty() {
                text.to_string()
            } else {
                rule_type.clone()
            };
            let style = dxf_style(rule.style());
            let priority = (rule.priority()).max(1) as u32;

            for (sr, sc, er, ec) in &ranges {
                rules.push(ConditionalFormatRule {
                    id: Uuid::new_v4().to_string(),
                    start_row: *sr,
                    start_col: *sc,
                    end_row: *er,
                    end_col: *ec,
                    rule_type: rule_type.clone(),
                    condition: condition.clone(),
                    style: style.clone(),
                    priority,
                });
            }
        }
    }

    if rules.is_empty() {
        None
    } else {
        Some(rules)
    }
}

/// Reduces a differential (dxf) style to the `CellStyle` fields the grid
/// renders: fill, font colour, family, size, weight, style and decoration.
/// Alignment and borders are not part of a conditional-format dxf.
fn dxf_style(style: Option<&umya_spreadsheet::structs::Style>) -> CellStyle {
    let Some(style) = style else {
        return CellStyle::default();
    };
    let background = style
        .fill()
        .and_then(|f| f.pattern_fill())
        .and_then(|pf| pf.foreground_color())
        .map(|c| argb_to_hex(&c.argb_str()))
        .filter(|c| c != "#FFFFFF");

    let font = style.font();
    let color = font
        .and_then(|f| f.color())
        .map(|c| argb_to_hex(&c.argb_str()))
        .filter(|c| c != "#000000");
    let font_family = font.map(|f| f.name().to_string());
    let font_size = font.map(|f| f.size().round() as u32);
    let font_weight = font.and_then(|f| if f.bold() { Some("bold".to_string()) } else { None });
    let font_style = font.and_then(|f| if f.italic() { Some("italic".to_string()) } else { None });
    let text_decoration = font.and_then(|f| {
        let mut dec = Vec::new();
        let underline = f.underline();
        if underline != "none" && !underline.is_empty() {
            dec.push("underline");
        }
        if f.strikethrough() {
            dec.push("line-through");
        }
        if dec.is_empty() {
            None
        } else {
            Some(dec.join(" "))
        }
    });

    let border = style.borders().and_then(|b| {
        super::xlsx_write::extract_border([b.top(), b.left(), b.bottom(), b.right()])
    });

    CellStyle {
        background,
        color,
        font_family,
        font_size,
        font_weight,
        font_style,
        text_decoration,
        border,
        ..CellStyle::default()
    }
}

fn argb_to_hex(argb: &str) -> String {
    if argb.len() >= 8 {
        format!("#{}", &argb[2..8])
    } else if argb.is_empty() {
        "#000000".to_string()
    } else {
        format!("#{argb}")
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Expands a literal (comma-separated) list validation into its allowed values.
fn list_values(validation_type: &str, formula1: &str) -> Option<Vec<String>> {
    if validation_type != "list" || formula1.is_empty() {
        return None;
    }
    // A range-backed list ("$A$1:$A$5") is resolved at render time, not here.
    if formula1.contains('$') || formula1.contains(':') {
        return None;
    }
    let values: Vec<String> = formula1
        .split(',')
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

/// Parses an A1 range (`A1:B10`) into zero-based (start_row, start_col, end_row, end_col).
fn parse_a1_range(a1: &str) -> Option<(u32, u32, u32, u32)> {
    let parts: Vec<&str> = a1.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let (sr, sc) = super::xlsx_layout::parse_cell_ref(parts[0])?;
    let (er, ec) = super::xlsx_layout::parse_cell_ref(parts[1])?;
    Some((sr, sc, er, ec))
}
