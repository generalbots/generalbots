use log::info;

pub fn convert_mail_line_with_substitution(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();
    let mut in_substitution = false;
    let mut current_var = String::new();
    let mut current_literal = String::new();

    while let Some(c) = chars.next() {
        match c {
            '$' => {
                if let Some(&'{') = chars.peek() {
                    chars.next();

                    if !current_literal.is_empty() {
                        if result.is_empty() {
                            result.push('"');
                            result.push_str(&current_literal.replace('"', "\\\""));
                            result.push('"');
                        } else {
                            result.push_str(" + \"");
                            result.push_str(&current_literal.replace('"', "\\\""));
                            result.push('"');
                        }
                        current_literal.clear();
                    }
                    in_substitution = true;
                    current_var.clear();
                } else {
                    current_literal.push(c);
                }
            }
            '}' if in_substitution => {
                in_substitution = false;
                if !current_var.is_empty() {
                    if result.is_empty() {
                        result.push_str(&current_var);
                    } else {
                        result.push_str(" + ");
                        result.push_str(&current_var);
                    }
                }
                current_var.clear();
            }
            _ if in_substitution => {
                if c.is_alphanumeric() || c == '_' || c == '(' || c == ')' || c == ',' || c == ' ' || c == '\"' {
                    current_var.push(c);
                }
            }
            _ => {
                if !in_substitution {
                    current_literal.push(c);
                }
            }
        }
    }

    if !current_literal.is_empty() {
        if result.is_empty() {
            result.push('"');
            result.push_str(&current_literal.replace('"', "\\\""));
            result.push('"');
        } else {
            result.push_str(" + \"");
            result.push_str(&current_literal.replace('"', "\\\""));
            result.push('"');
        }
    }

    info!("[TOOL] Converted mail line: '{}' → '{}'", line, result);
    result
}

pub fn convert_mail_block(recipient: &str, lines: &[String]) -> String {
    let mut subject = String::new();
    let mut body_lines: Vec<String> = Vec::new();
    // let mut in_subject = true; // Removed unused variable
    let mut skip_blank = true;

    for line in lines.iter() {
        if line.to_uppercase().starts_with("SUBJECT:") {
            subject = line[8..].trim().to_string();
            // in_subject = false; // Removed unused assignment
            skip_blank = true;
            continue;
        }

        if skip_blank && line.trim().is_empty() {
            skip_blank = false;
            continue;
        }

        skip_blank = false;
        let converted = convert_mail_line_with_substitution(line);
        body_lines.push(converted);
    }

    let mut result = String::new();
    let chunk_size = 5;
    let mut all_vars: Vec<String> = Vec::new();

    for (var_count, chunk) in body_lines.chunks(chunk_size).enumerate() {
        let var_name = format!("__mail_body_{}__", var_count);
        all_vars.push(var_name.clone());

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

    let body_expr = if all_vars.is_empty() {
        "\"\"".to_string()
    } else if all_vars.len() == 1 {
        all_vars[0].clone()
    } else {
        let mut expr = all_vars[0].clone();
        for var in &all_vars[1..] {
            expr.push_str(" + \"\\n\" + ");
            expr.push_str(var);
        }
        expr
    };

    let recipient_expr = if recipient.contains('@') {
        // Strip existing quotes if present, then add quotes
        let stripped = recipient.trim_matches('"');
        format!("\"{}\"", stripped)
    } else {
        recipient.to_string()
    };
    result.push_str(&format!("send_mail({}, \"{}\", {}, []);\n", recipient_expr, subject, body_expr));

    info!("[TOOL] Converted MAIL block → {}", result);
    result
}
