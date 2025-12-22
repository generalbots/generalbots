use rhai::Dynamic;
use rhai::Engine;

pub fn first_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(&["FIRST", "$expr$"], false, {
            move |context, inputs| {
                let input_string = context.eval_expression_tree(&inputs[0])?;
                let input_str = input_string.to_string();
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
