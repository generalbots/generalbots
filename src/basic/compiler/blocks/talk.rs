use log::info;

pub fn convert_talk_line_with_substitution(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();
    let mut in_substitution = false;
    let mut current_var = String::new();
    let mut current_literal = String::new();
    let mut paren_depth = 0;

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                if let Some(&'{') = chars.peek() {
                    chars.next();

                    if !current_literal.is_empty() {
                        // Output the literal with proper quotes
                        if result.is_empty() {
                            result.push_str("TALK \"");
                        } else {
                            result.push_str(" + \"");
                        }
                        let escaped = current_literal.replace('"', "\\\"");
                        result.push_str(&escaped);
                        result.push('"');
                        current_literal.clear();
                    }
                    in_substitution = true;
                    current_var.clear();
                    paren_depth = 0;
                } else {
                    current_literal.push(c);
                }
            }
            '}' if in_substitution => {
                if paren_depth == 0 {
                    in_substitution = false;
                    if !current_var.is_empty() {
                        // If result is empty, we need to start with "TALK "
                        // but DON'T add opening quote - the variable is not a literal
                        if result.is_empty() {
                            result.push_str("TALK ");
                        } else {
                            result.push_str(" + ");
                        }
                        result.push_str(&current_var);
                    }
                    current_var.clear();
                } else {
                    current_var.push(c);
                    paren_depth -= 1;
                }
            }
            _ if in_substitution => {
                if c.is_alphanumeric() || c == '_' || c == '.' || c == '[' || c == ']' || c == ',' || c == '"' {
                    current_var.push(c);
                } else if c == '(' {
                    current_var.push(c);
                    paren_depth += 1;
                } else if c == ')' && paren_depth > 0 {
                    current_var.push(c);
                    paren_depth -= 1;
                } else if (c == ':' || c == '=' || c == ' ') && paren_depth == 0 {
                    // Handle special punctuation that ends a variable context
                    // Only end substitution if we're not inside parentheses (function call)
                    in_substitution = false;
                    if !current_var.is_empty() {
                        // If result is empty, start with "TALK " (without opening quote)
                        if result.is_empty() {
                            result.push_str("TALK ");
                        } else {
                            result.push_str(" + ");
                        }
                        result.push_str(&current_var);
                    }
                    current_var.clear();
                    current_literal.push(c);
                } else if c == ' ' {
                    // Allow spaces inside function calls
                    current_var.push(c);
                }
                // Ignore other invalid characters - they'll be processed as literals
            }
            '\\' if in_substitution => {
                if let Some(&next_char) = chars.peek() {
                    current_var.push(next_char);
                    chars.next();
                }
            }
            _ => {
                current_literal.push(c);
            }
        }
    }

    if !current_literal.is_empty() {
        if result.is_empty() {
            result.push_str("TALK \"");
        } else {
            result.push_str(" + \"");
        }
        let escaped = current_literal.replace('"', "\\\"");
        result.push_str(&escaped);
        result.push('"');
    }

    if result.is_empty() {
        result = "TALK \"\"".to_string();
    }

    info!("[TOOL] Converted TALK line: '{}' → '{}'", line, result);
    result
}

pub fn convert_talk_block(lines: &[String]) -> String {
    // Convert all lines first
    let converted_lines: Vec<String> = lines.iter()
        .map(|line| convert_talk_line_with_substitution(line))
        .collect();

    // Extract content after "TALK " prefix
    let line_contents: Vec<String> = converted_lines.iter()
        .map(|line| {
            if line.starts_with("TALK ") {
                line[5..].trim().to_string()
            } else {
                line.clone()
            }
        })
        .collect();

    // Use chunking to reduce expression complexity (max 5 lines per chunk)
    let chunk_size = 5;
    let mut result = String::new();

    for (chunk_idx, chunk) in line_contents.chunks(chunk_size).enumerate() {
        let var_name = format!("__talk_chunk_{}__", chunk_idx);

        if chunk.len() == 1 {
            result.push_str(&format!("let {} = {};\n", var_name, chunk[0]));
        } else {
            let mut chunk_expr = chunk[0].clone();
            for line in &chunk[1..] {
                chunk_expr.push_str(" + \"\\n\" + ");
                chunk_expr.push_str(line);
            }
            result.push_str(&format!("let {} = {};\n", var_name, chunk_expr));
        }
    }

    // Combine all chunks into final TALK statement
    let num_chunks = (line_contents.len() + chunk_size - 1) / chunk_size;
    if line_contents.is_empty() {
        return "TALK \"\";\n".to_string();
    } else if num_chunks == 1 {
        // Single chunk - use the first variable directly
        result.push_str(&format!("TALK __talk_chunk_0__;\n"));
    } else {
        // Multiple chunks - need hierarchical chunking to avoid complexity
        // Combine chunks in groups of 5 to create intermediate variables
        let combine_chunk_size = 5;
        let mut chunk_vars: Vec<String> = (0..num_chunks)
            .map(|i| format!("__talk_chunk_{}__", i))
            .collect();

        // If we have many chunks, create intermediate combination variables
        if chunk_vars.len() > combine_chunk_size {
            let mut level = 0;
            while chunk_vars.len() > combine_chunk_size {
                let mut new_vars: Vec<String> = Vec::new();
                for (idx, sub_chunk) in chunk_vars.chunks(combine_chunk_size).enumerate() {
                    let var_name = format!("__talk_combined_{}_{}__", level, idx);
                    if sub_chunk.len() == 1 {
                        new_vars.push(sub_chunk[0].clone());
                    } else {
                        let mut expr = sub_chunk[0].clone();
                        for var in &sub_chunk[1..] {
                            expr.push_str(" + \"\\n\" + ");
                            expr.push_str(var);
                        }
                        result.push_str(&format!("let {} = {};\n", var_name, expr));
                        new_vars.push(var_name);
                    }
                }
                chunk_vars = new_vars;
                level += 1;
            }
        }

        // Final TALK statement with combined chunks
        if chunk_vars.len() == 1 {
            result.push_str(&format!("TALK {};\n", chunk_vars[0]));
        } else {
            let mut expr = chunk_vars[0].clone();
            for var in &chunk_vars[1..] {
                expr.push_str(" + \"\\n\" + ");
                expr.push_str(var);
            }
            result.push_str(&format!("TALK {};\n", expr));
        }
    }

    result
}
