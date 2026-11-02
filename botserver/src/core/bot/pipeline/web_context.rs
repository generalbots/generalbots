//! #1547/#1632 — live web grounding driven by the composer's Web switch.
//!
//! The switch is the decision (#1632 item 3): when it is on, the user's turn
//! is searched as written and the results are injected as the freshest system
//! message; when it is off, nothing is searched. There is no extra routing
//! completion that asks the model whether it "needs" the web.
//!
//! A failed search never blocks the turn: the answer is produced without web
//! context, and the model can still call the `web.search` command itself.

use serde_json::json;
use uuid::Uuid;

use botlib::models::BotResponse;

use super::sink::ChannelSink;
use super::types::SessionIdentity;

const MAX_RESULTS: usize = 5;
const REGION: &str = "wt-wt";
const SEARCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);
const MAX_QUERY_CHARS: usize = 160;

/// Collapses whitespace and caps the length so a pasted paragraph cannot turn
/// into an unbounded search request.
fn normalize_query(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_QUERY_CHARS)
        .collect()
}

/// Streams a one-line progress note into the assistant bubble's thinking
/// section. Channels without raw frames simply ignore it.
async fn status(
    sink: &dyn ChannelSink,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) {
    // The chat renders a live status line while the trip happens; channels
    // without raw frames fall back to the thinking stream.
    if sink.supports_raw_frames() {
        let frame = json!({ "type": "chat_status", "text": text });
        if sink.send_raw_json(&frame).await.is_ok() {
            return;
        }
    }
    let response = BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: String::new(),
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: false,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: format!("{text}\n"),
    };
    let _ = sink.send_bot_response(&response).await;
}

/// Searches the user's turn directly and injects the results. Called only when
/// the composer's Web switch is on.
pub async fn search_and_inject(
    identity: &SessionIdentity<'_>,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let query = normalize_query(user_text);
    if query.is_empty() {
        return;
    }

    status(
        sink,
        bot_uuid,
        session_id,
        user_id,
        &format!("Searching the web for “{query}”…"),
    )
    .await;

    let results = match tokio::time::timeout(
        SEARCH_TIMEOUT,
        botresearch::web_search::search_web(&query, MAX_RESULTS, REGION),
    )
    .await
    {
        Ok(Ok(results)) => results,
        Ok(Err(e)) => {
            log::info!("web context: search failed: {e}");
            status(sink, bot_uuid, session_id, user_id, "The search failed — answering from what I know.").await;
            return;
        }
        Err(_) => {
            log::info!("web context: search timed out");
            status(sink, bot_uuid, session_id, user_id, "The search timed out — answering from what I know.").await;
            return;
        }
    };
    if results.is_empty() {
        status(sink, bot_uuid, session_id, user_id, "No usable sources — answering from what I know.").await;
        return;
    }

    let domains: Vec<&str> = {
        let mut seen: Vec<&str> = Vec::new();
        for result in &results {
            if !seen.contains(&result.source.as_str()) {
                seen.push(result.source.as_str());
            }
        }
        seen
    };
    status(
        sink,
        bot_uuid,
        session_id,
        user_id,
        &format!(
            "Found {} source{} ({}). Reading them and writing the answer…",
            results.len(),
            if results.len() == 1 { "" } else { "s" },
            domains.join(", ")
        ),
    )
    .await;

    let mut block = String::from(
        "Fresh web search results are provided below. They are sufficient for \
         this turn: answer the user NOW from this material. Do not call \
         api.find, web.search or any other command to look for a search tool — \
         the search has already been done for you. Cite the source of every \
         factual or time-sensitive claim as a markdown link [title](url). Never \
         invent a URL or a source. If the results do not answer the question, \
         say what is missing.\n\n",
    );
    for result in &results {
        block.push_str(&format!(
            "--- {} ---\nURL: {}\n{}\n\n",
            result.title, result.url, result.snippet
        ));
    }
    messages.push(json!({ "role": "system", "content": block }));
    log::info!(
        "web context: injected {} results for query '{}'",
        results.len(),
        query
    );
}

#[cfg(test)]
mod tests {
    use super::normalize_query;

    #[test]
    fn collapses_whitespace() {
        assert_eq!(normalize_query("  umbrella   weather\n today "), "umbrella weather today");
    }

    #[test]
    fn caps_the_query_length() {
        let long = "x".repeat(500);
        assert_eq!(normalize_query(&long).chars().count(), 160);
    }

    #[test]
    fn empty_input_stays_empty() {
        assert!(normalize_query("   \n\t ").is_empty());
    }
}
