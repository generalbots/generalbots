//! Evaluation ordering for dependent cells.

use std::collections::{HashMap, HashSet, VecDeque};

use super::CellKey;

/// Returns the cells that must be recalculated after `roots` change, in an
/// order where every cell appears after the cells it depends on.
///
/// The result is limited to cells reachable from `roots` through the dependency
/// graph. Cells taking part in a reference cycle cannot be ordered and are
/// appended last so that a circular formula degrades to a single pass instead of
/// stalling the caller.
pub fn collect_dependents(
    roots: &[CellKey],
    graph: &HashMap<CellKey, HashSet<CellKey>>,
) -> Vec<CellKey> {
    // Reachable set: every dependent of every root, transitively.
    let mut reachable: HashSet<CellKey> = HashSet::new();
    let mut queue: VecDeque<CellKey> = VecDeque::new();
    for &r in roots {
        if reachable.insert(r) {
            queue.push_back(r);
        }
    }
    while let Some(node) = queue.pop_front() {
        let Some(dependents) = graph.get(&node) else {
            continue;
        };
        for &next in dependents {
            if reachable.insert(next) {
                queue.push_back(next);
            }
        }
    }

    // In-degree counted only over edges internal to the reachable subgraph.
    let mut in_degree: HashMap<CellKey, usize> = reachable.iter().map(|&k| (k, 0)).collect();
    for &node in &reachable {
        let Some(dependents) = graph.get(&node) else {
            continue;
        };
        for &next in dependents {
            if let Some(degree) = in_degree.get_mut(&next) {
                *degree += 1;
            }
        }
    }

    let mut ready: VecDeque<CellKey> = in_degree
        .iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(&key, _)| key)
        .collect();

    let mut ordered: Vec<CellKey> = Vec::with_capacity(reachable.len());
    let mut emitted: HashSet<CellKey> = HashSet::with_capacity(reachable.len());

    while let Some(node) = ready.pop_front() {
        if !emitted.insert(node) {
            continue;
        }
        ordered.push(node);
        let Some(dependents) = graph.get(&node) else {
            continue;
        };
        for &next in dependents {
            let Some(degree) = in_degree.get_mut(&next) else {
                continue;
            };
            *degree = degree.saturating_sub(1);
            if *degree == 0 && !emitted.contains(&next) {
                ready.push_back(next);
            }
        }
    }

    // Anything left is part of a cycle; evaluate it once in a stable order.
    if ordered.len() < reachable.len() {
        let mut cyclic: Vec<CellKey> = reachable
            .into_iter()
            .filter(|k| !emitted.contains(k))
            .collect();
        cyclic.sort_unstable();
        log::warn!(
            "circular reference detected among {} sheet cells; evaluating them once",
            cyclic.len()
        );
        ordered.extend(cyclic);
    }

    ordered
}

