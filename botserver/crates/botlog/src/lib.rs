//! Redacting logging facade (#1528, #1529, #1530).
//!
//! Sensitive identifiers must never reach stdout/journald in clear text.
//! Call sites use the `safe_*!` macros; the message is scrubbed here before
//! it is handled by the `log` crate. UUIDs, e-mail addresses and phone-like
//! sequences are replaced with neutral markers, preserving the operational
//! message for tracing.

use std::sync::OnceLock;

fn cached(slot: &'static OnceLock<Option<regex::Regex>>, pattern: &str) -> Option<&'static regex::Regex> {
    slot.get_or_init(|| regex::Regex::new(pattern).ok()).as_ref()
}

fn scrub_with(slot: &'static OnceLock<Option<regex::Regex>>, pattern: &str, replacement: &str, input: &str) -> String {
    match cached(slot, pattern) {
        Some(re) => re.replace_all(input, replacement).into_owned(),
        None => input.to_string(),
    }
}

static UUID_RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
static EMAIL_RE: OnceLock<Option<regex::Regex>> = OnceLock::new();
static PHONE_RE: OnceLock<Option<regex::Regex>> = OnceLock::new();

/// Replace identifiers with neutral markers. Idempotent.
pub fn scrub(message: &str) -> String {
    let message = scrub_with(
        &UUID_RE,
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        "[uuid]",
        message,
    );
    let message = scrub_with(
        &EMAIL_RE,
        r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}",
        "[email]",
        &message,
    );
    scrub_with(
        &PHONE_RE,
        r"\+?[0-9][0-9()\-\s]{7,}[0-9]",
        "[phone]",
        &message,
    )
}

/// Emit a scrubbed record at the given level.
pub fn emit(level: log::Level, message: std::fmt::Arguments<'_>) {
    let text = scrub(&message.to_string());
    log::log!(level, "{text}");
}

/// Print a scrubbed line to stdout (macros used by CLI helpers).
pub fn emit_stdout(message: std::fmt::Arguments<'_>) {
    println!("{}", scrub(&message.to_string()));
}

#[macro_export]
macro_rules! safe_info {
    ($($arg:tt)*) => { $crate::emit(log::Level::Info, format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! safe_warn {
    ($($arg:tt)*) => { $crate::emit(log::Level::Warn, format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! safe_error {
    ($($arg:tt)*) => { $crate::emit(log::Level::Error, format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! safe_debug {
    ($($arg:tt)*) => { $crate::emit(log::Level::Debug, format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! safe_trace {
    ($($arg:tt)*) => { $crate::emit(log::Level::Trace, format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! safe_println {
    ($($arg:tt)*) => { $crate::emit_stdout(format_args!($($arg)*)) };
}

#[cfg(test)]
mod tests {
    use super::scrub;

    #[test]
    fn scrubs_uuid_email_phone() {
        let out = scrub(
            "user 123e4567-e89b-12d3-a456-426614174000 mail a@b.com phone +55 11 98888-7777",
        );
        assert!(!out.contains("123e4567"));
        assert!(!out.contains("a@b.com"));
        assert!(!out.contains("98888"));
        assert!(out.contains("[uuid]"));
        assert!(out.contains("[email]"));
        assert!(out.contains("[phone]"));
    }
}
