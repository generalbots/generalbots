//! One-shot repair of a program that failed with a syntax error.
//!
//! The repair talks to the engine of the turn (the provider and model the
//! person chose), gets the person's request with the broken program, and must
//! keep writing the same deliverables into the conversation's `output/`
//! folder. A subscription agent fixes its own programs: no repair runs there.

use super::vm_turn_engine;

/// Extensions of files a program delivers to the person.
const DELIVERABLE_EXTENSIONS: [&str; 20] = [
    "docx", "pptx", "xlsx", "pdf", "csv", "odt", "ods", "odp", "md", "txt", "html", "json", "png",
    "jpg", "jpeg", "svg", "zip", "mp3", "mp4", "wav",
];

/// Whether a failed program gets a repair pass in this conversation.
pub(super) fn offered(session_id: uuid::Uuid) -> bool {
    vm_turn_engine::current(session_id).is_none_or(|engine| !engine.subscription)
}

/// Asks the turn's engine to rewrite a program that failed with a syntax
/// error. `Ok` is the corrected source; `Err(Some(why))` a rewrite that was
/// refused because it no longer writes the expected `output/` files;
/// `Err(None)` no usable answer.
pub(super) async fn repair_program(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    bot_uuid: uuid::Uuid,
    session_id: uuid::Uuid,
    language: &str,
    code: &str,
    error: &str,
) -> Result<String, Option<String>> {
    let turn = vm_turn_engine::current(session_id);
    let (provider, model, key, request) = match turn.as_ref() {
        Some(turn) => (turn.provider.clone(), turn.model.clone(), turn.key.clone(), turn.request.clone()),
        None => {
            let Some(provider) = state.llm_provider.clone() else {
                botlog::safe_warn!("program repair skipped: no LLM provider");
                return Err(None);
            };
            let (model, key) = super::super::tool_exec::bot_llm_credentials(&state.conn, bot_uuid);
            (provider, model, key, String::new())
        }
    };
    // Keyless gateways (free tiers) legitimately carry an empty key; only a
    // missing model makes the repair call impossible.
    if model.is_empty() {
        botlog::safe_warn!("program repair skipped: no model for this turn");
        return Err(None);
    }
    let expected = expected_outputs(code);
    let prompt = repair_prompt(session_id, &request, language, code, error, &expected);
    let reply = match provider.generate(&prompt, &serde_json::json!({}), &model, &key).await {
        Ok(text) => text,
        Err(e) => {
            botlog::safe_warn!("program repair LLM call failed: {e}");
            return Err(None);
        }
    };
    let cleaned = strip_code_fences(&reply);
    if cleaned.trim().is_empty() || cleaned.trim() == code.trim() {
        return Err(None);
    }
    match check_outputs(&cleaned, &expected) {
        Ok(()) => Ok(cleaned),
        Err(why) => {
            botlog::safe_warn!("program repair refused: session={session_id} {why}");
            Err(Some(why))
        }
    }
}

fn repair_prompt(
    session_id: uuid::Uuid,
    request: &str,
    language: &str,
    code: &str,
    error: &str,
    expected: &[String],
) -> String {
    let asked = if request.trim().is_empty() {
        String::new()
    } else {
        format!("The person asked:\n\n{}\n\n", request.trim())
    };
    let names = if expected.is_empty() {
        String::new()
    } else {
        format!(" Keep these file names: {}.", expected.join(", "))
    };
    format!(
        "{asked}The following {language} program, written for that request, failed with this \
         error:\n\n{error}\n\nProgram (it may be cut short):\n\n{code}\n\n\
         Rewrite the whole program so it runs correctly and delivers what was asked, completing \
         it where it is cut. It runs with the conversation folder as its working directory. Write \
         every file for the person into output/ (relative to the working directory, that is \
         /opt/vibe/conversations/{session_id}/output/), never into program/ and never relative to \
         the script file (__file__).{names} Keep the @@GB_TRACE@@ lines. A newline inside a \
         quoted string is invalid Python: use \\n escapes or triple-quoted strings for multi-line \
         text. Return ONLY the complete corrected program, no markdown fences and no explanation."
    )
}

/// File names with a deliverable extension written as string literals in
/// `code` (`"output/memo.docx"`, `os.path.join(OUT, "deck.pptx")`).
fn expected_outputs(code: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for literal in code.split(['"', '\'']) {
        if literal.len() > 160 || literal.contains(['\n', '{', '}', '*']) {
            continue;
        }
        let name = literal.rsplit('/').next().unwrap_or(literal).trim();
        let Some((stem, extension)) = name.rsplit_once('.') else {
            continue;
        };
        let deliverable = DELIVERABLE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str());
        if deliverable && !stem.trim().is_empty() && !names.iter().any(|known| known == name) {
            names.push(name.to_string());
        }
    }
    names
}

/// Refuses a rewrite that writes outside `output/` or drops an expected file.
fn check_outputs(program: &str, expected: &[String]) -> Result<(), String> {
    if program.contains("__file__") {
        return Err("the rewrite builds its paths from the script folder (program/) instead of output/".into());
    }
    if !program.contains("output") {
        return Err("the rewrite does not write into output/".into());
    }
    match expected.iter().find(|name| !program.contains(name.as_str())) {
        Some(name) => Err(format!("the rewrite no longer writes output/{name}")),
        None => Ok(()),
    }
}

fn strip_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let body: Vec<&str> = trimmed.lines().skip(1).collect();
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

#[cfg(test)]
mod tests {
    use super::{check_outputs, expected_outputs, repair_prompt};

    #[test]
    fn deliverable_names_are_read_from_string_literals() {
        let code = "OUT = 'output'\nprs.save(os.path.join(OUT, \"Apresentação.pptx\"))\n\
                    doc.save('output/memo.docx')\nopen(f'{name}.csv')\nimport run.py";
        assert_eq!(expected_outputs(code), vec!["Apresentação.pptx", "memo.docx"]);
    }

    #[test]
    fn a_rewrite_into_the_script_folder_or_without_the_file_is_refused() {
        let expected = vec!["memo.docx".to_string()];
        let script_folder = "BASE = os.path.dirname(os.path.abspath(__file__))\ndoc.save(os.path.join(BASE, 'output', 'memo.docx'))";
        assert!(check_outputs(script_folder, &expected).is_err());
        let renamed = "doc.save('output/documento.docx')";
        assert!(check_outputs(renamed, &expected).unwrap_err().contains("memo.docx"));
        assert!(check_outputs("doc.save('output/memo.docx')", &expected).is_ok());
        assert!(check_outputs("doc.save('memo.docx')", &[]).is_err());
    }

    #[test]
    fn the_prompt_carries_the_request_and_the_output_folder() {
        let session = uuid::Uuid::nil();
        let prompt = repair_prompt(session, "Um memorando em Word", "python", "x = (", "SyntaxError", &["memo.docx".to_string()]);
        assert!(prompt.starts_with("The person asked:\n\nUm memorando em Word"));
        assert!(prompt.contains(&format!("/opt/vibe/conversations/{session}/output/")));
        assert!(prompt.contains("Keep these file names: memo.docx."));
    }
}
