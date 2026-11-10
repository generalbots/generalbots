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
            // NOW()-style maps carry a date without a time component, so parse a
            // NaiveDate and promote it at midnight — parsing a date-only string
            // with NaiveDateTime always failed and silently returned "".
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&value_str, "%Y-%m-%d") {
                if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                    return Ok(apply_date_format(&dt, &pattern));
                }
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
    // Wrap format_impl to unwrap Result before returning to Rhai
    engine.register_fn("FORMAT", |value: Dynamic, pattern: String| -> String {
        match format_impl(value, pattern) {
            Ok(result) => result,
            Err(e) => {
                log::error!("FORMAT error: {}", e);
                String::new()
            }
        }
    });
    engine.register_fn("format", |value: Dynamic, pattern: String| -> String {
        match format_impl(value, pattern) {
            Ok(result) => result,
            Err(e) => {
                log::error!("format error: {}", e);
                String::new()
            }
        }
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(y, mo, d)
            .and_then(|date| date.and_hms_opt(h, mi, s))
            .expect("valid datetime")
    }

    #[test]
    fn parse_pattern_defaults_to_two_decimals_english() {
        assert_eq!(parse_pattern("N"), ("N".to_string(), 2, "en".to_string()));
        assert_eq!(parse_pattern("C0"), ("C".to_string(), 0, "en".to_string()));
        assert_eq!(parse_pattern("N3[pt]"), ("N".to_string(), 3, "pt".to_string()));
        assert_eq!(parse_pattern("C2[de]"), ("C".to_string(), 2, "de".to_string()));
    }

    #[test]
    fn parse_pattern_handles_missing_or_malformed_locale() {
        assert_eq!(parse_pattern("N1[fr"), ("N".to_string(), 1, "en".to_string()));
        assert_eq!(parse_pattern("N1[]"), ("N".to_string(), 1, "".to_string()));
    }

    #[test]
    fn get_locale_and_currency_symbol_cover_every_tag() {
        assert_eq!(get_locale("fr"), Locale::fr);
        assert_eq!(get_locale("de"), Locale::de);
        assert_eq!(get_locale("pt"), Locale::pt);
        assert_eq!(get_locale("it"), Locale::it);
        assert_eq!(get_locale("es"), Locale::es);
        assert_eq!(get_locale("xx"), Locale::en);
        assert_eq!(get_currency_symbol("pt"), "R$ ");
        assert_eq!(get_currency_symbol("fr"), "€");
        assert_eq!(get_currency_symbol("de"), "€");
        assert_eq!(get_currency_symbol("es"), "€");
        assert_eq!(get_currency_symbol("it"), "€");
        assert_eq!(get_currency_symbol("en"), "$");
    }

    #[test]
    fn apply_date_format_expands_every_token() {
        let value = dt(2026, 3, 7, 14, 5, 9);
        assert_eq!(apply_date_format(&value, "yyyy-MM-dd"), "2026-03-07");
        assert_eq!(apply_date_format(&value, "yy/M/d"), "26/3/7");
        assert_eq!(apply_date_format(&value, "HH:mm:ss"), "14:05:09");
        assert_eq!(apply_date_format(&value, "hh:mm tt"), "02:05 PM");
        assert_eq!(apply_date_format(&value, "h:m t"), "2:5 P");
        assert_eq!(apply_date_format(&value, "fff"), "000");
    }

    #[test]
    fn apply_date_format_midnight_and_noon_map_to_12() {
        assert_eq!(apply_date_format(&dt(2026, 1, 1, 0, 0, 0), "hh tt"), "12 AM");
        assert_eq!(apply_date_format(&dt(2026, 1, 1, 12, 0, 0), "hh tt"), "12 PM");
    }

    #[test]
    fn apply_text_placeholders_handles_case_markers() {
        assert_eq!(apply_text_placeholders("Ana", "Olá @"), "Olá Ana");
        assert_eq!(apply_text_placeholders("Ana", "@"), "Ana");
        assert_eq!(apply_text_placeholders("Ana", "&"), "ana");
        assert_eq!(apply_text_placeholders("Ana", "&>"), "ana");
        assert_eq!(apply_text_placeholders("Ana", "&!"), "ana!");
        assert_eq!(apply_text_placeholders("Ana", ">"), "ANA");
        assert_eq!(apply_text_placeholders("Ana", "!"), "ANA");
        assert_eq!(apply_text_placeholders("Ana", "plain"), "plain");
    }

    #[test]
    fn format_impl_formats_plain_numbers() {
        assert_eq!(format_impl(Dynamic::from(42_i64), "N0".into()).unwrap(), "42");
        assert_eq!(format_impl(Dynamic::from(1234_i64), "N0".into()).unwrap(), "1,234");
        assert_eq!(format_impl(Dynamic::from(12.5_f64), "n".into()).unwrap(), "12.50");
        assert_eq!(format_impl(Dynamic::from(12.5_f64), "F".into()).unwrap(), "12.50");
        assert_eq!(format_impl(Dynamic::from(0.25_f64), "0%".into()).unwrap(), "25%");
    }

    #[test]
    fn format_impl_applies_currency_and_locale_separators() {
        assert_eq!(format_impl(Dynamic::from(1234.5_f64), "C2".into()).unwrap(), "$1,234.50");
        assert_eq!(format_impl(Dynamic::from(1234.5_f64), "C2[pt]".into()).unwrap(), "R$ 1.234,50");
        assert_eq!(format_impl(Dynamic::from(1234.5_f64), "N2[de]".into()).unwrap(), "1.234,50");
        assert_eq!(format_impl(Dynamic::from(1234.5_f64), "N0[fr]".into()).unwrap(), "1\u{202f}234");
    }

    #[test]
    fn format_impl_formats_datetime_strings() {
        assert_eq!(
            format_impl(Dynamic::from("2026-11-10 08:30:00"), "dd/MM/yyyy".into()).unwrap(),
            "10/11/2026"
        );
    }

    #[test]
    fn format_impl_reads_now_map_formatted_property() {
        let mut map = Map::new();
        map.insert("formatted".into(), Dynamic::from("2026-11-10 08:30:00"));
        assert_eq!(format_impl(Dynamic::from(map), "yyyy".into()).unwrap(), "2026");
    }

    #[test]
    fn format_impl_falls_back_to_map_date_components() {
        let mut map = Map::new();
        map.insert("year".into(), Dynamic::from(2026_i64));
        map.insert("month".into(), Dynamic::from(2_i64));
        map.insert("day".into(), Dynamic::from(3_i64));
        assert_eq!(format_impl(Dynamic::from(map), "yyyy-MM-dd".into()).unwrap(), "2026-02-03");
    }

    #[test]
    fn format_impl_returns_empty_for_unsupported_map() {
        assert_eq!(format_impl(Dynamic::from(Map::new()), "yyyy".into()).unwrap(), "");
    }

    #[test]
    fn format_impl_treats_non_numeric_text_as_text() {
        assert_eq!(format_impl(Dynamic::from("Ana"), "Olá @".into()).unwrap(), "Olá Ana");
        assert_eq!(format_impl(Dynamic::from("Ana"), ">".into()).unwrap(), "ANA");
    }

    #[test]
    fn format_keyword_registers_both_casings() {
        let mut engine = Engine::new();
        format_keyword(&mut engine);
        assert_eq!(
            engine.eval::<String>(r#"FORMAT(1234, "N0")"#).expect("FORMAT"),
            "1,234"
        );
        assert_eq!(
            engine.eval::<String>(r#"format("Ana", "&")"#).expect("format"),
            "ana"
        );
    }
}
