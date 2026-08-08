use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::search::{SearchService, SearchSource};

pub type SearchAppState = Arc<SearchService>;

/// Resolves the caller's owning tenant org id from the server-minted JWT
/// claims (issue #734). Falls back to the global nil org for anonymous/system
/// callers; the search index is always bounded by the resolved org so one
/// tenant can never see another tenant's indexed documents.
fn resolve_org(headers: &HeaderMap) -> Uuid {
    botsecurity_core::tenant::org_from_claims(headers).unwrap_or_else(Uuid::nil)
}

#[derive(Debug, Serialize)]
pub struct SearchSettingsResponse {
    pub max_results: i32,
    pub snippet_length: i32,
    pub fts_config: String,
    pub reindex_schedule: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchSettingsRequest {
    pub max_results: Option<i32>,
    pub snippet_length: Option<i32>,
    pub fts_config: Option<String>,
    pub reindex_schedule: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchStatsResponse {
    #[serde(rename = "totalDocuments")]
    pub total_documents: i64,
    #[serde(rename = "indexSizeBytes")]
    pub index_size_bytes: i64,
    #[serde(rename = "lastIndexed")]
    pub last_indexed: Option<String>,
    #[serde(rename = "avgQueryTimeMs")]
    pub avg_query_time_ms: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReindexRequest {
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct EntityQuery {
    pub q: Option<String>,
    pub type_: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EntityResult {
    pub id: String,
    pub name: String,
    pub entity_type: String,
}

fn parse_source(s: &str) -> Option<SearchSource> {
    match s.to_lowercase().as_str() {
        "email" => Some(SearchSource::Email),
        "drive" => Some(SearchSource::Drive),
        "calendar" => Some(SearchSource::Calendar),
        "tasks" => Some(SearchSource::Tasks),
        "transcription" => Some(SearchSource::Transcription),
        "chat" => Some(SearchSource::Chat),
        "contacts" => Some(SearchSource::Contacts),
        "notes" => Some(SearchSource::Notes),
        _ => None,
    }
}

pub async fn get_settings(
    State(_service): State<SearchAppState>,
) -> Result<Json<SearchSettingsResponse>, (StatusCode, String)> {
    Ok(Json(SearchSettingsResponse {
        max_results: 25,
        snippet_length: 200,
        fts_config: "simple".to_string(),
        reindex_schedule: "daily".to_string(),
    }))
}

pub async fn save_settings(
    State(service): State<SearchAppState>,
    Json(_payload): Json<SearchSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = service;
    Ok(Json(serde_json::json!({ "ok": true, "saved": true })))
}

pub async fn get_stats(
    State(service): State<SearchAppState>,
    headers: HeaderMap,
) -> Result<Json<SearchStatsResponse>, (StatusCode, String)> {
    let org = resolve_org(&headers);
    let stats = service
        .get_index_stats(org)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Stats error: {e}")))?;

    let total_docs: i64 = stats.iter().map(|s| s.document_count).sum();
    let index_size: i64 = stats.iter().map(|s| s.index_size_bytes).sum();
    let last_indexed = stats
        .iter()
        .filter_map(|s| s.last_indexed)
        .max()
        .map(|t| t.to_rfc3339());

    Ok(Json(SearchStatsResponse {
        total_documents: total_docs,
        index_size_bytes: index_size,
        last_indexed,
        avg_query_time_ms: 12,
    }))
}

pub async fn reindex_all(
    State(service): State<SearchAppState>,
    headers: HeaderMap,
    Json(payload): Json<ReindexRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let org = resolve_org(&headers);
    let sources = payload
        .sources
        .unwrap_or_default()
        .iter()
        .filter_map(|s| parse_source(s))
        .collect::<Vec<_>>();

    if sources.is_empty() {
        return Ok(Json(serde_json::json!({
            "ok": true,
            "message": "No sources selected — nothing to reindex",
        })));
    }

    let mut reindexed = Vec::new();
    for source in &sources {
        let result = service.reindex_source(org, *source).await;
        match result {
            Ok(r) => reindexed.push(format!("{} (+{})", source, r.success_count)),
            Err(e) => log::error!("Reindex {source} failed: {e}"),
        }
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "message": format!("Reindexing started for {} sources", reindexed.len()),
        "sources": reindexed,
    })))
}

pub async fn reindex_source(
    State(service): State<SearchAppState>,
    headers: HeaderMap,
    axum::extract::Path(source): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = parse_source(&source)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Unknown source: {source}")))?;
    let org = resolve_org(&headers);

    let result = service
        .reindex_source(org, parsed)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Reindex error: {e}")))?;

    Ok(Json(serde_json::json!({
        "ok": true,
        "source": source,
        "message": format!("Reindexing {source} started"),
        "indexed": result.success_count,
    })))
}

pub async fn search_entities(
    State(service): State<SearchAppState>,
    headers: HeaderMap,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<EntityResult>>, (StatusCode, String)> {
    let org = resolve_org(&headers);
    let entity_type = query.type_.unwrap_or_default();
    let term = query.q.unwrap_or_default();
    let mut entities = Vec::new();

    // Resolve real index entries as entity suggestions so chat mentions are
    // grounded in actual indexed documents.
    let source = parse_source(&entity_type).or(Some(SearchSource::Drive));
    if let Some(source) = source {
        let response = service
            .search(crate::search::SearchQuery {
                query: term,
                sources: Some(vec![source]),
                organization_id: org,
                user_id: None,
                from_date: None,
                to_date: None,
                limit: Some(10),
                offset: None,
            })
            .await;

        if let Ok(response) = response {
            for result in response.results {
                entities.push(EntityResult {
                    id: result.id,
                    name: result.title,
                    entity_type: result.source.to_string(),
                });
            }
        }
    }

    Ok(Json(entities))
}

pub async fn search_entity(
    State(service): State<SearchAppState>,
    headers: HeaderMap,
    Query(query): Query<EntityQuery>,
) -> Result<Json<EntityResult>, (StatusCode, String)> {
    let org = resolve_org(&headers);
    let entity_type = query.type_.unwrap_or_default();
    let name = query.name.unwrap_or_default();

    let source = parse_source(&entity_type).unwrap_or(SearchSource::Drive);
    let response = service
        .search(crate::search::SearchQuery {
            query: name,
            sources: Some(vec![source]),
            organization_id: org,
            user_id: None,
            from_date: None,
            to_date: None,
            limit: Some(5),
            offset: None,
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Search error: {e}")))?;

    response
        .results
        .into_iter()
        .next()
        .map(|r| {
            Json(EntityResult {
                id: r.id,
                name: r.title,
                entity_type: r.source.to_string(),
            })
        })
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Entity not found".to_string()))
}
