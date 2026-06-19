use crate::formulas::helpers::{resolve_cell_value, split_args};
use crate::types::Worksheet;

pub fn evaluate_bot_ai_prompt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let paren_start = expr.find('(')?;
    let paren_end = expr.rfind(')')?;
    let args_str = &expr[paren_start + 1..paren_end];

    let args = split_args(args_str);
    if args.is_empty() {
        return Some("#ERROR!".to_string());
    }

    let mut prompt_parts: Vec<String> = Vec::new();

    for arg in args {
        let trimmed = arg.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let inner = &trimmed[1..trimmed.len() - 1];
            prompt_parts.push(inner.to_string());
        } else {
            let resolved = resolve_cell_value(trimmed, worksheet);
            prompt_parts.push(resolved);
        }
    }

    let resolved = prompt_parts.concat();
    Some(format!("__BOT_AI__:{}", resolved))
}
