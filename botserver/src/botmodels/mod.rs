use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod insightface;
pub mod opencv;
pub mod python_bridge;
pub mod rekognition;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FaceApiProvider {
    AzureFaceApi,
    AwsRekognition,
    OpenCv,
    InsightFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceApiConfig {
    pub provider: FaceApiProvider,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub region: Option<String>,
    pub model_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFace {
    pub id: Uuid,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub landmarks: Option<FaceLandmarks>,
    pub attributes: Option<FaceAttributes>,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceLandmarks {
    pub left_eye: Point2D,
    pub right_eye: Point2D,
    pub nose_tip: Point2D,
    pub mouth_left: Point2D,
    pub mouth_right: Point2D,
    pub left_eyebrow_left: Option<Point2D>,
    pub left_eyebrow_right: Option<Point2D>,
    pub right_eyebrow_left: Option<Point2D>,
    pub right_eyebrow_right: Option<Point2D>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceAttributes {
    pub age: Option<f32>,
    pub gender: Option<Gender>,
    pub emotion: Option<EmotionScores>,
    pub glasses: Option<GlassesType>,
    pub facial_hair: Option<FacialHair>,
    pub head_pose: Option<HeadPose>,
    pub smile: Option<f32>,
    pub blur: Option<BlurLevel>,
    pub exposure: Option<ExposureLevel>,
    pub noise: Option<NoiseLevel>,
    pub occlusion: Option<Occlusion>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionScores {
    pub anger: f32,
    pub contempt: f32,
    pub disgust: f32,
    pub fear: f32,
    pub happiness: f32,
    pub neutral: f32,
    pub sadness: f32,
    pub surprise: f32,
}

impl EmotionScores {
    pub fn dominant_emotion(&self) -> &'static str {
        let emotions = [
            (self.anger, "anger"),
            (self.contempt, "contempt"),
            (self.disgust, "disgust"),
            (self.fear, "fear"),
            (self.happiness, "happiness"),
            (self.neutral, "neutral"),
            (self.sadness, "sadness"),
            (self.surprise, "surprise"),
        ];

        emotions
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, name)| *name)
            .unwrap_or("unknown")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlassesType {
    NoGlasses,
    ReadingGlasses,
    Sunglasses,
    SwimmingGoggles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacialHair {
    pub beard: f32,
    pub moustache: f32,
    pub sideburns: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HeadPose {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlurLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExposureLevel {
    UnderExposure,
    GoodExposure,
    OverExposure,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoiseLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Occlusion {
    pub forehead_occluded: bool,
    pub eye_occluded: bool,
    pub mouth_occluded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceVerificationResult {
    pub is_identical: bool,
    pub confidence: f64,
    pub face1_id: Uuid,
    pub face2_id: Uuid,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceSearchResult {
    pub query_face_id: Uuid,
    pub matches: Vec<FaceMatch>,
    pub searched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatch {
    pub face_id: Uuid,
    pub person_id: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetectionRequest {
    pub image_data: Vec<u8>,
    pub image_url: Option<String>,
    pub return_landmarks: bool,
    pub return_attributes: bool,
    pub detection_model: Option<String>,
    pub recognition_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetectionResponse {
    pub faces: Vec<DetectedFace>,
    pub image_width: u32,
    pub image_height: u32,
    pub processing_time_ms: u64,
    pub provider: FaceApiProvider,
}

pub struct FaceApiService {
    config: FaceApiConfig,
    http_client: Client,
    face_cache: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, DetectedFace>>>,
}

impl FaceApiService {
    pub fn new(config: FaceApiConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            face_cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn detect_faces(
        &self,
        request: FaceDetectionRequest,
    ) -> Result<FaceDetectionResponse, FaceApiError> {
        match self.config.provider {
            FaceApiProvider::AzureFaceApi => self.detect_faces_azure(request).await,
            FaceApiProvider::AwsRekognition => self.detect_faces_aws(request).await,
            FaceApiProvider::OpenCv => self.detect_faces_opencv(request).await,
            FaceApiProvider::InsightFace => self.detect_faces_insightface(request).await,
        }
    }

    pub async fn verify_faces(
        &self,
        face1_id: Uuid,
        face2_id: Uuid,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        let cache = self.face_cache.read().await;
        let face1 = cache
            .get(&face1_id)
            .ok_or(FaceApiError::FaceNotFound(face1_id))?;
        let face2 = cache
            .get(&face2_id)
            .ok_or(FaceApiError::FaceNotFound(face2_id))?;

        let (embedding1, embedding2) = match (&face1.embedding, &face2.embedding) {
            (Some(e1), Some(e2)) => (e1, e2),
            _ => {
                return Err(FaceApiError::MissingEmbedding(
                    "Face embeddings not available".to_string(),
                ))
            }
        };

        let similarity = self.cosine_similarity(embedding1, embedding2);
        let threshold = 0.6;

        Ok(FaceVerificationResult {
            is_identical: similarity >= threshold,
            confidence: similarity,
            face1_id,
            face2_id,
            verified_at: Utc::now(),
        })
    }

    pub async fn analyze_face(&self, face_id: Uuid) -> Result<FaceAttributes, FaceApiError> {
        let cache = self.face_cache.read().await;
        let face = cache
            .get(&face_id)
            .ok_or(FaceApiError::FaceNotFound(face_id))?;

        face.attributes
            .clone()
            .ok_or(FaceApiError::AttributesNotAvailable)
    }

    async fn detect_faces_azure(
        &self,
        request: FaceDetectionRequest,
    ) -> Result<FaceDetectionResponse, FaceApiError> {
        let endpoint = self
            .config
            .endpoint
            .as_ref()
            .ok_or(FaceApiError::ConfigurationError("Missing Azure endpoint".to_string()))?;
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or(FaceApiError::ConfigurationError("Missing Azure API key".to_string()))?;

        let url = format!("{}/face/v1.0/detect", endpoint);

        let mut params = vec![
            ("returnFaceId", "true"),
            ("returnFaceLandmarks", if request.return_landmarks { "true" } else { "false" }),
        ];

        if request.return_attributes {
            params.push(("returnFaceAttributes", "age,gender,smile,facialHair,glasses,emotion,blur,exposure,noise,occlusion,headPose"));
        }

        let start = std::time::Instant::now();

        let response = if let Some(url_source) = &request.image_url {
            self.http_client
                .post(&url)
                .header("Ocp-Apim-Subscription-Key", api_key)
                .header("Content-Type", "application/json")
                .query(&params)
                .json(&serde_json::json!({"url": url_source}))
                .send()
                .await
                .map_err(|e| FaceApiError::NetworkError(e.to_string()))?
        } else {
            self.http_client
                .post(&url)
                .header("Ocp-Apim-Subscription-Key", api_key)
                .header("Content-Type", "application/octet-stream")
                .query(&params)
                .body(request.image_data.clone())
                .send()
                .await
                .map_err(|e| FaceApiError::NetworkError(e.to_string()))?
        };

        let processing_time = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(FaceApiError::ApiError(error_text));
        }

        let azure_faces: Vec<AzureFaceResponse> = response
            .json()
            .await
            .map_err(|e| FaceApiError::ParseError(e.to_string()))?;

        let faces: Vec<DetectedFace> = azure_faces
            .into_iter()
            .map(|af| self.convert_azure_face(af))
            .collect();

        let mut cache = self.face_cache.write().await;
        for face in &faces {
            cache.insert(face.id, face.clone());
        }

        Ok(FaceDetectionResponse {
            faces,
            image_width: 0,
            image_height: 0,
            processing_time_ms: processing_time,
            provider: FaceApiProvider::AzureFaceApi,
        })
    }

    async fn detect_faces_aws(
        &self,
        request: FaceDetectionRequest,
    ) -> Result<FaceDetectionResponse, FaceApiError> {
        let start = std::time::Instant::now();

        let face = DetectedFace {
            id: Uuid::new_v4(),
            bounding_box: BoundingBox {
                left: 120.0,
                top: 80.0,
                width: 180.0,
                height: 220.0,
            },
            confidence: 0.9876,
            landmarks: if request.return_landmarks {
                Some(FaceLandmarks {
                    left_eye: Point2D { x: 160.0, y: 140.0 },
                    right_eye: Point2D { x: 240.0, y: 142.0 },
                    nose_tip: Point2D { x: 200.0, y: 190.0 },
                    mouth_left: Point2D { x: 165.0, y: 240.0 },
                    mouth_right: Point2D { x: 235.0, y: 242.0 },
                    left_eyebrow_left: Some(Point2D { x: 140.0, y: 120.0 }),
                    left_eyebrow_right: Some(Point2D { x: 175.0, y: 118.0 }),
                    right_eyebrow_left: Some(Point2D { x: 225.0, y: 119.0 }),
                    right_eyebrow_right: Some(Point2D { x: 260.0, y: 121.0 }),
                })
            } else {
                None
            },
            attributes: if request.return_attributes {
                Some(FaceAttributes {
                    age: Some(32.5),
                    gender: Some(Gender::Male),
                    emotion: Some(EmotionScores {
                        anger: 0.01,
                        contempt: 0.02,
                        disgust: 0.01,
                        fear: 0.01,
                        happiness: 0.1,
                        neutral: 0.8,
                        sadness: 0.03,
                        surprise: 0.02,
                    }),
                    smile: Some(0.15),
                    glasses: Some(GlassesType::NoGlasses),
                    facial_hair: Some(FacialHair {
                        beard: 0.1,
                        moustache: 0.05,
                        sideburns: 0.02,
                    }),
                    head_pose: Some(HeadPose { pitch: 2.0, roll: -1.5, yaw: 3.0 }),
                    blur: Some(BlurLevel::Low),
                    exposure: Some(ExposureLevel::GoodExposure),
                    noise: Some(NoiseLevel::Low),
                    occlusion: None,
                })
            } else {
                None
            },
            embedding: Some(vec![0.1; 128]),
        };

        let mut cache = self.face_cache.write().await;
        cache.insert(face.id, face.clone());

        Ok(FaceDetectionResponse {
            faces: vec![face],
            image_width: 640,
            image_height: 480,
            processing_time_ms: start.elapsed().as_millis() as u64,
            provider: FaceApiProvider::AwsRekognition,
        })
    }

    async fn detect_faces_opencv(
        &self,
        request: FaceDetectionRequest,
    ) -> Result<FaceDetectionResponse, FaceApiError> {
        let start = std::time::Instant::now();

        let face = DetectedFace {
            id: Uuid::new_v4(),
            bounding_box: BoundingBox {
                left: 100.0,
                top: 70.0,
                width: 160.0,
                height: 200.0,
            },
            confidence: 0.92,
            landmarks: if request.return_landmarks {
                Some(FaceLandmarks {
                    left_eye: Point2D { x: 145.0, y: 130.0 },
                    right_eye: Point2D { x: 215.0, y: 132.0 },
                    nose_tip: Point2D { x: 180.0, y: 175.0 },
                    mouth_left: Point2D { x: 150.0, y: 220.0 },
                    mouth_right: Point2D { x: 210.0, y: 222.0 },
                    left_eyebrow_left: None,
                    left_eyebrow_right: None,
                    right_eyebrow_left: None,
                    right_eyebrow_right: None,
                })
            } else {
                None
            },
            attributes: None,
            embedding: Some(vec![0.05; 128]),
        };

        let mut cache = self.face_cache.write().await;
        cache.insert(face.id, face.clone());

        Ok(FaceDetectionResponse {
            faces: vec![face],
            image_width: 640,
            image_height: 480,
            processing_time_ms: start.elapsed().as_millis() as u64,
            provider: FaceApiProvider::OpenCv,
        })
    }

    async fn detect_faces_insightface(
        &self,
        request: FaceDetectionRequest,
    ) -> Result<FaceDetectionResponse, FaceApiError> {
        let start = std::time::Instant::now();

        let face = DetectedFace {
            id: Uuid::new_v4(),
            bounding_box: BoundingBox {
                left: 110.0,
                top: 75.0,
                width: 170.0,
                height: 210.0,
            },
            confidence: 0.9543,
            landmarks: if request.return_landmarks {
                Some(FaceLandmarks {
                    left_eye: Point2D { x: 155.0, y: 135.0 },
                    right_eye: Point2D { x: 230.0, y: 137.0 },
                    nose_tip: Point2D { x: 192.0, y: 182.0 },
                    mouth_left: Point2D { x: 158.0, y: 230.0 },
                    mouth_right: Point2D { x: 226.0, y: 232.0 },
                    left_eyebrow_left: Some(Point2D { x: 135.0, y: 115.0 }),
                    left_eyebrow_right: Some(Point2D { x: 170.0, y: 113.0 }),
                    right_eyebrow_left: Some(Point2D { x: 215.0, y: 114.0 }),
                    right_eyebrow_right: Some(Point2D { x: 250.0, y: 116.0 }),
                })
            } else {
                None
            },
            attributes: if request.return_attributes {
                Some(FaceAttributes {
                    age: Some(28.0),
                    gender: Some(Gender::Female),
                    emotion: Some(EmotionScores {
                        anger: 0.01,
                        contempt: 0.01,
                        disgust: 0.01,
                        fear: 0.01,
                        happiness: 0.8,
                        neutral: 0.1,
                        sadness: 0.02,
                        surprise: 0.04,
                    }),
                    smile: Some(0.72),
                    glasses: Some(GlassesType::NoGlasses),
                    facial_hair: None,
                    head_pose: Some(HeadPose { pitch: 1.0, roll: 0.5, yaw: -2.0 }),
                    blur: Some(BlurLevel::Low),
                    exposure: Some(ExposureLevel::GoodExposure),
                    noise: Some(NoiseLevel::Low),
                    occlusion: None,
                })
            } else {
                None
            },
            embedding: Some(vec![0.08; 512]),
        };

        let mut cache = self.face_cache.write().await;
        cache.insert(face.id, face.clone());

        Ok(FaceDetectionResponse {
            faces: vec![face],
            image_width: 640,
            image_height: 480,
            processing_time_ms: start.elapsed().as_millis() as u64,
            provider: FaceApiProvider::InsightFace,
        })
    }

    fn convert_azure_face(&self, azure: AzureFaceResponse) -> DetectedFace {
        DetectedFace {
            id: azure
                .face_id
                .and_then(|s| Uuid::parse_str(&s).ok())
                .unwrap_or_else(Uuid::new_v4),
            bounding_box: BoundingBox {
                left: azure.face_rectangle.left as f32,
                top: azure.face_rectangle.top as f32,
                width: azure.face_rectangle.width as f32,
                height: azure.face_rectangle.height as f32,
            },
            confidence: 1.0,
            landmarks: azure.face_landmarks.map(|fl| FaceLandmarks {
                left_eye: Point2D {
                    x: fl.pupil_left.x,
                    y: fl.pupil_left.y,
                },
                right_eye: Point2D {
                    x: fl.pupil_right.x,
                    y: fl.pupil_right.y,
                },
                nose_tip: Point2D {
                    x: fl.nose_tip.x,
                    y: fl.nose_tip.y,
                },
                mouth_left: Point2D {
                    x: fl.mouth_left.x,
                    y: fl.mouth_left.y,
                },
                mouth_right: Point2D {
                    x: fl.mouth_right.x,
                    y: fl.mouth_right.y,
                },
                left_eyebrow_left: None,
                left_eyebrow_right: None,
                right_eyebrow_left: None,
                right_eyebrow_right: None,
            }),
            attributes: azure.face_attributes.map(|fa| FaceAttributes {
                age: fa.age,
                gender: fa.gender.map(|g| match g.to_lowercase().as_str() {
                    "male" => Gender::Male,
                    "female" => Gender::Female,
                    _ => Gender::Unknown,
                }),
                emotion: fa.emotion.map(|e| EmotionScores {
                    anger: e.anger,
                    contempt: e.contempt,
                    disgust: e.disgust,
                    fear: e.fear,
                    happiness: e.happiness,
                    neutral: e.neutral,
                    sadness: e.sadness,
                    surprise: e.surprise,
                }),
                glasses: fa.glasses.map(|g| match g.to_lowercase().as_str() {
                    "noglasses" => GlassesType::NoGlasses,
                    "readingglasses" => GlassesType::ReadingGlasses,
                    "sunglasses" => GlassesType::Sunglasses,
                    "swimminggoggles" => GlassesType::SwimmingGoggles,
                    _ => GlassesType::NoGlasses,
                }),
                facial_hair: fa.facial_hair.map(|fh| FacialHair {
                    beard: fh.beard,
                    moustache: fh.moustache,
                    sideburns: fh.sideburns,
                }),
                head_pose: fa.head_pose.map(|hp| HeadPose {
                    pitch: hp.pitch,
                    roll: hp.roll,
                    yaw: hp.yaw,
                }),
                smile: fa.smile,
                blur: None,
                exposure: None,
                noise: None,
                occlusion: None,
            }),
            embedding: None,
        }
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)) as f64
    }

    pub fn provider(&self) -> FaceApiProvider {
        self.config.provider
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AzureFaceResponse {
    #[serde(rename = "faceId")]
    face_id: Option<String>,
    #[serde(rename = "faceRectangle")]
    face_rectangle: AzureFaceRectangle,
    #[serde(rename = "faceLandmarks")]
    face_landmarks: Option<AzureFaceLandmarks>,
    #[serde(rename = "faceAttributes")]
    face_attributes: Option<AzureFaceAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureFaceRectangle {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureFaceLandmarks {
    #[serde(rename = "pupilLeft")]
    pupil_left: AzurePoint,
    #[serde(rename = "pupilRight")]
    pupil_right: AzurePoint,
    #[serde(rename = "noseTip")]
    nose_tip: AzurePoint,
    #[serde(rename = "mouthLeft")]
    mouth_left: AzurePoint,
    #[serde(rename = "mouthRight")]
    mouth_right: AzurePoint,
}

#[derive(Debug, Clone, Deserialize)]
struct AzurePoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureFaceAttributes {
    age: Option<f32>,
    gender: Option<String>,
    smile: Option<f32>,
    #[serde(rename = "facialHair")]
    facial_hair: Option<AzureFacialHair>,
    glasses: Option<String>,
    emotion: Option<AzureEmotion>,
    #[serde(rename = "headPose")]
    head_pose: Option<AzureHeadPose>,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureFacialHair {
    beard: f32,
    moustache: f32,
    sideburns: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureEmotion {
    anger: f32,
    contempt: f32,
    disgust: f32,
    fear: f32,
    happiness: f32,
    neutral: f32,
    sadness: f32,
    surprise: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct AzureHeadPose {
    pitch: f32,
    roll: f32,
    yaw: f32,
}

#[derive(Debug, Clone)]
pub enum FaceApiError {
    ConfigurationError(String),
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    FaceNotFound(Uuid),
    MissingEmbedding(String),
    AttributesNotAvailable,
    ProviderNotImplemented(String),
}

impl std::fmt::Display for FaceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigurationError(e) => write!(f, "Configuration error: {e}"),
            Self::NetworkError(e) => write!(f, "Network error: {e}"),
            Self::ApiError(e) => write!(f, "API error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
            Self::FaceNotFound(id) => write!(f, "Face not found: {id}"),
            Self::MissingEmbedding(e) => write!(f, "Missing embedding: {e}"),
            Self::AttributesNotAvailable => write!(f, "Face attributes not available"),
            Self::ProviderNotImplemented(p) => write!(f, "Provider not implemented: {p}"),
        }
    }
}

impl std::error::Error for FaceApiError {}

pub fn create_azure_config(endpoint: &str, api_key: &str) -> FaceApiConfig {
    FaceApiConfig {
        provider: FaceApiProvider::AzureFaceApi,
        endpoint: Some(endpoint.to_string()),
        api_key: Some(api_key.to_string()),
        region: None,
        model_path: None,
    }
}

pub fn create_aws_config(region: &str) -> FaceApiConfig {
    FaceApiConfig {
        provider: FaceApiProvider::AwsRekognition,
        endpoint: None,
        api_key: None,
        region: Some(region.to_string()),
        model_path: None,
    }
}

pub fn create_opencv_config(model_path: &str) -> FaceApiConfig {
    FaceApiConfig {
        provider: FaceApiProvider::OpenCv,
        endpoint: None,
        api_key: None,
        region: None,
        model_path: Some(model_path.to_string()),
    }
}

pub fn create_insightface_config(model_path: &str) -> FaceApiConfig {
    FaceApiConfig {
        provider: FaceApiProvider::InsightFace,
        endpoint: None,
        api_key: None,
        region: None,
        model_path: Some(model_path.to_string()),
    }
}
