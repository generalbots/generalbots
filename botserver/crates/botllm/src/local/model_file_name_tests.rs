//! Offline tests for the llama.cpp model path reducer (issue #2075).

use super::*;

#[test]
fn legacy_relative_paths_are_reduced_to_the_bare_file_name() {
    let cases = [
        ("../../../../data/llm/model.gguf", "model.gguf"),
        ("/opt/gbo/models/llm/qwen.gguf", "qwen.gguf"),
        ("model.gguf", "model.gguf"),
        (r"C:\models\llm\qwen.gguf", "qwen.gguf"),
        ("  models/model.gguf  ", "model.gguf"),
    ];

    for (input, expected) in cases {
        assert_eq!(model_file_name(input), expected, "input {input}");
    }
}

#[test]
fn empty_and_blank_models_stay_empty() {
    for input in ["", "   ", "\t\n"] {
        assert_eq!(model_file_name(input), "", "input {input:?}");
    }
}

#[test]
fn a_trailing_separator_does_not_panic() {
    // A malformed config value must not crash the boot path.
    assert_eq!(model_file_name("models/"), "");
}
