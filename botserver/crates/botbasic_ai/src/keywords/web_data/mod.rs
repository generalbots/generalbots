pub(crate) use botbasic_types::UserSession;
pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use log::{debug, trace};
pub(crate) use rhai::{Array, Dynamic, Engine, EvalAltResult, Map, Position};
pub(crate) use scraper::{Html, Selector};

use reqwest::Url;
use std::sync::Arc;
use std::time::Duration;

// #2131 — every URL these keywords fetch comes from the script/LLM, so they go
// through the shared SSRF guard (public http(s) only, per-hop redirect checks).
pub(crate) use botsecurity::validation::{fetch_user_url, FetchPolicy};

mod static_selector;
mod register_scrape_images_keyword;

pub use static_selector::*;
use register_scrape_images_keyword::*;
