//! Text runs and cell formatting for the PDF content stream.

/// Longest cell text rendered; longer values are truncated with an ellipsis.
const MAX_CELL_CHARS: usize = 16;

/// One positioned text run: `x y Tm (escaped) Tj`.
pub(super) fn text_cmd(x: f32, y: f32, text: &str) -> String {
    format!("1 0 0 1 {x:.1} {y:.1} Tm ({}) Tj\n", escape_text(text))
}

/// Escapes PDF string specials and replaces characters outside WinAnsi.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            ch if (' '..='~').contains(&ch) => out.push(ch),
            _ => out.push('?'),
        }
    }
    out
}

/// Zero-based column index to spreadsheet letters (`0 -> A`, `27 -> AB`).
pub(super) fn column_letter(col: u32) -> String {
    if col > 16_383 {
        return "?".to_string();
    }
    let mut name = String::new();
    let mut n = col + 1;
    while n > 0 {
        let rem = (n - 1) % 26;
        name.insert(0, (b'A' + rem as u8) as char);
        n = (n - 1) / 26;
    }
    name
}

/// Truncates long values to keep a page legible.
pub(super) fn cell_to_text(value: &str) -> String {
    if value.chars().count() <= MAX_CELL_CHARS {
        return value.to_string();
    }
    let truncated: String = value.chars().take(MAX_CELL_CHARS - 1).collect();
    format!("{truncated}…")
}

pub(super) fn parse_key(key: &str) -> Option<(u32, u32)> {
    let (row, col) = key.split_once(',')?;
    Some((row.parse().ok()?, col.parse().ok()?))
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
