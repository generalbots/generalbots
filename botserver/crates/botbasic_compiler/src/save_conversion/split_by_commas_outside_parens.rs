

/// Split a string by commas that are not inside parentheses, brackets or
/// braces. Empty slots are dropped, so a trailing comma or `a,,b` never yields
/// an empty item.
pub(super) fn split_by_commas_outside_parens(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let part = s[start..i].trim();
                if !part.is_empty() {
                    parts.push(part);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        let part = s[start..].trim();
        if !part.is_empty() {
            parts.push(part);
        }
    }

    parts
}
