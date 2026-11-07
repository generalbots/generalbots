pub(crate) use super::kb_context_search;
pub(crate) use super::KbSearchResult;
pub(crate) use botcore::config::ConfigManager;
pub(crate) use botcore::kb::web_crawler::WebCrawler;
pub(crate) use botcore::shared::utils::DbPool;
pub(crate) use log::{debug, info};
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;

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
mod graph_search;
mod reciprocal_rank_fusion;

pub use from_label::*;
use graph_search::*;
use reciprocal_rank_fusion::*;
