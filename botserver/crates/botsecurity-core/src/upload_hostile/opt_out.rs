//! #2522 — the fail-closed malware requirement and its one explicit opt-out.
//!
//! [`super::HostileFilePolicy::default`] requires a malware scanner: with no
//! scanner registered, a zip import is refused with
//! [`crate::upload_error::UploadError::MalwareScannerMissing`] instead of
//! being accepted as "clean". A deployment that cannot deploy a scanner yet
//! can lower the requirement through [`UNCHECKED_IMPORT_OPT_OUT_ENV`]; the
//! scan decision is still recorded as `NotConfigured`, never fabricated as
//! `Clean`.
//!
//! [`malware_scanner_required_from`] interprets the same value independently
//! of its source, so a caller that reads the switch from Vault (or another
//! configuration store) reaches the identical decision without this crate
//! performing a blocking Vault lookup inside `Default`.

/// Environment switch that restores the pre-#2522 permissive behaviour. Only
/// `1`, `true`, `yes` or `on` (case-insensitive, surrounding whitespace
/// ignored) opt out; anything else — including an unset variable — keeps the
/// fail-closed default.
pub const UNCHECKED_IMPORT_OPT_OUT_ENV: &str = "GB_ALLOW_UNSCANNED_IMPORT";

/// Interprets one opt-out value. Split from [`malware_scanner_required`] so the
/// decision is testable without mutating process-wide state and so callers can
/// feed a value read from Vault or another configuration source.
pub fn malware_scanner_required_from(value: Option<&str>) -> bool {
    match value {
        Some(raw) => !matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => true,
    }
}

/// True when the shared policy requires a malware scanner (the default).
///
/// Reads [`UNCHECKED_IMPORT_OPT_OUT_ENV`] from the process environment. In
/// managed deployments that variable is injected from Vault at boot, so the
/// env lookup and a Vault-sourced value share the grammar interpreted by
/// [`malware_scanner_required_from`].
pub fn malware_scanner_required() -> bool {
    malware_scanner_required_from(std::env::var(UNCHECKED_IMPORT_OPT_OUT_ENV).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fail-closed guarantee lives in the `None` arm of the pure resolver:
    /// an absent variable requires a scanner. Feeding `None` directly proves
    /// it without mutating the process environment (which would be racy with
    /// other tests and is `unsafe` on the 2024 edition).
    #[test]
    fn absent_variable_is_fail_closed_without_touching_process_env() {
        assert!(
            malware_scanner_required_from(None),
            "an absent {UNCHECKED_IMPORT_OPT_OUT_ENV} must keep the scanner requirement"
        );
    }

    /// Every accepted spelling of the opt-out, across casing and surrounding
    /// whitespace, must lower the requirement. `trim()` is Unicode-aware and
    /// `to_ascii_lowercase()` is case-insensitive, so all of these opt out.
    #[test]
    fn explicit_opt_out_values_are_case_and_whitespace_insensitive() {
        for value in [
            "1",
            " 1 ",
            "true",
            "TRUE",
            "True",
            "tRuE",
            " true ",
            "\ttrue\n",
            "yes",
            "YES",
            "Yes",
            " yes ",
            "  yes\t",
            "on",
            "ON",
            "On",
            " on ",
            "\ton\r\n",
            "\u{2003}YES\u{2003}",
        ] {
            assert!(
                !malware_scanner_required_from(Some(value)),
                "{value:?} is an explicit opt-out and must lower the requirement"
            );
        }
    }

    /// Empty/whitespace-only values and every unrecognised spelling keep the
    /// fail-closed default. Near-misses guard against a too-permissive parser.
    #[test]
    fn empty_and_unknown_values_keep_the_fail_closed_default() {
        for value in [
            "",
            " ",
            "   ",
            "\t",
            "\n",
            "0",
            " 0 ",
            "2",
            "01",
            "false",
            "FALSE",
            "no",
            "NO",
            "off",
            "OFF",
            "y",
            "n",
            "enabled",
            "random",
            "yesplease",
            "onoff",
            "true-ish",
        ] {
            assert!(
                malware_scanner_required_from(Some(value)),
                "{value:?} is not a recognised opt-out and must keep the scanner requirement"
            );
        }
    }
}
