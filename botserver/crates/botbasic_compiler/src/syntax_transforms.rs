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
        declarations.push_str(&format!("let {};\n", v));
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

            // Handle inline IF/THEN with body on the same line
            // e.g., IF cond THEN stmt END IF  or  IF cond THEN stmt ELSE stmt2 END IF
            let after_then = trimmed[then_pos + 5..].trim();
            let has_inline_else = after_then.to_uppercase().contains(" ELSE ");
            let has_inline_end = after_then.to_uppercase().contains(" END IF");

            if has_inline_end || has_inline_else {
                // Extract inline THEN body (everything before ELSE or END IF)
                let inline_body = if has_inline_else {
                    let else_upper = after_then.to_uppercase();
                    let else_pos = else_upper.find(" ELSE ").unwrap();
                    after_then[..else_pos].trim().to_string()
                } else if has_inline_end {
                    let end_upper = after_then.to_uppercase();
                    let end_pos = end_upper.find(" END IF").unwrap();
                    after_then[..end_pos].trim().to_string()
                } else {
                    after_then.to_string()
                };

                result.push_str("if ");
                result.push_str(&condition);
                result.push_str(" {\n  ");
                result.push_str(&inline_body);
                result.push_str(";\n");

                if has_inline_else {
                    let else_upper = after_then.to_uppercase();
                    let else_pos = else_upper.find(" ELSE ").unwrap();
                    let after_else = after_then[else_pos + 6..].trim();
                    let else_body = if after_else.to_uppercase().ends_with(" END IF") {
                        after_else[..after_else.len() - 7].trim().to_string()
                    } else {
                        after_else.trim_end().to_string()
                    };
                    result.push_str("} else {\n  ");
                    result.push_str(&else_body);
                    result.push_str(";\n}\n");
                } else {
                    result.push_str("}\n");
                }
                // Don't push to if_stack since it's single-line and already closed
                continue;
            }

            result.push_str("if ");
            result.push_str(&condition);
            result.push_str(" {\n");
            if_stack.push(true);
            continue;
        }

        if upper.starts_with("ELSE IF ") && upper.contains(" THEN") {
            let then_pos = match upper.find(" THEN") {
                Some(pos) => pos,
                None => continue,
            };
            let condition = &trimmed[7..then_pos].trim();
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
            log::trace!("Converting ELSE IF statement: condition='{}'", condition);
            result.push_str("} else if ");
            result.push_str(&condition);
            result.push_str(" {\n");
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
                let values = parts[1..].join(",");
                // Unified 2-arg SAVE: SAVE "table", data
                // Rhai handler checks if data has an `id` field to decide insert vs upsert
                let converted = format!("SAVE \"{}\", {};\n", table, values);
                log::trace!("Unified SAVE syntax: '{}'", converted);
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
                    let first_word = trimmed_lower.split_whitespace().next().unwrap_or("");
                    let is_statement_keyword = matches!(first_word,
                        "if" | "else" | "while" | "for" | "update" | "save" | "insert"
                        | "delete" | "select" | "merge" | "talk" | "print" | "return"
                        | "switch" | "match" | "throw" | "import" | "export" | "const"
                    );
                    if !is_statement_keyword {
                        result.push_str("let ");
                    }
                }
            }
            result.push_str(&line_to_process);
            let is_keyword_stmt = upper.starts_with("INSERT ")
                || upper.starts_with("SAVE ")
                || upper.starts_with("TALK ")
                || upper.starts_with("PRINT ")
                || upper.starts_with("MERGE ")
                || upper.starts_with("UPDATE ");
            let ends_with_block_brace = trimmed.ends_with('}') && !is_keyword_stmt && !trimmed.contains("#{");
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

        (r#"CLEAR\s+KB"#, 0, 0, vec![]),
        (r#"CLEAR\s+SUGGESTIONS"#, 0, 0, vec![]),
        (r#"CLEAR\s+SWITCHERS"#, 0, 0, vec![]),
        (r#"CLEAR\s+TOOLS"#, 0, 0, vec![]),
        (r#"CLEAR\s+WEBSITES"#, 0, 0, vec![]),

        (r#"ADD\s+SUGGESTION\s+TOOL"#, 2, 2, vec!["tool", "text"]),
        (r#"ADD\s+SUGGESTION\s+TEXT"#, 2, 2, vec!["value", "text"]),
        (r#"ADD\s+SUGGESTION(?!\s+TOOL|\s+TEXT|_)"#, 2, 2, vec!["context", "text"]),
        (r#"ADD\s+SWITCHER"#, 2, 2, vec!["switcher", "text"]),
        (r#"ADD\s+MEMBER"#, 2, 2, vec!["name", "role"]),
        (r#"ADD\s+MEMBER"#, 2, 2, vec!["name", "role"]),

        (r#"CREATE\s+TASK"#, 1, 1, vec!["task"]),
        (r#"CREATE\s+DRAFT"#, 4, 4, vec!["to", "subject", "body", "attachments"]),
        (r#"CREATE\s+SITE"#, 1, 1, vec!["site"]),

        (r#"ON\s+FORM\s+SUBMIT"#, 1, 1, vec!["form"]),
        (r#"ON\s+EMAIL"#, 1, 1, vec!["filter"]),
        (r#"ON\s+EVENT"#, 1, 1, vec!["event"]),

        (r#"SEND\s+MAIL"#, 4, 4, vec!["to", "subject", "body", "attachments"]),
        (r#"SEND\s+TEAMS\s+MESSAGE"#, 2, 2, vec!["chat_id", "message"]),
        (r#"SEND\s+TO"#, 2, 2, vec!["target", "message"]),

        (r#"HAS\s+ROLE"#, 1, 1, vec!["role"]),
        (r#"BOOK"#, 1, 1, vec!["event"]),

        (r#"VIBE\s+RUN"#, 1, 1, vec!["intent"]),
        (r#"VIBE\s+STATUS"#, 1, 1, vec!["run_id"]),
        (r#"VIBE\s+APPROVE"#, 1, 1, vec!["run_id"]),
        (r#"VIBE\s+CANCEL"#, 1, 1, vec!["run_id"]),
        (r#"VIBE\s+EVENTS"#, 1, 1, vec!["run_id"]),
        (r#"VIBE\s+TOOLS"#, 0, 0, vec![]),
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

        // Check for multiword keywords in ANY position (not just start of line)
        for (pattern, min_params, max_params, _param_names) in &multiword_patterns {
            // Use a regex that captures prefix (indent + any text before keyword) and params after keyword
            // Zero-param keywords (e.g. CLEAR KB) have no trailing params, so make the param group optional
            let regex_str = if *min_params == 0 {
                format!(
                    r#"(?i)^(\s*)(.*?)\b{}(?:\s+(.*))?$"#,
                    pattern
                )
            } else {
                format!(
                    r#"(?i)^(\s*)(.*?)\b{}\s+(.*)$"#,
                    pattern
                )
            };

            if let Ok(re) = Regex::new(&regex_str) {
                if let Some(caps) = re.captures(line) {
                    if let Some(prefix_match) = caps.get(2) {
                        let indent = caps.get(1).map_or("", |m| m.as_str());
                        let prefix = prefix_match.as_str();
                        let params_str = caps.get(3).map_or("", |m| m.as_str().trim());
                        let mut params = if params_str.is_empty() {
                            Vec::new()
                        } else {
                            parse_parameters(params_str)
                        };
                        let mut param_count = params.len();

                        // Fallback: if comma-based parsing gave too few params, try splitting by '='
                        // e.g. SET BOT MEMORY "key" = value uses '=' not ','
                        if param_count < *min_params && params_str.contains('=') && !params_str.contains(','){
                            let eq_parts: Vec<&str> = params_str.splitn(2, '=').collect();
                            if eq_parts.len() == 2 {
                                let key = eq_parts[0].trim();
                                let value = eq_parts[1].trim();
                                let fallback_params = vec![key.to_string(), value.to_string()];
                                if fallback_params.len() >= *min_params && fallback_params.len() <= *max_params {
                                    params = fallback_params;
                                    param_count = params.len();
                                }
                            }
                        }

                        // Fallback: try splitting by ' AS ' (for ADD_SUGGESTION, ADD_SUGGESTION_TOOL, etc.)
                        if param_count < *min_params {
                            let as_upper = params_str.to_uppercase();
                            if let Some(pos) = as_upper.find(" AS ") {
                                let a = &params_str[..pos];
                                let b = &params_str[pos + 4..];
                                let fallback_params = vec![a.trim().to_string(), b.trim().to_string()];
                                if fallback_params.len() >= *min_params && fallback_params.len() <= *max_params {
                                    params = fallback_params;
                                    param_count = params.len();
                                }
                            }
                        }

                        if param_count >= *min_params && param_count <= *max_params {
                            let keyword = pattern.replace(r"\s+", "_").to_lowercase();

                            let output = if keyword == "add_switcher" {
                                let (switcher_id, label) = if params.len() == 2 {
                                    (params[0].clone(), params[1].clone())
                                } else if params.len() == 3 && params[1].eq_ignore_ascii_case("AS") {
                                    (params[0].clone(), params[2].clone())
                                } else {
                                    (params[0].clone(), params.last().cloned().unwrap_or_default())
                                };
                                let up_keyword = keyword.to_uppercase();
                                format!("{}{}{} {} as {};", indent, prefix, up_keyword, switcher_id, label)
                            } else if keyword.starts_with("add_suggestion") {
                                // ADD_SUGGESTION, ADD_SUGGESTION_TOOL, ADD_SUGGESTION_TEXT
                                // all use "AS" separator between param1 and param2 and are custom syntaxes, not functions
                                let (a, b) = if params.len() == 2 {
                                    (params[0].clone(), params[1].clone())
                                } else if params.len() >= 3 {
                                    let as_idx = params.iter().position(|p| p.eq_ignore_ascii_case("as")).unwrap_or(1);
                                    (params[0].clone(), params[as_idx + 1..].join(" "))
                                } else {
                                    (params[0].clone(), String::new())
                                };
                                let up_keyword = keyword.to_uppercase();
                                format!("{}{}{} {} as {};", indent, prefix, up_keyword, a, b)
                            } else {
                                let param_str = if params.is_empty() {
                                    String::new()
                                } else {
                                    params.join(", ")
                                };
                                format!("{}{}{}({});", indent, prefix, keyword, param_str)
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
            let trimmed_line = line.trim();
            let is_comment = trimmed_line.starts_with('\'') || trimmed_line.starts_with('#') || trimmed_line.starts_with("//");
            let is_decl = trimmed_line.starts_with("BEGIN ") || trimmed_line.starts_with("END ") || trimmed_line.starts_with("TABLE ");
            let is_control = trimmed_line.starts_with("IF ") || trimmed_line.starts_with("ELSE") || trimmed_line.starts_with("END ") || trimmed_line.starts_with("NEXT") || trimmed_line.starts_with("LOOP") || trimmed_line.starts_with("FOR ") || trimmed_line.starts_with("WHILE") || trimmed_line.starts_with("SWITCH") || trimmed_line.starts_with("CASE") || trimmed_line.starts_with("DEFAULT") || trimmed_line.starts_with("DO") || trimmed_line.starts_with("UNTIL") || trimmed_line.starts_with("WEND");
            if is_comment || trimmed_line.is_empty() {
                // skip comment/blank lines entirely — Rhai doesn't understand ' as comment
            } else if is_decl || is_control {
                result.push_str(line);
            } else if trimmed_line.ends_with(';') || trimmed_line.ends_with('{') || (trimmed_line.ends_with('}') && !trimmed_line.contains("#{")) || trimmed_line.ends_with(':') {
                result.push_str(line);
            } else {
                result.push_str(line);
                result.push(';');
            }
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
    let mut paren_depth: i32 = 0;
    let chars: Vec<char> = params_str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
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
            '(' if !in_quotes => {
                paren_depth += 1;
                current.push(c);
            }
            ')' if !in_quotes => {
                paren_depth -= 1;
                current.push(c);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                if !current.is_empty() {
                    params.push(current.trim().to_string());
                    current = String::new();
                }
            }
            _ => {
                current.push(c);
            }
        }
        i += 1;
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
