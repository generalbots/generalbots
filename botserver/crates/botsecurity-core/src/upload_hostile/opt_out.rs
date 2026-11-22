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

    #[test]
    fn an_unset_or_unrecognised_value_keeps_the_fail_closed_default() {
        assert!(malware_scanner_required_from(None));
        for value in ["", "0", "off", "false", "no", "random", " 0 "] {
            assert!(
                malware_scanner_required_from(Some(value)),
                "{value} must keep the scanner requirement"
            );
        }
    }

    #[test]
    fn only_explicit_opt_out_values_lower_the_requirement() {
        for value in ["1", "true", "TRUE", " yes ", "On"] {
            assert!(
                !malware_scanner_required_from(Some(value)),
                "{value} must opt out"
            );
        }
    }
}
