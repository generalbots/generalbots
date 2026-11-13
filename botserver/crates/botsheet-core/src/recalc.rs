//! Workbook-wide formula recalculation for imported workbooks.
//!
//! Producers that write formulas without a cached result (openpyxl, most
//! generated packages, and our own scripts) leave `value` empty. The grid then
//! renders a workbook with labels and no numbers, so every imported workbook
//! must be recalculated once after loading (#2152 follow-up).
//!
//! Evaluation is done in passes: a formula that references a cell another
//! formula fills in only settles on the next pass. Chains across worksheets
//! (`Dashboard!B6 = Indicators!B4`, `Indicators!B4 = SUM(Monthly!...)`) need a
//! handful of passes, and evaluation stops as soon as a pass changes nothing.

use crate::engine::evaluate_typed_in;
use crate::types::Worksheet;

/// Upper bound on recalculation passes. Deeper dependency chains are rare and
/// a workbook must never make the loader spin: the loop also stops as soon as
/// a pass produces no change.
pub const MAX_PASSES: usize = 12;

/// True when at least one formula cell never had its result computed.
///
/// Used to keep the loader cheap: workbooks that already carry cached values
/// (Excel-saved files) are returned untouched.
pub fn needs_recalculation(worksheets: &[Worksheet]) -> bool {
    worksheets.iter().any(|worksheet| {
        worksheet.data.values().any(|cell| {
            cell.formula.is_some()
                && cell
                    .value
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
        })
    })
}

/// Evaluates every formula cell against the whole workbook and writes the
/// result into `value`/`typed`. Returns the number of cells updated.
pub fn recalculate_workbook(worksheets: &mut [Worksheet]) -> usize {
    let mut updated = 0;
    for _ in 0..MAX_PASSES {
        // Evaluation reads the whole workbook while cells are being written,
        // so each pass works from a snapshot of the previous pass.
        let snapshot: Vec<Worksheet> = worksheets.to_vec();
        let mut pass_updates = 0;
        for (index, worksheet) in worksheets.iter_mut().enumerate() {
            for cell in worksheet.data.values_mut() {
                let Some(formula) = cell.formula.clone() else {
                    continue;
                };
                let result = evaluate_typed_in(&formula, &snapshot, index);
                let display = result.display();
                let unchanged = cell.value.as_deref().unwrap_or("") == display
                    && cell.typed.as_ref() == Some(&result);
                if unchanged {
                    continue;
                }
                cell.value = Some(display);
                cell.typed = Some(result);
                pass_updates += 1;
            }
        }
        updated += pass_updates;
        if pass_updates == 0 {
            break;
        }
    }
    updated
}

/// Recalculates only when the workbook still has unresolved formulas.
pub fn recalculate_if_needed(worksheets: &mut [Worksheet]) -> usize {
    if needs_recalculation(worksheets) {
        recalculate_workbook(worksheets)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::value::CellValue;
    use crate::types::CellData;
    use std::collections::HashMap;

    fn formula_cell(formula: &str) -> CellData {
        CellData {
            value: None,
            typed: Some(CellValue::Empty),
            formula: Some(formula.to_string()),
            style: None,
            format: None,
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        }
    }

    fn number_cell(value: f64) -> CellData {
        CellData {
            value: Some(value.to_string()),
            typed: Some(CellValue::Number(value)),
            formula: None,
            style: None,
            format: None,
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        }
    }

    fn worksheet(name: &str, cells: Vec<(&str, CellData)>) -> Worksheet {
        Worksheet {
            name: name.to_string(),
            data: cells
                .into_iter()
                .map(|(key, cell)| (key.to_string(), cell))
                .collect::<HashMap<_, _>>(),
            ..Worksheet::default()
        }
    }

    #[test]
    fn recalculates_same_sheet_sum() {
        let mut sheets = vec![worksheet(
            "Sheet1",
            vec![
                ("0,0", number_cell(1.0)),
                ("1,0", number_cell(2.0)),
                ("2,0", formula_cell("=SUM(A1:A2)")),
            ],
        )];
        assert!(needs_recalculation(&sheets));
        assert!(recalculate_workbook(&mut sheets) > 0);
        assert_eq!(
            sheets[0].data.get("2,0").and_then(|c| c.value.clone()),
            Some("3".to_string())
        );
        assert!(!needs_recalculation(&sheets));
    }

    #[test]
    fn recalculates_cross_sheet_chain() {
        let mut sheets = vec![
            worksheet("Dashboard", vec![("0,0", formula_cell("=Indicators!A1"))]),
            worksheet("Indicators", vec![("0,0", formula_cell("=SUM(Data!A1:A2)"))]),
            worksheet(
                "Data",
                vec![("0,0", number_cell(10.0)), ("1,0", number_cell(5.0))],
            ),
        ];
        recalculate_workbook(&mut sheets);
        assert_eq!(
            sheets[0].data.get("0,0").and_then(|c| c.value.clone()),
            Some("15".to_string())
        );
    }

    #[test]
    fn leaves_cached_workbook_untouched() {
        let mut cell = formula_cell("=SUM(A1:A2)");
        cell.value = Some("3".to_string());
        cell.typed = Some(CellValue::Number(3.0));
        let mut sheets = vec![worksheet(
            "Sheet1",
            vec![
                ("0,0", number_cell(1.0)),
                ("1,0", number_cell(2.0)),
                ("2,0", cell),
            ],
        )];
        assert!(!needs_recalculation(&sheets));
        assert_eq!(recalculate_if_needed(&mut sheets), 0);
    }
}
