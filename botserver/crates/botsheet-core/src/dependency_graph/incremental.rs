//! Cached, incrementally-maintained dependency graph (#784).
//!
//! Every edit used to rebuild the whole graph (`build_dependency_graph` +
//! `find_cycles`) before recalculating, which is O(cells) of formula parsing
//! per keystroke. A [`DepGraph`] is built once when a sheet session loads and
//! only the edited cells' edges change afterwards: `on_edit` removes the
//! previous referenced set of a formula cell and adds its new one, then
//! recomputes the cycle list. Recalculation reuses the cached topology, so the
//! per-keystroke cost is proportional to the edited formula only.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::types::Worksheet;

use super::extract::extract_referenced_cells;
use super::order::collect_dependents;
use super::{key_to_string, CellKey};

/// Mutable dependency graph for one worksheet.
///
/// `dependents` maps a referenced cell to the formula cells that read it;
/// `formula_refs` records, per formula cell, the cells it reads so an edit can
/// remove exactly the edges that no longer apply. `cycles` caches the cells
/// participating in reference loops, recomputed on structural edits.
#[derive(Default)]
pub struct DepGraph {
    dependents: HashMap<CellKey, HashSet<CellKey>>,
    formula_refs: HashMap<CellKey, HashSet<CellKey>>,
    cycles: Vec<CellKey>,
}

impl DepGraph {
    /// Builds the graph and cycle list for a whole worksheet (session load,
    /// worksheet addition, or any structural bulk change).
    pub fn build(worksheet: &Worksheet) -> Self {
        let mut graph = Self::default();
        let keys: Vec<CellKey> = worksheet
            .data
            .keys()
            .filter_map(|k| {
                k.split_once(',')
                    .and_then(|(r, c)| Some((r.parse().ok()?, c.parse().ok()?)))
            })
            .collect();
        graph.on_edit(worksheet, &keys);
        graph
    }

    /// Applies the formula of the given cells to the graph: edges of previous
    /// formulas are removed, edges of the current formulas are added, and the
    /// cycle list is refreshed. Cells without a formula are treated as
    /// formula-less (their old edges are removed).
    pub fn on_edit(&mut self, worksheet: &Worksheet, keys: &[CellKey]) {
        for &key in keys {
            if let Some(old_refs) = self.formula_refs.remove(&key) {
                for referenced in old_refs {
                    if let Some(dependents) = self.dependents.get_mut(&referenced) {
                        dependents.remove(&key);
                        if dependents.is_empty() {
                            self.dependents.remove(&referenced);
                        }
                    }
                }
            }
            let key_str = key_to_string(key);
            let Some(cell) = worksheet.data.get(&key_str) else {
                continue;
            };
            let Some(formula) = &cell.formula else {
                continue;
            };
            let new_refs: HashSet<CellKey> =
                extract_referenced_cells(formula).into_iter().collect();
            for referenced in &new_refs {
                self.dependents.entry(*referenced).or_default().insert(key);
            }
            self.formula_refs.insert(key, new_refs);
        }
        self.recompute_cycles();
    }

    /// The `referenced -> dependents` map, for topological traversal.
    pub fn dependent_map(&self) -> &HashMap<CellKey, HashSet<CellKey>> {
        &self.dependents
    }

    /// Cells currently participating in reference cycles, cached since the
    /// last structural edit. Recalculation skips them.
    pub fn cycles(&self) -> &[CellKey] {
        &self.cycles
    }

    /// Recalculates the dependents of a single changed cell (typed engine).
    pub fn recalc_cascade_typed(
        &self,
        worksheet: &mut Worksheet,
        changed: CellKey,
        max_iterations: usize,
    ) {
        self.recalc_cascade_typed_many(worksheet, std::slice::from_ref(&changed), max_iterations);
    }

    /// Recalculates the dependents of several roots (paste, clear, resize)
    /// using the cached topology, in dependency order. Cycle members are
    /// reported but not re-evaluated; `max_iterations` bounds a pass so a very
    /// large chain cannot monopolize the request thread.
    pub fn recalc_cascade_typed_many(
        &self,
        worksheet: &mut Worksheet,
        roots: &[CellKey],
        max_iterations: usize,
    ) {
        let order = collect_dependents(roots, &self.dependents);

        let mut iter = 0usize;
        for cell_key in order {
            if iter >= max_iterations {
                log::warn!(
                    "recalculation stopped after {max_iterations} cells; dependency fan-out from {roots:?} is larger than the configured limit"
                );
                break;
            }
            if self.cycles.contains(&cell_key) {
                continue;
            }
            let key_str = key_to_string(cell_key);
            let Some(cell) = worksheet.data.get(&key_str) else {
                continue;
            };
            let Some(formula) = cell.formula.clone() else {
                continue;
            };
            let result = crate::engine::evaluate_typed(&formula, worksheet);
            let display = result.display();
            if let Some(target) = worksheet.data.get_mut(&key_str) {
                if target.value.as_deref() != Some(display.as_str()) {
                    target.value = Some(display);
                }
                target.typed = Some(result);
            }
            iter += 1;
        }
    }

    fn recompute_cycles(&mut self) {
        self.cycles = find_cycles_in(&self.dependents);
    }
}

/// Consumes the graph, returning the `referenced -> dependents` map (used by
/// the legacy one-shot path).
pub fn into_dependents(graph: DepGraph) -> HashMap<CellKey, HashSet<CellKey>> {
    graph.dependents
}

/// Kahn-style cycle detection over a `referenced -> dependents` map: nodes
/// that survive full stripping take part in at least one cycle (#784).
pub fn find_cycles_in(graph: &HashMap<CellKey, HashSet<CellKey>>) -> Vec<CellKey> {
    let mut in_degree: HashMap<CellKey, usize> = HashMap::new();
    for node in graph.keys() {
        in_degree.entry(*node).or_insert(0);
    }
    for dependents in graph.values() {
        for d in dependents {
            *in_degree.entry(*d).or_insert(0) += 1;
        }
    }
    let mut queue: VecDeque<CellKey> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&k, _)| k)
        .collect();
    let mut removed: HashSet<CellKey> = HashSet::new();
    while let Some(node) = queue.pop_front() {
        removed.insert(node);
        if let Some(dependents) = graph.get(&node) {
            for d in dependents {
                if let Some(deg) = in_degree.get_mut(d) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 && !removed.contains(d) {
                        queue.push_back(*d);
                    }
                }
            }
        }
    }
    let mut cyclic: Vec<CellKey> = in_degree
        .keys()
        .filter(|k| !removed.contains(k))
        .copied()
        .collect();
    cyclic.sort_unstable();
    cyclic
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::types::CellData;

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
    fn build_tracks_dependents_and_cycles() {
        let w = ws(&[
            ("0,0", "=5"),
            ("0,1", "=A1+1"),
            ("0,2", "=B1+1"),
            ("1,0", "=B2+1"),
            ("1,1", "=A2+1"),
        ]);
        let g = DepGraph::build(&w);
        assert!(g.dependents.get(&(0, 0)).unwrap().contains(&(0, 1)));
        assert!(g.dependents.get(&(0, 1)).unwrap().contains(&(0, 2)));
        assert!(g.cycles.contains(&(1, 0)));
        assert!(g.cycles.contains(&(1, 1)));
    }

    #[test]
    fn edit_replaces_edges_incrementally() {
        let mut data = HashMap::new();
        data.insert("0,0".to_string(), cell(Some("=5")));
        data.insert("0,1".to_string(), cell(Some("=A1+1")));
        data.insert("0,2".to_string(), cell(Some("=B1+1")));
        let mut w = Worksheet {
            data,
            ..Worksheet::default()
        };
        let mut g = DepGraph::build(&w);
        // C1 is rewritten to read A1: the old C1 -> B1 dependence disappears
        // and a new C1 -> A1 dependence appears, without a full rebuild.
        if let Some(c) = w.data.get_mut("0,2") {
            c.formula = Some("=A1+1".to_string());
        }
        g.on_edit(&w, &[(0, 2)]);
        let dependents_of_a1 = &g.dependents[&(0, 0)];
        assert!(dependents_of_a1.contains(&(0, 1)));
        assert!(dependents_of_a1.contains(&(0, 2)));
        assert!(!g.dependents.contains_key(&(0, 1)));
    }

    #[test]
    fn rewrite_to_formula_less_removes_edges() {
        let mut data = HashMap::new();
        data.insert("0,0".to_string(), cell(Some("=5")));
        data.insert("0,1".to_string(), cell(Some("=A1+1")));
        let mut w = Worksheet {
            data,
            ..Worksheet::default()
        };
        let mut g = DepGraph::build(&w);
        assert!(g.dependents.contains_key(&(0, 0)));
        if let Some(c) = w.data.get_mut("0,1") {
            c.formula = None;
        }
        g.on_edit(&w, &[(0, 1)]);
        assert!(!g.dependents.contains_key(&(0, 0)));
        assert!(g.cycles.is_empty());
    }

    #[test]
    fn recalc_uses_cached_topology() {
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
        let mut g = DepGraph::build(&w);
        g.on_edit(&w, &[(0, 0), (0, 1), (0, 2)]);
        g.recalc_cascade_typed(&mut w, (0, 0), 1000);
        assert_eq!(w.data.get("0,1").and_then(|c| c.value.as_deref()), Some("101"));
        assert_eq!(w.data.get("0,2").and_then(|c| c.value.as_deref()), Some("102"));
    }
}