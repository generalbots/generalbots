//! Dependency tracking and cascading recalculation for worksheet formulas.

pub mod extract;
pub mod incremental;
pub mod order;

pub use extract::extract_referenced_cells;
pub use incremental::DepGraph;
pub use order::collect_dependents;

use std::collections::{HashMap, HashSet};

use crate::types::Worksheet;

pub type CellKey = (u32, u32);

pub fn cell_key(row: u32, col: u32) -> CellKey {
    (row, col)
}

pub fn key_to_string(k: CellKey) -> String {
    format!("{},{}", k.0, k.1)
}

pub fn key_from_string(s: &str) -> Option<CellKey> {
    let mut parts = s.split(',');
    let row: u32 = parts.next()?.parse().ok()?;
    let col: u32 = parts.next()?.parse().ok()?;
    Some((row, col))
}

pub fn build_dependency_graph(worksheet: &Worksheet) -> HashMap<CellKey, HashSet<CellKey>> {
    let mut deps: HashMap<CellKey, HashSet<CellKey>> = HashMap::new();

    for (key, cell) in &worksheet.data {
        let Some(formula) = &cell.formula else {
            continue;
        };
        let Some(target) = key_from_string(key) else {
            continue;
        };
        for referenced in extract_referenced_cells(formula) {
            // Self-references stay in the graph so `find_cycles` reports
            // them (`#CIRC!`); recalc skips cyclic nodes in dependency order.
            deps.entry(referenced).or_default().insert(target);
        }
    }

    deps
}

/// Recalculates every formula that depends on `changed`, in dependency order.
///
/// `max_iterations` bounds the number of formulas evaluated in one pass so that
/// a very large dependency chain cannot monopolize the request thread. When the
/// bound is reached the remaining cells keep their previous values and a warning
/// is logged.
pub fn recalc_cascade<F>(
    worksheet: &mut Worksheet,
    changed: CellKey,
    evaluator: F,
    max_iterations: usize,
) where
    F: Fn(&str, &Worksheet) -> String,
{
    let graph = build_dependency_graph(worksheet);
    let order = collect_dependents(&[changed], &graph);

    let mut iter = 0usize;
    for cell_key in order {
        if iter >= max_iterations {
            log::warn!(
                "recalculation stopped after {max_iterations} cells; dependency chain from {changed:?} is longer than the configured limit"
            );
            break;
        }
        let key_str = key_to_string(cell_key);
        let Some(cell) = worksheet.data.get(&key_str) else {
            continue;
        };
        let Some(formula) = cell.formula.clone() else {
            continue;
        };
        let result = evaluator(&formula, worksheet);
        if let Some(target) = worksheet.data.get_mut(&key_str) {
            if target.value.as_deref() != Some(result.as_str()) {
                target.value = Some(result);
            }
        }
        iter += 1;
    }
}

/// Finds the cells that participate in a reference cycle, if any (#784).
///
/// A formula cell is reported when it is reachable from itself through the
/// dependency graph. The caller can surface these as `#CIRC!` instead of
/// silently evaluating them in a degraded order.
pub fn find_cycles(worksheet: &Worksheet) -> Vec<CellKey> {
    incremental::find_cycles_in(&build_dependency_graph(worksheet))
}

/// Recalculates dependents of `changed` using the typed engine, stopping when
/// the limit is reached. Cycle members are reported but not re-evaluated.
///
/// One-shot flavor: builds a fresh cached graph for the call. Session-bound
/// callers use [`DepGraph`] directly so the topology survives across edits
/// (#784).
pub fn recalc_cascade_typed(
    worksheet: &mut Worksheet,
    changed: CellKey,
    max_iterations: usize,
) {
    let graph = DepGraph::build(worksheet);
    graph.recalc_cascade_typed(worksheet, changed, max_iterations);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CellData;
    use std::collections::HashMap;

    fn cell(formula: Option<&str>) -> CellData {
        CellData {
            value: None,
            typed: None,
            formula: formula.map(String::from),
            style: None,
            format: None,
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        }
    }

    fn ws(pairs: &[(&str, &str)]) -> Worksheet {
        let mut data = HashMap::new();
        for (k, f) in pairs {
            data.insert((*k).to_string(), cell(Some(f)));
        }
        Worksheet {
            data,
            ..Worksheet::default()
        }
    }

    #[test]
    fn extracts_simple_ref() {
        let refs = extract_referenced_cells("=A1+5");
        assert!(refs.contains(&(0, 0)));
    }

    #[test]
    fn extracts_range_ref() {
        let refs = extract_referenced_cells("=SUM(A1:A3)");
        assert!(refs.contains(&(0, 0)));
        assert!(refs.contains(&(1, 0)));
        assert!(refs.contains(&(2, 0)));
    }

    #[test]
    fn ignores_function_names() {
        let refs = extract_referenced_cells("=SUM(5,10)+A1");
        assert!(refs.contains(&(0, 0)));
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn builds_graph_for_chain() {
        let w = ws(&[("0,0", "=5"), ("0,1", "=A1+1"), ("0,2", "=B1+1")]);
        let g = build_dependency_graph(&w);
        let dependents = g.get(&(0, 0)).expect("A1 must have dependents");
        assert!(dependents.contains(&(0, 1)));
    }

    #[test]
    fn ignores_identifiers_that_are_not_cell_refs() {
        // A long alphanumeric token is a named range, not a column reference.
        assert!(extract_referenced_cells("=MYNAMEDRANGE2024+1").is_empty());
        // Row 0 does not exist in any reference syntax.
        assert!(extract_referenced_cells("=A0+1").is_empty());
        // Columns beyond XFD are out of range.
        assert!(extract_referenced_cells("=XFE1+1").is_empty());
    }

    #[test]
    fn tracks_absolute_references() {
        let refs = extract_referenced_cells("=SUM($A$1:$A$3)");
        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&(0, 0)));
        assert!(refs.contains(&(2, 0)));
        assert!(extract_referenced_cells("=$A$1+1").contains(&(0, 0)));
    }

    #[test]
    fn wide_range_records_only_its_endpoints() {
        let refs = extract_referenced_cells("=SUM(A1:XFD1048576)");
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&(0, 0)));
        assert!(refs.contains(&(1_048_575, 16_383)));
    }

    #[test]
    fn orders_dependents_after_their_inputs() {
        let w = ws(&[("0,1", "=A1+1"), ("0,2", "=B1+1")]);
        let g = build_dependency_graph(&w);
        let order = collect_dependents(&[(0, 0)], &g);
        assert_eq!(order, vec![(0, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn cyclic_references_are_returned_once() {
        let w = ws(&[("0,1", "=C1+1"), ("0,2", "=B1+1")]);
        let g = build_dependency_graph(&w);
        let order = collect_dependents(&[(0, 1)], &g);
        assert_eq!(order.len(), 2);
        assert!(order.contains(&(0, 1)));
        assert!(order.contains(&(0, 2)));
    }

    #[test]
    fn recalculates_transitive_dependents() {
        let mut data = HashMap::new();
        data.insert("0,0".to_string(), {
            let mut c = cell(None);
            c.value = Some("100".to_string());
            c
        });
        data.insert("0,1".to_string(), cell(Some("=A1+1")));
        data.insert("0,2".to_string(), cell(Some("=B1+1")));
        let mut w = Worksheet {
            data,
            ..Worksheet::default()
        };

        recalc_cascade(
            &mut w,
            (0, 0),
            |formula, sheet| crate::formulas::evaluate_formula(formula, sheet).value,
            1000,
        );

        assert_eq!(
            w.data.get("0,1").and_then(|c| c.value.as_deref()),
            Some("101")
        );
        assert_eq!(
            w.data.get("0,2").and_then(|c| c.value.as_deref()),
            Some("102")
        );
    }

    #[test]
    fn finds_two_cell_cycle() {
        let w = ws(&[("0,0", "=B1+1"), ("0,1", "=A1+1")]);
        let cycles = find_cycles(&w);
        assert_eq!(cycles.len(), 2);
        assert!(cycles.contains(&(0, 0)));
        assert!(cycles.contains(&(0, 1)));
    }

    #[test]
    fn acyclic_sheet_has_no_cycles() {
        let w = ws(&[("0,0", "=5"), ("0,1", "=A1+1"), ("0,2", "=B1+1")]);
        assert!(find_cycles(&w).is_empty());
    }

    #[test]
    fn typed_recalc_skips_cycles() {
        let mut data = HashMap::new();
        data.insert("0,0".to_string(), cell(Some("=10")));
        data.insert("0,1".to_string(), cell(Some("=A1+1")));
        // Real two-cell cycle: A2 and B2 refer to each other.
        data.insert("1,0".to_string(), cell(Some("=B2+1")));
        data.insert("1,1".to_string(), cell(Some("=A2+1")));
        let mut w = Worksheet {
            data,
            ..Worksheet::default()
        };
        recalc_cascade_typed(&mut w, (0, 0), 1000);
        assert_eq!(w.data.get("0,1").and_then(|c| c.value.as_deref()), Some("11"));
        // Cycle members are reported but not re-evaluated.
        let cycles = find_cycles(&w);
        assert!(cycles.contains(&(1, 0)));
        assert!(cycles.contains(&(1, 1)));
    }
}
