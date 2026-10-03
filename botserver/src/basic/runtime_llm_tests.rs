//! Unit tests for `super` (runtime_llm.rs).
//!
//! Split out of `runtime_llm.rs` to keep the file inside the project size
//! limit; the production code stays beside it.

use super::*;

#[test]
fn fallback_model_is_the_self_hosted_default() {
    assert_eq!(FALLBACK_MODEL, "llama3");
}

#[test]
fn generation_timeout_is_within_the_keyword_budget() {
    assert!(GENERATION_TIMEOUT <= Duration::from_secs(120));
}
