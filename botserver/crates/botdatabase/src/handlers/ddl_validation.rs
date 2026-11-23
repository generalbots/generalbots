//! #2772 — allowlist validation for the two DDL fragments that cannot be
//! bound as query parameters. PostgreSQL does not accept `$1` placeholders in
//! a column type or in a `DEFAULT` expression, so these client-supplied fields
//! are reduced to a canonical, injection-free form before interpolation.
//!
//! Everything else in the DDL path already flows through
//! [`sanitize_identifier`] and an equality check.

use super::*;

/// Reject oversized payloads outright instead of truncating them — truncation
/// can hide a second statement past the cut point.
const MAX_TOKEN_LEN: usize = 64;

/// SQL types accepted by the Database pane. The UI offers a subset of these;
/// the wider set keeps programmatic/AI clients working while still being a
/// closed allowlist (#2772).
const ALLOWED_BASE_TYPES: &[&str] = &[
    "uuid",
    "text",
    "varchar",
    "character varying",
    "char",
    "character",
    "bpchar",
    "integer",
    "int",
    "int4",
    "bigint",
    "int8",
    "smallint",
    "int2",
    "decimal",
    "numeric",
    "real",
    "float",
    "float4",
    "float8",
    "double precision",
    "money",
    "boolean",
    "bool",
    "date",
    "time",
    "timetz",
    "timestamp",
    "timestamptz",
    "timestamp with time zone",
    "timestamp without time zone",
    "json",
    "jsonb",
    "bytea",
    "serial",
    "bigserial",
    "smallserial",
    "interval",
    "inet",
    "cidr",
    "macaddr",
    "xml",
    "point",
    "line",
    "lseg",
    "box",
    "path",
    "polygon",
    "circle",
];

/// Safe, side-effect-free `DEFAULT` expressions accepted verbatim (from the
/// allowlist, never from the request body).
const SAFE_DEFAULT_EXPRS: &[&str] = &[
    "NULL",
    "TRUE",
    "FALSE",
    "NOW()",
    "CURRENT_TIMESTAMP",
    "CURRENT_DATE",
    "CURRENT_TIME",
    "GEN_RANDOM_UUID()",
    "UUID_GENERATE_V4()",
];

/// Validate a client-supplied column type and return a canonical fragment that
/// is safe to interpolate into `CREATE TABLE` / `ALTER TABLE`.
///
/// The returned string is rebuilt from the allowlisted base type, digits-only
/// precision/scale and a fixed `[]` suffix, so no request-body character other
/// than an allowlisted type name can reach the SQL text.
pub(super) fn validate_data_type(
    raw: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TOKEN_LEN {
        return Err(error_response("Invalid column data type"));
    }

    // Optional trailing array marker, e.g. TEXT[].
    let (core, array_suffix) = match trimmed.strip_suffix("[]") {
        Some(rest) => (rest.trim(), "[]"),
        None => (trimmed, ""),
    };

    // Split the base type from an optional (precision[, scale]) suffix.
    let (base_raw, params_raw) = match core.find('(') {
        Some(open) => {
            let base = core[..open].trim();
            let rest = core[open..].trim();
            if !rest.ends_with(')') {
                return Err(error_response("Invalid column data type"));
            }
            (base, Some(rest[1..rest.len() - 1].trim()))
        }
        None => (core, None),
    };

    // Collapse whitespace so `character  varying(255)` matches the allowlist;
    // `;`, `--`, quotes and comment markers never match an allowlisted name.
    let collapsed: String = base_raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let base = ALLOWED_BASE_TYPES
        .iter()
        .find(|known| known.eq_ignore_ascii_case(&collapsed))
        .ok_or_else(|| error_response("Unsupported column data type"))?;

    // Precision/scale is rebuilt from digits and separators only — never from
    // the raw substring, which could carry `); DROP TABLE ...`.
    let params = match params_raw {
        Some(p) => {
            if p.is_empty()
                || p.len() > 16
                || !p.chars().all(|c| c.is_ascii_digit() || c == ',' || c == ' ')
            {
                return Err(error_response("Invalid column data type precision"));
            }
            format!("({})", p.split_whitespace().collect::<Vec<_>>().join(","))
        }
        None => String::new(),
    };

    Ok(format!("{base}{params}{array_suffix}"))
}

/// Validate a client-supplied `DEFAULT` literal and return a single,
/// well-formed SQL expression. Bind parameters are not legal in a DDL default,
/// so the body string is never passed through: only allowlisted expressions,
/// numeric literals and strictly escaped single-quoted literals are emitted.
pub(super) fn validate_default(
    raw: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TOKEN_LEN {
        return Err(error_response("Invalid column default"));
    }

    let upper = trimmed.to_ascii_uppercase();
    if SAFE_DEFAULT_EXPRS.contains(&upper.as_str()) {
        return Ok(upper);
    }

    // Plain numeric literal: optional leading '-', digits and at most one '.'.
    let numeric = trimmed.strip_prefix('-').unwrap_or(trimmed);
    if !numeric.is_empty()
        && numeric.matches('.').count() <= 1
        && numeric
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
    {
        return Ok(trimmed.to_string());
    }

    // A single-quoted literal. Terminators and comment introducers are
    // rejected even inside the quotes; embedded quotes are doubled so the
    // result stays one literal and cannot close the string early.
    if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        let inner = &trimmed[1..trimmed.len() - 1];
        // Decode existing doubled quotes before inspecting the value, so a
        // legitimately escaped `'O''Brien'` round-trips unchanged.
        let decoded = inner.replace("''", "'");
        let has_forbidden_char = decoded
            .chars()
            .any(|c| matches!(c, ';' | '\\' | '\n' | '\r'));
        if !has_forbidden_char
            && !decoded.contains("--")
            && !decoded.contains("/*")
            && !decoded.contains("*/")
        {
            return Ok(format!("'{}'", decoded.replace('\'', "''")));
        }
    }

    Err(error_response("Invalid column default"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_type(raw: &str) -> String {
        match validate_data_type(raw) {
            Ok(v) => v,
            Err(_) => panic!("expected valid data type: {raw}"),
        }
    }

    fn ok_default(raw: &str) -> String {
        match validate_default(raw) {
            Ok(v) => v,
            Err(_) => panic!("expected valid default: {raw}"),
        }
    }

    #[test]
    fn rejects_injection_payloads_in_data_type() {
        let attacks = [
            "UUID); DROP TABLE users; --",
            "VARCHAR(255); DROP TABLE users; --",
            "TEXT; DELETE FROM users; --",
            "INT--",
            "INT /**/",
            "TEXT' OR '1'='1",
            "TEXT /* comment */",
            "VARCHAR(255) /**/--",
            "UUID(1); --",
            "TEXT\n; DROP TABLE users",
            "VARCHAR(255))",
            "NUMERIC(10,(2))",
            "TEXT[]; DROP TABLE users",
            "'; DROP TABLE users; --",
            "TEXT --",
        ];
        for payload in attacks {
            assert!(
                validate_data_type(payload).is_err(),
                "data-type payload wasn't rejected: {payload}"
            );
        }
    }

    #[test]
    fn accepts_legitimate_data_types() {
        let cases = [
            ("UUID", "uuid"),
            ("TEXT", "text"),
            ("VARCHAR(255)", "varchar(255)"),
            ("character varying(255)", "character varying(255)"),
            ("INTEGER", "integer"),
            ("BIGINT", "bigint"),
            ("DECIMAL", "decimal"),
            ("DECIMAL(10,2)", "decimal(10,2)"),
            ("BOOLEAN", "boolean"),
            ("DATE", "date"),
            ("TIMESTAMPTZ", "timestamptz"),
            ("JSONB", "jsonb"),
            ("TEXT[]", "text[]"),
            ("VARCHAR ( 255 )", "varchar(255)"),
        ];
        for (input, expected) in cases {
            assert_eq!(ok_type(input), expected, "type: {input}");
        }
    }

    #[test]
    fn rejects_injection_payloads_in_default() {
        let attacks = [
            "'; DROP TABLE users; --",
            "1; --",
            "0, x); DROP TABLE users; --",
            "0); DROP TABLE users",
            "now(); DROP TABLE users",
            "'x'; DROP TABLE users",
            "true--",
            "NULL; DELETE FROM users",
            "'a'||(SELECT 1)--",
            "'a\nb'; DROP TABLE users",
            "0 /* comment */",
            "gen_random_uuid(); DROP TABLE users",
            "';--",
            "1' OR '1'='1",
        ];
        for payload in attacks {
            assert!(
                validate_default(payload).is_err(),
                "default payload wasn't rejected: {payload}"
            );
        }
    }

    #[test]
    fn accepts_legitimate_defaults() {
        let cases = [
            ("0", "0"),
            ("-1", "-1"),
            ("1.5", "1.5"),
            ("TRUE", "TRUE"),
            ("FALSE", "FALSE"),
            ("NULL", "NULL"),
            ("now()", "NOW()"),
            ("gen_random_uuid()", "GEN_RANDOM_UUID()"),
            ("'active'", "'active'"),
            ("'O''Brien'", "'O''Brien'"),
        ];
        for (input, expected) in cases {
            assert_eq!(ok_default(input), expected, "default: {input}");
        }
    }
}
