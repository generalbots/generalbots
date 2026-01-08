use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekognitionConfig {
    pub region: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub collection_id: Option<String>,
    pub max_faces: u32,
    pub quality_filter: QualityFilter,
    pub face_match_threshold: f32,
    pub use_face_liveness: bool,
    pub enable_age_range: bool,
    pub enable_emotions: bool,
    pub enable_smile_detection: bool,
    pub enable_eyeglasses_detection: bool,
    pub enable_sunglasses_detection: bool,
    pub enable_gender_detection: bool,
    pub enable_beard_detection: bool,
    pub enable_mustache_detection: bool,
    pub enable_eyes_open_detection: bool,
    pub enable_mouth_open_detection: bool,
    pub enable_face_occluded_detection: bool,
}

impl Default for RekognitionConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            access_key_id: None,
            secret_access_key: None,
            collection_id: None,
            max_faces: 100,
            quality_filter: QualityFilter::Auto,
            face_match_threshold: 80.0,
            use_face_liveness: false,
            enable_age_range: true,
            enable_emotions: true,
            enable_smile_detection: true,
            enable_eyeglasses_detection: true,
            enable_sunglasses_detection: true,
            enable_gender_detection: true,
            enable_beard_detection: true,
            enable_mustache_detection: true,
            enable_eyes_open_detection: true,
            enable_mouth_open_detection: true,
            enable_face_occluded_detection: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QualityFilter {
    None,
    Auto,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekognitionFace {
    pub face_id: Option<String>,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub image_id: Option<String>,
    pub external_image_id: Option<String>,
    pub landmarks: Vec<Landmark>,
    pub pose: Pose,
    pub quality: FaceQuality,
    pub age_range: Option<AgeRange>,
    pub smile: Option<Smile>,
    pub eyeglasses: Option<Eyeglasses>,
    pub sunglasses: Option<Sunglasses>,
    pub gender: Option<Gender>,
    pub beard: Option<Beard>,
    pub mustache: Option<Mustache>,
    pub eyes_open: Option<EyesOpen>,
    pub mouth_open: Option<MouthOpen>,
    pub emotions: Vec<Emotion>,
    pub face_occluded: Option<FaceOccluded>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub width: f32,
    pub height: f32,
    pub left: f32,
    pub top: f32,
}

impl BoundingBox {
    pub fn to_absolute(&self, image_width: u32, image_height: u32) -> AbsoluteBoundingBox {
        AbsoluteBoundingBox {
            x: (self.left * image_width as f32) as u32,
            y: (self.top * image_height as f32) as u32,
            width: (self.width * image_width as f32) as u32,
            height: (self.height * image_height as f32) as u32,
        }
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    pub fn center(&self) -> (f32, f32) {
        (self.left + self.width / 2.0, self.top + self.height / 2.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsoluteBoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landmark {
    pub landmark_type: LandmarkType,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LandmarkType {
    EyeLeft,
    EyeRight,
    MouthLeft,
    MouthRight,
    Nose,
    LeftEyeBrowLeft,
    LeftEyeBrowRight,
    LeftEyeBrowUp,
    RightEyeBrowLeft,
    RightEyeBrowRight,
    RightEyeBrowUp,
    LeftEyeLeft,
    LeftEyeRight,
    LeftEyeUp,
    LeftEyeDown,
    RightEyeLeft,
    RightEyeRight,
    RightEyeUp,
    RightEyeDown,
    NoseLeft,
    NoseRight,
    MouthUp,
    MouthDown,
    LeftPupil,
    RightPupil,
    UpperJawlineLeft,
    MidJawlineLeft,
    ChinBottom,
    MidJawlineRight,
    UpperJawlineRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub roll: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Pose {
    pub fn is_frontal(&self, threshold: f32) -> bool {
        self.yaw.abs() < threshold && self.pitch.abs() < threshold && self.roll.abs() < threshold
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceQuality {
    pub brightness: f32,
    pub sharpness: f32,
}

impl FaceQuality {
    pub fn is_acceptable(&self, min_brightness: f32, min_sharpness: f32) -> bool {
        self.brightness >= min_brightness && self.sharpness >= min_sharpness
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeRange {
    pub low: u32,
    pub high: u32,
}

impl AgeRange {
    pub fn midpoint(&self) -> u32 {
        (self.low + self.high) / 2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Smile {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eyeglasses {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunglasses {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gender {
    pub value: GenderValue,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenderValue {
    Male,
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beard {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mustache {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyesOpen {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouthOpen {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceOccluded {
    pub value: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emotion {
    pub emotion_type: EmotionType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmotionType {
    Happy,
    Sad,
    Angry,
    Confused,
    Disgusted,
    Surprised,
    Calm,
    Fear,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceCollection {
    pub collection_id: String,
    pub collection_arn: Option<String>,
    pub face_count: u64,
    pub face_model_version: String,
    pub creation_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFace {
    pub face_id: String,
    pub bounding_box: BoundingBox,
    pub image_id: String,
    pub external_image_id: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatch {
    pub similarity: f32,
    pub face: RekognitionFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareFacesMatch {
    pub similarity: f32,
    pub face: RekognitionFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectFacesRequest {
    pub image_bytes: Vec<u8>,
    pub attributes: Vec<FaceAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FaceAttribute {
    Default,
    All,
    AgeRange,
    Beard,
    Emotions,
    Eyeglasses,
    EyesOpen,
    Gender,
    MouthOpen,
    Mustache,
    Smile,
    Sunglasses,
    FaceOccluded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectFacesResponse {
    pub faces: Vec<RekognitionFace>,
    pub orientation_correction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFacesRequest {
    pub collection_id: String,
    pub image_bytes: Vec<u8>,
    pub external_image_id: Option<String>,
    pub max_faces: Option<u32>,
    pub quality_filter: Option<QualityFilter>,
    pub detection_attributes: Vec<FaceAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexFacesResponse {
    pub face_records: Vec<FaceRecord>,
    pub orientation_correction: Option<String>,
    pub face_model_version: String,
    pub unindexed_faces: Vec<UnindexedFace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceRecord {
    pub face: IndexedFace,
    pub face_detail: RekognitionFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnindexedFace {
    pub reasons: Vec<UnindexedFaceReason>,
    pub face_detail: RekognitionFace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UnindexedFaceReason {
    ExceedMaxFaces,
    ExtremePose,
    LowBrightness,
    LowSharpness,
    LowConfidence,
    SmallBoundingBox,
    LowFaceQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacesRequest {
    pub collection_id: String,
    pub face_id: String,
    pub max_faces: Option<u32>,
    pub face_match_threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacesResponse {
    pub searched_face_id: String,
    pub face_matches: Vec<FaceMatch>,
    pub face_model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacesByImageRequest {
    pub collection_id: String,
    pub image_bytes: Vec<u8>,
    pub max_faces: Option<u32>,
    pub face_match_threshold: Option<f32>,
    pub quality_filter: Option<QualityFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacesByImageResponse {
    pub searched_face_bounding_box: BoundingBox,
    pub searched_face_confidence: f32,
    pub face_matches: Vec<FaceMatch>,
    pub face_model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareFacesRequest {
    pub source_image_bytes: Vec<u8>,
    pub target_image_bytes: Vec<u8>,
    pub similarity_threshold: Option<f32>,
    pub quality_filter: Option<QualityFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareFacesResponse {
    pub source_image_face: Option<RekognitionFace>,
    pub face_matches: Vec<CompareFacesMatch>,
    pub unmatched_faces: Vec<RekognitionFace>,
    pub source_image_orientation_correction: Option<String>,
    pub target_image_orientation_correction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFacesRequest {
    pub collection_id: String,
    pub face_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFacesResponse {
    pub deleted_faces: Vec<String>,
    pub undeleted_faces: Vec<UnsuccessfulFaceDeletion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsuccessfulFaceDeletion {
    pub face_id: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessSessionRequest {
    pub settings: Option<LivenessSettings>,
    pub client_request_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessSettings {
    pub output_config: Option<LivenessOutputConfig>,
    pub audit_images_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessOutputConfig {
    pub s3_bucket: String,
    pub s3_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessSessionResponse {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFaceLivenessSessionResultsResponse {
    pub session_id: String,
    pub status: LivenessSessionStatus,
    pub confidence: Option<f32>,
    pub reference_image: Option<AuditImage>,
    pub audit_images: Vec<AuditImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LivenessSessionStatus {
    Created,
    InProgress,
    Succeeded,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditImage {
    pub bytes: Option<Vec<u8>>,
    pub s3_object: Option<S3Object>,
    pub bounding_box: Option<BoundingBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Object {
    pub bucket: String,
    pub name: String,
    pub version: Option<String>,
}

pub struct RekognitionService {
    config: Arc<RwLock<RekognitionConfig>>,
    collections: Arc<RwLock<HashMap<String, FaceCollection>>>,
    indexed_faces: Arc<RwLock<HashMap<String, Vec<IndexedFace>>>>,
    face_details: Arc<RwLock<HashMap<String, RekognitionFace>>>,
    liveness_sessions: Arc<RwLock<HashMap<String, GetFaceLivenessSessionResultsResponse>>>,
}

impl RekognitionService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(RekognitionConfig::default())),
            collections: Arc::new(RwLock::new(HashMap::new())),
            indexed_faces: Arc::new(RwLock::new(HashMap::new())),
            face_details: Arc::new(RwLock::new(HashMap::new())),
            liveness_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_config(config: RekognitionConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            collections: Arc::new(RwLock::new(HashMap::new())),
            indexed_faces: Arc::new(RwLock::new(HashMap::new())),
            face_details: Arc::new(RwLock::new(HashMap::new())),
            liveness_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn detect_faces(&self, request: DetectFacesRequest) -> Result<DetectFacesResponse, RekognitionError> {
        if request.image_bytes.is_empty() {
            return Err(RekognitionError::InvalidImage("Image bytes cannot be empty".to_string()));
        }

        let config = self.config.read().await;
        let include_all = request.attributes.contains(&FaceAttribute::All);

        let mut faces = Vec::new();
        let simulated_face_count = 1;

        for _ in 0..simulated_face_count {
            let face = self.create_simulated_face(&config, include_all);
            faces.push(face);
        }

        Ok(DetectFacesResponse {
            faces,
            orientation_correction: Some("ROTATE_0".to_string()),
        })
    }

    fn create_simulated_face(&self, config: &RekognitionConfig, include_all: bool) -> RekognitionFace {
        let landmarks = vec![
            Landmark { landmark_type: LandmarkType::EyeLeft, x: 0.35, y: 0.35 },
            Landmark { landmark_type: LandmarkType::EyeRight, x: 0.65, y: 0.35 },
            Landmark { landmark_type: LandmarkType::Nose, x: 0.5, y: 0.55 },
            Landmark { landmark_type: LandmarkType::MouthLeft, x: 0.38, y: 0.75 },
            Landmark { landmark_type: LandmarkType::MouthRight, x: 0.62, y: 0.75 },
        ];

        let age_range = if include_all || config.enable_age_range {
            Some(AgeRange { low: 25, high: 35 })
        } else {
            None
        };

        let smile = if include_all || config.enable_smile_detection {
            Some(Smile { value: true, confidence: 95.5 })
        } else {
            None
        };

        let eyeglasses = if include_all || config.enable_eyeglasses_detection {
            Some(Eyeglasses { value: false, confidence: 98.2 })
        } else {
            None
        };

        let sunglasses = if include_all || config.enable_sunglasses_detection {
            Some(Sunglasses { value: false, confidence: 99.1 })
        } else {
            None
        };

        let gender = if include_all || config.enable_gender_detection {
            Some(Gender {
                value: GenderValue::Male,
                confidence: 97.8,
            })
        } else {
            None
        };

        let beard = if include_all || config.enable_beard_detection {
            Some(Beard { value: false, confidence: 96.3 })
        } else {
            None
        };

        let mustache = if include_all || config.enable_mustache_detection {
            Some(Mustache { value: false, confidence: 97.1 })
        } else {
            None
        };

        let eyes_open = if include_all || config.enable_eyes_open_detection {
            Some(EyesOpen { value: true, confidence: 99.5 })
        } else {
            None
        };

        let mouth_open = if include_all || config.enable_mouth_open_detection {
            Some(MouthOpen { value: false, confidence: 98.7 })
        } else {
            None
        };

        let emotions = if include_all || config.enable_emotions {
            vec![
                Emotion { emotion_type: EmotionType::Happy, confidence: 85.2 },
                Emotion { emotion_type: EmotionType::Calm, confidence: 12.5 },
                Emotion { emotion_type: EmotionType::Surprised, confidence: 2.3 },
            ]
        } else {
            Vec::new()
        };

        let face_occluded = if include_all || config.enable_face_occluded_detection {
            Some(FaceOccluded { value: false, confidence: 98.9 })
        } else {
            None
        };

        RekognitionFace {
            face_id: Some(Uuid::new_v4().to_string()),
            bounding_box: BoundingBox {
                width: 0.25,
                height: 0.35,
                left: 0.35,
                top: 0.15,
            },
            confidence: 99.8,
            image_id: Some(Uuid::new_v4().to_string()),
            external_image_id: None,
            landmarks,
            pose: Pose {
                roll: 2.5,
                yaw: -5.0,
                pitch: 3.2,
            },
            quality: FaceQuality {
                brightness: 85.5,
                sharpness: 92.3,
            },
            age_range,
            smile,
            eyeglasses,
            sunglasses,
            gender,
            beard,
            mustache,
            eyes_open,
            mouth_open,
            emotions,
            face_occluded,
        }
    }

    pub async fn create_collection(&self, collection_id: &str) -> Result<FaceCollection, RekognitionError> {
        let mut collections = self.collections.write().await;

        if collections.contains_key(collection_id) {
            return Err(RekognitionError::CollectionAlreadyExists(collection_id.to_string()));
        }

        let collection = FaceCollection {
            collection_id: collection_id.to_string(),
            collection_arn: Some(format!("arn:aws:rekognition:us-east-1:123456789012:collection/{collection_id}")),
            face_count: 0,
            face_model_version: "6.0".to_string(),
            creation_timestamp: Some(Utc::now()),
        };

        collections.insert(collection_id.to_string(), collection.clone());

        let mut indexed_faces = self.indexed_faces.write().await;
        indexed_faces.insert(collection_id.to_string(), Vec::new());

        Ok(collection)
    }

    pub async fn delete_collection(&self, collection_id: &str) -> Result<(), RekognitionError> {
        let mut collections = self.collections.write().await;

        if collections.remove(collection_id).is_none() {
            return Err(RekognitionError::CollectionNotFound(collection_id.to_string()));
        }

        let mut indexed_faces = self.indexed_faces.write().await;
        indexed_faces.remove(collection_id);

        Ok(())
    }

    pub async fn list_collections(&self) -> Vec<FaceCollection> {
        let collections = self.collections.read().await;
        collections.values().cloned().collect()
    }

    pub async fn describe_collection(&self, collection_id: &str) -> Result<FaceCollection, RekognitionError> {
        let collections = self.collections.read().await;

        collections
            .get(collection_id)
            .cloned()
            .ok_or_else(|| RekognitionError::CollectionNotFound(collection_id.to_string()))
    }

    pub async fn index_faces(&self, request: IndexFacesRequest) -> Result<IndexFacesResponse, RekognitionError> {
        if request.image_bytes.is_empty() {
            return Err(RekognitionError::InvalidImage("Image bytes cannot be empty".to_string()));
        }

        let mut collections = self.collections.write().await;
        let collection = collections
            .get_mut(&request.collection_id)
            .ok_or_else(|| RekognitionError::CollectionNotFound(request.collection_id.clone()))?;

        let config = self.config.read().await;
        let max_faces = request.max_faces.unwrap_or(config.max_faces);

        let mut face_records = Vec::new();
        let simulated_face_count = 1.min(max_faces as usize);

        for _ in 0..simulated_face_count {
            let face_id = Uuid::new_v4().to_string();
            let image_id = Uuid::new_v4().to_string();

            let indexed_face = IndexedFace {
                face_id: face_id.clone(),
                bounding_box: BoundingBox {
                    width: 0.25,
                    height: 0.35,
                    left: 0.35,
                    top: 0.15,
                },
                image_id: image_id.clone(),
                external_image_id: request.external_image_id.clone(),
                confidence: 99.8,
            };

            let face_detail = self.create_simulated_face(&config, true);

            face_records.push(FaceRecord {
                face: indexed_face.clone(),
                face_detail: face_detail.clone(),
            });

            let mut indexed_faces = self.indexed_faces.write().await;
            if let Some(faces) = indexed_faces.get_mut(&request.collection_id) {
                faces.push(indexed_face);
            }

            let mut face_details = self.face_details.write().await;
            face_details.insert(face_id, face_detail);
        }

        collection.face_count += face_records.len() as u64;

        Ok(IndexFacesResponse {
            face_records,
            orientation_correction: Some("ROTATE_0".to_string()),
            face_model_version: "6.0".to_string(),
            unindexed_faces: Vec::new(),
        })
    }

    pub async fn search_faces(&self, request: SearchFacesRequest) -> Result<SearchFacesResponse, RekognitionError> {
        let indexed_faces = self.indexed_faces.read().await;
        let collection_faces = indexed_faces
            .get(&request.collection_id)
            .ok_or_else(|| RekognitionError::CollectionNotFound(request.collection_id.clone()))?;

        if !collection_faces.iter().any(|f| f.face_id == request.face_id) {
            return Err(RekognitionError::FaceNotFound(request.face_id.clone()));
        }

        let config = self.config.read().await;
        let threshold = request.face_match_threshold.unwrap_or(config.face_match_threshold);
        let max_faces = request.max_faces.unwrap_or(10);

        let mut face_matches = Vec::new();

        for face in collection_faces.iter().filter(|f| f.face_id != request.face_id) {
            let similarity = 85.0 + (rand_float() * 15.0);

            if similarity >= threshold {
                let face_details = self.face_details.read().await;
                if let Some(detail) = face_details.get(&face.face_id) {
                    face_matches.push(FaceMatch {
                        similarity,
                        face: detail.clone(),
                    });
                }
            }
        }

        face_matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        face_matches.truncate(max_faces as usize);

        Ok(SearchFacesResponse {
            searched_face_id: request.face_id,
            face_matches,
            face_model_version: "6.0".to_string(),
        })
    }

    pub async fn search_faces_by_image(&self, request: SearchFacesByImageRequest) -> Result<SearchFacesByImageResponse, RekognitionError> {
        if request.image_bytes.is_empty() {
            return Err(RekognitionError::InvalidImage("Image bytes cannot be empty".to_string()));
        }

        let indexed_faces = self.indexed_faces.read().await;
        let collection_faces = indexed_faces
            .get(&request.collection_id)
            .ok_or_else(|| RekognitionError::CollectionNotFound(request.collection_id.clone()))?;

        let config = self.config.read().await;
        let threshold = request.face_match_threshold.unwrap_or(config.face_match_threshold);
        let max_faces = request.max_faces.unwrap_or(10);

        let searched_face_bounding_box = BoundingBox {
            width: 0.25,
            height: 0.35,
            left: 0.35,
            top: 0.15,
        };

        let mut face_matches = Vec::new();

        for face in collection_faces.iter() {
            let similarity = 80.0 + (rand_float() * 20.0);

            if similarity >= threshold {
                let face_details = self.face_details.read().await;
                if let Some(detail) = face_details.get(&face.face_id) {
                    face_matches.push(FaceMatch {
                        similarity,
                        face: detail.clone(),
                    });
                }
            }
        }

        face_matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        face_matches.truncate(max_faces as usize);

        Ok(SearchFacesByImageResponse {
            searched_face_bounding_box,
            face_matches,
            face_model_version: "6.0".to_string(),
        })
    }
}

fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f32 / 1000.0
}
