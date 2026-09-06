use anyhow::Result;
use log::{debug, info, warn};
use uuid::Uuid;

use botcore::shared::utils::DbPool;
#[cfg(any(feature = "research", feature = "llm"))]
use botcore::kb::embedding_generator::{EmbeddingConfig, KbEmbeddingGenerator};

use super::KbSearchResult;

fn get_vectordb_url() -> String {
    std::env::var("QDRANT_URL")
        .ok()
        .or_else(|| std::env::var("VECTORDB_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:6333".to_string())
}

fn get_vectordb_api_key() -> String {
    std::env::var("QDRANT_API_KEY")
        .ok()
        .or_else(|| std::env::var("VECTORDB_API_KEY").ok())
        .unwrap_or_default()
}

#[cfg(any(feature = "research", feature = "llm"))]
async fn generate_query_embedding(query: &str, bot_id: Uuid, db_pool: &DbPool) -> Option<Vec<f32>> {
    let config = EmbeddingConfig::from_bot_config(db_pool, &bot_id);

    if config.embedding_url.is_empty() {
        debug!("No embedding URL configured for bot {}, using Qdrant with hash fallback", bot_id);
        return None;
    }

    let generator = KbEmbeddingGenerator::new(config);
    match generator.generate_single_embedding(query).await {
        Ok(embedding) => {
            info!("Generated real embedding for query ({} dims)", embedding.vector.len());
            Some(embedding.vector)
        }
        Err(e) => {
            warn!("Failed to generate real embedding for query: {}, using hash fallback", e);
            None
        }
    }
}

#[cfg(any(feature = "research", feature = "llm"))]
fn generate_hash_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut embedding = Vec::with_capacity(dimensions);
    for i in 0..dimensions {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        i.hash(&mut hasher);
        let hash = hasher.finish();
        embedding.push((hash as f32) / (u64::MAX as f32) * 2.0 - 1.0);
    }
    embedding
}

#[cfg(any(feature = "research", feature = "llm"))]
pub async fn search_qdrant(
    collection_name: &str,
    query: &str,
    limit: usize,
    bot_id: Uuid,
    db_pool: &DbPool,
) -> Result<Vec<KbSearchResult>> {
    let qdrant_url = get_vectordb_url();
    let api_key = get_vectordb_api_key();

    let search_url = format!("{}/collections/{}/points/search", qdrant_url.trim_end_matches('/'), collection_name);

    let client = botlib::security::create_tls_client(None);

    let check_url = format!("{}/collections/{}", qdrant_url.trim_end_matches('/'), collection_name);
    let dim = {
        let resp = client.get(&check_url)
            .header("api-key", &api_key)
            .send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let info: serde_json::Value = r.json().await.unwrap_or_default();
                info["result"]["config"]["params"]["vectors"]["size"].as_u64().map(|d| d as usize)
            }
            _ => None,
        }
    };

    let dimensions = dim.unwrap_or(384);

    let use_real_embedding = matches!(generate_query_embedding(query, bot_id, db_pool).await, Some(ref v) if v.len() == dimensions);

    if !use_real_embedding {
        info!("No embedding model configured for KB '{}', using keyword search fallback", collection_name);
        return search_qdrant_by_keyword(&client, &qdrant_url, collection_name, query, limit, &api_key).await;
    }

    let vector = generate_query_embedding(query, bot_id, db_pool).await
        .unwrap_or_else(|| generate_hash_embedding(query, dimensions));

    let mut request = client
        .post(&search_url)
        .json(&serde_json::json!({
            "limit": limit,
            "with_vector": false,
            "with_payload": true,
            "params": {
                "hnsw_ef": 128,
                "exact": false
            },
            "vector": vector
        }));

    if !api_key.is_empty() {
        request = request.header("api-key", &api_key);
    }

    let response = request.send().await?;
    let status = response.status();

    if status == 404 {
        debug!("Qdrant collection '{}' not found, skipping", collection_name);
        return Ok(Vec::new());
    }

    if !status.is_success() {
        warn!("Qdrant search failed for '{}': status={}", collection_name, status);
        return Ok(Vec::new());
    }

    let result: serde_json::Value = response.json().await?;

    let points = result["result"]
        .as_array()
        .map(|a| a.to_vec())
        .unwrap_or_default();

    if points.is_empty() {
        debug!("No points found in Qdrant collection '{}'", collection_name);
        return Ok(Vec::new());
    }

    let mut search_results = Vec::new();
    for point in &points {
        let score = point.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

        if score < 0.20 {
            continue;
        }

        let payload = point.get("payload");
        let content = payload
            .and_then(|p| p.get("content").or_else(|| p.get("text")).or_else(|| p.get("data")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let document_path = payload
            .and_then(|p| p.get("document_path").or_else(|| p.get("source")).or_else(|| p.get("file")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if content.is_empty() || content.len() < 20 {
            continue;
        }

        search_results.push(KbSearchResult {
            content,
            document_path,
            score,
        });
    }

    Ok(search_results)
}

#[cfg(any(feature = "research", feature = "llm"))]
async fn search_qdrant_by_keyword(
    client: &reqwest::Client,
    qdrant_url: &str,
    collection_name: &str,
    query: &str,
    limit: usize,
    api_key: &str,
) -> Result<Vec<KbSearchResult>> {
    let scroll_url = format!("{}/collections/{}/points/scroll", qdrant_url.trim_end_matches('/'), collection_name);

    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace()
        .filter(|t| t.len() > 2)
        .collect();

    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_points = Vec::new();
    let mut offset: Option<serde_json::Value> = None;

    loop {
        let body = if let Some(ref off) = offset {
            serde_json::json!({
                "limit": 100,
                "with_vector": false,
                "with_payload": true,
                "offset": off
            })
        } else {
            serde_json::json!({
                "limit": 100,
                "with_vector": false,
                "with_payload": true
            })
        };

        let mut request = client.post(&scroll_url).json(&body);
        if !api_key.is_empty() {
            request = request.header("api-key", api_key);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("Qdrant scroll failed for '{}': status={}", collection_name, response.status());
            break;
        }

        let result: serde_json::Value = response.json().await?;
        let points = result["result"]["points"].as_array().cloned().unwrap_or_default();
        let next_offset = result["result"]["next_page_offset"].clone();

        for point in points {
            let payload = point.get("payload").cloned().unwrap_or_default();
            let content = payload
                .get("content").or_else(|| payload.get("text")).or_else(|| payload.get("data"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if content.is_empty() || content.len() < 20 {
                continue;
            }

            let content_lower = content.to_lowercase();
            let match_score = query_terms.iter()
                .filter(|&&term| content_lower.contains(term))
                .count() as f32 / query_terms.len() as f32;

            if match_score > 0.0 {
                let document_path = payload
                    .get("document_path").or_else(|| payload.get("source")).or_else(|| payload.get("file"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                all_points.push(KbSearchResult {
                    content: content.to_string(),
                    document_path,
                    score: match_score,
                });
            }
        }

        if next_offset.is_null() || all_points.len() >= limit * 3 {
            break;
        }
        offset = Some(next_offset);
    }

    all_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    all_points.truncate(limit);

    info!("Keyword search for '{}' in '{}': {} results", query, collection_name, all_points.len());
    Ok(all_points)
}

#[cfg(any(feature = "research", feature = "llm"))]
pub async fn search_keyword_only(
    collection_name: &str,
    query: &str,
    limit: usize,
) -> Vec<KbSearchResult> {
    let client = botlib::security::create_tls_client(None);
    let qdrant_url = get_vectordb_url();
    let api_key = get_vectordb_api_key();
    search_qdrant_by_keyword(&client, &qdrant_url, collection_name, query, limit, &api_key)
        .await
        .unwrap_or_default()
}

#[cfg(not(any(feature = "research", feature = "llm")))]
pub async fn search_keyword_only(
    _collection_name: &str,
    _query: &str,
    _limit: usize,
) -> Vec<KbSearchResult> {
    Vec::new()
}

#[cfg(not(any(feature = "research", feature = "llm")))]
pub async fn search_qdrant(
    _collection_name: &str,
    _query: &str,
    _limit: usize,
    _bot_id: Uuid,
    _db_pool: &DbPool,
) -> Result<Vec<KbSearchResult>> {
    Ok(Vec::new())
}
