use std::collections::HashMap;

pub(super) fn col_letter_to_index(s: &str) -> Option<u32> {
    let mut n: u32 = 0;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        let up = c.to_ascii_uppercase() as u32 - 'A' as u32 + 1;
        n = n.saturating_mul(26).saturating_add(up);
    }
    if n == 0 {
        None
    } else {
        Some(n - 1)
    }
}

pub(super) fn parse_range(range: &str) -> Option<(u32, u32, u32, u32)> {
    let s = range.replace('$', "");
    let parts: Vec<&str> = s.split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let (start_ref, end_ref) = if parts.len() == 1 {
        (parts[0], parts[0])
    } else {
        (parts[0], parts[1])
    };
    let split_ref = |r: &str| -> Option<(u32, u32)> {
        let p = r.find(|c: char| c.is_ascii_digit())?;
        if p == 0 || p == r.len() {
            return None;
        }
        let col = col_letter_to_index(&r[..p])?;
        // Spreadsheet refs are 1-based; the sheet cell map is 0-based. Without
        // this conversion the header row was read as the first data row and
        // every pivot was shifted by one.
        let row: u32 = r[p..].parse::<u32>().ok()?.checked_sub(1)?;
        Some((row, col))
    };
    let (r1, c1) = split_ref(start_ref)?;
    let (r2, c2) = split_ref(end_ref)?;
    let (rr1, rr2) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
    let (cc1, cc2) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
    Some((rr1, cc1, rr2, cc2))
}

pub(super) fn cell_to_string(v: Option<&crate::types::CellData>) -> String {
    match v.and_then(|c| c.value.as_ref()) {
        Some(val) => val.clone(),
        None => String::new(),
    }
}

pub(super) fn cell_to_number(v: Option<&crate::types::CellData>) -> Option<f64> {
    // Only typed Numbers aggregate; text that merely looks numeric (`0123`)
    // must not be summed. Untyped cells (CSV/ODS) keep the parse fallback.
    match v.and_then(|c| c.typed.as_ref()) {
        Some(botsheet_core::engine::value::CellValue::Number(n)) => Some(*n),
        Some(_) => None,
        None => cell_to_string(v).trim().parse::<f64>().ok(),
    }
}

pub(super) fn collect_field_index(
    header_row: u32,
    start_col: u32,
    end_col: u32,
    data: &HashMap<String, crate::types::CellData>,
) -> HashMap<String, u32> {
    let mut idx = HashMap::new();
    for col in start_col..=end_col {
        let key = format!("{},{}", header_row, col);
        let name = cell_to_string(data.get(&key)).trim().to_string();
        if !name.is_empty() {
            idx.insert(name, col);
        }
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::parse_range;

    #[test]
    fn a1_ranges_are_zero_based_and_inclusive() {
        assert_eq!(parse_range("A1:C3"), Some((0, 0, 2, 2)));
        assert_eq!(parse_range("$B$2:$D$5"), Some((1, 1, 4, 3)));
        assert_eq!(parse_range("C3:A1"), Some((0, 0, 2, 2)));
    }

    #[test]
    fn invalid_ranges_are_rejected() {
        assert_eq!(parse_range(""), None);
        assert_eq!(parse_range("A"), None);
        assert_eq!(parse_range("1:3"), None);
    }
}
