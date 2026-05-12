use regex::Regex;
use std::collections::BTreeSet;

pub fn predeclare_variables(script: &str) -> String {
    let reserved: std::collections::HashSet<&str> = [
        "if", "else", "while", "for", "loop", "return", "break", "continue",
        "let", "fn", "true", "false", "in", "do", "match", "switch", "case",
        "mod", "and", "or", "not", "rem", "call", "talk", "hear", "save",
        "insert", "update", "delete", "find", "get", "set", "print",
    ].iter().cloned().collect();

    let mut vars: BTreeSet<String> = BTreeSet::new();

    for line in script.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("//") || t.starts_with('\'') || t.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = t.find('=') {
            let after_char = t.as_bytes().get(eq_pos + 1).copied();
            let prev_char = if eq_pos > 0 { t.as_bytes().get(eq_pos - 1).copied() } else { None };
            if after_char == Some(b'=') { continue; }
            if matches!(prev_char, Some(b'!') | Some(b'<') | Some(b'>') | Some(b'+') | Some(b'-') | Some(b'*') | Some(b'/')) { continue; }
            let before = &t[..eq_pos];
            let lhs = before.trim();
            if lhs.is_empty() || lhs.contains(' ') || lhs.contains('"') || lhs.contains('(') || lhs.contains('[') {
                continue;
            }
            if !lhs.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_') {
                continue;
            }
            if !lhs.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let lower = lhs.to_lowercase();
            if reserved.contains(lower.as_str()) {
                continue;
            }
            vars.insert(lhs.to_string());
        }
    }

    if vars.is_empty() {
        return script.to_string();
    }

    let mut declarations = String::new();
    for v in &vars {
        declarations.push_str(&format!("let {} = ();\n", v));
    }
    declarations.push('\n');
    declarations.push_str(script);
    declarations
}

pub fn convert_if_then_syntax(script: &str) -> String {
    let mut result = String::new();
    let mut if_stack: Vec<bool> = Vec::new();
    let mut while_depth: usize = 0;
    let mut in_with_block = false;
    let mut in_line_continuation = false;

    log::trace!("Converting IF/THEN syntax, input has {} lines", script.lines().count());

    for line in script.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("while ") && trimmed.ends_with('{') {
            while_depth += 1;
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }
        if trimmed == "}" && while_depth > 0 && if_stack.is_empty() {
            while_depth -= 1;
            result.push_str("}\n");
            continue;
        }

        if upper.starts_with("IF ") && upper.contains(" THEN") {
            let then_pos = match upper.find(" THEN") {
                Some(pos) => pos,
                None => continue,
            };
            let condition = &trimmed[3..then_pos].trim();
            let condition = condition.replace(" NOT IN ", " !in ").replace(" not in ", " !in ");
            let condition = condition.replace(" AND ", " && ").replace(" and ", " && ")
                .replace(" OR ", " || ").replace(" or ", " || ");
            let condition = if !condition.contains("==") && !condition.contains("!=")
                && !condition.contains("<=") && !condition.contains(">=")
                && !condition.contains("+=") && !condition.contains("-=")
                && !condition.contains("*=") && !condition.contains("/=") {
                condition.replace("=", "==")
            } else {
                condition.to_string()
            };
            log::trace!("Converting IF statement: condition='{}'", condition);
            result.push_str("if ");
            result.push_str(&condition);
            result.push_str(" {\n");
            if_stack.push(true);
            continue;
        }

        if upper == "ELSE" {
            log::trace!("Converting ELSE statement");
            result.push_str("} else {\n");
            continue;
        }

        if upper.starts_with("ELSEIF ") && upper.contains(" THEN") {
            let then_pos = match upper.find(" THEN") {
                Some(pos) => pos,
                None => continue,
            };
            let condition = &trimmed[6..then_pos].trim();
            let condition = condition.replace(" NOT IN ", " !in ").replace(" not in ", " !in ");
            let condition = condition.replace(" AND ", " && ").replace(" and ", " && ")
                .replace(" OR ", " || ").replace(" or ", " || ");
            let condition = if !condition.contains("==") && !condition.contains("!=")
                && !condition.contains("<=") && !condition.contains(">=")
                && !condition.contains("+=") && !condition.contains("-=")
                && !condition.contains("*=") && !condition.contains("/=") {
                condition.replace("=", "==")
            } else {
                condition.to_string()
            };
            log::trace!("Converting ELSEIF statement: condition='{}'", condition);
            result.push_str("} else if ");
            result.push_str(&condition);
            result.push_str(" {\n");
            continue;
        }

        if upper == "END IF" {
            log::trace!("Converting END IF statement");
            if if_stack.pop().is_some() {
                result.push_str("}\n");
            }
            continue;
        }

        if upper.starts_with("WITH ") {
            let object_name = &trimmed[5..].trim();
            log::trace!("Converting WITH statement: object='{}'", object_name);
            result.push_str("let ");
            result.push_str(object_name);
            result.push_str(" = #{\n");
            in_with_block = true;
            continue;
        }

        if upper == "END WITH" {
            log::trace!("Converting END WITH statement");
            result.push_str("};\n");
            in_with_block = false;
            continue;
        }

        if in_with_block {
            if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") && !trimmed.contains("+=") && !trimmed.contains("-=") {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let property_name = parts[0].trim();
                    let property_value = parts[1].trim();
                    let property_value = property_value.trim_end_matches(';');
                    result.push_str(&format!(" {}: {},\n", property_name, property_value));
                    continue;
                }
            }
            result.push_str("  ");
        }

        if upper.starts_with("SAVE") && upper.contains(',') {
            log::trace!("Processing SAVE line: '{}'", trimmed);
            let after_save = &trimmed[4..].trim();
            let parts: Vec<&str> = after_save.split(',').collect();
            log::trace!("SAVE parts: {:?}", parts);

            if parts.len() >= 2 {
                let table = parts[0].trim().trim_matches('"');

                if parts.len() == 2 {
                    let object_name = parts[1].trim().trim_end_matches(';');
                    let converted = format!("INSERT \"{}\", {};\n", table, object_name);
                    log::trace!("Converted SAVE to INSERT (old syntax): '{}'", converted);
                    result.push_str(&converted);
                    continue;
                }

                let values = parts[1..].join(", ");
                let converted = format!("SAVE \"{}\", {};\n", table, values);
                log::trace!("Keeping SAVE syntax (modern): '{}'", converted);
                result.push_str(&converted);
                continue;
            }
        }

        if upper.starts_with("SEND EMAIL") {
            log::trace!("Processing SEND EMAIL line: '{}'", trimmed);
            let after_send = &trimmed[11..].trim();
            let parts: Vec<&str> = after_send.split(',').collect();
            log::trace!("SEND EMAIL parts: {:?}", parts);
            if parts.len() == 3 {
                let to = parts[0].trim();
                let subject = parts[1].trim();
                let body = parts[2].trim().trim_end_matches(';');
                let converted = format!("send_mail({}, {}, {}, []);\n", to, subject, body);
                log::trace!("Converted SEND EMAIL to: '{}'", converted);
                result.push_str(&converted);
                continue;
            }
        }

        if !if_stack.is_empty() {
            result.push_str("  ");
        }

        if !upper.starts_with("IF ") && !upper.starts_with("ELSE") && !upper.starts_with("END IF") {
            let is_var_assignment = trimmed.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
                && trimmed.contains('=')
                && !trimmed.contains("==")
                && !trimmed.contains("!=")
                && !trimmed.contains("<=")
                && !trimmed.contains(">=")
                && !trimmed.contains("+=")
                && !trimmed.contains("-=")
                && !trimmed.contains("*=")
                && !trimmed.contains("/=");

            let ends_with_comma = trimmed.ends_with(',');

            let line_to_process = if in_line_continuation && !is_var_assignment
                && !trimmed.contains('=') && !trimmed.starts_with('"') && !upper.starts_with("IF ") {
                let escaped = trimmed.replace('"', "\\\"");
                format!("\"{}\\n\"", escaped)
            } else {
                trimmed.to_string()
            };

            if is_var_assignment {
                let trimmed_lower = trimmed.to_lowercase();
                let in_block = !if_stack.is_empty() || while_depth > 0;
                if !in_block && !trimmed_lower.starts_with("let ") {
                    result.push_str("let ");
                }
            }
            result.push_str(&line_to_process);
            let is_keyword_stmt = upper.starts_with("INSERT ")
                || upper.starts_with("SAVE ")
                || upper.starts_with("TALK ")
                || upper.starts_with("PRINT ")
                || upper.starts_with("MERGE ")
                || upper.starts_with("UPDATE ");
            let ends_with_block_brace = trimmed.ends_with('}') && !is_keyword_stmt;
            let needs_semicolon = !trimmed.ends_with(';')
                && !trimmed.ends_with('{')
                && !ends_with_block_brace
                && !upper.starts_with("SELECT ")
                && !upper.starts_with("CASE ")
                && upper != "END SELECT"
                && !upper.starts_with("WHILE ")
                && !upper.starts_with("WEND")
                && !ends_with_comma
                && !in_line_continuation;
            if needs_semicolon {
                result.push(';');
            }
            result.push('\n');

            in_line_continuation = ends_with_comma;
        } else {
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    log::trace!("IF/THEN conversion complete, output has {} lines", result.lines().count());

    result.replace(" <> ", " != ")
}

pub fn convert_while_wend_syntax(script: &str) -> String {
    let mut result = String::new();
    for line in script.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if upper.starts_with("WHILE ") {
            let condition = &trimmed[6..];
            result.push_str(&format!("while {} {{\n", condition));
        } else if upper == "WEND" {
            result.push_str("}\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

pub fn convert_select_case_syntax(script: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = script.lines().collect();
    let mut i = 0;

    log::trace!("Converting SELECT/CASE syntax to if-else chains");

    fn strip_let_from_assignment(line: &str) -> String {
        let trimmed = line.trim();
        let trimmed_lower = trimmed.to_lowercase();
        if trimmed_lower.starts_with("let ") && trimmed.contains('=') {
            trimmed[4..].trim().to_string()
        } else {
            trimmed.to_string()
        }
    }

    while i < lines.len() {
        let trimmed = lines[i].trim();
        let upper = trimmed.to_uppercase();

        if upper.starts_with("SELECT ") && !upper.contains(" THEN") {
            let select_var = trimmed[7..].trim();
            log::trace!("Converting SELECT statement for variable: '{}'", select_var);

            i += 1;

            let mut current_case_body: Vec<String> = Vec::new();
            let mut in_case = false;
            let mut is_first_case = true;

            while i < lines.len() {
                let case_trimmed = lines[i].trim();
                let case_upper = case_trimmed.to_uppercase();

                if case_trimmed.is_empty() || case_trimmed.starts_with('\'') || case_trimmed.starts_with('#') {
                    i += 1;
                    continue;
                }

                if case_upper == "END SELECT" {
                    if in_case {
                        for body_line in &current_case_body {
                            result.push_str("  ");
                            let processed_line = strip_let_from_assignment(body_line);
                            result.push_str(&processed_line);
                            if !processed_line.ends_with(';') && !processed_line.ends_with('{') && !processed_line.ends_with('}') {
                                result.push(';');
                            }
                            result.push('\n');
                        }
                        result.push_str(" }\n");
                        current_case_body.clear();
                    }
                    i += 1;
                    break;
                } else if case_upper.starts_with("SELECT ") {
                    if in_case {
                        for body_line in &current_case_body {
                            result.push_str("  ");
                            let processed_line = strip_let_from_assignment(body_line);
                            result.push_str(&processed_line);
                            if !processed_line.ends_with(';') && !processed_line.ends_with('{') && !processed_line.ends_with('}') {
                                result.push(';');
                            }
                            result.push('\n');
                        }
                        result.push_str(" }\n");
                        current_case_body.clear();
                    }
                    break;
                } else if case_upper.starts_with("CASE ") {
                    if in_case {
                        for body_line in &current_case_body {
                            result.push_str("  ");
                            let processed_line = strip_let_from_assignment(body_line);
                            result.push_str(&processed_line);
                            if !processed_line.ends_with(';') && !processed_line.ends_with('{') && !processed_line.ends_with('}') {
                                result.push(';');
                            }
                            result.push('\n');
                        }
                        current_case_body.clear();
                    }

                    let case_value = if case_trimmed[5..].trim().starts_with('"') {
                        case_trimmed[5..].trim().to_string()
                    } else {
                        format!("\"{}\"", case_trimmed[5..].trim())
                    };

                    if is_first_case {
                        result.push_str(&format!("if {} == {} {{\n", select_var, case_value));
                        is_first_case = false;
                    } else {
                        result.push_str(&format!("}} else if {} == {} {{\n", select_var, case_value));
                    }
                    in_case = true;
                    i += 1;
                } else if in_case {
                    current_case_body.push(lines[i].to_string());
                    i += 1;
                } else {
                    i += 1;
                }
            }

            continue;
        }

        if i < lines.len() {
            result.push_str(lines[i]);
            result.push('\n');
            i += 1;
        }
    }

    result
}

pub fn convert_keywords_to_lowercase(script: &str) -> String {
    let rhai_builtins = [
        "IF", "ELSE", "WHILE", "FOR", "IN", "LOOP", "RETURN", "LET",
        "CONST", "IMPORT", "EXPORT", "FN", "PRIVATE", "SWITCH", "MATCH",
        "TRUE", "FALSE", "BREAK", "CONTINUE", "DO", "TRY", "CATCH", "THROW",
        "AS",
    ];

    let mut result = String::new();
    for line in script.lines() {
        let mut processed_line = line.to_string();
        for keyword in &rhai_builtins {
            let pattern = format!(r"\b{}\b", regex::escape(keyword));
            if let Ok(re) = Regex::new(&pattern) {
                processed_line = re.replace_all(&processed_line, keyword.to_lowercase()).to_string();
            }
        }
        result.push_str(&processed_line);
        result.push('\n');
    }
    result
}

pub fn convert_multiword_keywords(script: &str) -> String {
    let multiword_patterns = vec![
        (r#"USE\s+WEBSITE"#, 1, 2, vec!["url", "refresh"]),
        (r#"USE\s+MODEL"#, 1, 1, vec!["model"]),
        (r#"USE\s+KB"#, 1, 1, vec!["kb_name"]),
        (r#"USE\s+TOOL"#, 1, 1, vec!["tool_path"]),

        (r#"SET\s+BOT\s+MEMORY"#, 2, 2, vec!["key", "value"]),
        (r#"SET\s+CONTEXT"#, 2, 2, vec!["key", "value"]),
        (r#"SET\s+USER"#, 1, 1, vec!["user_id"]),

        (r#"GET\s+BOT\s+MEMORY"#, 1, 1, vec!["key"]),

        (r#"CLEAR\s+SUGGESTIONS"#, 0, 0, vec![]),
        (r#"CLEAR\s+SWITCHERS"#, 0, 0, vec![]),
        (r#"CLEAR\s+TOOLS"#, 0, 0, vec![]),
        (r#"CLEAR\s+WEBSITES"#, 0, 0, vec![]),

        (r#"ADD_SUGGESTION_TOOL"#, 2, 2, vec!["tool", "text"]),
        (r#"ADD_SUGGESTION_TEXT"#, 2, 2, vec!["value", "text"]),
        (r#"ADD_SUGGESTION(?!\\s+TOOL|\\s+TEXT|_)"#, 2, 2, vec!["context", "text"]),
        (r#"ADD_SWITCHER"#, 2, 2, vec!["switcher", "text"]),
        (r#"ADD_MEMBER"#, 2, 2, vec!["name", "role"]),
        (r#"ADD\s+MEMBER"#, 2, 2, vec!["name", "role"]),

        (r#"CREATE\s+TASK"#, 1, 1, vec!["task"]),
        (r#"CREATE\s+DRAFT"#, 4, 4, vec!["to", "subject", "body", "attachments"]),
        (r#"CREATE\s+SITE"#, 1, 1, vec!["site"]),

        (r#"ON\s+FORM\s+SUBMIT"#, 1, 1, vec!["form"]),
        (r#"ON\s+EMAIL"#, 1, 1, vec!["filter"]),
        (r#"ON\s+EVENT"#, 1, 1, vec!["event"]),

        (r#"SEND\s+MAIL"#, 4, 4, vec!["to", "subject", "body", "attachments"]),

        (r#"BOOK"#, 1, 1, vec!["event"]),
    ];

    let mut result = String::new();

    for line in script.lines() {
        let trimmed = line.trim();
        let mut converted = false;

        let trimmed_upper = trimmed.to_uppercase();
        if trimmed_upper.contains("ADD_SUGGESTION_TOOL") ||
            trimmed_upper.contains("ADD_SUGGESTION_TEXT") ||
            trimmed_upper.starts_with("ADD_SUGGESTION_") ||
            (trimmed_upper.starts_with("ADD_SWITCHER") && trimmed_upper.contains(" AS ")) ||
            trimmed_upper.starts_with("ADD_MEMBER") ||
            (trimmed_upper.starts_with("CLEAR_SWITCHERS") && trimmed.contains('(')) ||
            (trimmed_upper.starts_with("USE_") && trimmed.contains('(')) {
            result.push_str(line);
            if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                result.push(';');
            }
            result.push('\n');
            continue;
        }

        for (pattern, min_params, max_params, _param_names) in &multiword_patterns {
            let regex_str = format!(
                r#"(?i)^\s*{}\s+(.*?)(?:\s*)$"#,
                pattern
            );

            if let Ok(re) = Regex::new(&regex_str) {
                if let Some(caps) = re.captures(trimmed) {
                    if let Some(params_str) = caps.get(1) {
                        let params = parse_parameters(params_str.as_str());
                        let param_count = params.len();

                        if param_count >= *min_params && param_count <= *max_params {
                            let keyword = pattern.replace(r"\s+", "_");

                            let output = if keyword == "ADD_SWITCHER" {
                                let (switcher_id, label) = if params.len() == 2 {
                                    (params[0].clone(), params[1].clone())
                                } else if params.len() == 3 && params[1].eq_ignore_ascii_case("AS") {
                                    (params[0].clone(), params[2].clone())
                                } else {
                                    (params[0].clone(), params.last().cloned().unwrap_or_default())
                                };
                                format!("ADD_SWITCHER {} as {};", switcher_id, label)
                            } else {
                                let params_str = if params.is_empty() {
                                    String::new()
                                } else {
                                    params.join(", ")
                                };
                                format!("{}({});", keyword, params_str)
                            };

                            result.push_str(&output);
                            result.push('\n');
                            converted = true;
                            break;
                        }
                    }
                }
            }
        }

        if !converted {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

fn parse_parameters(params_str: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    let chars = params_str.chars().peekable();

    for c in chars {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
                current.push(c);
            }
            '"' | '\'' if in_quotes && c == quote_char => {
                in_quotes = false;
                current.push(c);
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    params.push(current.trim().to_string());
                    current = String::new();
                }
            }
            ',' if !in_quotes => {
                if !current.is_empty() {
                    params.push(current.trim().to_string());
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        params.push(current.trim().to_string());
    }

    params
}

pub fn preprocess_llm_keyword(script: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = script.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let remaining: String = chars[i..].iter().collect();
        let remaining_upper = remaining.to_uppercase();

        if remaining_upper.starts_with("LLM ") {
            result.push_str("LLM ");
            i += 4;

            if i < chars.len() && chars[i] == '"' {
                result.push('"');
                i += 1;

                while i < chars.len() && chars[i] != '"' {
                    result.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == '"' {
                    result.push('"');
                    i += 1;
                }

                let before_with = result.trim_end_matches('"');
                if !before_with.to_uppercase().contains("WITH OPTIMIZE") {
                    result = format!("{} WITH OPTIMIZE FOR \"speed\"", before_with);
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}
