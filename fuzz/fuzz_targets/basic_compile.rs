#![no_main]
//! BASIC script compile (#2119).
//!
//! `.bas` scripts come from bot repositories and from `AUTOTASK` generation, so
//! the preprocessor and the syntax transforms must survive any byte sequence:
//! an unterminated block, a stray quote or a `GOTO` to nowhere is a compile
//! error, never a panic and never a hang.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    // Whole-script transforms, in the order the compiler applies them. They are
    // pure text functions: garbage in must produce deterministic text out.
    let _ = botbasic_compiler::goto_transform::has_goto_constructs(source);
    let goto = botbasic_compiler::goto_transform::transform_goto(source);
    let if_then = botbasic_compiler::syntax_transforms::convert_if_then_syntax(source);
    let unquoted = botbasic_compiler::syntax_transforms::preprocess_llm_keyword(source);

    assert_eq!(
        goto,
        botbasic_compiler::goto_transform::transform_goto(source),
        "GOTO transform is not deterministic"
    );
    assert_eq!(
        if_then,
        botbasic_compiler::syntax_transforms::convert_if_then_syntax(source),
        "IF/THEN transform is not deterministic"
    );
    assert_eq!(
        unquoted,
        botbasic_compiler::syntax_transforms::preprocess_llm_keyword(source),
        "unquote transform is not deterministic"
    );
});
