//! Hybrid Search Module for RAG 2.0
//!
//! Implements hybrid search combining sparse (BM25) and dense (embedding) retrieval
//! with Reciprocal Rank Fusion (RRF) for optimal results.
//!
//! Config.csv properties:
//! ```csv
//! rag-hybrid-enabled,true
//! rag-dense-weight,0.7
//! rag-sparse-weight,0.3
//! rag-reranker-enabled,true
//! rag-reranker-model,cross-encoder/ms-marco-MiniLM-L-6-v2
//! ```

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

/// Configuration for hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    /// Weight for dense (embedding) search results (0.0 - 1.0)
    pub dense_weight: f32,
    /// Weight for sparse (BM25) search results (0.0 - 1.0)
    pub sparse_weight: f32,
    /// Whether to use reranker for final results
    pub reranker_enabled: bool,
    /// Reranker model name/path
    pub reranker_model: String,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Minimum score threshold (0.0 - 1.0)
    pub min_score: f32,
    /// K parameter for RRF (typically 60)
    pub rrf_k: u32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            dense_weight: 0.7,
            sparse_weight: 0.3,
            reranker_enabled: false,
            reranker_model: "cross-encoder/ms-marco-MiniLM-L-6-v2".to_string(),
            max_results: 10,
            min_score: 0.0,
            rrf_k: 60,
        }
    }
}

impl HybridSearchConfig {
    /// Load config from bot configuration
    pub fn from_bot_config(state: &AppState, bot_id: Uuid) -> Self {
        use diesel::prelude::*;

        let mut config = Self::default();

        if let Ok(mut conn) = state.conn.get() {
            #[derive(QueryableByName)]
            struct ConfigRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_key: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                config_value: String,
            }

            let configs: Vec<ConfigRow> = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration \
                 WHERE bot_id = $1 AND config_key LIKE 'rag-%'",
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load(&mut conn)
            .unwrap_or_default();

            for row in configs {
                match row.config_key.as_str() {
                    "rag-dense-weight" => {
                        config.dense_weight = row.config_value.parse().unwrap_or(0.7);
                    }
                    "rag-sparse-weight" => {
                        config.sparse_weight = row.config_value.parse().unwrap_or(0.3);
                    }
                    "rag-reranker-enabled" => {
                        config.reranker_enabled = row.config_value.to_lowercase() == "true";
                    }
                    "rag-reranker-model" => {
                        config.reranker_model = row.config_value;
                    }
                    "rag-max-results" => {
                        config.max_results = row.config_value.parse().unwrap_or(10);
                    }
                    "rag-min-score" => {
                        config.min_score = row.config_value.parse().unwrap_or(0.0);
                    }
                    "rag-rrf-k" => {
                        config.rrf_k = row.config_value.parse().unwrap_or(60);
                    }
                    _ => {}
                }
            }
        }

        // Normalize weights
        let total = config.dense_weight + config.sparse_weight;
        if total > 0.0 {
            config.dense_weight /= total;
            config.sparse_weight /= total;
        }

        config
    }
}

/// Search result from any retrieval method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique document identifier
    pub doc_id: String,
    /// Document content
    pub content: String,
    /// Source file/email/etc path
    pub source: String,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Search method that produced this result
    pub search_method: SearchMethod,
}

/// Search method used to retrieve a result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchMethod {
    Dense,
    Sparse,
    Hybrid,
    Reranked,
}

/// BM25 search index for sparse retrieval
pub struct BM25Index {
    /// Document frequency for each term
    doc_freq: HashMap<String, usize>,
    /// Total number of documents
    doc_count: usize,
    /// Average document length
    avg_doc_len: f32,
    /// Document lengths
    doc_lengths: HashMap<String, usize>,
    /// Term frequencies per document
    term_freqs: HashMap<String, HashMap<String, usize>>,
    /// BM25 parameters
    k1: f32,
    b: f32,
}

impl BM25Index {
    pub fn new() -> Self {
        Self {
            doc_freq: HashMap::new(),
            doc_count: 0,
            avg_doc_len: 0.0,
            doc_lengths: HashMap::new(),
            term_freqs: HashMap::new(),
            k1: 1.2,
            b: 0.75,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, doc_id: &str, content: &str) {
        let terms = self.tokenize(content);
        let doc_len = terms.len();

        // Update document length
        self.doc_lengths.insert(doc_id.to_string(), doc_len);

        // Calculate term frequencies
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        let mut seen_terms: std::collections::HashSet<String> = std::collections::HashSet::new();

        for term in &terms {
            *term_freq.entry(term.clone()).or_insert(0) += 1;

            // Update document frequency (only once per document per term)
            if !seen_terms.contains(term) {
                *self.doc_freq.entry(term.clone()).or_insert(0) += 1;
                seen_terms.insert(term.clone());
            }
        }

        self.term_freqs.insert(doc_id.to_string(), term_freq);
        self.doc_count += 1;

        // Update average document length
        let total_len: usize = self.doc_lengths.values().sum();
        self.avg_doc_len = total_len as f32 / self.doc_count as f32;
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: &str) {
        if let Some(term_freq) = self.term_freqs.remove(doc_id) {
            // Update document frequencies
            for term in term_freq.keys() {
                if let Some(freq) = self.doc_freq.get_mut(term) {
                    *freq = freq.saturating_sub(1);
                    if *freq == 0 {
                        self.doc_freq.remove(term);
                    }
                }
            }
        }

        self.doc_lengths.remove(doc_id);
        self.doc_count = self.doc_count.saturating_sub(1);

        // Update average document length
        if self.doc_count > 0 {
            let total_len: usize = self.doc_lengths.values().sum();
            self.avg_doc_len = total_len as f32 / self.doc_count as f32;
        } else {
            self.avg_doc_len = 0.0;
        }
    }

    /// Search the index with BM25 scoring
    pub fn search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        let query_terms = self.tokenize(query);
        let mut scores: HashMap<String, f32> = HashMap::new();

        for term in &query_terms {
            let df = *self.doc_freq.get(term).unwrap_or(&0);
            if df == 0 {
                continue;
            }

            // IDF calculation
            let idf = ((self.doc_count as f32 - df as f32 + 0.5) / (df as f32 + 0.5) + 1.0).ln();

            for (doc_id, term_freqs) in &self.term_freqs {
                if let Some(&tf) = term_freqs.get(term) {
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&1) as f32;
                    let tf_normalized = (tf as f32 * (self.k1 + 1.0))
                        / (tf as f32
                            + self.k1 * (1.0 - self.b + self.b * (doc_len / self.avg_doc_len)));

                    *scores.entry(doc_id.clone()).or_insert(0.0) += idf * tf_normalized;
                }
            }
        }

        // Sort by score and return top results
        let mut results: Vec<(String, f32)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);

        results
    }

    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 2) // Filter out very short tokens
            .map(|s| s.to_string())
            .collect()
    }

    /// Get index statistics
    pub fn stats(&self) -> BM25Stats {
        BM25Stats {
            doc_count: self.doc_count,
            unique_terms: self.doc_freq.len(),
            avg_doc_len: self.avg_doc_len,
        }
    }
}

impl Default for BM25Index {
    fn default() -> Self {
        Self::new()
    }
}

/// BM25 index statistics
#[derive(Debug, Clone)]
pub struct BM25Stats {
    pub doc_count: usize,
    pub unique_terms: usize,
    pub avg_doc_len: f32,
}

/// Hybrid search engine combining dense and sparse retrieval
pub struct HybridSearchEngine {
    /// BM25 sparse index
    bm25_index: BM25Index,
    /// Document store for content retrieval
    documents: HashMap<String, DocumentEntry>,
    /// Configuration
    config: HybridSearchConfig,
    /// Qdrant URL for dense search
    qdrant_url: String,
    /// Collection name
    collection_name: String,
}

/// Document entry in the store
#[derive(Debug, Clone)]
struct DocumentEntry {
    pub content: String,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

impl HybridSearchEngine {
    pub fn new(config: HybridSearchConfig, qdrant_url: &str, collection_name: &str) -> Self {
        Self {
            bm25_index: BM25Index::new(),
            documents: HashMap::new(),
            config,
            qdrant_url: qdrant_url.to_string(),
            collection_name: collection_name.to_string(),
        }
    }

    /// Index a document for both dense and sparse search
    pub async fn index_document(
        &mut self,
        doc_id: &str,
        content: &str,
        source: &str,
        metadata: HashMap<String, String>,
        embedding: Option<Vec<f32>>,
    ) -> Result<(), String> {
        // Add to BM25 index
        self.bm25_index.add_document(doc_id, content);

        // Store document
        self.documents.insert(
            doc_id.to_string(),
            DocumentEntry {
                content: content.to_string(),
                source: source.to_string(),
                metadata,
            },
        );

        // If embedding provided, add to Qdrant
        if let Some(emb) = embedding {
            self.upsert_to_qdrant(doc_id, &emb).await?;
        }

        Ok(())
    }

    /// Remove a document from all indexes
    pub async fn remove_document(&mut self, doc_id: &str) -> Result<(), String> {
        self.bm25_index.remove_document(doc_id);
        self.documents.remove(doc_id);
        self.delete_from_qdrant(doc_id).await?;
        Ok(())
    }

    /// Perform hybrid search
    pub async fn search(
        &self,
        query: &str,
        query_embedding: Option<Vec<f32>>,
    ) -> Result<Vec<SearchResult>, String> {
        let fetch_count = self.config.max_results * 3; // Fetch more for fusion

        // Sparse search (BM25)
        let sparse_results = self.bm25_index.search(query, fetch_count);
        trace!(
            "BM25 search returned {} results for query: {}",
            sparse_results.len(),
            query
        );

        // Dense search (Qdrant)
        let dense_results = if let Some(embedding) = query_embedding {
            self.search_qdrant(&embedding, fetch_count).await?
        } else {
            Vec::new()
        };
        trace!(
            "Dense search returned {} results for query: {}",
            dense_results.len(),
            query
        );

        // Reciprocal Rank Fusion
        let fused_results = self.reciprocal_rank_fusion(&sparse_results, &dense_results);
        trace!("RRF produced {} fused results", fused_results.len());

        // Convert to SearchResult
        let mut results: Vec<SearchResult> = fused_results
            .into_iter()
            .filter_map(|(doc_id, score)| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    doc_id,
                    content: doc.content.clone(),
                    source: doc.source.clone(),
                    score,
                    metadata: doc.metadata.clone(),
                    search_method: SearchMethod::Hybrid,
                })
            })
            .filter(|r| r.score >= self.config.min_score)
            .take(self.config.max_results)
            .collect();

        // Optional reranking
        if self.config.reranker_enabled && !results.is_empty() {
            results = self.rerank(query, results).await?;
        }

        Ok(results)
    }

    /// Perform only sparse (BM25) search
    pub fn sparse_search(&self, query: &str) -> Vec<SearchResult> {
        let results = self.bm25_index.search(query, self.config.max_results);

        results
            .into_iter()
            .filter_map(|(doc_id, score)| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    doc_id,
                    content: doc.content.clone(),
                    source: doc.source.clone(),
                    score,
                    metadata: doc.metadata.clone(),
                    search_method: SearchMethod::Sparse,
                })
            })
            .collect()
    }

    /// Perform only dense (embedding) search
    pub async fn dense_search(
        &self,
        query_embedding: Vec<f32>,
    ) -> Result<Vec<SearchResult>, String> {
        let results = self
            .search_qdrant(&query_embedding, self.config.max_results)
            .await?;

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .filter_map(|(doc_id, score)| {
                self.documents.get(&doc_id).map(|doc| SearchResult {
                    doc_id,
                    content: doc.content.clone(),
                    source: doc.source.clone(),
                    score,
                    metadata: doc.metadata.clone(),
                    search_method: SearchMethod::Dense,
                })
            })
            .collect();

        Ok(search_results)
    }

    /// Reciprocal Rank Fusion algorithm
    fn reciprocal_rank_fusion(
        &self,
        sparse: &[(String, f32)],
        dense: &[(String, f32)],
    ) -> Vec<(String, f32)> {
        let k = self.config.rrf_k as f32;
        let mut scores: HashMap<String, f32> = HashMap::new();

        // Score from sparse results
        for (rank, (doc_id, _)) in sparse.iter().enumerate() {
            let rrf_score = self.config.sparse_weight / (k + rank as f32 + 1.0);
            *scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
        }

        // Score from dense results
        for (rank, (doc_id, _)) in dense.iter().enumerate() {
            let rrf_score = self.config.dense_weight / (k + rank as f32 + 1.0);
            *scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
        }

        // Sort by combined score
        let mut results: Vec<(String, f32)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Normalize scores to 0-1 range
        if let Some((_, max_score)) = results.first() {
            if *max_score > 0.0 {
                for (_, score) in &mut results {
                    *score /= max_score;
                }
            }
        }

        results
    }

    /// Rerank results using cross-encoder model
    async fn rerank(
        &self,
        query: &str,
        results: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, String> {
        // In a full implementation, this would call a cross-encoder model
        // For now, we'll use a simple relevance heuristic
        let mut reranked = results;

        for result in &mut reranked {
            // Simple reranking based on query term overlap
            let query_terms: std::collections::HashSet<&str> =
                query.to_lowercase().split_whitespace().collect();
            let content_lower = result.content.to_lowercase();

            let mut overlap_score = 0.0;
            for term in &query_terms {
                if content_lower.contains(term) {
                    overlap_score += 1.0;
                }
            }

            // Combine original score with overlap
            let overlap_normalized = overlap_score / query_terms.len().max(1) as f32;
            result.score = result.score * 0.7 + overlap_normalized * 0.3;
            result.search_method = SearchMethod::Reranked;
        }

        // Re-sort by new scores
        reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(reranked)
    }

    /// Search Qdrant for similar vectors
    async fn search_qdrant(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>, String> {
        let client = reqwest::Client::new();

        let search_request = serde_json::json!({
            "vector": embedding,
            "limit": limit,
            "with_payload": false
        });

        let response = client
            .post(&format!(
                "{}/collections/{}/points/search",
                self.qdrant_url, self.collection_name
            ))
            .json(&search_request)
            .send()
            .await
            .map_err(|e| format!("Qdrant search failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Qdrant search error: {}", error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Qdrant response: {}", e))?;

        let points = result["result"]
            .as_array()
            .ok_or("Invalid Qdrant response format")?;

        let results: Vec<(String, f32)> = points
            .iter()
            .filter_map(|p| {
                let id = p["id"].as_str().map(|s| s.to_string())?;
                let score = p["score"].as_f64()? as f32;
                Some((id, score))
            })
            .collect();

        Ok(results)
    }

    /// Upsert vector to Qdrant
    async fn upsert_to_qdrant(&self, doc_id: &str, embedding: &[f32]) -> Result<(), String> {
        let client = reqwest::Client::new();

        let upsert_request = serde_json::json!({
            "points": [{
                "id": doc_id,
                "vector": embedding
            }]
        });

        let response = client
            .put(&format!(
                "{}/collections/{}/points",
                self.qdrant_url, self.collection_name
            ))
            .json(&upsert_request)
            .send()
            .await
            .map_err(|e| format!("Qdrant upsert failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Qdrant upsert error: {}", error_text));
        }

        Ok(())
    }

    /// Delete vector from Qdrant
    async fn delete_from_qdrant(&self, doc_id: &str) -> Result<(), String> {
        let client = reqwest::Client::new();

        let delete_request = serde_json::json!({
            "points": [doc_id]
        });

        let response = client
            .post(&format!(
                "{}/collections/{}/points/delete",
                self.qdrant_url, self.collection_name
            ))
            .json(&delete_request)
            .send()
            .await
            .map_err(|e| format!("Qdrant delete failed: {}", e))?;

        if !response.status().is_success() {
            warn!(
                "Qdrant delete may have failed for {}: {}",
                doc_id,
                response.status()
            );
        }

        Ok(())
    }

    /// Get engine statistics
    pub fn stats(&self) -> HybridSearchStats {
        let bm25_stats = self.bm25_index.stats();

        HybridSearchStats {
            total_documents: self.documents.len(),
            bm25_doc_count: bm25_stats.doc_count,
            unique_terms: bm25_stats.unique_terms,
            avg_doc_len: bm25_stats.avg_doc_len,
            config: self.config.clone(),
        }
    }
}

/// Hybrid search engine statistics
#[derive(Debug, Clone)]
pub struct HybridSearchStats {
    pub total_documents: usize,
    pub bm25_doc_count: usize,
    pub unique_terms: usize,
    pub avg_doc_len: f32,
    pub config: HybridSearchConfig,
}

/// Query decomposition for complex questions
pub struct QueryDecomposer {
    llm_endpoint: String,
    api_key: String,
}

impl QueryDecomposer {
    pub fn new(llm_endpoint: &str, api_key: &str) -> Self {
        Self {
            llm_endpoint: llm_endpoint.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Decompose a complex query into simpler sub-queries
    pub async fn decompose(&self, query: &str) -> Result<Vec<String>, String> {
        // Simple heuristic decomposition for common patterns
        // A full implementation would use an LLM

        let mut sub_queries = Vec::new();

        // Check for conjunctions
        let conjunctions = ["and", "also", "as well as", "in addition to"];
        let mut parts: Vec<&str> = vec![query];

        for conj in &conjunctions {
            parts = parts
                .iter()
                .flat_map(|p| p.split(conj))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
        }

        if parts.len() > 1 {
            for part in parts {
                sub_queries.push(part.to_string());
            }
        } else {
            // Try question word splitting
            let question_words = ["what", "how", "why", "when", "where", "who"];
            let lower = query.to_lowercase();

            let mut has_multiple_questions = false;
            for qw in &question_words {
                if lower.matches(qw).count() > 1 {
                    has_multiple_questions = true;
                    break;
                }
            }

            if has_multiple_questions {
                // Split on question marks or question words
                for part in query.split('?') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        sub_queries.push(format!("{}?", trimmed));
                    }
                }
            }
        }

        // If no decomposition happened, return original query
        if sub_queries.is_empty() {
            sub_queries.push(query.to_string());
        }

        Ok(sub_queries)
    }

    /// Synthesize answers from multiple sub-query results
    pub fn synthesize(&self, query: &str, sub_answers: &[String]) -> String {
        if sub_answers.len() == 1 {
            return sub_answers[0].clone();
        }

        // Simple concatenation with context
        let mut synthesis = format!("Based on your question about \"{}\", here's what I found:\n\n", query);

        for (i, answer) in sub_answers.iter().enumerate() {
            synthesis.push_str(&format!("{}. {}\n\n", i + 1, answer));
        }

        synthesis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_index_basic() {
        let mut index = BM25Index::new();

        index.add_document("doc1", "The quick brown fox jumps over the lazy dog");
        index.add_document("doc2", "A quick brown dog runs in the park");
        index.add_document("doc3", "The lazy cat sleeps all day");

        let stats = index.stats();
        assert_eq!(stats.doc_count, 3);
        assert!(stats.avg_doc_len > 0.0);
    }

    #[test]
    fn test_bm25_search() {
        let mut index = BM25Index::new();

        index.add_document("doc1", "machine learning artificial intelligence");
        index.add_document("doc2", "natural language processing NLP");
        index.add_document("doc3", "computer vision image recognition");

        let results = index.search("machine learning", 10);

        assert!(!results.is_empty());
        assert_eq!(results[0].0, "doc1"); // doc1 should be first
    }

    #[test]
    fn test_bm25_remove_document() {
        let mut index = BM25Index::new();

        index.add_document("doc1", "test document one");
        index.add_document("doc2", "test document two");

        assert_eq!(index.stats().doc_count, 2);

        index.remove_document("doc1");

        assert_eq!(index.stats().doc_count, 1);

        let results = index.search("one", 10);
        assert!(results.is_empty() || results[0].0 != "doc1");
    }

    #[test]
    fn test_hybrid_config_default() {
        let config = HybridSearchConfig::default();

        assert_eq!(config.dense_weight, 0.7);
        assert_eq!(config.sparse_weight, 0.3);
        assert!(!config.reranker_enabled);
        assert_eq!(config.max_results, 10);
    }

    #[test]
    fn test_reciprocal_rank_fusion() {
        let config = HybridSearchConfig::default();
        let engine = HybridSearchEngine::new(config, "http://localhost:6333", "test");

        let sparse = vec![
            ("doc1".to_string(), 0.9),
            ("doc2".to_string(), 0.7),
            ("doc3".to_string(), 0.5),
        ];

        let dense = vec![
            ("doc2".to_string(), 0.95),
            ("doc1".to_string(), 0.8),
            ("doc4".to_string(), 0.6),
        ];

        let fused = engine.reciprocal_rank_fusion(&sparse, &dense);

        // doc1 and doc2 should be in top results as they appear in both
        assert!(!fused.is_empty());
        let top_ids: Vec<&str> = fused.iter().take(2).map(|(id, _)| id.as_str()).collect();
        assert!(top_ids.contains(&"doc1") || top_ids.contains(&"doc2"));
    }

    #[test]
    fn test_query_decomposer_simple() {
        let decomposer = QueryDecomposer::new("http://localhost:8081", "none");

        // Use tokio runtime for async test
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async {
            decomposer
                .decompose("What is machine learning and how does it work?")
                .await
        });

        assert!(result.is_ok());
        let queries = result.unwrap();
        assert!(!queries.is_empty());
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            doc_id: "test123".to_string(),
            content: "Test content".to_string(),
            source: "/path/to/file".to_string(),
            score: 0.85,
            metadata: HashMap::new(),
            search_method: SearchMethod::Hybrid,
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());

        let parsed: Result<SearchResult, _> = serde_json::from_str(&json.unwrap());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().doc_id, "test123");
    }
}
