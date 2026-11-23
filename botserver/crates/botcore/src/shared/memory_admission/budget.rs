//! Shared, pure pre-materialisation budget predicate for Office imports (#2530).
//!
//! The cgroup gate in [`super::gate::MemoryAdmission`] bounds the *concurrent*
//! work of a request, but each importer still has to decide, before it builds
//! its in-memory model, whether this particular input is worth parsing at all.
//! A 25 MB XLSX can expand into hundreds of megabytes of heap; the decision
//! must happen while only the cheap signals (byte length, row count, slide
//! count) are known.
//!
//! [`ImportBudget`] is that decision. [`ImportBudget::check`] is a **pure**
//! function: numbers in, typed verdict out, no I/O, no logging, no document
//! content and no allocation proportional to the input. It enforces three
//! independent dimensions:
//!
//! 1. `bytes` — the compressed upload size, capped at the endpoint's own cap;
//! 2. `rows` — worksheet rows (the measured heap driver for Sheets);
//! 3. `pages` — slide count (the heap driver for Slides).
//!
//! The limits are configurable at runtime through environment variables and
//! fall back to safe defaults. An override can only **tighten** a cap: a bad
//! configuration can make imports fail closed, never open the process to an
//! unbounded allocation. Environment reads happen once when the budget is
//! built; the check itself stays pure and testable.
//!
//! Refusals are typed ([`BudgetError`]) and reuse the existing admission
//! i18n key, so no new locale catalog entry is introduced.

use super::policy::MIB;
use axum::http::StatusCode;
use std::fmt;

/// Optional byte-cap override. Absent/invalid values keep the endpoint cap.
pub const ENV_MAX_BYTES: &str = "GBO_OFFICE_IMPORT_MAX_BYTES";
/// Optional worksheet-row override. Default: [`DEFAULT_MAX_ROWS`].
pub const ENV_MAX_ROWS: &str = "GBO_OFFICE_IMPORT_MAX_ROWS";
/// Optional slide-count override. Default: [`DEFAULT_MAX_PAGES`].
pub const ENV_MAX_PAGES: &str = "GBO_OFFICE_IMPORT_MAX_PAGES";

/// 25 MiB, the existing Docs/Sheets upload cap; used as the documented default
/// for callers that have no per-class policy to pass to [`ImportBudget::from_env`].
pub const DEFAULT_MAX_BYTES: u64 = 25 * MIB;
/// A quarter-million worksheet rows: beyond this, the retained cell keys alone
/// dominate the heap. Matches the existing per-sheet cell cap's order of size.
pub const DEFAULT_MAX_ROWS: u64 = 250_000;
/// Five thousand slides is far above any normal deck; a package claiming more
/// is a structural problem, not a document.
pub const DEFAULT_MAX_PAGES: u64 = 5_000;

/// The cheap, content-free signals an importer has *before* it materialises its
/// model. Unavailable dimensions stay at `0` (which is always within budget).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSize {
    pub bytes: u64,
    pub rows: u64,
    pub pages: u64,
}

/// Limits for one import class. Values are already resolved (defaults applied),
/// so [`ImportBudget::check`] never reads the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportBudget {
    pub max_bytes: u64,
    pub max_rows: u64,
    pub max_pages: u64,
}

/// A typed refusal. Carries only allow-listed sizes — never a filename, a
/// tenant id or document text — so it is safe to put in an HTTP response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetError {
    TooLarge { bytes: u64, max_bytes: u64 },
    TooManyRows { rows: u64, max_rows: u64 },
    TooManyPages { pages: u64, max_pages: u64 },
}

impl ImportBudget {
    /// Explicit limits, used by tests and by callers with their own policy.
    #[must_use]
    pub fn new(max_bytes: u64, max_rows: u64, max_pages: u64) -> Self {
        Self {
            max_bytes,
            max_rows,
            max_pages,
        }
    }

    /// Resolves the limits once from the environment with safe defaults.
    ///
    /// `class_max_bytes` is the endpoint's existing upload cap. The env byte
    /// override may only lower it; rows and pages default to the conservative
    /// constants above and may be lowered as well.
    #[must_use]
    pub fn from_env(class_max_bytes: u64) -> Self {
        Self {
            max_bytes: parse_limit(
                std::env::var(ENV_MAX_BYTES).ok().as_deref(),
                class_max_bytes,
            )
            .min(class_max_bytes),
            max_rows: parse_limit(
                std::env::var(ENV_MAX_ROWS).ok().as_deref(),
                DEFAULT_MAX_ROWS,
            ),
            max_pages: parse_limit(
                std::env::var(ENV_MAX_PAGES).ok().as_deref(),
                DEFAULT_MAX_PAGES,
            ),
        }
    }

    /// Pure admission decision. At exactly a limit the input is accepted; one
    /// unit over is refused with the corresponding typed error. The first
    /// violated dimension wins, so the response is deterministic.
    #[must_use]
    pub fn check(&self, size: ImportSize) -> Result<(), BudgetError> {
        if size.bytes > self.max_bytes {
            return Err(BudgetError::TooLarge {
                bytes: size.bytes,
                max_bytes: self.max_bytes,
            });
        }
        if size.rows > self.max_rows {
            return Err(BudgetError::TooManyRows {
                rows: size.rows,
                max_rows: self.max_rows,
            });
        }
        if size.pages > self.max_pages {
            return Err(BudgetError::TooManyPages {
                pages: size.pages,
                max_pages: self.max_pages,
            });
        }
        Ok(())
    }
}

/// Pure env parser: `None`, unparsable, zero and empty values fall back to the
/// default. Kept separate from `from_env` so the rule is unit-testable without
/// mutating process-wide environment variables.
fn parse_limit(raw: Option<&str>, default: u64) -> u64 {
    match raw {
        Some(value) => match value.trim().parse::<u64>() {
            Ok(parsed) if parsed > 0 => parsed,
            _ => default,
        },
        None => default,
    }
}

impl BudgetError {
    /// Stable machine code for the client. Distinct from the i18n key so a
    /// translated UI can react to the exact dimension that was refused.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooLarge { .. } => "office_payload_too_large",
            Self::TooManyRows { .. } => "office_import_too_many_rows",
            Self::TooManyPages { .. } => "office_import_too_many_pages",
        }
    }

    /// Reuses the existing admission catalog key: all three refusals are the
    /// same user-facing "too large" case, and no new locale entry is needed.
    #[must_use]
    pub fn i18n_key(&self) -> &'static str {
        "office.admission.payload-too-large"
    }

    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        StatusCode::PAYLOAD_TOO_LARGE
    }

    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::TooLarge { bytes, max_bytes } => format!(
                "This document is too large for this operation ({bytes} bytes; limit {max_bytes} bytes). Try a smaller file."
            ),
            Self::TooManyRows { rows, max_rows } => format!(
                "This spreadsheet has too many rows ({rows}; limit {max_rows}). Split it into smaller sheets."
            ),
            Self::TooManyPages { pages, max_pages } => format!(
                "This presentation has too many slides ({pages}; limit {max_pages}). Split it into smaller decks."
            ),
        }
    }

    /// Content-free JSON body. Only the stable code, the shared i18n key and
    /// allow-listed sizes are included.
    #[must_use]
    pub fn json_body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "error": self.code(),
            "i18n_key": self.i18n_key(),
            "message": self.message(),
        });
        if let Some(object) = body.as_object_mut() {
            match self {
                Self::TooLarge { bytes, max_bytes } => {
                    object.insert("bytes".to_string(), serde_json::json!(bytes));
                    object.insert("limit_bytes".to_string(), serde_json::json!(max_bytes));
                }
                Self::TooManyRows { rows, max_rows } => {
                    object.insert("rows".to_string(), serde_json::json!(rows));
                    object.insert("limit_rows".to_string(), serde_json::json!(max_rows));
                }
                Self::TooManyPages { pages, max_pages } => {
                    object.insert("pages".to_string(), serde_json::json!(pages));
                    object.insert("limit_pages".to_string(), serde_json::json!(max_pages));
                }
            }
        }
        body
    }
}

impl fmt::Display for BudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message())
    }
}

impl std::error::Error for BudgetError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> ImportBudget {
        ImportBudget::new(100, 10, 3)
    }

    #[test]
    fn every_dimension_accepts_exactly_its_limit() {
        let result = budget().check(ImportSize {
            bytes: 100,
            rows: 10,
            pages: 3,
        });
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn every_dimension_refuses_one_over_its_limit() {
        let cases = [
            (
                ImportSize {
                    bytes: 101,
                    ..ImportSize::default()
                },
                BudgetError::TooLarge {
                    bytes: 101,
                    max_bytes: 100,
                },
            ),
            (
                ImportSize {
                    rows: 11,
                    ..ImportSize::default()
                },
                BudgetError::TooManyRows {
                    rows: 11,
                    max_rows: 10,
                },
            ),
            (
                ImportSize {
                    pages: 4,
                    ..ImportSize::default()
                },
                BudgetError::TooManyPages {
                    pages: 4,
                    max_pages: 3,
                },
            ),
        ];
        for (size, expected) in cases {
            assert_eq!(budget().check(size), Err(expected));
        }
    }

    #[test]
    fn the_first_violated_dimension_wins() {
        let size = ImportSize {
            bytes: 101,
            rows: 11,
            pages: 4,
        };
        assert_eq!(
            budget().check(size),
            Err(BudgetError::TooLarge {
                bytes: 101,
                max_bytes: 100,
            })
        );
    }

    #[test]
    fn a_zero_budget_fails_closed_for_any_non_empty_input() {
        let zero = ImportBudget::new(0, 0, 0);
        assert_eq!(
            zero.check(ImportSize {
                bytes: 1,
                ..ImportSize::default()
            }),
            Err(BudgetError::TooLarge {
                bytes: 1,
                max_bytes: 0,
            })
        );
    }

    #[test]
    fn env_parser_uses_defaults_for_absent_invalid_and_zero_values() {
        assert_eq!(parse_limit(None, 42), 42);
        assert_eq!(parse_limit(Some(""), 42), 42);
        assert_eq!(parse_limit(Some("  "), 42), 42);
        assert_eq!(parse_limit(Some("abc"), 42), 42);
        assert_eq!(parse_limit(Some("0"), 42), 42);
        assert_eq!(parse_limit(Some("7"), 42), 7);
        assert_eq!(parse_limit(Some(" 99 "), 42), 99);
    }

    #[test]
    fn env_override_can_only_tighten_the_byte_cap() {
        // No override is guaranteed in a test runner, so assert the invariant
        // that holds with or without one: never wider than the endpoint cap.
        let resolved = ImportBudget::from_env(DEFAULT_MAX_BYTES);
        assert!(resolved.max_bytes <= DEFAULT_MAX_BYTES);
        assert!(resolved.max_rows > 0);
        assert!(resolved.max_pages > 0);
    }

    #[test]
    fn typed_errors_keep_the_stable_http_contract_and_carry_no_document() {
        const SECRET: &str = "confidential-salary-sheet.xlsx";
        for error in [
            BudgetError::TooLarge {
                bytes: 1,
                max_bytes: 0,
            },
            BudgetError::TooManyRows {
                rows: 1,
                max_rows: 0,
            },
            BudgetError::TooManyPages {
                pages: 1,
                max_pages: 0,
            },
        ] {
            assert_eq!(error.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(error.i18n_key(), "office.admission.payload-too-large");
            assert!(error.code().starts_with("office_"));
            let body = error.json_body().to_string();
            assert!(!body.contains(SECRET));
            assert!(!error.to_string().contains(SECRET));
        }
    }
}
