use crate::types::Worksheet;

use super::helpers::{format_number, resolve_cell_value, split_args};

pub fn evaluate_textsplit(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TEXTSPLIT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let text = parts[0].trim().trim_matches('"');
    let delim = parts.get(1).map(|s| s.trim().trim_matches('"')).unwrap_or(",");
    let col_delim = parts.get(2).map(|s| s.trim().trim_matches('"'));
    let mut rows: Vec<String> = vec![text.to_string()];
    rows = rows.iter().flat_map(|r| r.split(col_delim.unwrap_or("\n")).map(|s| s.to_string()).collect::<Vec<_>>()).collect();
    rows = rows.iter().flat_map(|r| r.split(delim).map(|s| s.to_string()).collect::<Vec<_>>()).collect();
    Some(rows.join(","))
}

pub fn evaluate_textafter(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TEXTAFTER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let text = parts[0].trim().trim_matches('"');
    let delim = parts[1].trim().trim_matches('"');
    let n: usize = if parts.len() > 2 { parts[2].trim().parse().unwrap_or(1) } else { 1 };
    match text.find(delim) {
        Some(pos) => {
            let after = &text[pos + delim.len()..];
            let mut result = after.to_string();
            for _ in 1..n {
                match result.find(delim) {
                    Some(p) => result = result[p + delim.len()..].to_string(),
                    None => break,
                }
            }
            Some(result)
        }
        None => Some(String::new()),
    }
}

pub fn evaluate_textbefore(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TEXTBEFORE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let text = parts[0].trim().trim_matches('"');
    let delim = parts[1].trim().trim_matches('"');
    let n: usize = if parts.len() > 2 { parts[2].trim().parse().unwrap_or(1) } else { 1 };
    let mut pos = 0usize;
    let mut count = 0;
    while count < n {
        match text[pos..].find(delim) {
            Some(p) => {
                pos += p;
                count += 1;
                if count == n { break; }
                pos += delim.len();
            }
            None => return Some(String::new()),
        }
    }
    Some(text[..pos].to_string())
}

pub fn evaluate_arraytotext(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ARRAYTOTEXT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[12..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let format = if parts.len() > 1 && parts[1].trim().to_uppercase().contains("STRICT") { "\"{}\"" } else { "{}" };
    Some(values.iter().map(|v| format.replace("{}", v)).collect::<Vec<_>>().join(","))
}

pub fn evaluate_valuetotext(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("VALUETOTEXT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[12..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let v = resolve_cell_value(parts[0].trim(), worksheet);
    if v.parse::<f64>().is_ok() { Some(v) } else { Some(format!("\"{}\"", v)) }
}

pub fn evaluate_numbervalue(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("NUMBERVALUE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[12..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let text = parts[0].trim().trim_matches('"').replace(',', ".");
    let dec_sep = parts.get(1).map(|s| s.trim().trim_matches('"')).unwrap_or(".");
    let clean = text.replace(dec_sep, ".");
    let cleaned: String = clean.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == 'e' || *c == 'E').collect();
    match cleaned.parse::<f64>() {
        Ok(v) => Some(format_number(v)),
        Err(_) => Some("#VALUE!".to_string()),
    }
}

pub fn evaluate_unichar(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UNICHAR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let n: u32 = inner.trim().parse().unwrap_or(0);
    match char::from_u32(n) {
        Some(c) => Some(c.to_string()),
        None => Some("#VALUE!".to_string()),
    }
}

pub fn evaluate_unicode(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UNICODE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    match text.chars().next() {
        Some(c) => Some((c as u32).to_string()),
        None => Some("0".to_string()),
    }
}

pub fn evaluate_arabic(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ARABIC(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet).to_uppercase();
    let roman_values: [(&str, i64); 13] = [
        ("M", 1000), ("CM", 900), ("D", 500), ("CD", 400), ("C", 100),
        ("XC", 90), ("L", 50), ("XL", 40), ("X", 10), ("IX", 9),
        ("V", 5), ("IV", 4), ("I", 1),
    ];
    let mut result = 0i64;
    let mut remaining = text.as_str();
    while !remaining.is_empty() {
        let mut matched = false;
        for (r, v) in &roman_values {
            if remaining.starts_with(r) {
                result += v;
                remaining = &remaining[r.len()..];
                matched = true;
                break;
            }
        }
        if !matched { break; }
    }
    Some(result.to_string())
}

pub fn evaluate_roman(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROMAN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let n: i64 = inner.trim().parse().unwrap_or(0);
    if n <= 0 || n >= 4000 {
        return Some("#VALUE!".to_string());
    }
    let values: [(i64, &str); 13] = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"),
        (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"),
        (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut num = n;
    let mut out = String::new();
    for (v, s) in &values {
        while num >= *v {
            out.push_str(s);
            num -= v;
        }
    }
    Some(out)
}

pub fn evaluate_base64encode(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("BASE64_ENCODE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[14..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(base64_encode(&text))
}

pub fn evaluate_base64decode(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("BASE64_DECODE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[14..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(base64_decode(&text))
}

fn base64_encode(s: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if i + 1 < bytes.len() { CHARS[((triple >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if i + 2 < bytes.len() { CHARS[(triple & 0x3F) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

fn base64_decode(s: &str) -> String {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let bytes = s.bytes();
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for c in bytes {
        if c == b'=' { break; }
        if let Some(idx) = chars.find(c as char) {
            buf = (buf << 6) | (idx as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((buf >> bits) as u8);
                buf &= (1u32 << bits) - 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn evaluate_urlencode(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("URL_ENCODE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            out.push(c);
        } else {
            for b in c.to_string().as_bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    Some(out)
}
