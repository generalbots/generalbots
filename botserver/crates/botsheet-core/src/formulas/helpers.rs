use crate::types::Worksheet;
use regex::Regex;

pub fn col_name_to_index(name: &str) -> u32 {
    let mut col: u32 = 0;
    for ch in name.chars() {
        col = col * 26 + (ch as u32 - 'A' as u32 + 1);
    }
    col - 1
}

pub fn format_number(num: f64) -> String {
    if num.fract() == 0.0 {
        format!("{}", num as i64)
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
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default()
    } else {
        value.to_string()
    }
}

pub fn split_args(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

pub fn parse_range(range: &str) -> Option<((u32, u32), (u32, u32))> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_cell_ref(parts[0].trim())?;
    let end = parse_cell_ref(parts[1].trim())?;
    Some((start, end))
}

pub fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let cell_ref = cell_ref.trim();
    if cell_ref.is_empty() {
        return None;
    }
    let bytes = cell_ref.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let col_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    let col_end = i;
    
    let row_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let row_end = i;
    
    if col_start == col_end || row_start == row_end {
        return None;
    }
    
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i < bytes.len() {
        return None;
    }
    
    let col_str = std::str::from_utf8(&bytes[col_start..col_end]).ok()?;
    let col_str_upper = col_str.to_ascii_uppercase();
    let col = col_name_to_index(&col_str_upper);
    
    let row_str = std::str::from_utf8(&bytes[row_start..row_end]).ok()?;
    let row = row_str.parse::<u32>().ok()? - 1;
    Some((row, col))
}

pub fn get_range_values(range: &str, worksheet: &Worksheet) -> Vec<f64> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        if let Ok(val) = resolve_cell_value(range.trim(), worksheet).parse::<f64>() {
            return vec![val];
        }
        return Vec::new();
    }
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    let mut key = String::with_capacity(32);
    use std::fmt::Write;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            key.clear();
            let _ = write!(&mut key, "{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                if let Some(ref value) = cell.value {
                    if let Ok(num) = value.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        }
    }
    values
}

pub fn get_range_string_values(range: &str, worksheet: &Worksheet) -> Vec<String> {
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    let mut key = String::with_capacity(32);
    use std::fmt::Write;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            key.clear();
            let _ = write!(&mut key, "{},{}", row, col);
            let value = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.clone())
                .unwrap_or_default();
            values.push(value);
        }
    }
    values
}

pub fn resolve_cell_references(expr: &str, worksheet: &Worksheet) -> String {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let regex = RE.get_or_init(|| regex::Regex::new(r"\b([A-Z]{1,2})(\d+)\b").unwrap());

    regex.replace_all(expr, |cap: &regex::Captures| {
        let col_str = &cap[1];
        let row_str = &cap[2];
        let col = col_name_to_index(col_str);
        let row: u32 = row_str.parse::<u32>().unwrap_or(1).saturating_sub(1);
        let key = format!("{},{}", row, col);

        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_else(|| "0".to_string())
    }).into_owned()
}

pub fn evaluate_condition(condition: &str, worksheet: &Worksheet) -> bool {
    let condition = condition.trim();
    if condition.eq_ignore_ascii_case("TRUE") {
        return true;
    }
    if condition.eq_ignore_ascii_case("FALSE") {
        return false;
    }

    let operators = [">=", "<=", "<>", "!=", "=", ">", "<"];
    for op in &operators {
        if let Some(pos) = condition.find(op) {
            let left = resolve_cell_value(condition[..pos].trim(), worksheet);
            let right = resolve_cell_value(
                condition[pos + op.len()..].trim().trim_matches('"'),
                worksheet,
            );

            let left_num = left.parse::<f64>().ok();
            let right_num = right.parse::<f64>().ok();

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
    if criteria.starts_with(">=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v >= c;
        }
    } else if criteria.starts_with("<=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v <= c;
        }
    } else if criteria.starts_with("<>") || criteria.starts_with("!=") {
        let c = &criteria[2..];
        return !value.eq_ignore_ascii_case(c);
    } else if criteria.starts_with('>') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v > c;
        }
    } else if criteria.starts_with('<') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v < c;
        }
    } else if criteria.starts_with('=') {
        return value.eq_ignore_ascii_case(&criteria[1..]);
    } else if criteria.contains('*') || criteria.contains('?') {
        let pattern = criteria.replace('*', ".*").replace('?', ".");
        if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
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
