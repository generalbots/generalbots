//! Pure helpers of the Vault persistence layer (#2109).

use super::{bare_name, is_active, is_loopback};
use crate::on_device_ai::catalog::find;

#[test]
fn bare_name_strips_legacy_relative_paths() {
    assert_eq!(bare_name("../../../../data/llm/model.gguf"), "model.gguf");
    assert_eq!(bare_name("C:\\models\\model.gguf"), "model.gguf");
    assert_eq!(bare_name("  model.gguf  "), "model.gguf");
    assert_eq!(bare_name(""), "");
}

#[test]
fn active_matches_the_configured_model_by_filename() {
    let spec = find("qwen3.5-9b-q4km").expect("catalogue entry");
    assert!(is_active(spec.file, spec));
    assert!(is_active(&format!("../../data/llm/{}", spec.file), spec));
    assert!(!is_active("", spec));
    assert!(!is_active("other-model.gguf", spec));
}

#[test]
fn only_loopback_urls_are_treated_as_local() {
    assert!(is_loopback("http://localhost:8081/v1/chat/completions"));
    assert!(is_loopback("http://127.0.0.1:8081/v1"));
    assert!(is_loopback("http://[::1]:8081/v1"));
    assert!(!is_loopback("https://api.openai.com/v1"));
    assert!(!is_loopback(""));
}
