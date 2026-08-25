//! Text and JSON merge helpers backing the `merge` plan step kind.
//!
//! `merge_text` performs a line-based three-way-less merge: equal lines from
//! both inputs are kept once; diverging blocks are captured as conflict hunks
//! instead of being silently dropped. `merge_json` unions object keys and
//! reports differing values as conflicts.

use serde_json::Value;

const LOOKAHEAD: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct MergeOutcome {
    pub merged: String,
    pub conflicts: Vec<String>,
}

impl MergeOutcome {
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }
}

fn render_conflict(a_block: &[&str], b_block: &[&str]) -> String {
    let mut hunk = String::from("<<<<<<< a\n");
    for line in a_block {
        hunk.push_str(line);
        hunk.push('\n');
    }
    hunk.push_str("=======\n");
    for line in b_block {
        hunk.push_str(line);
        hunk.push('\n');
    }
    hunk.push_str(">>>>>>> b");
    hunk
}

/// Line-based merge of two documents. Lines present identically in both are
/// kept in order; contiguous divergences become one conflict hunk each.
pub fn merge_text(a: &str, b: &str) -> MergeOutcome {
    let la: Vec<&str> = a.lines().collect();
    let lb: Vec<&str> = b.lines().collect();
    let mut merged = String::new();
    let mut conflicts = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);

    while i < la.len() || j < lb.len() {
        if i < la.len() && j < lb.len() && la[i] == lb[j] {
            merged.push_str(la[i]);
            merged.push('\n');
            i += 1;
            j += 1;
            continue;
        }
        // Locate the next realignment point: the smallest combined offset
        // where an identical line appears ahead in both inputs, so divergent
        // blocks stay bounded instead of swallowing the remainder.
        let mut split: Option<(usize, usize)> = None;
        'search: for total in 1..=LOOKAHEAD {
            for k in 0..=total {
                let m = total - k;
                if k == 0 && m == 0 {
                    continue;
                }
                if la.get(i + k).is_some() && la.get(i + k) == lb.get(j + m) {
                    split = Some((k, m));
                    break 'search;
                }
            }
        }
        match split {
            Some((a_len, b_len)) => {
                conflicts.push(render_conflict(&la[i..i + a_len], &lb[j..j + b_len]));
                merged.push_str(&render_conflict(&la[i..i + a_len], &lb[j..j + b_len]));
                merged.push('\n');
                i += a_len;
                j += b_len;
            }
            None => {
                let rest_a = &la[i.min(la.len())..];
                let rest_b = &lb[j.min(lb.len())..];
                let hunk = render_conflict(rest_a, rest_b);
                conflicts.push(hunk.clone());
                merged.push_str(&hunk);
                merged.push('\n');
                i = la.len();
                j = lb.len();
            }
        }
    }

    if merged.ends_with('\n') && !(a.is_empty() && b.is_empty()) {
        merged.pop();
    }
    MergeOutcome { merged, conflicts }
}

/// JSON merge: object keys are unioned; identical nested values collapse.
/// Conflicting scalars/arrays are reported as `key: a_value | b_value`.
pub fn merge_json(a: &Value, b: &Value) -> MergeOutcome {
    let mut conflicts = Vec::new();
    let merged = merge_objects(a, b, String::new(), &mut conflicts);
    MergeOutcome {
        merged: serde_json::to_string_pretty(&merged).unwrap_or_else(|_| "{}".to_string()),
        conflicts,
    }
}

fn merge_objects(a: &Value, b: &Value, path: String, conflicts: &mut Vec<String>) -> Value {
    match (a, b) {
        (Value::Object(ma), Value::Object(mb)) => {
            let mut out = serde_json::Map::new();
            for (key, va) in ma {
                match mb.get(key) {
                    Some(vb) => {
                        let child_path = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        };
                        out.insert(key.clone(), merge_objects(va, vb, child_path, conflicts));
                    }
                    None => {
                        out.insert(key.clone(), va.clone());
                    }
                }
            }
            for (key, vb) in mb {
                if !ma.contains_key(key) {
                    out.insert(key.clone(), vb.clone());
                }
            }
            Value::Object(out)
        }
        (x, y) if x == y => x.clone(),
        (x, y) => {
            conflicts.push(format!(
                "{path}: {} | {}",
                serde_json::to_string(x).unwrap_or_else(|_| "?".to_string()),
                serde_json::to_string(y).unwrap_or_else(|_| "?".to_string())
            ));
            y.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_inputs_merge_without_conflicts() {
        let outcome = merge_text("same\nlines\nhere\n", "same\nlines\nhere\n");
        assert!(outcome.conflicts.is_empty());
        assert_eq!(outcome.merged, "same\nlines\nhere");
    }

    #[test]
    fn common_suffix_realigns_after_divergence() {
        let outcome = merge_text("head\nmine\ntail\n", "head\ntheirs\ntail\n");
        assert_eq!(outcome.conflicts.len(), 1);
        assert!(outcome.conflicts[0].contains("<<<<<<<"));
        assert!(outcome.conflicts[0].contains("======="));
        assert!(outcome.conflicts[0].contains(">>>>>>>"));
        assert!(outcome.merged.starts_with("head"));
        assert!(outcome.merged.ends_with("tail"));
        assert!(outcome.merged.contains("mine"));
        assert!(outcome.merged.contains("theirs"));
    }

    #[test]
    fn fully_disjoint_documents_yield_one_hunk() {
        let outcome = merge_text("alpha\nbeta\n", "gamma\ndelta\n");
        assert_eq!(outcome.conflicts.len(), 1);
        assert!(outcome.merged.contains("alpha"));
        assert!(outcome.merged.contains("delta"));
    }

    #[test]
    fn json_merge_unions_keys_and_reports_scalar_conflicts() {
        let a: Value = serde_json::json!({"name": "x", "keep": 1});
        let b: Value = serde_json::json!({"name": "y", "extra": true});
        let outcome = merge_json(&a, &b);
        assert!(outcome.has_conflicts());
        let merged: Value =
            serde_json::from_str(&outcome.merged).expect("merged output is valid json");
        assert_eq!(merged["keep"], serde_json::json!(1));
        assert_eq!(merged["extra"], serde_json::json!(true));
    }

    #[test]
    fn json_merge_equal_values_do_not_conflict() {
        let outcome = merge_json(
            &serde_json::json!({"a": [1, 2]}),
            &serde_json::json!({"a": [1, 2]}),
        );
        assert!(!outcome.has_conflicts());
    }
}
