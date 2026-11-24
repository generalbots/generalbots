//! How a turn ends: the tool-round budget, the last round that offers no
//! tools, and the final text frame.
//!
//! A model that keeps calling tools used to run one more tool at the depth
//! limit and then stop with no text at all. The last follow-up now offers no
//! tools and asks for an answer with what was found; if it still brings no
//! text, the person is told the search could not be finished.

use super::super::types::SessionIdentity;

/// Follow-up model calls after tool results, for most tools.
const MAX_TOOL_FOLLOWUPS: usize = 2;
/// Follow-ups when every call of the round is a web tool: a search often
/// needs other words and a page read. `web_search` keeps its own cap of
/// searches per answer.
const MAX_WEB_FOLLOWUPS: usize = 4;
const WEB_TOOLS: [&str; 2] = ["web_search", "web_fetch"];

const TOOL_CALL_MARKER: &str = "\"__tool_call__\":";

fn call_name(chunk: &str) -> Option<String> {
    let start = chunk.find('{')?;
    let end = chunk.rfind('}')?;
    let value: serde_json::Value = serde_json::from_str(chunk.get(start..=end)?).ok()?;
    value.get("name")?.as_str().map(str::to_string)
}

/// Follow-ups a turn may take after the tool calls in `full_response`.
pub(super) fn max_followups(full_response: &str) -> usize {
    let names: Vec<Option<String>> = full_response
        .split('\n')
        .filter(|line| line.contains(TOOL_CALL_MARKER))
        .map(call_name)
        .collect();
    let web_only = !names.is_empty()
        && names
            .iter()
            .all(|name| name.as_deref().is_some_and(|n| WEB_TOOLS.contains(&n)));
    if web_only {
        MAX_WEB_FOLLOWUPS
    } else {
        MAX_TOOL_FOLLOWUPS
    }
}

/// Whether the follow-up after the tool round at `tool_depth` is the last.
pub(super) fn next_is_final(tool_depth: usize, full_response: &str) -> bool {
    tool_depth + 1 >= max_followups(full_response)
}

/// The tools a round offers: none in the last round.
pub(super) fn tools_for_round<T>(tools: Vec<T>, final_round: bool) -> Option<Vec<T>> {
    (!final_round && !tools.is_empty()).then_some(tools)
}

/// Whether a reply's tool calls run: never in the last round, where a call
/// the model still makes is dropped and its text, if any, is the answer.
pub(super) fn runs_tools(full_response: &str, final_round: bool) -> bool {
    !final_round && full_response.contains(TOOL_CALL_MARKER)
}

/// The answer text of a reply: in the last round, a tool call the model
/// still wrote as text is dropped, so it never reaches the person as JSON.
pub(super) fn answer_text(content: &str, final_round: bool) -> String {
    if !final_round || !content.contains(TOOL_CALL_MARKER) {
        return content.to_string();
    }
    content
        .split('\n')
        .filter(|line| !line.contains(TOOL_CALL_MARKER))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// The message that hands tool output back to the model.
pub(super) fn followup_instruction(output: &str, final_round: bool) -> String {
    if final_round {
        format!(
            "The tool finished with this result:\n\n{output}\n\nNo more tools can run for this answer. \
             Answer the user's original request now with what you have, in plain text and without \
             any tool call. Say plainly what you could not confirm."
        )
    } else {
        format!(
            "The tool finished with this result:\n\n{output}\n\nAnswer the user's original request now \
             using this result. Do not call another tool unless it is strictly necessary."
        )
    }
}

/// What the person reads when the last round brought no answer.
pub(super) fn unfinished_text(language: &str) -> &'static str {
    if language.starts_with("pt") {
        "Não consegui concluir a pesquisa para esta resposta. Tente de novo ou reformule a \
         pergunta; se quiser, diga onde devo procurar."
    } else {
        "I could not finish the research for this answer. Try again or rephrase the question; \
         you can also tell me where to look."
    }
}

/// Sends the turn's final text with its end frame (`is_complete`).
pub(super) async fn send_text(identity: &SessionIdentity<'_>, content: String, reasoning: &str) {
    let final_resp = botlib::models::BotResponse {
        bot_id: identity.bot_uuid.to_string(),
        user_id: identity.user_id.to_string(),
        session_id: identity.session_id.to_string(),
        channel: "web".to_string(),
        content,
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: reasoning.trim().to_string(),
    };
    // #2084 — a transport error is logged, never swallowed.
    if let Err(error) = identity.sink.send_bot_response(&final_resp).await {
        log::error!(
            "final LLM reply not delivered ({} bytes): {error}",
            final_resp.content.len()
        );
    }
}

/// Ends a last round that brought no text; returns what was sent.
pub(super) async fn send_unfinished(identity: &SessionIdentity<'_>) -> &'static str {
    let locale = botcore::i18n::locale_for_user_pool(&identity.state.conn, identity.user_id);
    let text = unfinished_text(locale.language());
    log::warn!("last tool round brought no text; the turn ends with the unfinished notice");
    send_text(identity, text.to_string(), "").await;
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(name: &str) -> String {
        format!("{{\"__tool_call__\": true, \"name\": \"{name}\", \"arguments\": {{}}}}")
    }

    #[test]
    fn web_rounds_get_the_larger_budget() {
        assert_eq!(max_followups(&call("web_search")), MAX_WEB_FOLLOWUPS);
        let both = format!("{}\n{}", call("web_search"), call("web_fetch"));
        assert_eq!(max_followups(&both), MAX_WEB_FOLLOWUPS);
    }

    #[test]
    fn other_or_mixed_rounds_get_the_default_budget() {
        assert_eq!(max_followups(&call("vm_exec")), MAX_TOOL_FOLLOWUPS);
        let mixed = format!("{}\n{}", call("web_search"), call("drive_save"));
        assert_eq!(max_followups(&mixed), MAX_TOOL_FOLLOWUPS);
        assert_eq!(max_followups("no call at all"), MAX_TOOL_FOLLOWUPS);
        assert_eq!(max_followups("{\"__tool_call__\": true, broken"), MAX_TOOL_FOLLOWUPS);
    }

    #[test]
    fn the_depth_limit_makes_the_next_round_final() {
        let web = call("web_search");
        assert!(!next_is_final(0, &web));
        assert!(!next_is_final(2, &web));
        assert!(next_is_final(3, &web));
        let other = call("vm_exec");
        assert!(!next_is_final(0, &other));
        assert!(next_is_final(1, &other));
        assert!(next_is_final(5, &other));
    }

    #[test]
    fn the_last_round_offers_and_runs_no_tools() {
        assert_eq!(tools_for_round(vec![1, 2], false), Some(vec![1, 2]));
        assert_eq!(tools_for_round(vec![1, 2], true), None);
        assert_eq!(tools_for_round(Vec::<u8>::new(), false), None);
        assert!(runs_tools(&call("web_search"), false));
        assert!(!runs_tools(&call("web_search"), true));
        assert!(!runs_tools("plain answer", false));
    }

    #[test]
    fn the_last_round_drops_a_tool_call_written_as_text() {
        let reply = format!("Python 3.14 is the latest.\n{}", call("web_search"));
        assert_eq!(answer_text(&reply, true), "Python 3.14 is the latest.");
        assert_eq!(answer_text(&call("web_search"), true), "");
        assert_eq!(answer_text(&reply, false), reply);
        let prefixed = format!("Searching: {}", call("web_fetch"));
        assert_eq!(max_followups(&prefixed), MAX_WEB_FOLLOWUPS);
    }

    #[test]
    fn the_final_instruction_forbids_tools_and_asks_for_gaps() {
        let last = followup_instruction("RESULT", true);
        assert!(last.contains("RESULT"));
        assert!(last.contains("without any tool call"));
        assert!(last.contains("could not confirm"));
        assert!(!followup_instruction("RESULT", false).contains("No more tools"));
    }

    #[test]
    fn the_unfinished_notice_follows_the_language() {
        assert!(unfinished_text("pt").starts_with("Não consegui concluir a pesquisa"));
        assert!(unfinished_text("en").starts_with("I could not finish"));
    }
}
