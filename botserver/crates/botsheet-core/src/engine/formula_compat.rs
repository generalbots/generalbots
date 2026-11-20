//! Deterministic formula normalization for XLSX round-trips (#2572).
//!
//! A formula authored in the local preview may use a localized function name,
//! a locale decimal mark and a semicolon argument separator. XLSX stores the
//! canonical English name, a dot decimal mark, a comma separator and uppercase
//! error literals. Normalizing to that single form on both import and export is
//! what makes an open/save cycle semantics-preserving and idempotent.

use super::value::canonical_error_literal;
use crate::types::FormulaLocale;

/// Single source of truth for localized function names, shared by the legacy
/// dispatcher and by workbook normalization. Keys are matched case-insensitively.
pub fn canonical_function_name(name: &str) -> Option<&'static str> {
    let key = name.trim().to_uppercase();
    Some(match key.as_str() {
        "SOMA" | "SUMA" | "SOMME" | "SUMME" | "合計" | "합계" | "求和" => "SUM",
        "MÉDIA" | "MEDIA" | "PROMEDIO" | "MOYENNE" | "MITTELWERT" | "平均" | "평균"
        | "平均值" => "AVERAGE",
        "CONT" | "CONTAR" | "NB" | "ANZAHL" | "CONT.NÚM" | "件数" | "개수" | "计数" => {
            "COUNT"
        }
        "CONT.VALORES" => "COUNTA",
        "CONT.SE" | "CONTAR.SI" | "NB.SI" | "ZÄHLENWENN" | "COUNTIF" => "COUNTIF",
        "SOMASE" => "SUMIF",
        "MÉDIASE" | "MEDIASE" => "AVERAGEIF",
        "SOMASES" => "SUMIFS",
        "MÉDIASES" | "MEDIASES" => "AVERAGEIFS",
        "CONT.SES" => "COUNTIFS",
        "MÁXIMO" | "MAXIMO" | "最大" | "최대" | "最大值" => "MAX",
        "MÍNIMO" | "MINIMO" | "最小" | "최소" | "最小值" => "MIN",
        "SE" | "SI" | "WENN" | "もし" | "만약" | "如果" => "IF",
        "ARRED" | "ARREDONDA" | "REDONDEAR" | "ARRONDI" | "RUNDEN" | "四捨五入"
        | "반올림" | "四舍五入" => "ROUND",
        "MULT" => "PRODUCT",
        _ => return None,
    })
}

/// Uppercases the formula skeleton while leaving quoted text untouched, then
/// rewrites every call name to its canonical English spelling.
///
/// The legacy dispatcher used to uppercase the whole formula, which broke
/// `TEXTJOIN("-",TRUE,"a","","b")` by returning `A-B`. Canonicalizing each call
/// name separately keeps aliases and nested calls working without changing the
/// user's string literals.
pub fn canonicalize_dispatch_formula(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let mut in_string = false;

    while i < chars.len() {
        let current = chars[i];
        if in_string {
            out.push(current);
            if current == '"' {
                if chars.get(i + 1).is_some_and(|next| *next == '"') {
                    out.push('"');
                    i += 2;
                    continue;
                }
                in_string = false;
            }
            i += 1;
            continue;
        }
        if current == '"' {
            in_string = true;
            out.push(current);
            i += 1;
            continue;
        }
        if current.is_alphabetic() || current == '_' || current == '$' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || matches!(chars[i], '_' | '$' | '!' | '.'))
            {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            let upper = raw.to_uppercase();
            let mut lookahead = i;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && chars[lookahead] == '(' {
                if let Some(canonical) = canonical_function_name(&upper) {
                    out.push_str(canonical);
                } else {
                    out.push_str(&upper);
                }
            } else {
                out.push_str(&upper);
            }
            continue;
        }
        out.push(current);
        i += 1;
    }
    out
}

/// Rewrites a formula into the canonical XLSX form for `locale`.
///
/// * localized call names become English uppercase (nested calls included);
/// * locale decimal/group marks become a single dot;
/// * `;` becomes the comma argument separator;
/// * error literals become their canonical uppercase spelling;
/// * `[...]` preview arrays keep their brackets and dot-decimal literals;
/// * quoted text, `$` anchors, sheet qualifiers and `A1:B2` ranges are kept.
pub fn normalize_formula(input: &str, locale: FormulaLocale) -> String {
    // A formula that already uses canonical punctuation and English names is
    // left alone, so normalizing an exported formula twice is a no-op. A
    // semicolon or a localized call name proves the formula is still locale
    // authored and must be converted with the caller's `locale`.
    let source = if locale.decimal_separator() == ',' && !input.contains(';') && !has_localized_call(input) {
        FormulaLocale::En
    } else {
        locale
    };
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let mut in_string = false;
    let mut bracket_depth = 0usize;

    while i < chars.len() {
        let current = chars[i];
        if in_string {
            out.push(current);
            if current == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    out.push('"');
                    i += 2;
                    continue;
                }
                in_string = false;
            }
            i += 1;
            continue;
        }
        if current == '"' {
            in_string = true;
            out.push(current);
            i += 1;
            continue;
        }
        // Preview arrays always use comma-separated, dot-decimal literals,
        // even under a comma-decimal locale. The brackets are preserved so
        // array-taking functions such as IRR still see one argument.
        let number_locale = if bracket_depth > 0 {
            FormulaLocale::En
        } else {
            source
        };
        if current.is_ascii_digit()
            || current == number_locale.decimal_separator()
            || current == number_locale.group_separator()
        {
            if let Some((number, next)) = read_number(&chars, i, number_locale) {
                out.push_str(&number);
                i = next;
                continue;
            }
        }
        if current == '#' && chars.get(i + 1).is_some_and(|ch| ch.is_alphabetic()) {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || "#/?!_%".contains(chars[i]))
            {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            match canonical_error_literal(&raw) {
                Some(canonical) => out.push_str(canonical),
                None => out.push_str(&raw),
            }
            continue;
        }
        if current.is_alphabetic() || current == '_' || current == '$' {
            let start = i;
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_alphanumeric() || ch == '_' || ch == '$' || ch == '!' || ch == '.' {
                    i += 1;
                } else {
                    break;
                }
            }
            let raw: String = chars[start..i].iter().collect();
            let mut lookahead = i;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && chars[lookahead] == '(' {
                if let Some(canonical) = canonical_function_name(&raw) {
                    out.push_str(canonical);
                } else {
                    out.push_str(&raw.to_uppercase());
                }
            } else {
                out.push_str(&raw);
            }
            continue;
        }
        match current {
            ';' => out.push(','),
            '[' => {
                bracket_depth += 1;
                out.push('[');
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                out.push(']');
            }
            other => out.push(other),
        }
        i += 1;
    }
    out
}

/// True when at least one call uses a localized name rather than its canonical
/// English one. Used to avoid re-interpreting canonical numbers as locale text.
fn has_localized_call(input: &str) -> bool {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if !chars[index].is_alphabetic() && chars[index] != '_' {
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len()
            && (chars[index].is_alphanumeric() || chars[index] == '_' || chars[index] == '.')
        {
            index += 1;
        }
        let name: String = chars[start..index].iter().collect();
        let mut lookahead = index;
        while lookahead < chars.len() && chars[lookahead].is_whitespace() {
            lookahead += 1;
        }
        if lookahead < chars.len() && chars[lookahead] == '(' {
            if let Some(canonical) = canonical_function_name(&name) {
                if name.to_uppercase() != canonical {
                    return true;
                }
            }
        }
    }
    false
}

fn read_number(chars: &[char], start: usize, locale: FormulaLocale) -> Option<(String, usize)> {
    let decimal = locale.decimal_separator();
    let group = locale.group_separator();
    // A comma is a grouping mark only in locales whose decimal mark is a
    // comma; elsewhere it stays an argument separator.
    let grouped_decimal = decimal == ',';
    let mut i = start;
    let mut raw = String::new();
    let mut seen_digit = false;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_ascii_digit() {
            raw.push(ch);
            seen_digit = true;
            i += 1;
            continue;
        }
        let is_mark = ch == decimal || (grouped_decimal && group != ' ' && ch == group);
        if is_mark && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            raw.push(ch);
            i += 1;
            continue;
        }
        break;
    }
    if !seen_digit {
        return None;
    }
    if grouped_decimal && raw.contains(',') {
        let canonical: String = raw
            .chars()
            .filter(|ch| group == ' ' || *ch != group)
            .map(|ch| if ch == ',' { '.' } else { ch })
            .collect();
        Some((canonical, i))
    } else {
        Some((raw, i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_br_formula_normalizes_to_xlsx_form() {
        assert_eq!(
            normalize_formula("=SOMA(1,5;2,5)", FormulaLocale::PtBr),
            "=SUM(1.5,2.5)"
        );
        assert_eq!(
            normalize_formula("=MÉDIA(A1:A3)", FormulaLocale::PtBr),
            "=AVERAGE(A1:A3)"
        );
    }

    #[test]
    fn nested_localized_calls_are_canonicalized() {
        let normalized = normalize_formula("=ARRED(MÉDIA(A1;A2);2)", FormulaLocale::PtBr);
        assert_eq!(normalized, "=ROUND(AVERAGE(A1,A2),2)");
    }

    #[test]
    fn separators_and_anchors_survive() {
        assert_eq!(
            normalize_formula("=SOMASE($A$1:$A$3;\">1\";Sheet2!B1)", FormulaLocale::PtBr),
            "=SUMIF($A$1:$A$3,\">1\",Sheet2!B1)"
        );
        assert_eq!(
            normalize_formula("=SUM(A1:B2,1.5)", FormulaLocale::En),
            "=SUM(A1:B2,1.5)"
        );
    }

    #[test]
    fn error_literals_and_arrays_are_canonical() {
        // Bracket array literals are the engine's preview syntax. They stay
        // bracketed so array-taking functions such as IRR still receive one
        // argument; only their numeric literals are canonicalized to dot
        // decimals, even under a comma-decimal locale (see differential_tests).
        assert_eq!(
            normalize_formula("=IFNA(#n/a;IRR([-1000,100,200]))", FormulaLocale::PtBr),
            "=IFNA(#N/A,IRR([-1000,100,200]))"
        );
        assert_eq!(
            normalize_formula("=IF(#div/0!;0;1)", FormulaLocale::PtBr),
            "=IF(#DIV/0!,0,1)"
        );
    }

    #[test]
    fn normalization_is_idempotent() {
        for locale in FormulaLocale::ALL {
            for formula in [
                "=SOMA(1.234,50;2,5)",
                "=SUM(A1:B2,1.5)",
                "=MÁXIMO($A$1:$A$3)",
                "=IF(TRUE;\"Olá, mundo\";0)",
            ] {
                let once = normalize_formula(formula, locale);
                assert_eq!(normalize_formula(&once, locale), once, "unstable for {formula}");
            }
        }
    }

    #[test]
    fn all_preview_aliases_map_to_dispatched_names() {
        assert_eq!(canonical_function_name("soma"), Some("SUM"));
        assert_eq!(canonical_function_name("CONT.SE"), Some("COUNTIF"));
        assert_eq!(canonical_function_name("ZÄHLENWENN"), Some("COUNTIF"));
        assert_eq!(canonical_function_name("求和"), Some("SUM"));
        assert_eq!(canonical_function_name("UNKNOWN"), None);
    }
}
