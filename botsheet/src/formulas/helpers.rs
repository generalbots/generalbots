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
    let cell_ref = cell_ref.trim().to_uppercase();
    let mut col_str = String::new();
    let mut row_str = String::new();
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col_str.push(ch);
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }
    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }
    let col = col_name_to_index(&col_str);
    let row: u32 = row_str.parse::<u32>().ok()? - 1;
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
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
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
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
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
    let mut result = expr.to_string();
    let re = Regex::new(r"([A-Z]+)(\d+)").ok();

    if let Some(regex) = re {
        for cap in regex.captures_iter(expr) {
            if let (Some(col_match), Some(row_match)) = (cap.get(1), cap.get(2)) {
                let col = col_name_to_index(col_match.as_str());
                let row: u32 = row_match.as_str().parse().unwrap_or(1) - 1;
                let key = format!("{},{}", row, col);

                let value = worksheet
                    .data
                    .get(&key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_else(|| "0".to_string());

                let cell_ref = format!("{}{}", col_match.as_str(), row_match.as_str());
                result = result.replace(&cell_ref, &value);
            }
        }
    }
    result
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
