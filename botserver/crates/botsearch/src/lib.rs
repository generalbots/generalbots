pub mod handlers;
pub mod routes;
pub mod search;

pub use routes::configure_search_routes;
pub use search::{
    create_search_index_migration, DocumentToIndex, IndexResult, IndexStats, SearchConfig,
    SearchError, SearchQuery, SearchResponse, SearchResult, SearchService, SearchSource,
};
