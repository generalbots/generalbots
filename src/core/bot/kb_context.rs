use anyhow::Result;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::kb::KnowledgeBaseManager;
use crate::core::shared::utils::DbPool;
use crate::core::kb::{EmbeddingConfig, KbIndexer, QdrantConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKbAssociation {
    pub kb_name: String,
    pub qdrant_collection: String,
    pub kb_folder_path: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWebsiteAssociation {
    pub website_url: String,
    pub collection_name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbContext {
    pub kb_name: String,
    pub search_results: Vec<KbSearchResult>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbSearchResult {
    pub content: String,
    pub document_path: String,
    pub score: f32,
    pub chunk_tokens: usize,
}

pub struct KbInjectionContext<'a> {
    pub session_id: Uuid,
    pub bot_id: Uuid,
    pub bot_name: &'a str,
    pub user_query: &'a str,
    pub messages: &'a mut serde_json::Value,
    pub max_context_tokens: usize,
}

#[derive(Debug)]
pub struct KbContextManager {
    kb_manager: Arc<KnowledgeBaseManager>,
    db_pool: DbPool,
}

impl KbContextManager {
    pub fn new(kb_manager: Arc<KnowledgeBaseManager>, db_pool: DbPool) -> Self {
        Self {
            kb_manager,
            db_pool,
        }
    }

    pub fn get_active_kbs(&self, session_id: Uuid) -> Result<Vec<SessionKbAssociation>> {
        let mut conn = self.db_pool.get()?;

        let query = diesel::sql_query(
            "SELECT kb_name, qdrant_collection, kb_folder_path, is_active
             FROM session_kb_associations
             WHERE session_id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id);

        #[derive(QueryableByName)]
        struct KbAssocRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            kb_name: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            qdrant_collection: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            kb_folder_path: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let rows: Vec<KbAssocRow> = query.load(&mut conn)?;

        Ok(rows
            .into_iter()
            .map(|row| SessionKbAssociation {
                kb_name: row.kb_name,
                qdrant_collection: row.qdrant_collection,
                kb_folder_path: row.kb_folder_path,
                is_active: row.is_active,
            })
            .collect())
    }

    pub fn get_active_websites(&self, session_id: Uuid) -> Result<Vec<SessionWebsiteAssociation>> {
        let mut conn = self.db_pool.get()?;

        let query = diesel::sql_query(
            "SELECT website_url, collection_name, is_active
             FROM session_website_associations
             WHERE session_id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id);

        #[derive(QueryableByName)]
        struct WebsiteAssocRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            website_url: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            collection_name: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let rows: Vec<WebsiteAssocRow> = query.load(&mut conn)?;

        Ok(rows
            .into_iter()
            .map(|row| SessionWebsiteAssociation {
                website_url: row.website_url,
                collection_name: row.collection_name,
                is_active: row.is_active,
            })
            .collect())
    }

    pub async fn search_active_kbs(
        &self,
        session_id: Uuid,
        bot_id: Uuid,
        bot_name: &str,
        query: &str,
        max_results_per_kb: usize,
        max_total_tokens: usize,
    ) -> Result<Vec<KbContext>> {
        let active_kbs = self.get_active_kbs(session_id)?;

        if active_kbs.is_empty() {
            debug!("No active KBs for session {}", session_id);
            return Ok(Vec::new());
        }

        info!(
            "Searching {} active KBs for session {}: {:?}",
            active_kbs.len(),
            session_id,
            active_kbs.iter().map(|kb| &kb.kb_name).collect::<Vec<_>>()
        );

        let mut kb_contexts = Vec::new();
        let mut total_tokens_used = 0;

        for kb_assoc in active_kbs {
            if total_tokens_used >= max_total_tokens {
                warn!("Reached max token limit, skipping remaining KBs");
                break;
            }

            match self
                .search_single_kb(
                    bot_id,
                    bot_name,
                    &kb_assoc.kb_name,
                    query,
                    max_results_per_kb,
                    max_total_tokens - total_tokens_used,
                )
                .await
            {
                Ok(context) => {
                    total_tokens_used += context.total_tokens;
                    info!(
                        "Found {} results from KB '{}' using {} tokens",
                        context.search_results.len(),
                        context.kb_name,
                        context.total_tokens
                    );
                    kb_contexts.push(context);
                }
                Err(e) => {
                    error!("Failed to search KB '{}': {}", kb_assoc.kb_name, e);
                }
            }
        }

        Ok(kb_contexts)
    }

    pub async fn search_active_websites(
        &self,
        session_id: Uuid,
        query: &str,
        max_results_per_website: usize,
        max_total_tokens: usize,
    ) -> Result<Vec<KbContext>> {
        let active_websites = self.get_active_websites(session_id)?;

        if active_websites.is_empty() {
            debug!("No active websites for session {}", session_id);
            return Ok(Vec::new());
        }

        info!(
            "Searching {} active websites for session {}: {:?}",
            active_websites.len(),
            session_id,
            active_websites.iter().map(|w| &w.website_url).collect::<Vec<_>>()
        );

        let mut kb_contexts = Vec::new();
        let mut total_tokens_used = 0;

        for website_assoc in active_websites {
            if total_tokens_used >= max_total_tokens {
                warn!("Reached max token limit, skipping remaining websites");
                break;
            }

            match self
                .search_single_collection(
                    &website_assoc.collection_name,
                    &website_assoc.website_url,
                    query,
                    max_results_per_website,
                    max_total_tokens - total_tokens_used,
                )
                .await
            {
                Ok(context) => {
                    total_tokens_used += context.total_tokens;
                    info!(
                        "Found {} results from website '{}' using {} tokens",
                        context.search_results.len(),
                        context.kb_name,
                        context.total_tokens
                    );
                    kb_contexts.push(context);
                }
                Err(e) => {
                    error!("Failed to search website '{}': {}", website_assoc.website_url, e);
                }
            }
        }

        Ok(kb_contexts)
    }

        async fn get_collection_dimension(&self, qdrant_config: &QdrantConfig, collection_name: &str) -> Result<Option<usize>> {
            let http_client = crate::core::shared::utils::create_tls_client(Some(10));
            let check_url = format!("{}/collections/{}", qdrant_config.url, collection_name);

            let response = http_client.get(&check_url).send().await?;

            if !response.status().is_success() {
                debug!("Could not get collection info for '{}', using default dimension", collection_name);
                return Ok(None);
            }

            let info_json: serde_json::Value = response.json().await?;
            let dimension = info_json["result"]["config"]["params"]["vectors"]["size"]
                .as_u64()
                .map(|d| d as usize);

            Ok(dimension)
        }

        async fn search_single_collection(
            &self,
            collection_name: &str,
            display_name: &str,
            query: &str,
            max_results: usize,
            max_tokens: usize,
        ) -> Result<KbContext> {
            debug!("Searching collection '{}' with query: {}", collection_name, query);

            // Extract bot_name from collection_name (format: "{bot_name}_{kb_name}")
            let bot_name = collection_name.split('_').next().unwrap_or("default");

            // Get bot_id from bot_name
            let bot_id = self.get_bot_id_by_name(bot_name).await?;

            // Load embedding config from database for this bot
            let mut embedding_config = EmbeddingConfig::from_bot_config(&self.db_pool, &bot_id);
            let qdrant_config = if let Some(sm) = crate::core::shared::utils::get_secrets_manager_sync() {
                let (url, api_key) = sm.get_vectordb_config_sync();
                crate::core::kb::QdrantConfig {
                    url,
                    api_key,
                    timeout_secs: 30,
                }
            } else {
                crate::core::kb::QdrantConfig::default()
            };

            // Query Qdrant to get the collection's actual vector dimension
            let collection_dimension = self.get_collection_dimension(&qdrant_config, collection_name).await?;

            // Override the embedding config dimension to match the collection
            if let Some(dim) = collection_dimension {
                if dim != embedding_config.dimensions {
                    debug!(
                        "Overriding embedding dimension from {} to {} to match collection '{}'",
                        embedding_config.dimensions, dim, collection_name
                    );
                    embedding_config.dimensions = dim;
                }
            }

            // Create a temporary indexer with bot-specific config
            let indexer = KbIndexer::new(embedding_config, qdrant_config);

        // Use the bot-specific indexer for search
        let search_results = indexer
            .search(collection_name, query, max_results * 3)
            .await?;

        let deduplicated = self.deduplicate_by_document(search_results);
        let kb_search_results = self.filter_by_tokens(deduplicated, max_tokens);

        Ok(KbContext {
            kb_name: display_name.to_string(),
            search_results: kb_search_results,
            total_tokens: 0,
        })
        }

        async fn get_bot_id_by_name(&self, bot_name: &str) -> Result<Uuid> {
            use crate::core::shared::models::schema::bots::dsl::*;

            let mut conn = self.db_pool.get()?;

            let bot_uuid: Uuid = bots
                .filter(name.eq(bot_name))
                .select(id)
                .first(&mut conn)
                .map_err(|e| anyhow::anyhow!("Failed to find bot '{}': {}", bot_name, e))?;

            Ok(bot_uuid)
        }

    async fn search_single_kb(
        &self,
        bot_id: Uuid,
        bot_name: &str,
        kb_name: &str,
        query: &str,
        max_results: usize,
        max_tokens: usize,
    ) -> Result<KbContext> {
        debug!("Searching KB '{}' with query: {}", kb_name, query);

        let search_results = self
            .kb_manager
            .search(bot_id, bot_name, kb_name, query, max_results * 3)
            .await?;

        let deduplicated = self.deduplicate_by_document(search_results);
        let kb_search_results = self.filter_by_tokens(deduplicated, max_tokens);

        Ok(KbContext {
            kb_name: kb_name.to_string(),
            search_results: kb_search_results,
            total_tokens: 0,
        })
    }

    fn deduplicate_by_document(&self, results: Vec<crate::core::kb::SearchResult>) -> Vec<crate::core::kb::SearchResult> {
        use std::collections::HashMap;

        let mut best_by_doc: HashMap<String, crate::core::kb::SearchResult> = HashMap::new();

        for result in results {
            let doc_key = if result.document_path.is_empty() {
                format!("unknown_{}", result.content.len())
            } else {
                result.document_path.clone()
            };

            best_by_doc
                .entry(doc_key)
                .and_modify(|existing| {
                    if result.score > existing.score {
                        *existing = result.clone();
                    }
                })
                .or_insert(result);
        }

        let mut results: Vec<_> = best_by_doc.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn filter_by_tokens(
        &self,
        results: Vec<crate::core::kb::SearchResult>,
        max_tokens: usize,
    ) -> Vec<KbSearchResult> {
        let mut kb_search_results = Vec::new();
        let mut total_tokens = 0;

        for result in results {
            let tokens = estimate_tokens(&result.content);
            
            info!("KB result - score: {:.3}, tokens: {}, content_len: {}, path: {}", 
                  result.score, tokens, result.content.len(), result.document_path);

            if total_tokens + tokens > max_tokens {
                debug!(
                    "Skipping result due to token limit ({} + {} > {})",
                    total_tokens, tokens, max_tokens
                );
                break;
            }

            if result.score < 0.20 {
                debug!("Skipping low-relevance result (score: {})", result.score);
                continue;
            }

            kb_search_results.push(KbSearchResult {
                content: result.content,
                document_path: result.document_path,
                score: result.score,
                chunk_tokens: tokens,
            });

            total_tokens += tokens;
        }

        kb_search_results
    }

    pub fn build_context_string(&self, kb_contexts: &[KbContext]) -> String {
        if kb_contexts.is_empty() {
            return String::new();
        }

        let mut context_parts = vec!["\n--- Knowledge Base Context ---".to_string()];

        for kb_context in kb_contexts {
            if kb_context.search_results.is_empty() {
                continue;
            }

            context_parts.push(format!(
                "\n## From '{}':",
                kb_context.kb_name
            ));

            for (idx, result) in kb_context.search_results.iter().enumerate() {
                context_parts.push(format!(
                    "\n### Result {} (relevance: {:.2}):\n{}",
                    idx + 1,
                    result.score,
                    result.content
                ));

                if !result.document_path.is_empty() {
                    context_parts.push(format!("Source: {}", result.document_path));
                }
            }
        }

        context_parts.push("\n--- End Knowledge Base Context ---\n".to_string());
        let full_context = context_parts.join("\n");

        // Truncate KB context to fit within token limits (max 8000 tokens for KB context)
        crate::core::shared::utils::truncate_text_for_model(&full_context, "local", 8000)
    }

    pub fn get_active_tools(&self, session_id: Uuid) -> Result<Vec<String>> {
        let mut conn = self.db_pool.get()?;

        let query = diesel::sql_query(
            "SELECT tool_name
             FROM session_tool_associations
             WHERE session_id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id);

        #[derive(QueryableByName)]
        struct ToolRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            tool_name: String,
        }

        let rows: Vec<ToolRow> = query.load(&mut conn)?;
        Ok(rows.into_iter().map(|row| row.tool_name).collect())
    }
}

fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

pub async fn inject_kb_context(
    kb_manager: Arc<KnowledgeBaseManager>,
    db_pool: DbPool,
    context: KbInjectionContext<'_>,
) -> Result<()> {
    let context_manager = KbContextManager::new(kb_manager.clone(), db_pool.clone());

    let kb_contexts = context_manager
        .search_active_kbs(context.session_id, context.bot_id, context.bot_name, context.user_query, 20, context.max_context_tokens / 2)
        .await?;

    let website_contexts = context_manager
        .search_active_websites(context.session_id, context.user_query, 20, context.max_context_tokens / 2)
        .await?;

    let mut all_contexts = kb_contexts;
    all_contexts.extend(website_contexts);

    if all_contexts.is_empty() {
        debug!("No KB or website context found for session {}", context.session_id);
        return Ok(());
    }

    let context_string = context_manager.build_context_string(&all_contexts);

    if context_string.is_empty() {
        return Ok(());
    }

    // Sanitize context to remove UTF-16 surrogate characters that can't be encoded in UTF-8
    let sanitized_context = context_string
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            !(0xD800..=0xDBFF).contains(&cp) && !(0xDC00..=0xDFFF).contains(&cp)
        })
        .collect::<String>();

    if sanitized_context.is_empty() {
        return Ok(());
    }

    info!(
        "Injecting {} characters of KB/website context into prompt for session {}",
        sanitized_context.len(),
        context.session_id
    );

    if let Some(messages_array) = context.messages.as_array_mut() {
        let system_msg_idx = messages_array.iter().position(|m| m["role"] == "system");

        if let Some(idx) = system_msg_idx {
            if let Some(content) = messages_array[idx]["content"].as_str() {
                let new_content = format!("{}\n{}", content, sanitized_context);
                messages_array[idx]["content"] = serde_json::Value::String(new_content);
            }
        } else {
            messages_array.insert(
                0,
                serde_json::json!({
                    "role": "system",
                    "content": sanitized_context
                }),
            );
        }
    }

    Ok(())
}
