use super::*;

impl HybridSearchEngine {
    pub(crate) async fn upsert_to_qdrant(&self, doc_id: &str, embedding: &[f32]) -> Result<(), String> {
        let client = reqwest::Client::new();

        let upsert_request = serde_json::json!({
            "points": [{
                "id": doc_id,
                "vector": embedding
            }]
        });

        let request = client
            .put(format!(
                "{}/collections/{}/points",
                self.qdrant_url, self.collection_name
            ))
            .json(&upsert_request);
        let request = match crate::qdrant_native::resolve_qdrant_api_key() {
            Some(key) => request.header("api-key", key),
            None => request,
        };
        let response = request
            .send()
            .await
            .map_err(|e| format!("Qdrant upsert failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Qdrant upsert error: {}", error_text));
        }

        Ok(())
    }

    pub(crate) async fn delete_from_qdrant(&self, doc_id: &str) -> Result<(), String> {
        let client = reqwest::Client::new();

        let delete_request = serde_json::json!({
            "points": [doc_id]
        });

        let request = client
            .post(format!(
                "{}/collections/{}/points/delete",
                self.qdrant_url, self.collection_name
            ))
            .json(&delete_request);
        let request = match crate::qdrant_native::resolve_qdrant_api_key() {
            Some(key) => request.header("api-key", key),
            None => request,
        };
        let response = request
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

    pub fn stats(&self) -> HybridSearchStats {
        let bm25_stats = self.bm25_index.stats();

        HybridSearchStats {
            total_documents: self.documents.len(),
            bm25_doc_count: bm25_stats.doc_count,
            unique_terms: bm25_stats.unique_terms,
            avg_doc_len: bm25_stats.avg_doc_len,
            bm25_enabled: bm25_stats.enabled,
            config: self.config.clone(),
        }
    }
}
