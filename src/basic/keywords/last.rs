use rhai::Dynamic;
use rhai::Engine;

pub fn last_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(&["LAST", "(", "$expr$", ")"], false, {
            move |context, inputs| {
                let input_string = context.eval_expression_tree(&inputs[0])?;
                let input_str = input_string.to_string();
                if input_str.trim().is_empty() {
                    return Ok(Dynamic::from(""));
                }
                let words: Vec<&str> = input_str.split_whitespace().collect();
                let last_word = words.last().copied().unwrap_or("");
                Ok(Dynamic::from(last_word.to_string()))
            }
        })
        .ok();
}
