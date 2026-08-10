//! Discovery of the cell references a formula reads.

use std::collections::HashSet;

use crate::formulas::parse_cell_ref;

use super::CellKey;

/// Largest row span expanded cell-by-cell when recording range dependencies.
const MAX_RANGE_ROWS: u32 = 1024;

/// Largest column span expanded cell-by-cell when recording range dependencies.
const MAX_RANGE_COLS: u32 = 256;

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
        if c.is_ascii_alphabetic() || c == b'$' {
            let start = i;
            // `$` belongs to the token: `$A$1` is one absolute reference.
            while i < n
                && (bytes[i].is_ascii_alphabetic()
                    || bytes[i].is_ascii_digit()
                    || bytes[i] == b'_'
                    || bytes[i] == b'$')
            {
                i += 1;
            }
            if i < n && bytes[i] == b'(' {
                continue;
            }
            let token = &formula[start..i];
            let is_cross_sheet = start > 0 && bytes[start - 1] == b'!';
            let after = &formula[i..];
            let is_range = after.starts_with(':');
            if is_cross_sheet {
                // `Sheet2!A1` and `Sheet2!A1:B3` reference another worksheet;
                // the local dependency graph must not record the qualified
                // cells as dependencies of this sheet (#783, #784).
                if is_range {
                    while i < n
                        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b':' || bytes[i] == b'$')
                    {
                        i += 1;
                    }
                }
                continue;
            }
            if is_function_name(token) {
                continue;
            }
            let parsed = parse_cell_ref(token);
            if is_range {
                let range_end_relaxed = after.trim_start_matches(':');
                let end_first = range_end_relaxed
                    .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '$')
                    .next()
                    .unwrap_or("");
                if let (Some((start_r, start_c)), Some((end_r, end_c))) =
                    (parsed, parse_cell_ref(end_first))
                {
                    let lo_r = start_r.min(end_r);
                    let hi_r = start_r.max(end_r);
                    let lo_c = start_c.min(end_c);
                    let hi_c = start_c.max(end_c);
                    // Wide ranges are tracked by their anchor only: expanding a
                    // whole-column reference would add millions of graph nodes.
                    if hi_r - lo_r <= MAX_RANGE_ROWS && hi_c - lo_c <= MAX_RANGE_COLS {
                        for rr in lo_r..=hi_r {
                            for cc in lo_c..=hi_c {
                                out.insert((rr, cc));
                            }
                        }
                    } else {
                        out.insert((start_r, start_c));
                        out.insert((end_r, end_c));
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

