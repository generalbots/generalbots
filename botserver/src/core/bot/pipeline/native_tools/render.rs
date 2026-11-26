//! Formats workspace output for the chat: stdout/stderr plus the files the run
//! left in `output/`.

/// Maximum characters of process output echoed back into chat. A runaway
/// program must not paste megabytes into the conversation.
const OUTPUT_LIMIT: usize = 8_000;

/// The words around the program's own output.
struct Labels {
    error: &'static str,
    no_output: &'static str,
    truncated: &'static str,
    files: &'static str,
}

/// For the model, which answers the person in their language.
const FOR_MODEL: Labels = Labels {
    error: "**Error**",
    no_output: "The run finished without output.",
    truncated: "… (output truncated)",
    files: "**Files**",
};

/// For the person, who reads a background job's result in the conversation.
const FOR_PERSON: Labels = Labels {
    error: "**Erro**",
    no_output: "A execução terminou sem saída.",
    truncated: "… (saída cortada)",
    files: "**Arquivos**",
};

fn truncate(text: &str, labels: &Labels) -> String {
    let mut out: String = text.chars().take(OUTPUT_LIMIT).collect();
    if text.chars().count() > OUTPUT_LIMIT {
        out.push('\n');
        out.push_str(labels.truncated);
    }
    out
}

/// Formats stdout/stderr and the files left in `output/` as plain command
/// output (#1632 item 4): no "program" wording, just what the run produced.
pub(super) async fn render_result(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    ok: bool,
    stdout: &str,
    stderr: &str,
) -> String {
    let files = output_files(user_id, session_id).await;
    format_result(&FOR_MODEL, session_id, ok, stdout, stderr, &files)
}

/// [`render_result`] with the person's wording, for a result saved straight
/// into the conversation.
pub(super) async fn render_result_for_person(
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    ok: bool,
    stdout: &str,
    stderr: &str,
) -> String {
    let files = output_files(user_id, session_id).await;
    format_result(&FOR_PERSON, session_id, ok, stdout, stderr, &files)
}

async fn output_files(user_id: uuid::Uuid, session_id: uuid::Uuid) -> Vec<String> {
    crate::core::bot::agent_vm_hook::list_user_vm_output(user_id, session_id)
        .await
        .unwrap_or_else(|| { log::debug!("using default for vm output files"); Default::default() })
}

fn format_result(
    labels: &Labels,
    session_id: uuid::Uuid,
    ok: bool,
    stdout: &str,
    stderr: &str,
    files: &[String],
) -> String {
    let mut message = String::new();
    let out = stdout.trim();
    if !out.is_empty() {
        message.push_str("```\n");
        message.push_str(&truncate(out, labels));
        message.push_str("\n```\n");
    }
    let err = stderr.trim();
    if !err.is_empty() || !ok {
        message.push('\n');
        message.push_str(labels.error);
        message.push('\n');
        if err.is_empty() {
            message.push_str(&format!("```\n{}\n```\n", labels.no_output));
        } else {
            message.push_str("```\n");
            message.push_str(&truncate(err, labels));
            message.push_str("\n```\n");
        }
    }
    if !files.is_empty() {
        let conversation = session_id.to_string();
        message.push('\n');
        message.push_str(labels.files);
        message.push('\n');
        for file in files {
            message.push_str(&format!(
                "- [{file}](vmfile:{conversation}/output/{file})\n"
            ));
        }
    }
    message.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_person_reads_a_background_result_in_portuguese() {
        let session = uuid::Uuid::new_v4();
        let files = vec!["a.csv".to_string()];
        let person = format_result(&FOR_PERSON, session, false, "", "", &files);
        assert!(person.contains("**Erro**"));
        assert!(person.contains("A execução terminou sem saída."));
        assert!(person.contains("**Arquivos**"));
        assert!(person.contains(&format!("vmfile:{session}/output/a.csv")));
        assert!(!person.contains("Error") && !person.contains("Files"));

        let long = "x".repeat(OUTPUT_LIMIT + 1);
        assert!(format_result(&FOR_PERSON, session, true, &long, "", &[]).contains("(saída cortada)"));

        let model = format_result(&FOR_MODEL, session, false, "", "", &files);
        assert!(model.contains("**Error**") && model.contains("**Files**"));
    }
}
