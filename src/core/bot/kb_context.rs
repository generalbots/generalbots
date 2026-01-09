use anyhow::Result;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::kb::KnowledgeBaseManager;
use crate::shared::utils::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKbAssociation {
    pub kb_name: String,
    pub qdrant_collection: String,
    pub kb_folder_path: String,
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

    pub async fn search_active_kbs(
        &self,
        session_id: Uuid,
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

    async fn search_single_kb(
        &self,
        bot_name: &str,
        kb_name: &str,
        query: &str,
        max_results: usize,
        max_tokens: usize,
    ) -> Result<KbContext> {
        debug!("Searching KB '{}' with query: {}", kb_name, query);

        let search_results = self
            .kb_manager
            .search(bot_name, kb_name, query, max_results)
            .await?;

        let mut kb_search_results = Vec::new();
        let mut total_tokens = 0;

        for result in search_results {
            let tokens = estimate_tokens(&result.content);

            if total_tokens + tokens > max_tokens {
                debug!(
                    "Skipping result due to token limit ({} + {} > {})",
                    total_tokens, tokens, max_tokens
                );
                break;
            }

            kb_search_results.push(KbSearchResult {
                content: result.content,
                document_path: result.document_path,
                score: result.score,
                chunk_tokens: tokens,
            });

            total_tokens += tokens;

            if result.score < 0.7 {
                debug!("Skipping low-relevance result (score: {})", result.score);
                break;
            }
        }

        Ok(KbContext {
            kb_name: kb_name.to_string(),
            search_results: kb_search_results,
            total_tokens,
        })
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
                "\n## From '{}' knowledge base:",
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
        context_parts.join("\n")
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
    session_id: Uuid,
    bot_name: &str,
    user_query: &str,
    messages: &mut serde_json::Value,
    max_context_tokens: usize,
) -> Result<()> {
    let context_manager = KbContextManager::new(kb_manager, db_pool);

    let kb_contexts = context_manager
        .search_active_kbs(session_id, bot_name, user_query, 5, max_context_tokens)
        .await?;

    if kb_contexts.is_empty() {
        debug!("No KB context found for session {}", session_id);
        return Ok(());
    }

    let context_string = context_manager.build_context_string(&kb_contexts);

    if context_string.is_empty() {
        return Ok(());
    }

    info!(
        "Injecting {} characters of KB context into prompt for session {}",
        context_string.len(),
        session_id
    );

    if let Some(messages_array) = messages.as_array_mut() {
        let system_msg_idx = messages_array.iter().position(|m| m["role"] == "system");

        if let Some(idx) = system_msg_idx {
            if let Some(content) = messages_array[idx]["content"].as_str() {
                let new_content = format!("{}\n{}", content, context_string);
                messages_array[idx]["content"] = serde_json::Value::String(new_content);
            }
        } else {
            messages_array.insert(
                0,
                serde_json::json!({
                    "role": "system",
                    "content": context_string
                }),
            );
        }
    }

    Ok(())
}
