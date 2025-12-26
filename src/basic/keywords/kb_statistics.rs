use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub vectors_count: u64,
    pub points_count: u64,
    pub segments_count: u64,
    pub disk_data_size: u64,
    pub ram_data_size: u64,
    pub indexed_vectors_count: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KBStatistics {
    pub total_collections: u64,
    pub total_documents: u64,
    pub total_vectors: u64,
    pub total_disk_size_mb: f64,
    pub total_ram_size_mb: f64,
    pub documents_added_last_week: u64,
    pub documents_added_last_month: u64,
    pub collections: Vec<CollectionStats>,
}

pub fn kb_statistics_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("KB STATISTICS", move || -> Dynamic {
        let state = Arc::clone(&state_clone);
        let user = user_clone.clone();

        trace!(
            "KB STATISTICS called for bot {} by user {}",
            user.bot_id,
            user.user_id
        );

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            error!("KB STATISTICS: No tokio runtime available");
            return Dynamic::UNIT;
        }

        let result = rt
            .unwrap()
            .block_on(async { get_kb_statistics(&state, &user).await });

        match result {
            Ok(stats) => match serde_json::to_value(&stats) {
                Ok(json) => Dynamic::from(json.to_string()),
                Err(e) => {
                    error!("Failed to serialize KB statistics: {}", e);
                    Dynamic::UNIT
                }
            },
            Err(e) => {
                error!("Failed to get KB statistics: {}", e);
                Dynamic::UNIT
            }
        }
    });

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine.register_fn(
        "KB COLLECTION STATS",
        move |collection_name: &str| -> Dynamic {
            let state = Arc::clone(&state_clone2);
            let user = user_clone2.clone();

            trace!(
                "KB COLLECTION STATS called for collection '{}' bot {} by user {}",
                collection_name,
                user.bot_id,
                user.user_id
            );

            let rt = tokio::runtime::Handle::try_current();
            if rt.is_err() {
                error!("KB COLLECTION STATS: No tokio runtime available");
                return Dynamic::UNIT;
            }

            let collection = collection_name.to_string();
            let result = rt
                .unwrap()
                .block_on(async { get_collection_statistics(&state, &collection).await });

            match result {
                Ok(stats) => match serde_json::to_value(&stats) {
                    Ok(json) => Dynamic::from(json.to_string()),
                    Err(e) => {
                        error!("Failed to serialize collection statistics: {}", e);
                        Dynamic::UNIT
                    }
                },
                Err(e) => {
                    error!("Failed to get collection statistics: {}", e);
                    Dynamic::UNIT
                }
            }
        },
    );

    let state_clone3 = Arc::clone(&state);
    let user_clone3 = user.clone();

    engine.register_fn("KB DOCUMENTS COUNT", move || -> i64 {
        let state = Arc::clone(&state_clone3);
        let user = user_clone3.clone();

        trace!(
            "KB DOCUMENTS COUNT called for bot {} by user {}",
            user.bot_id,
            user.user_id
        );

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            error!("KB DOCUMENTS COUNT: No tokio runtime available");
            return 0;
        }

        let result = get_documents_count(&state, &user);

        result.unwrap_or(0)
    });

    let state_clone4 = Arc::clone(&state);
    let user_clone4 = user.clone();

    engine.register_fn("KB DOCUMENTS ADDED SINCE", move |days: i64| -> i64 {
        let state = Arc::clone(&state_clone4);
        let user = user_clone4.clone();

        trace!(
            "KB DOCUMENTS ADDED SINCE {} days called for bot {} by user {}",
            days,
            user.bot_id,
            user.user_id
        );

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            error!("KB DOCUMENTS ADDED SINCE: No tokio runtime available");
            return 0;
        }

        let result = get_documents_added_since(&state, &user, days);

        result.unwrap_or(0)
    });

    let state_clone5 = Arc::clone(&state);
    let user_clone5 = user.clone();

    engine.register_fn("KB LIST COLLECTIONS", move || -> Dynamic {
        let state = Arc::clone(&state_clone5);
        let user = user_clone5.clone();

        trace!(
            "KB LIST COLLECTIONS called for bot {} by user {}",
            user.bot_id,
            user.user_id
        );

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            error!("KB LIST COLLECTIONS: No tokio runtime available");
            return Dynamic::UNIT;
        }

        let result = rt
            .unwrap()
            .block_on(async { list_collections(&state, &user).await });

        match result {
            Ok(collections) => {
                let arr: Vec<Dynamic> = collections.into_iter().map(Dynamic::from).collect();
                Dynamic::from(arr)
            }
            Err(e) => {
                error!("Failed to list collections: {}", e);
                Dynamic::UNIT
            }
        }
    });

    let state_clone6 = Arc::clone(&state);
    let user_clone6 = user;

    engine.register_fn("KB STORAGE SIZE", move || -> f64 {
        let state = Arc::clone(&state_clone6);
        let user = user_clone6.clone();

        trace!(
            "KB STORAGE SIZE called for bot {} by user {}",
            user.bot_id,
            user.user_id
        );

        let rt = tokio::runtime::Handle::try_current();
        if rt.is_err() {
            error!("KB STORAGE SIZE: No tokio runtime available");
            return 0.0;
        }

        let result = rt
            .unwrap()
            .block_on(async { get_storage_size(&state, &user).await });

        result.unwrap_or(0.0)
    });
}

async fn get_kb_statistics(
    state: &AppState,
    user: &UserSession,
) -> Result<KBStatistics, Box<dyn std::error::Error + Send + Sync>> {
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "https://localhost:6334".to_string());
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let collections_response = client
        .get(format!("{}/collections", qdrant_url))
        .send()
        .await?;

    let collections_json: serde_json::Value = collections_response.json().await?;
    let collection_names: Vec<String> = collections_json["result"]["collections"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
        .filter(|name| name.starts_with(&format!("kb_{}", user.bot_id)))
        .collect();

    let mut total_documents = 0u64;
    let mut total_vectors = 0u64;
    let mut total_disk_size = 0u64;
    let mut total_ram_size = 0u64;
    let mut collections = Vec::new();

    for collection_name in &collection_names {
        if let Ok(stats) = get_collection_statistics(state, collection_name).await {
            total_documents += stats.points_count;
            total_vectors += stats.vectors_count;
            total_disk_size += stats.disk_data_size;
            total_ram_size += stats.ram_data_size;
            collections.push(stats);
        }
    }

    let documents_added_last_week = get_documents_added_since(state, user, 7).unwrap_or(0) as u64;
    let documents_added_last_month = get_documents_added_since(state, user, 30).unwrap_or(0) as u64;

    Ok(KBStatistics {
        total_collections: collection_names.len() as u64,
        total_documents,
        total_vectors,
        total_disk_size_mb: total_disk_size as f64 / (1024.0 * 1024.0),
        total_ram_size_mb: total_ram_size as f64 / (1024.0 * 1024.0),
        documents_added_last_week,
        documents_added_last_month,
        collections,
    })
}

async fn get_collection_statistics(
    _state: &AppState,
    collection_name: &str,
) -> Result<CollectionStats, Box<dyn std::error::Error + Send + Sync>> {
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "https://localhost:6334".to_string());
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client
        .get(format!("{}/collections/{}", qdrant_url, collection_name))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let result = &json["result"];

    Ok(CollectionStats {
        name: collection_name.to_string(),
        vectors_count: result["vectors_count"].as_u64().unwrap_or(0),
        points_count: result["points_count"].as_u64().unwrap_or(0),
        segments_count: result["segments_count"].as_u64().unwrap_or(0),
        disk_data_size: result["disk_data_size"].as_u64().unwrap_or(0),
        ram_data_size: result["ram_data_size"].as_u64().unwrap_or(0),
        indexed_vectors_count: result["indexed_vectors_count"].as_u64().unwrap_or(0),
        status: result["status"].as_str().unwrap_or("unknown").to_string(),
    })
}

fn get_documents_count(
    state: &AppState,
    user: &UserSession,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::BigInt;

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let mut conn = state.conn.get()?;
    let bot_id = user.bot_id.to_string();

    let result: CountResult =
        sql_query("SELECT COUNT(*) as count FROM kb_documents WHERE bot_id = $1")
            .bind::<diesel::sql_types::Text, _>(&bot_id)
            .get_result(&mut *conn)?;

    Ok(result.count)
}

fn get_documents_added_since(
    state: &AppState,
    user: &UserSession,
    days: i64,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::{BigInt, Integer, Text};

    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let mut conn = state.conn.get()?;
    let bot_id = user.bot_id.to_string();

    let result: CountResult = sql_query(
        "SELECT COUNT(*) as count FROM kb_documents
         WHERE bot_id = $1
         AND created_at >= NOW() - INTERVAL '1 day' * $2",
    )
    .bind::<Text, _>(&bot_id)
    .bind::<Integer, _>(days as i32)
    .get_result(&mut *conn)?;

    Ok(result.count)
}

async fn list_collections(
    _state: &AppState,
    user: &UserSession,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "https://localhost:6334".to_string());
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client
        .get(format!("{}/collections", qdrant_url))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let collections: Vec<String> = json["result"]["collections"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
        .filter(|name| name.starts_with(&format!("kb_{}", user.bot_id)))
        .collect();

    Ok(collections)
}

async fn get_storage_size(
    state: &AppState,
    user: &UserSession,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let stats = get_kb_statistics(state, user).await?;
    Ok(stats.total_disk_size_mb)
}
