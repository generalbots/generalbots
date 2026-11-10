//! Pure CSV cell/amount helpers shared by the leads and contacts surfaces
//! (#1451, #1456).
//!
//! Everything here works on in-memory strings: no database, no filesystem, no
//! clock. Splitting them out of `csv_io` keeps both files inside the 450-line
//! budget and makes the import rules testable without a pool.

use axum::http::StatusCode;

use crate::requests::CsvLeadRow;

/// Quotes one CSV cell when it contains a separator, a quote or a newline.
pub(crate) fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Quotes an optional cell, emitting an empty field for `None`.
pub(crate) fn csv_opt(value: &Option<String>) -> String {
    csv_cell(value.as_deref().unwrap_or(""))
}

/// Renders an optional number, emitting an empty field for `None`.
pub(crate) fn csv_number(value: Option<f64>) -> String {
    value.map(|v| format!("{v}")).unwrap_or_default()
}

/// Parses an amount written in any of the formats spreadsheets produce:
/// `1000`, `1000.50`, `1.234,56` (pt-BR), `1,234.56` (en-US), `R$ 2.500,00`.
///
/// A single comma is read as a decimal separator (pt-BR `1,5`); when both
/// separators appear, the rightmost one is the decimal separator. Currency
/// symbols and spaces are ignored. `None` means "no amount", never "zero".
pub(crate) fn parse_amount(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'))
        .collect();
    if cleaned.is_empty() {
        return None;
    }

    let normalised = match (cleaned.rfind('.'), cleaned.rfind(',')) {
        (Some(dot), Some(comma)) if comma > dot => cleaned.replace('.', "").replace(',', "."),
        (Some(_), Some(_)) => cleaned.replace(',', ""),
        (None, Some(_)) => cleaned.replace(',', "."),
        _ => cleaned,
    };

    normalised.parse::<f64>().ok()
}

/// Trims and lowercases an e-mail so the dedupe key is stable across
/// spreadsheets: `Ana@Example.test` and `ana@example.test` are one contact.
/// A blank cell is "no e-mail", not an empty address.
pub(crate) fn normalise_email(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
}

/// Parses the leads import body. Header names are matched case-insensitively,
/// values are trimmed, blank cells become `None` and unknown columns are
/// ignored, so an operator's spreadsheet imports without a schema agreement.
pub(crate) fn parse_leads_csv(body: &str) -> Result<Vec<CsvLeadRow>, (StatusCode, String)> {
    let mut rows = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(body.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid CSV headers: {e}")))?
        .clone();
    let idx_of = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));
    for rec in reader.records() {
        let rec = rec.map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid CSV: {e}")))?;
        let get = |name: &str| -> Option<String> {
            let idx = idx_of(name)?;
            rec.get(idx)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        rows.push(CsvLeadRow {
            title: get("title"),
            first_name: get("first_name"),
            last_name: get("last_name"),
            email: get("email"),
            phone: get("phone"),
            company: get("company"),
            job_title: get("job_title"),
            source: get("source"),
            value: get("value").and_then(|v| parse_amount(&v)),
            currency: get("currency"),
            stage: get("stage"),
        });
    }
    Ok(rows)
}

#[cfg(test)]
#[path = "csv_parse_tests.rs"]
mod csv_parse_tests;
