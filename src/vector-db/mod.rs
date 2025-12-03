//! Vector Database Module for RAG 2.0
//!
//! This module provides hybrid search capabilities combining:
//! - **Sparse Search (BM25)**: Powered by Tantivy when `vectordb` feature is enabled
//! - **Dense Search**: Uses Qdrant for embedding-based similarity search
//! - **Hybrid Fusion**: Reciprocal Rank Fusion (RRF) combines both methods
//!
//! # Features
//!
//! Enable the `vectordb` feature in Cargo.toml to use Tantivy-based BM25:
//! ```toml
//! [features]
//! vectordb = ["dep:qdrant-client", "dep:tantivy"]
//! ```
//!
//! # Configuration
//!
//! Configure via config.csv:
//! ```csv
//! # Enable/disable BM25 sparse search
//! bm25-enabled,true
//! bm25-k1,1.2
//! bm25-b,0.75
//!
//! # Hybrid search weights
//! rag-dense-weight,0.7
//! rag-sparse-weight,0.3
//! ```

pub mod bm25_config;
pub mod hybrid_search;
pub mod vectordb_indexer;

// BM25 Configuration exports
pub use bm25_config::{is_stopword, Bm25Config, DEFAULT_STOPWORDS};

// Hybrid Search exports
pub use hybrid_search::{
    BM25Stats, HybridSearchConfig, HybridSearchEngine, HybridSearchStats, QueryDecomposer,
    SearchMethod, SearchResult,
};

// Tantivy BM25 index (when vectordb feature enabled)
#[cfg(feature = "vectordb")]
pub use hybrid_search::TantivyBM25Index;

// Fallback BM25 index (when vectordb feature NOT enabled)
#[cfg(not(feature = "vectordb"))]
pub use hybrid_search::BM25Index;

// VectorDB Indexer exports
pub use vectordb_indexer::{IndexingStats, IndexingStatus, VectorDBIndexer};
