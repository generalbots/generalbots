//! Process-wide cached `Regex` compilation (#1493, #1738).
//!
//! Before #1738 this module was copied into 13 crates; each copy owned its own
//! `dynamic()` memo, so the same runtime-built pattern was compiled once per
//! crate and the 256-entry limit was applied 13 times independently. There is
//! now one implementation in `botlib` and every dependent crate re-exports it.
//!
//! Constant patterns go through the [`cached_re!`] macro, which expands to a
//! block-scoped `OnceLock`: the pattern stays beside the code that uses it and
//! is compiled once per call site. Patterns assembled at runtime (user text,
//! cell criteria, workbook part names) cannot be `static` and go through
//! [`dynamic`], whose memo is a single process-wide cache.
//!
//! Following `botconnectors::redact`, a pattern that fails to compile is logged
//! and reported as `None`; the caller skips that step instead of panicking
//! inside the running server (#1368).

use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Compile a constant pattern once per call site.
///
/// Expands to a block-scoped `OnceLock`, so the pattern stays beside the code
/// that uses it while every call site keeps its own cached `Regex`.
#[macro_export]
macro_rules! cached_re {
    ($pattern:expr) => {{
        static RE: ::std::sync::OnceLock<::core::option::Option<::regex::Regex>> =
            ::std::sync::OnceLock::new();
        RE.get_or_init(|| $crate::re_cache::compile($pattern))
            .as_ref()
    }};
}

pub use crate::cached_re;

/// `cached_re!` for call sites that need an owned `Regex` (e.g. a struct
/// field). Cloning a `Regex` only bumps the `Arc`s it already holds.
#[macro_export]
macro_rules! cached_re_owned {
    ($pattern:expr) => {
        $crate::re_cache::cached_re!($pattern).cloned()
    };
}

pub use crate::cached_re_owned;

/// Compile a constant pattern, reporting a failed pattern as `None`.
pub fn compile(pattern: &'static str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(re) => Some(re),
        Err(e) => {
            log::error!(
                "{}: pattern failed to compile ({pattern}): {e}",
                module_path!()
            );
            None
        }
    }
}

/// Patterns assembled at runtime cannot be `static`. This memo compiles each
/// distinct pattern once and returns a cheap clone: `regex::Regex` holds `Arc`s
/// internally, so cloning never recompiles. The map is cleared at the limit, so
/// a workload with many distinct patterns degrades to the previous behaviour
/// instead of growing without bound.
const DYNAMIC_LIMIT: usize = 256;

/// Returns `None` for a pattern that does not compile, exactly like the
/// `if let Ok(re)` call sites this replaces.
pub fn dynamic(pattern: &str) -> Option<Regex> {
    static CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(pattern) {
            return Some(cached.clone());
        }
    }
    let compiled = Regex::new(pattern).ok()?;
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= DYNAMIC_LIMIT {
            guard.clear();
        }
        guard.insert(pattern.to_string(), compiled.clone());
    }
    Some(compiled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_returns_some_for_a_valid_pattern() {
        let re = compile(r"^[a-z]+$").expect("valid pattern");
        assert!(re.is_match("abc"));
        assert!(!re.is_match("ABC"));
    }

    #[test]
    fn compile_returns_none_instead_of_panicking() {
        assert!(compile("(unclosed").is_none());
    }

    #[test]
    fn dynamic_compiles_and_memoizes_a_runtime_pattern() {
        let first = dynamic(r"\d{4}-\d{2}-\d{2}").expect("valid pattern");
        let second = dynamic(r"\d{4}-\d{2}-\d{2}").expect("cached pattern");
        assert!(first.is_match("2026-11-10"));
        assert!(second.is_match("2026-11-10"));
    }

    #[test]
    fn dynamic_returns_none_for_an_invalid_pattern() {
        assert!(dynamic("(unclosed").is_none());
    }

    #[test]
    fn cached_re_macro_compiles_once_per_call_site() {
        let re = cached_re!(r"^A[0-9]+$").expect("macro pattern");
        assert!(re.is_match("A12"));
        assert!(!re.is_match("B12"));
    }
}
