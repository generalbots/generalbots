/// Qdrant HTTP Client - Native implementation without qdrant-client crate
/// Uses reqwest for HTTP communication with Qdrant REST API

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use log::{debug, error, info};

/// Qdrant client using native HTTP (old name for backward compatibility)
pub type Qdrant = QdrantClient;

/// Qdrant client using native HTTP
#[derive(Clone)]
pub struct QdrantClient {
    client: Client,
    url: String,
}

/// Builder trait for Qdrant client
pub trait Build {
    fn build(self) -> Result<QdrantClient>;
}

/// URL builder for Qdrant
pub struct QdrantFromUrl {
    url: String,
}

impl QdrantFromUrl {
    pub fn build(self) -> Result<QdrantClient> {
        Ok(QdrantClient::new(&self.url))
    }
}

impl QdrantClient {
    /// Create new Qdrant client from URL (returns a builder)
    pub fn from_url(url: &str) -> QdrantFromUrl {
        QdrantFromUrl { url: url.to_string() }
    }

    /// Create new Qdrant client directly from URL
    pub fn new(url: &str) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            url: url.trim_end_matches('/').to_string(),
        }
    }

    /// Get full URL for collections endpoint
    fn collections_url(&self) -> String {
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

        debug!("Creating collection: {} at {}", name, url);
        
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
            info!("Collection '{}' created successfully", name);
            Ok(())
        } else {
            error!("Failed to create collection '{}': {} - {}", name, status, text);
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
            error!("Failed to list collections: {} - {}", status, text);
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
            error!("Failed to get collection info: {} - {}", status, text);
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
            error!("Failed to delete collection '{}': {} - {}", name, status, text);
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

        debug!("Upserting {} points to {}", points.len(), collection_name);
        
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
            error!("Failed to upsert points: {} - {}", status, text);
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
                .as_array()
                .map(|arr| arr.clone())
                .unwrap_or_default();
            debug!("Found {} search results", points.len());
            Ok(points)
        } else {
            let text = response.text().await.unwrap_or_default();
            error!("Failed to search points: {} - {}", status, text);
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
            error!("Failed to delete points: {} - {}", status, text);
            Err(anyhow::anyhow!("HTTP {}: {}", status, text))
        }
    }
}

/// Response for list collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsResponse {
    pub collections: Vec<CollectionInfo>,
}

/// Collection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
}

/// Collection info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfoResponse {
    #[serde(default)]
    pub points_count: Option<u64>,
}

/// Builder for creating collections
pub struct CreateCollectionBuilder {
    name: String,
    vector_size: u64,
    distance: String,
}

impl CreateCollectionBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vector_size: 0,
            distance: "Cosine".to_string(),
        }
    }

    pub fn vector(mut self, params: VectorParamsBuilder) -> Self {
        self.vector_size = params.size;
        self.distance = params.distance;
        self
    }

    pub fn vectors_config(mut self, params: VectorParams) -> Self {
        self.vector_size = params.size;
        self.distance = format!("{:?}", params.distance);
        self
    }

    pub async fn build(self, client: &QdrantClient) -> Result<()> {
        client
            .create_collection(&self.name, self.vector_size, &self.distance)
            .await
    }
}

/// Builder for vector parameters
pub struct VectorParamsBuilder {
    size: u64,
    distance: String,
}

impl VectorParamsBuilder {
    pub fn new(size: u64, distance: Distance) -> Self {
        Self {
            size,
            distance: format!("{:?}", distance),
        }
    }
}

/// Distance metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Distance {
    #[default]
    Cosine,
    Euclid,
    Dot,
    Manhattan,
}

impl std::fmt::Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distance::Cosine => write!(f, "Cosine"),
            Distance::Euclid => write!(f, "Euclid"),
            Distance::Dot => write!(f, "Dot"),
            Distance::Manhattan => write!(f, "Manhattan"),
        }
    }
}

/// Point structure for Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointStruct {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: serde_json::Map<String, Value>,
}

impl PointStruct {
    pub fn new(id: String, vector: Vec<f32>, payload: serde_json::Map<String, Value>) -> Self {
        Self { id, vector, payload }
    }
}

/// Filter for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub must: Vec<Condition>,
}

impl Filter {
    pub fn must(conditions: Vec<Condition>) -> Self {
        Self { must: conditions }
    }
}

/// Search condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(flatten)]
    pub condition_type: ConditionType,
}

impl Condition {
    pub fn matches(field: &str, value: Value) -> Self {
        Self {
            condition_type: ConditionType::Field {
                key: field.to_string(),
                value,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionType {
    Field { key: String, value: Value },
}

/// Builder for upsert points
pub struct UpsertPointsBuilder {
    collection_name: String,
    points: Vec<Value>,
}

impl UpsertPointsBuilder {
    pub fn new(collection_name: &str, points: Vec<PointStruct>) -> Self {
        let points: Vec<Value> = points
            .into_iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "vector": p.vector,
                    "payload": p.payload
                })
            })
            .collect();

        Self {
            collection_name: collection_name.to_string(),
            points,
        }
    }

    pub async fn build(self, client: &QdrantClient) -> Result<()> {
        client.upsert_points(&self.collection_name, self.points).await
    }
}

/// Builder for search points
pub struct SearchPointsBuilder {
    collection_name: String,
    vector: Vec<f32>,
    limit: usize,
    filter: Option<Value>,
}

impl SearchPointsBuilder {
    pub fn new(collection_name: &str, vector: Vec<f32>, limit: usize) -> Self {
        Self {
            collection_name: collection_name.to_string(),
            vector,
            limit,
            filter: None,
        }
    }

    pub fn filter(mut self, filter: Option<Filter>) -> Self {
        self.filter = filter.map(|f| json!(f));
        self
    }

    pub fn with_payload(self, _with: bool) -> Self {
        self
    }

    pub async fn build(self, client: &QdrantClient) -> Result<SearchResponse> {
        let points = client
            .search_points(&self.collection_name, &self.vector, self.limit, self.filter)
            .await?;
        
        Ok(SearchResponse {
            result: points,
        })
    }
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub result: Vec<Value>,
}

/// Payload type alias
pub type Payload = serde_json::Map<String, serde_json::Value>;

/// Point ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointId {
    pub id: String,
}

impl PointId {
    pub fn from(id: String) -> Self {
        Self { id }
    }
}

/// Vector parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorParams {
    pub size: u64,
    pub distance: Distance,
}

/// Delete points builder
pub struct DeletePointsBuilder {
    collection_name: String,
    points: Vec<PointId>,
}

impl DeletePointsBuilder {
    pub fn new(collection_name: &str) -> Self {
        Self {
            collection_name: collection_name.to_string(),
            points: Vec::new(),
        }
    }

    pub fn points(mut self, point_ids: Vec<PointId>) -> Self {
        self.points = point_ids;
        self
    }

    pub async fn build(self, client: &QdrantClient) -> Result<()> {
        let point_ids: Vec<String> = self.points.into_iter().map(|p| p.id).collect();
        client.delete_points(&self.collection_name, point_ids).await
    }
}
