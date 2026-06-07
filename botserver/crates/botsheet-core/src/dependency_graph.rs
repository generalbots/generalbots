use std::collections::{HashMap, HashSet, VecDeque};

use crate::formulas::parse_cell_ref;
use crate::types::{CellData, Worksheet};

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
            if referenced == target {
                continue;
            }
            deps.entry(referenced).or_default().insert(target);
        }
    }

    deps
}

pub fn extract_referenced_cells(formula: &str) -> HashSet<CellKey> {
    let mut out = HashSet::new();
    let bytes = formula.as_bytes();
    let mut i = 0;
    let n = bytes.len();

    while i < n {
        let c = bytes[i];
        let is_quote = c == b'"';
        if is_quote {
            i += 1;
            while i < n && bytes[i] != b'"' {
                i += 1;
            }
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() {
            let start = i;
            while i < n && (bytes[i].is_ascii_alphabetic() || bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                i += 1;
            }
            if i < n && bytes[i] == b'(' {
                continue;
            }
            let token = &formula[start..i];
            if is_function_name(token) {
                continue;
            }
            let parsed = parse_cell_ref(token);
            let after = &formula[i..];
            if after.starts_with(':') {
                let range_end_relaxed = after.trim_start_matches(':');
                let end_first = range_end_relaxed
                    .split(|ch: char| !ch.is_ascii_alphanumeric())
                    .next()
                    .unwrap_or("");
                if let (Some((r, col)), Some((end_r, end_c))) = (parsed, parse_cell_ref(end_first)) {
                    let start_r = r;
                    let start_c = col;
                    let lo_r = start_r.min(end_r);
                    let hi_r = start_r.max(end_r);
                    let lo_c = start_c.min(end_c);
                    let hi_c = start_c.max(end_c);
                    if hi_r - lo_r <= 1024 && hi_c - lo_c <= 256 {
                        for rr in lo_r..=hi_r {
                            for cc in lo_c..=hi_c {
                                out.insert((rr, cc));
                            }
                        }
                    } else {
                        out.insert((r, col));
                    }
                }
            } else if let Some((r, col)) = parsed {
                out.insert((r, col));
            }
        } else {
            i += 1;
        }
    }

    out
}

fn is_function_name(token: &str) -> bool {
    matches!(
        token.to_ascii_uppercase().as_str(),
        "SUM" | "AVERAGE" | "MIN" | "MAX" | "COUNT"
            | "COUNTA" | "COUNTBLANK" | "IF" | "AND" | "OR" | "NOT"
            | "VLOOKUP" | "HLOOKUP" | "LOOKUP" | "MATCH" | "INDEX"
            | "IFERROR" | "IFNA" | "ABS" | "ROUND" | "ROUNDUP" | "ROUNDDOWN"
            | "SQRT" | "POWER" | "MOD" | "INT" | "CEILING" | "FLOOR"
            | "CONCATENATE" | "CONCAT" | "LEFT" | "RIGHT" | "MID" | "LEN"
            | "UPPER" | "LOWER" | "PROPER" | "TRIM" | "TEXT" | "VALUE"
            | "TODAY" | "NOW" | "YEAR" | "MONTH" | "DAY" | "HOUR" | "MINUTE" | "SECOND"
            | "DATE" | "DATEDIF" | "DAYS" | "EOMONTH" | "EDATE"
            | "RAND" | "RANDBETWEEN" | "TRUE" | "FALSE"
            | "MEDIAN" | "MODE" | "STDEV" | "STDEVP" | "VAR" | "VARP"
            | "LARGE" | "SMALL" | "RANK" | "PERCENTILE" | "QUARTILE"
            | "EXP" | "LN" | "LOG" | "LOG10" | "PI" | "E"
            | "SIN" | "COS" | "TAN" | "ASIN" | "ACOS" | "ATAN" | "ATAN2"
            | "DEGREES" | "RADIANS" | "SIGN"
            | "CHAR" | "CODE" | "EXACT" | "FIND" | "SEARCH" | "REPLACE" | "SUBSTITUTE"
            | "REPT" | "T" | "FIXED" | "DOLLAR" | "ROMAN"
            | "ROWS" | "COLUMNS" | "ROW" | "COLUMN" | "ADDRESS" | "INDIRECT"
            | "OFFSET" | "CHOOSE" | "AREAS" | "TRANSPOSE" | "COLUMNREF"
            | "SUMIF" | "SUMIFS" | "COUNTIF" | "COUNTIFS" | "AVERAGEIF" | "AVERAGEIFS"
            | "MINIFS" | "MAXIFS" | "PRODUCT" | "SUMPRODUCT"
            | "XLOOKUP" | "XMATCH" | "FILTER" | "SORT" | "SORTBY" | "UNIQUE"
            | "SEQUENCE" | "RANDARRAY" | "TOCOL" | "TOROW" | "WRAPCOLS" | "WRAPROWS"
            | "HSTACK" | "VSTACK" | "CHOOSEROWS" | "CHOOSECOLS" | "TAKE" | "DROP"
            | "EXPAND" | "TRIMRANGE" | "PIVOTBY" | "GROUPBY"
    )
}

pub fn collect_dependents(
    roots: &[CellKey],
    graph: &HashMap<CellKey, HashSet<CellKey>>,
) -> Vec<CellKey> {
    let mut visited: HashSet<CellKey> = HashSet::new();
    let mut queue: VecDeque<CellKey> = VecDeque::new();
    let mut topo: Vec<CellKey> = Vec::new();

    for &r in roots {
        if visited.insert(r) {
            queue.push_back(r);
        }
    }

    let mut in_degree: HashMap<CellKey, usize> = HashMap::new();
    let mut local: HashMap<CellKey, HashSet<CellKey>> = HashMap::new();

    for (src, dests) in graph {
        for &d in dests {
            if visited.contains(src) {
                local.entry(*src).or_default().insert(d);
                *in_degree.entry(d).or_insert(0) += 1;
            } else {
                let mut q2: VecDeque<CellKey> = VecDeque::new();
                let mut seen2: HashSet<CellKey> = HashSet::new();
                if seen2.insert(*src) {
                    q2.push_back(*src);
                }
                while let Some(n) = q2.pop_front() {
                    if !visited.insert(n) {
                        continue;
                    }
                    if let Some(nexts) = graph.get(&n) {
                        for &nx in nexts {
                            if seen2.insert(nx) {
                                q2.push_back(nx);
                            }
                        }
                    }
                }
            }
        }
    }

    for &r in roots {
        let mut q2: VecDeque<CellKey> = VecDeque::new();
        q2.push_back(r);
        let mut seen2: HashSet<CellKey> = HashSet::new();
        while let Some(n) = q2.pop_front() {
            if !seen2.insert(n) {
                continue;
            }
            if let Some(nexts) = graph.get(&n) {
                for &nx in nexts {
                    *in_degree.entry(nx).or_insert(0) += 1;
                    local.entry(n).or_default().insert(nx);
                    q2.push_back(nx);
                }
            }
        }
    }

    let mut ready: VecDeque<CellKey> = VecDeque::new();
    for &r in roots {
        if in_degree.get(&r).copied().unwrap_or(0) == 0 {
            ready.push_back(r);
        }
    }
    for (k, _) in &local {
        if in_degree.get(k).copied().unwrap_or(0) == 0 && !ready.contains(k) {
            ready.push_back(*k);
        }
    }

    while let Some(n) = ready.pop_front() {
        if !visited.contains(&n) {
            visited.insert(n);
        }
        if !topo.contains(&n) {
            topo.push(n);
        }
        if let Some(nexts) = local.get(&n) {
            for &nx in nexts {
                let d = in_degree.entry(nx).or_insert(0);
                *d = d.saturating_sub(1);
                if *d == 0 && !topo.contains(&nx) {
                    ready.push_back(nx);
                }
            }
        }
    }

    topo
}

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

pub fn ensure_cell(worksheet: &mut Worksheet, row: u32, col: u32) {
    let k = format!("{row},{col}");
    worksheet.data.entry(k).or_insert_with(|| CellData {
        value: None,
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cell(formula: Option<&str>) -> CellData {
        CellData {
            value: None,
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
        assert!(g.get(&(0, 0)).is_some());
        assert!(g.get(&(0, 0)).unwrap().contains(&(0, 1)));
    }
}
