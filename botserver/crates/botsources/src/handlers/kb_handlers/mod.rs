pub(crate) use crate::handlers::extractors::{chunk_text, compute_content_hash, estimate_tokens, extract_text_content};
pub(crate) use crate::models::{KnowledgeSourceRow, QueryResponse, QueryRequest as KbQueryRequest, QueryResult as KbQueryResult, SearchResultRow, SourceType, UploadResponse, ListSourcesQuery, ReindexRequest};
pub(crate) use crate::renderers::html_escape;
pub(crate) use crate::state::AppState;
pub(crate) use axum::{ extract::{Multipart, Path, Query, State}, response::{Html, IntoResponse}, Json, };
pub(crate) use diesel::prelude::*;
pub(crate) use log::{info, warn};
pub(crate) use std::fmt::Write;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;

mod resolve_bot_id;
mod handle_query_knowledge_base;
mod handle_get_stats;

pub use resolve_bot_id::*;
pub use handle_query_knowledge_base::*;
pub use handle_get_stats::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(branch: u128) -> BotScope {
        BotScope {
            id: Uuid::from_u128(10),
            branch_id: Uuid::from_u128(branch),
        }
    }

    #[test]
    fn same_tenant_bot_is_authorized() {
        let bot = scope(1);
        assert_eq!(
            authorize_bot_for_branch(bot, Some(bot.branch_id)),
            Some(bot.id)
        );
    }

    #[test]
    fn cross_tenant_bot_is_denied() {
        let other_branch = Uuid::from_u128(2);
        assert_eq!(authorize_bot_for_branch(scope(1), Some(other_branch)), None);
    }

    #[test]
    fn unresolved_branch_is_denied() {
        assert_eq!(authorize_bot_for_branch(scope(1), None), None);
    }

    #[test]
    fn missing_claims_never_touch_the_database() {
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://invalid:invalid@127.0.0.1:1/invalid",
        );
        let pool = diesel::r2d2::Pool::builder().build_unchecked(manager);
        assert_eq!(
            resolve_bot_id_for_branch(&pool, &axum::http::HeaderMap::new()),
            None
        );
    }
}
