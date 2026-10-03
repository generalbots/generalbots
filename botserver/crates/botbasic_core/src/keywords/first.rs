use rhai::Dynamic;
use rhai::Engine;

pub fn first_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(["FIRST", "$expr$"], false, {
            move |context, inputs| {
                let input = context.eval_expression_tree(&inputs[0])?;
                // Arrays must keep element type: stringifying an array turns
                // FIRST(SPLIT(...)) into Rhai debug text like `["inbox"` —
                // the classify_media media-filing bug (2026-09).
                if input.is_array() {
                    let arr = input.into_array().unwrap_or_default();
                    return Ok(arr.first().cloned().unwrap_or(Dynamic::UNIT));
                }
                let input_str = input.to_string();
                let first_word = input_str
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                Ok(Dynamic::from(first_word))
            }
        })
        .ok();
}
