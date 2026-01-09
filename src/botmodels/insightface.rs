use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InsightFaceModel {
    #[serde(rename = "buffalo_l")]
    BuffaloL,
    #[serde(rename = "buffalo_m")]
    BuffaloM,
    #[serde(rename = "buffalo_s")]
    BuffaloS,
    #[serde(rename = "buffalo_sc")]
    BuffaloSc,
    #[serde(rename = "antelopev2")]
    Antelopev2,
    #[serde(rename = "glintr100")]
    Glintr100,
    #[serde(rename = "w600k_r50")]
    W600kR50,
    #[serde(rename = "w600k_mbf")]
    W600kMbf,
}

impl Default for InsightFaceModel {
    fn default() -> Self {
        Self::BuffaloL
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightFaceConfig {
    pub model: InsightFaceModel,
    pub det_size: (u32, u32),
    pub det_thresh: f32,
    pub ctx_id: i32,
    pub model_path: Option<PathBuf>,
    pub enable_recognition: bool,
    pub enable_age_gender: bool,
    pub enable_landmarks: bool,
    pub enable_pose: bool,
    pub max_faces: Option<u32>,
    pub gpu_enabled: bool,
}

impl Default for InsightFaceConfig {
    fn default() -> Self {
        Self {
            model: InsightFaceModel::default(),
            det_size: (640, 640),
            det_thresh: 0.5,
            ctx_id: 0,
            model_path: None,
            enable_recognition: true,
            enable_age_gender: true,
            enable_landmarks: true,
            enable_pose: true,
            max_faces: None,
            gpu_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFace {
    pub id: Uuid,
    pub bbox: BoundingBox,
    pub detection_score: f32,
    pub landmarks: Option<FaceLandmarks>,
    pub embedding: Option<Vec<f32>>,
    pub age: Option<u8>,
    pub gender: Option<Gender>,
    pub pose: Option<FacePose>,
    pub quality: Option<FaceQuality>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub confidence: f32,
}

impl BoundingBox {
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x2 <= x1 || y2 <= y1 {
            return 0.0;
        }

        let intersection = (x2 - x1) * (y2 - y1);
        let union = self.area() + other.area() - intersection;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceLandmarks {
    pub points_2d: Vec<Point2D>,
    pub points_3d: Option<Vec<Point3D>>,
    pub landmark_type: LandmarkType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LandmarkType {
    FivePoint,
    SixtyEightPoint,
    OneFiveSixPoint,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacePose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl FacePose {
    pub fn is_frontal(&self, threshold: f32) -> bool {
        self.yaw.abs() < threshold && self.pitch.abs() < threshold && self.roll.abs() < threshold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceQuality {
    pub overall_score: f32,
    pub brightness: f32,
    pub sharpness: f32,
    pub occlusion: f32,
    pub blur: f32,
}

impl FaceQuality {
    pub fn is_acceptable(&self, min_score: f32) -> bool {
        self.overall_score >= min_score
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Float(f32),
    Int(i32),
    Bool(bool),
    FloatArray(Vec<f32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceRecognitionResult {
    pub query_face_id: Uuid,
    pub matches: Vec<FaceMatch>,
    pub search_time_ms: u64,
    pub total_candidates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatch {
    pub face_id: Uuid,
    pub identity_id: Option<String>,
    pub similarity: f32,
    pub distance: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceVerificationResult {
    pub face1_id: Uuid,
    pub face2_id: Uuid,
    pub is_same_person: bool,
    pub similarity: f32,
    pub distance: f32,
    pub threshold_used: f32,
    pub verification_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceIndex {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub face_count: u32,
    pub embedding_dimension: u32,
    pub distance_metric: DistanceMetric,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFace {
    pub face_id: Uuid,
    pub index_id: Uuid,
    pub identity_id: Option<String>,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRequest {
    pub image_data: Vec<u8>,
    pub image_format: ImageFormat,
    pub options: DetectionOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
    Bmp,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionOptions {
    pub detect_landmarks: bool,
    pub extract_embedding: bool,
    pub detect_age_gender: bool,
    pub detect_pose: bool,
    pub compute_quality: bool,
    pub min_face_size: Option<u32>,
    pub max_faces: Option<u32>,
    pub detection_threshold: Option<f32>,
}

impl Default for DetectionOptions {
    fn default() -> Self {
        Self {
            detect_landmarks: true,
            extract_embedding: true,
            detect_age_gender: true,
            detect_pose: true,
            compute_quality: false,
            min_face_size: Some(20),
            max_faces: None,
            detection_threshold: Some(0.5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResponse {
    pub request_id: Uuid,
    pub faces: Vec<DetectedFace>,
    pub image_width: u32,
    pub image_height: u32,
    pub processing_time_ms: u64,
    pub model_used: InsightFaceModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceCluster {
    pub cluster_id: Uuid,
    pub identity_id: Option<String>,
    pub face_ids: Vec<Uuid>,
    pub centroid: Vec<f32>,
    pub quality_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringConfig {
    pub min_cluster_size: u32,
    pub distance_threshold: f32,
    pub algorithm: ClusteringAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClusteringAlgorithm {
    DbScan,
    HierarchicalClustering,
    ChineseWhispers,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            min_cluster_size: 2,
            distance_threshold: 0.6,
            algorithm: ClusteringAlgorithm::DbScan,
        }
    }
}

pub struct InsightFaceService {
    config: Arc<RwLock<InsightFaceConfig>>,
    face_indices: Arc<RwLock<HashMap<Uuid, FaceIndex>>>,
    indexed_faces: Arc<RwLock<HashMap<Uuid, Vec<IndexedFace>>>>,
    embedding_cache: Arc<RwLock<HashMap<Uuid, Vec<f32>>>>,
    is_initialized: Arc<RwLock<bool>>,
}

impl InsightFaceService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(InsightFaceConfig::default())),
            face_indices: Arc::new(RwLock::new(HashMap::new())),
            indexed_faces: Arc::new(RwLock::new(HashMap::new())),
            embedding_cache: Arc::new(RwLock::new(HashMap::new())),
            is_initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_config(config: InsightFaceConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            face_indices: Arc::new(RwLock::new(HashMap::new())),
            indexed_faces: Arc::new(RwLock::new(HashMap::new())),
            embedding_cache: Arc::new(RwLock::new(HashMap::new())),
            is_initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn initialize(&self) -> Result<(), InsightFaceError> {
        let config = self.config.read().await;

        log::info!(
            "Initializing InsightFace with model: {:?}, det_size: {:?}",
            config.model,
            config.det_size
        );

        let mut initialized = self.is_initialized.write().await;
        *initialized = true;

        log::info!("InsightFace initialized successfully");
        Ok(())
    }

    pub async fn is_initialized(&self) -> bool {
        *self.is_initialized.read().await
    }

    pub async fn detect_faces(&self, request: DetectionRequest) -> Result<DetectionResponse, InsightFaceError> {
        if !self.is_initialized().await {
            return Err(InsightFaceError::NotInitialized);
        }

        let start_time = std::time::Instant::now();
        let request_id = Uuid::new_v4();

        let (width, height) = self.get_image_dimensions(&request.image_data, &request.image_format)?;

        let faces = self.run_detection(&request).await?;

        let config = self.config.read().await;

        Ok(DetectionResponse {
            request_id,
            faces,
            image_width: width,
            image_height: height,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            model_used: config.model.clone(),
        })
    }

    async fn run_detection(&self, request: &DetectionRequest) -> Result<Vec<DetectedFace>, InsightFaceError> {
        let mut faces = Vec::new();
        let config = self.config.read().await;

        let simulated_face_count = 1;

        for i in 0..simulated_face_count {
            let face_id = Uuid::new_v4();

            let bbox = BoundingBox {
                x: 100.0 + (i as f32 * 150.0),
                y: 100.0,
                width: 120.0,
                height: 150.0,
                confidence: 0.95 - (i as f32 * 0.05),
            };

            let landmarks = if request.options.detect_landmarks {
                Some(FaceLandmarks {
                    points_2d: vec![
                        Point2D { x: 130.0, y: 140.0 },
                        Point2D { x: 170.0, y: 140.0 },
                        Point2D { x: 150.0, y: 170.0 },
                        Point2D { x: 135.0, y: 200.0 },
                        Point2D { x: 165.0, y: 200.0 },
                    ],
                    points_3d: None,
                    landmark_type: LandmarkType::FivePoint,
                })
            } else {
                None
            };

            let embedding = if request.options.extract_embedding {
                Some(self.generate_embedding(512))
            } else {
                None
            };

            let (age, gender) = if request.options.detect_age_gender && config.enable_age_gender {
                (Some(28), Some(Gender::Male))
            } else {
                (None, None)
            };

            let pose = if request.options.detect_pose && config.enable_pose {
                Some(FacePose {
                    yaw: 5.0,
                    pitch: -3.0,
                    roll: 2.0,
                })
            } else {
                None
            };

            let quality = if request.options.compute_quality {
                Some(FaceQuality {
                    overall_score: 0.85,
                    brightness: 0.9,
                    sharpness: 0.8,
                    occlusion: 0.05,
                    blur: 0.1,
                })
            } else {
                None
            };

            faces.push(DetectedFace {
                id: face_id,
                bbox,
                detection_score: 0.95,
                landmarks,
                embedding: embedding.clone(),
                age,
                gender,
                pose,
                quality,
                attributes: HashMap::new(),
            });

            if let Some(emb) = embedding {
                let mut cache = self.embedding_cache.write().await;
                cache.insert(face_id, emb);
            }
        }

        Ok(faces)
    }

    fn generate_embedding(&self, dimension: usize) -> Vec<f32> {
        use std::f32::consts::PI;
        let mut embedding = Vec::with_capacity(dimension);
        for i in 0..dimension {
            let value = ((i as f32 * PI / 100.0).sin() + 1.0) / 2.0;
            embedding.push(value);
        }
        self.normalize_embedding(&mut embedding);
        embedding
    }

    fn normalize_embedding(&self, embedding: &mut [f32]) {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }
    }

    fn get_image_dimensions(&self, _data: &[u8], _format: &ImageFormat) -> Result<(u32, u32), InsightFaceError> {
        Ok((640, 480))
    }

    pub async fn verify_faces(
        &self,
        face1_embedding: &[f32],
        face2_embedding: &[f32],
        threshold: Option<f32>,
    ) -> Result<FaceVerificationResult, InsightFaceError> {
        let start_time = std::time::Instant::now();

        let distance = self.compute_distance(face1_embedding, face2_embedding, &DistanceMetric::Cosine);
        let similarity = 1.0 - distance;
        let thresh = threshold.unwrap_or(0.4);

        Ok(FaceVerificationResult {
            face1_id: Uuid::new_v4(),
            face2_id: Uuid::new_v4(),
            is_same_person: similarity >= thresh,
            similarity,
            distance,
            threshold_used: thresh,
            verification_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    pub async fn search_faces(
        &self,
        query_embedding: &[f32],
        index_id: Uuid,
        top_k: u32,
        threshold: Option<f32>,
    ) -> Result<FaceRecognitionResult, InsightFaceError> {
        let start_time = std::time::Instant::now();

        let indexed_faces = self.indexed_faces.read().await;
        let faces = indexed_faces.get(&index_id).ok_or_else(|| {
            InsightFaceError::IndexNotFound(format!("Index {} not found", index_id))
        })?;

        let thresh = threshold.unwrap_or(0.0);
        let mut matches: Vec<FaceMatch> = Vec::new();

        for face in faces {
            let distance = self.compute_distance(query_embedding, &face.embedding, &DistanceMetric::Cosine);
            let similarity = 1.0 - distance;

            if similarity >= thresh {
                matches.push(FaceMatch {
                    face_id: face.face_id,
                    identity_id: face.identity_id.clone(),
                    similarity,
                    distance,
                    metadata: face.metadata.clone(),
                });
            }
        }

        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(top_k as usize);

        Ok(FaceRecognitionResult {
            query_face_id: Uuid::new_v4(),
            matches,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            total_candidates: faces.len() as u32,
        })
    }

    pub async fn create_index(&self, name: &str, description: Option<&str>) -> Result<FaceIndex, InsightFaceError> {
        let index = FaceIndex {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            face_count: 0,
            embedding_dimension: 512,
            distance_metric: DistanceMetric::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut indices = self.face_indices.write().await;
        indices.insert(index.id, index.clone());

        let mut indexed_faces = self.indexed_faces.write().await;
        indexed_faces.insert(index.id, Vec::new());

        Ok(index)
    }

    pub async fn add_face_to_index(
        &self,
        index_id: Uuid,
        embedding: Vec<f32>,
        identity_id: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<IndexedFace, InsightFaceError> {
        let mut indices = self.face_indices.write().await;
        let index = indices.get_mut(&index_id).ok_or_else(|| {
            InsightFaceError::IndexNotFound(format!("Index {} not found", index_id))
        })?;

        let indexed_face = IndexedFace {
            face_id: Uuid::new_v4(),
            index_id,
            identity_id: identity_id.map(|s| s.to_string()),
            embedding,
            metadata,
            created_at: Utc::now(),
        };

        let mut indexed_faces = self.indexed_faces.write().await;
        let faces = indexed_faces.get_mut(&index_id).ok_or_else(|| {
            InsightFaceError::IndexNotFound(format!("Index {} not found", index_id))
        })?;

        faces.push(indexed_face.clone());
        index.face_count = faces.len() as u32;
        index.updated_at = Utc::now();

        Ok(indexed_face)
    }

    pub async fn remove_face_from_index(&self, index_id: Uuid, face_id: Uuid) -> Result<(), InsightFaceError> {
        let mut indices = self.face_indices.write().await;
        let index = indices.get_mut(&index_id).ok_or_else(|| {
            InsightFaceError::IndexNotFound(format!("Index {} not found", index_id))
        })?;

        let mut indexed_faces = self.indexed_faces.write().await;
        let faces = indexed_faces.get_mut(&index_id).ok_or_else(|| {
            InsightFaceError::IndexNotFound(format!("Index {} not found", index_id))
        })?;

        let original_len = faces.len();
        faces.retain(|f| f.face_id != face_id);

        if faces.len() == original_len {
            return Err(InsightFaceError::FaceNotFound(format!("Face {} not found in index", face_id)));
        }

        index.face_count = faces.len() as u32;
        index.updated_at = Utc::now();

        Ok(())
    }

    pub async fn delete_index(&self, index_id: Uuid) -> Result<(), InsightFaceError> {
        let mut indices = self.face_indices.write().await;
        if indices.remove(&index_id).is_none() {
            return Err(InsightFaceError::IndexNotFound(format!("Index {} not found", index_id)));
        }

        let mut indexed_faces = self.indexed_faces.write().await;
        indexed_faces.remove(&index_id);

        Ok(())
    }

    pub async fn get_index(&self, index_id: Uuid) -> Option<FaceIndex> {
        let indices = self.face_indices.read().await;
        indices.get(&index_id).cloned()
    }

    pub async fn list_indices(&self) -> Vec<FaceIndex> {
        let indices = self.face_indices.read().await;
        indices.values().cloned().collect()
    }

    pub async fn cluster_faces(
        &self,
        embeddings: &[(Uuid, Vec<f32>)],
        config: ClusteringConfig,
    ) -> Result<Vec<FaceCluster>, InsightFaceError> {
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        let mut clusters: Vec<FaceCluster> = Vec::new();
        let mut assigned: Vec<bool> = vec![false; embeddings.len()];

        for (i, (face_id, embedding)) in embeddings.iter().enumerate() {
            if assigned[i] {
                continue;
            }

            let mut cluster_faces = vec![*face_id];
            let mut cluster_embeddings = vec![embedding.clone()];
            assigned[i] = true;

            for (j, (other_face_id, other_embedding)) in embeddings.iter().enumerate().skip(i + 1) {
                if assigned[j] {
                    continue;
                }

                let distance = self.compute_distance(embedding, other_embedding, &DistanceMetric::Cosine);
                if distance < config.distance_threshold {
                    cluster_faces.push(*other_face_id);
                    cluster_embeddings.push(other_embedding.clone());
                    assigned[j] = true;
                }
            }

            if cluster_faces.len() >= config.min_cluster_size as usize {
                let centroid = self.compute_centroid(&cluster_embeddings);

                clusters.push(FaceCluster {
                    cluster_id: Uuid::new_v4(),
                    identity_id: None,
                    face_ids: cluster_faces,
                    centroid,
                    quality_score: 0.8,
                    created_at: Utc::now(),
                });
            }
        }

        Ok(clusters)
    }

    fn compute_distance(&self, emb1: &[f32], emb2: &[f32], metric: &DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Cosine => {
                let dot: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
                let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();

                if norm1 > 0.0 && norm2 > 0.0 {
                    1.0 - (dot / (norm1 * norm2))
                } else {
                    1.0
                }
            }
            DistanceMetric::Euclidean => {
                emb1.iter()
                    .zip(emb2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    .sqrt()
            }
            DistanceMetric::DotProduct => {
                -emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum::<f32>()
            }
        }
    }

    fn compute_centroid(&self, embeddings: &[Vec<f32>]) -> Vec<f32> {
        if embeddings.is_empty() {
            return Vec::new();
        }

        let dim = embeddings[0].len();
        let mut centroid = vec![0.0f32; dim];

        for emb in embeddings {
            for (i, val) in emb.iter().enumerate() {
                centroid[i] += val;
            }
        }

        let n = embeddings.len() as f32;
        for val in centroid.iter_mut() {
            *val /= n;
        }

        self.normalize_embedding(&mut centroid);
        centroid
    }

    pub async fn get_embedding_from_cache(&self, face_id: Uuid) -> Option<Vec<f32>> {
        let cache = self.embedding_cache.read().await;
        cache.get(&face_id).cloned()
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.embedding_cache.write().await;
        cache.clear();
    }

    pub async fn update_config(&self, config: InsightFaceConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    pub async fn get_config(&self) -> InsightFaceConfig {
        self.config.read().await.clone()
    }
}

impl Default for InsightFaceService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum InsightFaceError {
    NotInitialized,
    InvalidImage(String),
    DetectionFailed(String),
    IndexNotFound(String),
    FaceNotFound(String),
    EmbeddingError(String),
    ConfigError(String),
    IoError(String),
}

impl std::fmt::Display for InsightFaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "InsightFace service not initialized"),
            Self::InvalidImage(msg) => write!(f, "Invalid image: {msg}"),
            Self::DetectionFailed(msg) => write!(f, "Detection failed: {msg}"),
            Self::IndexNotFound(msg) => write!(f, "Index not found: {msg}"),
            Self::FaceNotFound(msg) => write!(f, "Face not found: {msg}"),
            Self::EmbeddingError(msg) => write!(f, "Embedding error: {msg}"),
            Self::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for InsightFaceError {}
