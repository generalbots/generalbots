use super::*;

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
/// `CASE 1` must stay numeric: comparing an integer subject with the string
/// `"1"` is always false in Rhai, which silently skipped every branch of an
/// integer `SELECT`.
fn is_numeric_literal(raw: &str) -> bool {
    !raw.is_empty() && raw.parse::<f64>().is_ok()
}

pub fn convert_select_case_syntax(script: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = script.lines().collect();
    let mut i = 0;

    log::trace!("Converting SELECT/CASE syntax to if-else chains");

    pub(super) fn strip_let_from_assignment(line: &str) -> String {
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

        // `SELECT ... FROM` is SQL, not the SELECT/CASE construct.
        if upper.starts_with("SELECT ") && !upper.contains(" THEN") && !upper.contains(" FROM ") {
            let subject = trimmed[7..].trim();
            // `SELECT CASE expr` is the VBA spelling used by the shipped
            // templates; the subject is `expr`, not `CASE expr`.
            let select_var = if subject.to_uppercase().starts_with("CASE ") {
                subject[5..].trim().to_string()
            } else {
                subject.to_string()
            };
            log::trace!("Converting SELECT statement for variable: '{select_var}'");

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
                        result.push_str("}\n");
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
                        result.push_str("}\n");
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

                    let case_raw = case_trimmed[5..].trim();

                    // `CASE ELSE` is the default branch, not a literal to match.
                    if case_raw.eq_ignore_ascii_case("ELSE") {
                        if is_first_case {
                            result.push_str("if true {\n");
                            is_first_case = false;
                        } else {
                            result.push_str("} else {\n");
                        }
                        in_case = true;
                        i += 1;
                        continue;
                    }

                    let case_value = if case_raw.starts_with('"') || is_numeric_literal(case_raw) {
                        case_raw.to_string()
                    } else {
                        format!("\"{case_raw}\"")
                    };

                    if is_first_case {
                        result.push_str(&format!("if {select_var} == {case_value} {{\n"));
                        is_first_case = false;
                    } else {
                        result.push_str(&format!("}} else if {select_var} == {case_value} {{\n"));
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
        result.push_str(&lowercase_builtins_outside_literals(line, &rhai_builtins));
        result.push('\n');
    }
    result
}

/// Lowercases Rhai builtins in code only. Text inside a string literal is user
/// data — `TALK "IF you dare"` must not come out as `TALK "if you dare"`.
fn lowercase_builtins_outside_literals(line: &str, builtins: &[&str]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut code = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for ch in line.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            out.push_str(&lowercase_builtins(&code, builtins));
            code.clear();
            out.push(ch);
            in_string = true;
        } else {
            code.push(ch);
        }
    }
    out.push_str(&lowercase_builtins(&code, builtins));
    out
}

fn lowercase_builtins(code: &str, builtins: &[&str]) -> String {
    let mut processed = code.to_string();
    for keyword in builtins {
        let pattern = format!(r"\b{}\b", regex::escape(keyword));
        if let Some(re) = crate::re_cache::dynamic(&pattern) {
            processed = re.replace_all(&processed, keyword.to_lowercase()).to_string();
        }
    }
    processed
}

/// Outside string literals, bare `TODAY`/`NOW` tokens become calls
/// `TODAY()`/`NOW()`: the runtime registers them as functions
/// (botbasic_core datetime/now.rs), but BASIC scripts write them as bare
/// variables, and Rhai reads a bare identifier as a variable lookup
/// ("Variable not found: TODAY"). Old ASTs heal on load because the
/// execution-time self-heal runs this same transform.
pub(super) fn rewrite_bare_datetime_tokens(line: &str) -> String {
    let Some(re) = crate::re_cache::cached_re!(r"\b(TODAY|NOW)\b") else {
        return line.to_string();
    };
    // Append the token and normalize what follows: bare `TODAY` gains `()`;
    // an existing `TODAY()` is copied through unchanged. A trailing `\b`
    // after an optional `\(\)` group would backtrack (word end boundary
    // fails after `)`), matching the bare token inside `TODAY()` and
    // producing the invalid double call `TODAY()()` — hence the manual
    // paren check instead of regex lookahead (unsupported in Rust regex).
    pub(super) fn rewrite_plain(re: &Regex, plain: &str, out: &mut String) {
        let mut last = 0;
        for m in re.find_iter(plain) {
            out.push_str(&plain[last..m.start()]);
            out.push_str(m.as_str());
            let rest = &plain[m.end()..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with('(') {
                if let Some(rel) = trimmed.find(')') {
                    let skip = rest.len() - trimmed.len() + rel + 1;
                    out.push_str(&rest[..skip]);
                    last = m.end() + skip;
                    continue;
                }
            }
            out.push_str("()");
            last = m.end();
        }
        out.push_str(&plain[last..]);
    }
    let mut out = String::with_capacity(line.len() + 8);
    let mut plain = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in line.chars() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            rewrite_plain(re, &plain, &mut out);
            plain.clear();
            out.push(ch);
            in_string = true;
        } else {
            plain.push(ch);
        }
    }
    rewrite_plain(re, &plain, &mut out);
    out
}
