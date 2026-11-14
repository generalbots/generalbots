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
use super::web_sources;

/// Eight hits instead of five: index pages are now removed before the prompt
/// (#2146), so a news turn must start with enough raw hits to keep 3–5
/// articles worth citing.
const MAX_RESULTS: usize = 8;

// #1835 — re-exported from turn_hooks so the classifier stays available even
// when the research/web-search features are disabled (the VM skip uses it).
pub use super::turn_hooks::is_informational_request;
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
///
/// Returns the sources as structured citations (`title`, `url`, `host`) so the
/// caller can attach them to the final answer frame (#1635). An empty answer
/// means the turn had no usable sources; the reason is already streamed as a
/// status line.
pub async fn search_and_inject(
    identity: &SessionIdentity<'_>,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) -> Vec<serde_json::Value> {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let query = normalize_query(user_text);
    if query.is_empty() {
        return Vec::new();
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
            status(
                sink,
                bot_uuid,
                session_id,
                user_id,
                "The search failed — answering from what I know.",
            )
            .await;
            inject_no_sources_notice(messages, "The live web search failed for this request.");
            return Vec::new();
        }
        Err(_) => {
            log::info!("web context: search timed out");
            status(
                sink,
                bot_uuid,
                session_id,
                user_id,
                "The search timed out — answering from what I know.",
            )
            .await;
            inject_no_sources_notice(messages, "The live web search timed out for this request.");
            return Vec::new();
        }
    };
    if results.is_empty() {
        status(
            sink,
            bot_uuid,
            session_id,
            user_id,
            "No usable sources — answering from what I know.",
        )
        .await;
        inject_no_sources_notice(
            messages,
            "The live web search returned no usable sources for this request.",
        );
        return Vec::new();
    }
    // Search engines still return removed articles and hub pages. A citation
    // the reader cannot open — or that leads to a daily-changing index — is
    // worse than no citation (TS-01/UAT-03), so dead URLs and section indexes
    // are dropped here and the survivors are probed for their publish date.
    let results = web_sources::refine(results).await;
    if results.is_empty() {
        status(
            sink,
            bot_uuid,
            session_id,
            user_id,
            "No usable sources — answering from what I know.",
        )
        .await;
        inject_no_sources_notice(
            messages,
            "The live web search returned no usable, dated article sources for this request.",
        );
        return Vec::new();
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

    let mut block = web_sources::prompt_block(&results);
    block.push_str(
        "\nCite the source of every factual or time-sensitive claim as a \
         markdown link [headline](article_url). Never invent a URL, an outlet or \
         a date.\n",
    );
    messages.push(json!({ "role": "system", "content": block }));
    log::info!(
        "web context: injected {} results for query '{}'",
        results.len(),
        query
    );

    // #1635 — the same sources that went into the prompt become the structured
    // citations of the answer, so the chips always describe what the model was
    // actually given (never an invented list).
    results.iter().map(citation_for).collect()
}

/// Maps one search result to the citation shape the chat renderer consumes
/// (`30_citations.js`): `{title, url, host, date}` plus the optional favicon.
fn citation_for(result: &botresearch::web_search::WebSearchResult) -> serde_json::Value {
    json!({
        "title": result.title,
        "url": result.url,
        "host": result.source,
        "favicon": result.favicon,
        // #2146 — the chip shows the outlet *and* the article date, so the
        // "every item needs a date" check survives the prose.
        "date": web_sources::item_date(result),
    })
}

/// #2146 — when the Web switch is on but no trustworthy sources survive,
/// the model must not answer from memory by inventing headlines, URLs or
/// dates. This instruction is injected as the freshest system message so the
/// visible reply is an explicit failure instead of fabricated news.
fn inject_no_sources_notice(messages: &mut Vec<serde_json::Value>, reason: &str) {
    messages.push(json!({
        "role": "system",
        "content": format!(
            "{reason}\n\n\
             WEB SOURCES UNAVAILABLE\n\
             - Tell the user clearly that current web sources could not be retrieved.\n\
             - Do NOT invent news items, headlines, outlets, publication dates or URLs.\n\
             - Do NOT cite or imply sources you do not have.\n\
             - Offer to try again or suggest a narrower query.\n\
             - Answer in the language of the user's message."
        ),
    }));
}

#[cfg(test)]
mod tests {
    use super::{citation_for, normalize_query};

    #[test]
    fn collapses_whitespace() {
        assert_eq!(
            normalize_query("  umbrella   weather\n today "),
            "umbrella weather today"
        );
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

    /// #1635 — the final frame must carry the exact url/host of the result that
    /// went into the prompt; the renderer keys the chip target off `url`.
    #[test]
    fn citation_carries_title_url_and_host() {
        let result = botresearch::web_search::WebSearchResult {
            title: "Umbrella - Wikipedia".to_string(),
            url: "https://en.wikipedia.org/wiki/Umbrella".to_string(),
            snippet: "An umbrella is a canopy device.".to_string(),
            source: "en.wikipedia.org".to_string(),
            favicon: Some("https://en.wikipedia.org/favicon.ico".to_string()),
            published_date: None,
        };
        let citation = citation_for(&result);
        assert_eq!(citation["title"], "Umbrella - Wikipedia");
        assert_eq!(citation["url"], "https://en.wikipedia.org/wiki/Umbrella");
        assert_eq!(citation["host"], "en.wikipedia.org");
        assert_eq!(citation["favicon"], "https://en.wikipedia.org/favicon.ico");
        // #2146 — a hit without a date carries an explicit null, never an
        // invented date.
        assert!(citation["date"].is_null());
    }

    /// #2146 — the chip shape carries the article date the uat asked for.
    #[test]
    fn citation_carries_the_article_date() {
        let result = botresearch::web_search::WebSearchResult {
            title: "Lula recebe ministros".to_string(),
            url: "https://g1.globo.com/politica/noticia/2026/11/12/lula-fala.ghtml".to_string(),
            snippet: String::new(),
            source: "g1.globo.com".to_string(),
            favicon: None,
            published_date: Some("2026-11-12T09:10:00-03:00".to_string()),
        };
        assert_eq!(citation_for(&result)["date"], "2026-11-12");
    }
}
