use rhai::Dynamic;
use rhai::Engine;

pub fn last_keyword(engine: &mut Engine) {
    engine
        .register_custom_syntax(["LAST", "(", "$expr$", ")"], false, {
            move |context, inputs| {
                let input = context.eval_expression_tree(&inputs[0])?;
                // Arrays must keep element type: stringifying an array turns
                // LAST(SPLIT(...)) into Rhai debug text like
                // `"test_video.mp4"]` — the classify_media media-filing bug
                // filed objects under media/2026/09/unsorted/"test_video.mp4"].
                if input.is_array() {
                    let arr = input.into_array().unwrap_or_default();
                    return Ok(arr.last().cloned().unwrap_or(Dynamic::UNIT));
                }
                let input_str = input.to_string();
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
