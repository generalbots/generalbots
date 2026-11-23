use super::*;
use reqwest::header::{HeaderMap, HeaderValue};

/// The Qdrant REST data plane requires the `api-key` header on every call once
/// `service.api_key` is configured. Prefer `QDRANT_API_KEY`, fall back to the
/// generic `VECTORDB_API_KEY` used by the rest of the tree, and ignore the
/// placeholder seeded before Vault provisioning.
pub(crate) fn resolve_qdrant_api_key() -> Option<String> {
    std::env::var("QDRANT_API_KEY")
        .ok()
        .or_else(|| std::env::var("VECTORDB_API_KEY").ok())
        .filter(|key| !key.is_empty() && key != "none")
}

/// Convert a credential into a valid HTTP header value. An unusable value is
/// dropped rather than sent malformed: Qdrant then rejects the request instead
/// of silently granting access.
fn api_key_header(api_key: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(api_key).ok()
}

fn build_http_client(api_key: Option<&str>) -> Client {
    let mut builder = Client::builder();
    match api_key.and_then(api_key_header) {
        Some(value) => {
            let mut headers = HeaderMap::new();
            headers.insert("api-key", value);
            builder = builder.default_headers(headers);
        }
        None => {
            log::warn!(
                "Qdrant client has no usable API key; requests will be rejected by an authenticated data plane"
            );
        }
    }
    builder.build().unwrap_or_else(|_| Client::new())
}

impl QdrantFromUrl {
    /// Inject the data-plane credential explicitly. When omitted, the client
    /// resolves it from the environment at build time.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn build(self) -> Result<QdrantClient> {
        Ok(QdrantClient::new_with_api_key(&self.url, self.api_key))
    }
}
impl QdrantClient {
    /// Create new Qdrant client from URL (returns a builder)
    pub fn from_url(url: &str) -> QdrantFromUrl {
        QdrantFromUrl {
            url: url.to_string(),
            api_key: None,
        }
    }

    /// Create new Qdrant client directly from URL
    pub fn new(url: &str) -> Self {
        Self::new_with_api_key(url, None)
    }

    /// Create a client with an explicit credential, falling back to the
    /// environment when none is supplied by the caller.
    pub fn new_with_api_key(url: &str, api_key: Option<String>) -> Self {
        let api_key = api_key.or_else(resolve_qdrant_api_key);
        let client = build_http_client(api_key.as_deref());

        Self {
            client,
            url: url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    /// Normalized base URL (used by tests and diagnostics).
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The credential attached to every request, if one was configured.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Get full URL for collections endpoint
    pub(super) fn collections_url(&self) -> String {
        format!("{}/collections", self.url)
    }

    /// Create collection
    pub async fn create_collection(
        &self,
        name: &str,
        vector_size: u64,
        distance: &str,
    ) -> Result<()> {
        let url = self.collections_url();

        botlog::safe_debug!("Creating collection: {} at {}", name, url);
        
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "name": name,
                "vectors": {
                    "size": vector_size,
                    "distance": distance
                }
            }))
            .send()
            .await
            .context("Failed to send create collection request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        
        if status.is_success() {
            botlog::safe_info!("Collection '{}' created successfully", name);
            Ok(())
        } else {
            botlog::safe_error!("Failed to create collection '{}': {} - {}", name, status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<CollectionsResponse> {
        let url = self.collections_url();
        
        debug!("Listing collections at {}", url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send list collections request")?;
        
        let status = response.status();
        if status.is_success() {
            let result: CollectionsResponse = response.json().await
                .context("Failed to parse collections response")?;
            debug!("Found {} collections", result.collections.len());
            Ok(result)
        } else {
            let text = response.text().await.unwrap_or_default();
            log::error!("Failed to list collections: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// Check if collection exists
    pub async fn collection_exists(&self, name: &str) -> Result<bool> {
        let url = format!("{}/{}", self.collections_url(), name);

        debug!("Checking collection: {} at {}", name, url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send collection exists request")?;

        Ok(response.status().is_success())
    }

    /// Get collection info
    pub async fn collection_info(&self, name: &str) -> Result<CollectionInfoResponse> {
        let url = format!("{}/{}", self.collections_url(), name);

        debug!("Getting collection info: {} at {}", name, url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send collection info request")?;

        let status = response.status();
        if status.is_success() {
            let result: CollectionInfoResponse = response.json().await
                .context("Failed to parse collection info response")?;
            Ok(result)
        } else {
            let text = response.text().await.unwrap_or_default();
            log::error!("Failed to get collection info: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// Delete collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        let url = format!("{}/{}", self.collections_url(), name);

        debug!("Deleting collection: {} at {}", name, url);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send delete collection request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if status.is_success() {
            info!("Collection '{}' deleted successfully", name);
            Ok(())
        } else {
            log::error!("Failed to delete collection '{}': {} - {}", name, status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// Upsert points into collection
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<Value>,
    ) -> Result<()> {
        let url = format!("{}/{}/upsert", self.collections_url(), collection_name);
        let body = json!({
            "points": points
        });

        botlog::safe_debug!("Upserting {} points to {}", points.len(), collection_name);
        
        let response = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send upsert request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        
        if status.is_success() {
            debug!("Successfully upserted {} points", points.len());
            Ok(())
        } else {
            log::error!("Failed to upsert points: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// Search points in collection
    pub async fn search_points(
        &self,
        collection_name: &str,
        vector: &[f32],
        limit: usize,
        filter: Option<Value>,
    ) -> Result<Vec<Value>> {
        let url = format!("{}/{}/search", self.collections_url(), collection_name);
        let mut body = json!({
            "vector": vector,
            "limit": limit
        });

        if let Some(f) = filter {
            body["filter"] = f;
        }

        debug!("Searching {} points in {}", limit, collection_name);
        
        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send search request")?;

        let status = response.status();
        if status.is_success() {
            let result: Value = response.json().await.context("Failed to parse search response")?;
            let points = result["result"]
                .as_array().cloned()
                .unwrap_or_default();
            debug!("Found {} search results", points.len());
            Ok(points)
        } else {
            let text = response.text().await.unwrap_or_default();
            log::error!("Failed to search points: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }

    /// Delete points from collection
    pub async fn delete_points(
        &self,
        collection_name: &str,
        point_ids: Vec<String>,
    ) -> Result<()> {
        let url = format!("{}/{}/delete", self.collections_url(), collection_name);
        let body = json!({
            "points": point_ids
        });

        debug!("Deleting {} points from {}", point_ids.len(), collection_name);
        
        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send delete request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        
        if status.is_success() {
            info!("Successfully deleted {} points", point_ids.len());
            Ok(())
        } else {
            log::error!("Failed to delete points: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_url_normalizes_url_and_carries_explicit_key() {
        let built = QdrantClient::from_url("http://127.0.0.1:6333///")
            .with_api_key("unit-test-key")
            .build();
        assert!(built.is_ok());
        let client = match built {
            Ok(client) => client,
            Err(_) => return,
        };
        assert_eq!(client.url(), "http://127.0.0.1:6333");
        assert_eq!(client.api_key(), Some("unit-test-key"));
        assert_eq!(
            client.collections_url(),
            "http://127.0.0.1:6333/collections"
        );
    }

    #[test]
    fn explicit_key_takes_precedence_over_environment() {
        let client = QdrantClient::new_with_api_key(
            "http://127.0.0.1:6333/",
            Some("explicit-key".to_string()),
        );
        assert_eq!(client.api_key(), Some("explicit-key"));
    }

    #[test]
    fn api_key_header_value_is_validated() {
        assert!(matches!(
            api_key_header("unit-test-key").as_ref().map(HeaderValue::to_str),
            Some(Ok("unit-test-key"))
        ));
        assert!(api_key_header("bad\nkey").is_none());
        assert!(api_key_header("").is_some());
    }
}
