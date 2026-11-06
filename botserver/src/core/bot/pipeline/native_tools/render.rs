//! Formats workspace output for the chat: stdout/stderr plus the files the run
//! left in `output/`.

/// Maximum characters of process output echoed back into chat. A runaway
/// program must not paste megabytes into the conversation.
const OUTPUT_LIMIT: usize = 8_000;

fn truncate(text: &str) -> String {
    let mut out: String = text.chars().take(OUTPUT_LIMIT).collect();
    if text.chars().count() > OUTPUT_LIMIT {
        out.push_str("\n… (output truncated)");
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
    let mut message = String::new();
    let out = stdout.trim();
    if !out.is_empty() {
        message.push_str("```\n");
        message.push_str(&truncate(out));
        message.push_str("\n```\n");
    }
    let err = stderr.trim();
    if !err.is_empty() || !ok {
        message.push_str("\n**Error**\n");
        if err.is_empty() {
            message.push_str("```\nThe run finished without output.\n```\n");
        } else {
            message.push_str("```\n");
            message.push_str(&truncate(err));
            message.push_str("\n```\n");
        }
    }
    let files = crate::core::bot::agent_vm_hook::list_user_vm_output(user_id, session_id)
        .await
        .unwrap_or_default();
    if !files.is_empty() {
        let conversation = session_id.to_string();
        message.push_str("\n**Files**\n");
        for file in &files {
            message.push_str(&format!(
                "- [{file}](vmfile:{conversation}/output/{file})\n"
            ));
        }
    }
    message.trim_end().to_string()
}
