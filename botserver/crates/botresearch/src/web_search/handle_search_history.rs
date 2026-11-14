// Web-search HTTP handlers and the provider chain (DuckDuckGo, Wikipedia,
// Mojeek, TinyFish, Google News). Split by responsibility (#1730); the public
// handlers and the provider entry points keep their paths through the
// re-exports below.

mod duckduckgo;
mod google_news;
mod history;
mod instant;
mod mojeek;
mod provider_chain;
mod query;
mod tinyfish;
mod wikipedia;

pub use history::handle_search_history;
pub use instant::handle_instant_answer;
pub use provider_chain::{search_web, search_web_with_source, WebSearchOutcome};
pub use query::clean_query;
