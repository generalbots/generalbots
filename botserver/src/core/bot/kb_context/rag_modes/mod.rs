pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::Arc;
pub(crate) use log::{debug, info};
pub(crate) use uuid::Uuid;
pub(crate) use botcore::config::ConfigManager;
pub(crate) use botcore::kb::web_crawler::WebCrawler;
pub(crate) use botcore::shared::utils::DbPool;
pub(crate) use super::kb_context_search;
pub(crate) use super::KbSearchResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RagMode {
    Standard,
    Hybrid,
    Corrective,
    Graph,
    Agentic,
    Multimodal,
}

const RRF_K: f64 = 60.0;

mod from_label;
mod reciprocal_rank_fusion;
mod graph_search;

pub use from_label::*;
use reciprocal_rank_fusion::*;
use graph_search::*;
