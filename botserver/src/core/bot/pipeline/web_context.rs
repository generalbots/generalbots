//! #1547 — fresh web context for answers that must cite sources.
//!
//! The pipeline has no tool round-trip after the model answers, so search runs
//! BEFORE the LLM call: results go in as a system message and the model is
//! instructed to cite them as markdown links.

use serde_json::json;

const MAX_RESULTS: usize = 5;
const REGION: &str = "wt-wt";

/// Questions and time-sensitive asks benefit from a search; commands,
/// document work and short chatter do not.
fn should_search(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    if lower.chars().count() < 8 {
        return false;
    }
    let command_starts = [
        "faz", "faça", "crie", "cria", "gere", "gera", "salve", "use ", "call ",
        "executa", "rode", "abra", "abre", "edite", "corrija", "analise", "analisa",
        "resuma", "traduza", "crie", "monte", "desenhe",
    ];
    if command_starts.iter().any(|c| lower.starts_with(c)) && !lower.contains('?') {
        return false;
    }
    let cues = [
        "?", "quem", "qual", "quais", "quando", "onde", "como", "por que", "porque",
        "quanto", "what", "who", "when", "where", "why", "how", "which", "latest",
        "current", "hoje", "agora", "notícia", "noticia", "news", "cotação",
        "cotacao", "preço", "preco", "temperatura", "clima", "weather", "população",
        "populacao", "eleição", "eleicao",
    ];
    cues.iter().any(|cue| lower.contains(cue))
}

pub async fn inject(user_text: &str, messages: &mut Vec<serde_json::Value>) {
    let wanted = should_search(user_text);
    log::info!("web context: evaluate '{}' -> {}", botlib::text::truncate_with_suffix(user_text, 80, "…"), wanted);
    if !wanted {
        return;
    }
    let search = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        botresearch::web_search::search_web(user_text, MAX_RESULTS, REGION),
    )
    .await;
    let results = match search {
        Ok(Ok(results)) => results,
        Ok(Err(e)) => {
            log::info!("web context: search failed: {e}");
            return;
        }
        Err(_) => {
            log::info!("web context: search timed out");
            return;
        }
    };
    if results.is_empty() {
        log::info!("web context: no results");
        return;
    }
    let mut block = String::from(
        "Fresh web search results are provided below. They are sufficient for \
         this turn: answer the user NOW from this material. Do not call \
         api.find, web.search or any other command to look for a search tool — \
         the search has already been done for you. Cite the source of every \
         factual or time-sensitive claim as a markdown link [title](url). Never \
         invent a URL or a source. If the results do not answer the question, \
         say what is missing.\n\n",
    );
    for (index, result) in results.iter().enumerate() {
        block.push_str(&format!(
            "{}. [{}]({})\n{}\n\n",
            index + 1,
            result.title,
            result.url,
            result.snippet
        ));
    }
    messages.push(json!({ "role": "system", "content": block }));
    log::info!("web context: injected {} results", results.len());
}
