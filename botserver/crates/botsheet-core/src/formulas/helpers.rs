//! Value formatting, argument splitting and criteria matching.

use crate::types::Worksheet;

use super::refs::parse_cell_ref;

pub fn format_number(num: f64) -> String {
    if num.fract() == 0.0 {
        // `{:.0}` (not `as i64`) — an `as` cast saturates for magnitudes beyond
        // i64 (e.g. =10^20), silently showing a wrong value.
        format!("{:.0}", num)
    } else {
        format!("{:.6}", num)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn resolve_cell_value(value: &str, worksheet: &Worksheet) -> String {
    if let Some((row, col)) = parse_cell_ref(value) {
        let key = format!("{},{}", row, col);
        // Unset cell or empty value is an ordinary blank cell.
        match worksheet.data.get(&key).and_then(|c| c.value.clone()) {
            Some(value) => value,
            None => String::new(),
        }
    } else {
        value.to_string()
    }
}

pub fn split_args(s: &str) -> Vec<&str> {
    // #2572 — `;` is the argument separator in pt-BR (where `,` is the decimal
    // mark). Accept both at top level so `SOMA(1;2)` and `SUM(1,2)` agree.
    // Brackets and quoted strings are data, never separators: `SOMA("a;b",1)`
    // and `SOMA([1;2],1)` must not be split inside their literal.
    let mut parts = Vec::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut start = 0;
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let mut index = 0;
    while index < chars.len() {
        let (byte, ch) = chars[index];
        if in_string {
            if ch == '"' {
                if chars.get(index + 1).is_some_and(|(_, next)| *next == '"') {
                    index += 2;
                    continue;
                }
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' | ';' if paren_depth == 0 && bracket_depth == 0 => {
                parts.push(&s[start..byte]);
                start = byte + 1;
            }
            _ => {}
        }
        index += 1;
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

#[cfg(test)]
mod split_args_tests {
    use super::split_args;

    #[test]
    fn comma_and_semicolon_are_both_separators() {
        assert_eq!(split_args("1,2"), vec!["1", "2"]);
        assert_eq!(split_args("1;2"), vec!["1", "2"]);
        assert_eq!(split_args("1; 2,3"), vec!["1", " 2", "3"]);
    }

    #[test]
    fn separators_inside_nested_calls_arrays_and_strings_are_data() {
        assert_eq!(split_args("SOMA(1;2);3"), vec!["SOMA(1;2)", "3"]);
        assert_eq!(split_args("[1;2];3"), vec!["[1;2]", "3"]);
        assert_eq!(split_args("\"a;b\";3"), vec!["\"a;b\"", "3"]);
        assert_eq!(split_args("\"a\"\"b;c\";3"), vec!["\"a\"\"b;c\"", "3"]);
    }
}

pub fn evaluate_condition(condition: &str, worksheet: &Worksheet) -> bool {
    let condition = condition.trim();
    if condition.eq_ignore_ascii_case("TRUE") {
        return true;
    }
    if condition.eq_ignore_ascii_case("FALSE") {
        return false;
    }
    // #2067 — a bare number is a truth value in every spreadsheet: AND/OR/NOT
    // with literal arguments used to read `1` as false because the condition
    // parser only understood comparison operators.
    if let Ok(number) = condition.parse::<f64>() {
        return number != 0.0;
    }

    let operators = [">=", "<=", "<>", "!=", "=", ">", "<"];
    for op in &operators {
        if let Some(pos) = condition.find(op) {
            let left = resolve_cell_value(condition[..pos].trim(), worksheet);
            let right = resolve_cell_value(
                condition[pos + op.len()..].trim().trim_matches('"'),
                worksheet,
            );

            let left_num = match left.parse::<f64>() {
                Ok(v) => Some(v),
                Err(e) => { log::debug!("left operand is not numeric: {e}"); None }
            };
            let right_num = match right.parse::<f64>() {
                Ok(v) => Some(v),
                Err(e) => { log::debug!("right operand is not numeric: {e}"); None }
            };

            return match (*op, left_num, right_num) {
                (">=", Some(l), Some(r)) => l >= r,
                ("<=", Some(l), Some(r)) => l <= r,
                ("<>" | "!=", Some(l), Some(r)) => (l - r).abs() > f64::EPSILON,
                ("<>" | "!=", _, _) => left != right,
                ("=", Some(l), Some(r)) => (l - r).abs() < f64::EPSILON,
                ("=", _, _) => left.eq_ignore_ascii_case(&right),
                (">", Some(l), Some(r)) => l > r,
                ("<", Some(l), Some(r)) => l < r,
                _ => false,
            };
        }
    }
    false
}

pub fn matches_criteria(value: &str, criteria: &str) -> bool {
    if let Some(rest) = criteria.strip_prefix(">=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), rest.parse::<f64>()) {
            return v >= c;
        }
    } else if let Some(rest) = criteria.strip_prefix("<=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), rest.parse::<f64>()) {
            return v <= c;
        }
    } else if criteria.starts_with("<>") || criteria.starts_with("!=") {
        let c = &criteria[2..];
        return !value.eq_ignore_ascii_case(c);
    } else if let Some(rest) = criteria.strip_prefix('>') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), rest.parse::<f64>()) {
            return v > c;
        }
    } else if let Some(rest) = criteria.strip_prefix('<') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), rest.parse::<f64>()) {
            return v < c;
        }
    } else if let Some(rest) = criteria.strip_prefix('=') {
        return value.eq_ignore_ascii_case(rest);
    } else if criteria.contains('*') || criteria.contains('?') {
        let pattern = criteria.replace('*', ".*").replace('?', ".");
        if let Some(re) = crate::re_cache::dynamic(&format!("^{}$", pattern)) {
            return re.is_match(value);
        }
    }
    value.eq_ignore_ascii_case(criteria)
}

pub fn count_matching(values: &[String], criteria: &str) -> usize {
    values
        .iter()
        .filter(|v| matches_criteria(v, criteria))
        .count()
}

