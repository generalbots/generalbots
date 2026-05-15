use anyhow::Result;
use diesel::prelude::*;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botcore::shared::utils::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbSearchResult {
    pub content: String,
    pub document_path: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbContext {
    pub kb_name: String,
    pub search_results: Vec<KbSearchResult>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKbAssociation {
    pub kb_name: String,
    pub qdrant_collection: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWebsiteAssociation {
    pub website_url: String,
    pub collection_name: String,
    pub is_active: bool,
}

pub fn get_active_kbs(db_pool: &DbPool, session_id: Uuid) -> Vec<SessionKbAssociation> {
    let mut conn = match db_pool.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to get DB connection for KB lookup: {}", e);
            return Vec::new();
        }
    };

    #[derive(QueryableByName)]
    struct KbAssocRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        kb_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        qdrant_collection: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let query = diesel::sql_query(
        "SELECT kb_name, qdrant_collection, is_active
         FROM session_kb_associations
         WHERE session_id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id);

    match query.load::<KbAssocRow>(&mut conn) {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SessionKbAssociation {
                kb_name: r.kb_name,
                qdrant_collection: r.qdrant_collection,
                is_active: r.is_active,
            })
            .collect(),
        Err(e) => {
            debug!("No active KBs for session {}: {}", session_id, e);
            Vec::new()
        }
    }
}

pub fn get_active_websites(db_pool: &DbPool, session_id: Uuid) -> Vec<SessionWebsiteAssociation> {
    let mut conn = match db_pool.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to get DB connection for website lookup: {}", e);
            return Vec::new();
        }
    };

    #[derive(QueryableByName)]
    struct WebsiteAssocRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        website_url: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        collection_name: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
    }

    let query = diesel::sql_query(
        "SELECT website_url, collection_name, is_active
         FROM session_website_associations
         WHERE session_id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id);

    match query.load::<WebsiteAssocRow>(&mut conn) {
        Ok(rows) => rows
            .into_iter()
            .map(|r| SessionWebsiteAssociation {
                website_url: r.website_url,
                collection_name: r.collection_name,
                is_active: r.is_active,
            })
            .collect(),
        Err(e) => {
            debug!("No active websites for session {}: {}", session_id, e);
            Vec::new()
        }
    }
}

fn get_vectordb_url() -> String {
    std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| std::env::var("VECTORDB_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".to_string()))
}

fn get_vectordb_api_key() -> String {
    std::env::var("QDRANT_API_KEY")
        .unwrap_or_else(|_| std::env::var("VECTORDB_API_KEY").unwrap_or_default())
}

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

async fn search_qdrant(
    collection_name: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<KbSearchResult>> {
    let qdrant_url = get_vectordb_url();
    let api_key = get_vectordb_api_key();

    let search_url = format!("{}/collections/{}/points/search", qdrant_url.trim_end_matches('/'), collection_name);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Try to get collection info to determine vector dimension
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
    let vector = generate_hash_embedding(query, dimensions);

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

fn build_context_string(kb_contexts: &[KbContext]) -> String {
    if kb_contexts.is_empty() {
        return String::new();
    }

    let mut parts = vec!["\n--- Informações de Contexto (Base de Conhecimento) ---".to_string()];

    for ctx in kb_contexts {
        if ctx.search_results.is_empty() {
            continue;
        }

        parts.push(format!("\n## De '{}':", ctx.kb_name));

        for (idx, result) in ctx.search_results.iter().enumerate() {
            parts.push(format!(
                "\n### Resultado {} (relevância: {:.2}):\n{}",
                idx + 1,
                result.score,
                result.content
            ));
            if !result.document_path.is_empty() {
                parts.push(format!("Fonte: {}", result.document_path));
            }
        }
    }

    parts.push("\n--- Fim do Contexto ---\n".to_string());
    parts.join("\n")
}

fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

fn truncate_text(text: &str, max_tokens: usize) -> String {
    let mut tokens = 0usize;
    let mut result = String::new();
    for line in text.lines() {
        let line_tokens = estimate_tokens(line) + 1;
        if tokens + line_tokens > max_tokens {
            break;
        }
        tokens += line_tokens;
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Inject KB and website context into messages array for LLM
/// Searches active KBs and websites associated with the session,
/// then injects the results as a system message.
pub async fn inject_kb_context(
    db_pool: &DbPool,
    session_id: Uuid,
    user_query: &str,
    messages: &mut serde_json::Value,
    max_context_tokens: usize,
) {
    let active_kbs = get_active_kbs(db_pool, session_id);
    let active_websites = get_active_websites(db_pool, session_id);

    if active_kbs.is_empty() && active_websites.is_empty() {
        debug!("No active KBs or websites for session {}", session_id);
        return;
    }

    info!(
        "Injecting context for session {}: {} KB(s), {} website(s)",
        session_id,
        active_kbs.len(),
        active_websites.len()
    );

    let mut all_contexts = Vec::new();

    for kb in &active_kbs {
        match search_qdrant(&kb.qdrant_collection, user_query, 10).await {
            Ok(results) if !results.is_empty() => {
                let total_tokens: usize = results.iter().map(|r| estimate_tokens(&r.content)).sum();
                info!("Found {} results from KB '{}' ({} tokens)", results.len(), kb.kb_name, total_tokens);
                all_contexts.push(KbContext {
                    kb_name: kb.kb_name.clone(),
                    total_tokens,
                    search_results: results,
                });
            }
            Ok(_) => debug!("No results from KB '{}'", kb.kb_name),
            Err(e) => warn!("Failed to search KB '{}': {}", kb.kb_name, e),
        }
    }

    for website in &active_websites {
        match search_qdrant(&website.collection_name, user_query, 10).await {
            Ok(results) if !results.is_empty() => {
                let total_tokens: usize = results.iter().map(|r| estimate_tokens(&r.content)).sum();
                info!("Found {} results from website '{}' ({} tokens)", results.len(), website.website_url, total_tokens);
                all_contexts.push(KbContext {
                    kb_name: website.website_url.clone(),
                    total_tokens,
                    search_results: results,
                });
            }
            Ok(_) => debug!("No results from website '{}'", website.website_url),
            Err(e) => warn!("Failed to search website '{}': {}", website.website_url, e),
        }
    }

    if all_contexts.is_empty() {
        info!("No KB/website content found for session {}", session_id);
        return;
    }

    let context_string = build_context_string(&all_contexts);
    let truncated = truncate_text(&context_string, max_context_tokens);

    if truncated.is_empty() {
        return;
    }

    info!(
        "Injecting {} chars (est. {} tokens) of KB/website context into prompt for session {}",
        truncated.len(),
        estimate_tokens(&truncated),
        session_id
    );

    if let Some(msgs_array) = messages.as_array_mut() {
        if let Some(idx) = msgs_array.iter().position(|m| m["role"] == "system") {
            if let Some(content) = msgs_array[idx]["content"].as_str() {
                msgs_array[idx]["content"] = serde_json::Value::String(format!("{}\n{}", content, truncated));
            }
        } else {
            msgs_array.insert(0, serde_json::json!({
                "role": "system",
                "content": truncated
            }));
        }
    }
}
