use chrono::{Datelike, NaiveDateTime, Timelike};
use num_format::{Locale, ToFormattedString};
use rhai::{Dynamic, Engine, Map};
use std::str::FromStr;

/// Format a value (number, date, or Map from NOW()) according to a pattern
fn format_impl(value: Dynamic, pattern: String) -> Result<String, String> {
    // Handle Map objects (like NOW() which returns a Map with formatted property)
    if value.is::<Map>() {
        let map = value.cast::<Map>();
        // Try to get the 'formatted' property first
        if let Some(formatted_val) = map.get("formatted") {
            let value_str = formatted_val.to_string();
            if let Ok(dt) = NaiveDateTime::parse_from_str(&value_str, "%Y-%m-%d %H:%M:%S") {
                let formatted = apply_date_format(&dt, &pattern);
                return Ok(formatted);
            }
        }
        // If no formatted property or parsing failed, try to construct from components
        if let (Some(year), Some(month), Some(day)) = (
            map.get("year").and_then(|v| v.as_int().ok()),
            map.get("month").and_then(|v| v.as_int().ok()),
            map.get("day").and_then(|v| v.as_int().ok()),
        ) {
            let value_str = format!("{:04}-{:02}-{:02}", year, month, day);
            if let Ok(dt) = NaiveDateTime::parse_from_str(&value_str, "%Y-%m-%d") {
                let formatted = apply_date_format(&dt, &pattern);
                return Ok(formatted);
            }
        }
        // Fallback: return empty string for unsupported map format
        return Ok(String::new());
    }

    // Handle string/number values
    let value_str = value.to_string();
    if let Ok(num) = f64::from_str(&value_str) {
        let formatted = if pattern.starts_with('N') || pattern.starts_with('C') {
            let (prefix, decimals, locale_tag) = parse_pattern(&pattern);
            let locale = get_locale(&locale_tag);
            let symbol = if prefix == "C" {
                get_currency_symbol(&locale_tag)
            } else {
                ""
            };
            let int_part = num.trunc() as i64;
            let frac_part = num.fract();
            if decimals == 0 {
                format!("{}{}", symbol, int_part.to_formatted_string(&locale))
            } else {
                let frac_scaled =
                    ((frac_part * 10f64.powi(decimals as i32)).round()) as i64;
                let decimal_sep = match locale_tag.as_str() {
                    "pt" | "fr" | "es" | "it" | "de" => ",",
                    _ => ".",
                };
                format!(
                    "{}{}{}{:0width$}",
                    symbol,
                    int_part.to_formatted_string(&locale),
                    decimal_sep,
                    frac_scaled,
                    width = decimals
                )
            }
        } else {
            match pattern.as_str() {
                "n" | "F" => format!("{num:.2}"),
                "0%" => format!("{:.0}%", num * 100.0),
                _ => format!("{num}"),
            }
        };
        return Ok(formatted);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&value_str, "%Y-%m-%d %H:%M:%S") {
        let formatted = apply_date_format(&dt, &pattern);
        return Ok(formatted);
    }
    let formatted = apply_text_placeholders(&value_str, &pattern);
    Ok(formatted)
}

pub fn format_keyword(engine: &mut Engine) {
    // Register FORMAT as a regular function with two parameters
    engine.register_fn("FORMAT", format_impl);
    engine.register_fn("format", format_impl);
}
fn parse_pattern(pattern: &str) -> (String, usize, String) {
    let mut prefix = String::new();
    let mut decimals: usize = 2;
    let mut locale_tag = "en".to_string();
    if pattern.starts_with('C') {
        prefix = "C".to_string();
    } else if pattern.starts_with('N') {
        prefix = "N".to_string();
    }
    let rest = &pattern[1..];
    let mut num_part = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_digit() {
            num_part.push(ch);
        } else {
            break;
        }
    }
    if !num_part.is_empty() {
        decimals = num_part.parse().unwrap_or(2);
    }
    if let Some(start) = pattern.find('[') {
        if let Some(end) = pattern.find(']') {
            if end > start {
                locale_tag = pattern[start + 1..end].to_string();
            }
        }
    }
    (prefix, decimals, locale_tag)
}
fn get_locale(tag: &str) -> Locale {
    match tag {
        "fr" => Locale::fr,
        "de" => Locale::de,
        "pt" => Locale::pt,
        "it" => Locale::it,
        "es" => Locale::es,
        _ => Locale::en,
    }
}
fn get_currency_symbol(tag: &str) -> &'static str {
    match tag {
        "pt" => "R$ ",
        "fr" | "de" | "es" | "it" => "€",
        _ => "$",
    }
}
fn apply_date_format(dt: &NaiveDateTime, pattern: &str) -> String {
    let mut output = pattern.to_string();
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    let hour24 = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    let millis = dt.and_utc().timestamp_subsec_millis();
    output = output.replace("yyyy", &format!("{year:04}"));
    output = output.replace("yy", &format!("{:02}", year % 100));
    output = output.replace("MM", &format!("{month:02}"));
    output = output.replace('M', &format!("{month}"));
    output = output.replace("dd", &format!("{day:02}"));
    output = output.replace('d', &format!("{day}"));
    output = output.replace("HH", &format!("{hour24:02}"));
    output = output.replace('H', &format!("{hour24}"));
    let mut hour12 = hour24 % 12;
    if hour12 == 0 {
        hour12 = 12;
    }
    output = output.replace("hh", &format!("{hour12:02}"));
    output = output.replace('h', &format!("{hour12}"));
    output = output.replace("mm", &format!("{minute:02}"));
    output = output.replace('m', &format!("{minute}"));
    output = output.replace("ss", &format!("{second:02}"));
    output = output.replace('s', &format!("{second}"));
    output = output.replace("fff", &format!("{millis:03}"));
    output = output.replace("tt", if hour24 < 12 { "AM" } else { "PM" });
    output = output.replace('t', if hour24 < 12 { "A" } else { "P" });
    output
}
fn apply_text_placeholders(value: &str, pattern: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    while i < chars.len() {
        match chars[i] {
            '@' => result.push_str(value),
            '&' => {
                result.push_str(&value.to_lowercase());
                if i + 1 < chars.len() {
                    match chars[i + 1] {
                        '!' => {
                            result.push('!');
                            i += 1;
                        }
                        '>' => {
                            i += 1;
                        }
                        _ => (),
                    }
                }
            }
            '>' | '!' => result.push_str(&value.to_uppercase()),
            _ => result.push(chars[i]),
        }
        i += 1;
    }
    result
}
