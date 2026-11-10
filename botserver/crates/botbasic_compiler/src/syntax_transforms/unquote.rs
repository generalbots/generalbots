

/// Unquote a BASIC token that may arrive wrapped in double quotes.
pub(super) fn unquote(s: &str) -> &str {
    s.trim().trim_matches('"')
}

/// Removes a trailing statement terminator `;` from a parameter string,
/// but only when the semicolon sits OUTSIDE quoted literals. The compiled
/// `.ast` files produced by BasicCompiler append `;` to every line; without
/// this the terminator leaks into the last function-call argument and the
/// rewritten script fails to parse (e.g. `vibe_run("build an app";)`).
pub(super) fn strip_trailing_stmt_semicolon(s: &str) -> String {
    let trimmed = s.trim_end();
    if !trimmed.ends_with(';') {
        return trimmed.to_string();
    }
    let mut in_quote: Option<char> = None;
    for (i, c) in trimmed.char_indices() {
        if let Some(q) = in_quote {
            if c == q {
                in_quote = None;
            }
        } else if c == '"' || c == '\'' {
            in_quote = Some(c);
        }
        if c == ';' && in_quote.is_none() && i == trimmed.len() - 1 {
            return trimmed[..i].to_string();
        }
    }
    trimmed.to_string()
}
pub(super) fn parse_parameters(params_str: &str) -> Vec<String> {
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
                // Clamp at zero: an unbalanced `)` must not push the depth
                // negative, which would stop every later comma from splitting.
                // (`i32::saturating_sub` clamps at `i32::MIN`, not at zero.)
                paren_depth = (paren_depth - 1).max(0);
                current.push(c);
            }
            '[' | '{' if !in_quotes => {
                paren_depth += 1;
                current.push(c);
            }
            ']' | '}' if !in_quotes => {
                paren_depth = (paren_depth - 1).max(0);
                current.push(c);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    params.push(current.trim().to_string());
                }
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
        i += 1;
    }

    if !current.trim().is_empty() {
        params.push(current.trim().to_string());
    }

    params
}
/// `LLM "<prompt>"` gains the default routing clause `WITH OPTIMIZE FOR
/// "speed"` unless the statement already declares one (on the same line or on
/// the continuation line). The prompt literal is copied verbatim: dropping its
/// closing quote turned `LLM "ask"` into the malformed
/// `LLM "ask WITH OPTIMIZE FOR "speed"`.
pub fn preprocess_llm_keyword(script: &str) -> String {
    let lines: Vec<&str> = script.lines().collect();
    let mut result = String::new();

    for (idx, line) in lines.iter().enumerate() {
        let next_line = lines.get(idx + 1).copied().unwrap_or("");
        let next_declares_clause = next_line
            .trim_start()
            .to_uppercase()
            .starts_with("WITH OPTIMIZE");
        result.push_str(&rewrite_llm_statement(line, next_declares_clause));
        result.push('\n');
    }

    result
}

/// True when `chars[i..]` starts with the `LLM ` keyword (case-insensitive and
/// not part of a longer identifier).
fn is_llm_keyword_at(chars: &[char], i: usize) -> bool {
    const KEYWORD: [char; 4] = ['L', 'L', 'M', ' '];
    if i + KEYWORD.len() > chars.len() {
        return false;
    }
    let matches_keyword = chars[i..i + KEYWORD.len()]
        .iter()
        .zip(KEYWORD)
        .all(|(c, expected)| c.to_ascii_uppercase() == expected);
    if !matches_keyword {
        return false;
    }
    i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')
}

fn rewrite_llm_statement(line: &str, next_declares_clause: bool) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len() + 32);
    let mut i = 0;

    while i < chars.len() {
        if !is_llm_keyword_at(&chars, i) {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        out.push_str("LLM ");
        i += 4;

        // Unquoted expressions (`LLM prompt_variable`) are left as written.
        if i >= chars.len() || chars[i] != '"' {
            continue;
        }

        out.push('"');
        i += 1;
        while i < chars.len() && chars[i] != '"' {
            out.push(chars[i]);
            i += 1;
        }
        if i < chars.len() {
            out.push('"');
            i += 1;
        }

        let tail: String = chars[i..].iter().collect();
        let already_declared = next_declares_clause
            || tail.to_uppercase().contains("WITH OPTIMIZE");
        if !already_declared {
            out.push_str(" WITH OPTIMIZE FOR \"speed\"");
        }
    }

    out
}

