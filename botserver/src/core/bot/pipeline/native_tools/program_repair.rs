//! One-shot repair of a program that failed with a syntax error.

/// Asks the model to rewrite a program that failed with a syntax error.
/// Returns only the corrected source, or None when the provider is absent.
pub(super) async fn repair_program(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    bot_uuid: uuid::Uuid,
    language: &str,
    code: &str,
    error: &str,
) -> Option<String> {
    let provider = match state.llm_provider.as_ref() {
        Some(p) => p,
        None => {
            botlog::safe_warn!("program repair skipped: no LLM provider");
            return None;
        }
    };
    let (model, key) = super::super::tool_exec::bot_llm_credentials(&state.conn, bot_uuid);
    // Keyless gateways (Kilo free tier) legitimately carry an empty key; only
    // a missing model makes the repair call impossible.
    if model.is_empty() {
        botlog::safe_warn!("program repair skipped: no llm-model configured");
        return None;
    }
    let prompt = format!(
        "The following {language} program failed with this error:\n\n{error}\n\n\
         Program:\n\n{code}\n\n\
         Rewrite the whole program so it runs correctly. Preserve the intent and \
         the output path. A newline inside a quoted string is invalid Python: use \
         \\n escapes or triple-quoted strings for multi-line text. Return ONLY the \
         complete corrected program, no markdown fences and no explanation."
    );
    let reply = match provider
        .generate(&prompt, &serde_json::json!({}), &model, &key)
        .await
    {
        Ok(text) => text,
        Err(e) => {
            botlog::safe_warn!("program repair LLM call failed: {e}");
            return None;
        }
    };
    let cleaned = strip_code_fences(&reply);
    if cleaned.trim().is_empty() || cleaned.trim() == code.trim() {
        None
    } else {
        Some(cleaned)
    }
}

fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let mut lines = trimmed.lines();
    let _ = lines.next();
    let body: Vec<&str> = lines.collect();
    let mut out = body.join("\n");
    if let Some(pos) = out.rfind("```") {
        out.truncate(pos);
    }
    out.trim().to_string()
}

/// True when a failed run looks like a malformed program rather than a runtime
/// error (those are reported to the user, not silently rewritten).
pub(super) fn looks_like_syntax_error(stderr: &str) -> bool {
    stderr.contains("SyntaxError")
        || stderr.contains("IndentationError")
        || stderr.contains("TabError")
        || stderr.contains("Unexpected token")
        || stderr.contains("Unexpected end of input")
}
