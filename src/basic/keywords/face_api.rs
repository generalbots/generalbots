//! Face API BASIC Keywords
//!
//! Provides face detection, verification, and analysis capabilities through BASIC keywords.
//! Supports Azure Face API, AWS Rekognition, and local OpenCV fallback.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::botmodels::{
    DetectedFace, EmotionScores, FaceApiConfig, FaceApiProvider, FaceAttributes,
    Gender, BoundingBox,
};

// ============================================================================
// Keyword Definitions
// ============================================================================

/// DETECT FACES keyword - Detect faces in an image
///
/// Syntax:
///   faces = DETECT FACES image_url
///   faces = DETECT FACES image_url WITH OPTIONS options
///
/// Examples:
///   faces = DETECT FACES "https://example.com/photo.jpg"
///   faces = DETECT FACES photo WITH OPTIONS { "return_landmarks": true, "return_attributes": true }
///
/// Returns: Array of detected faces with bounding boxes and optional attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectFacesKeyword {
    pub image_source: ImageSource,
    pub options: DetectionOptions,
}

/// VERIFY FACE keyword - Verify if two faces belong to the same person
///
/// Syntax:
///   result = VERIFY FACE face1 AGAINST face2
///   result = VERIFY FACE image1 AGAINST image2
///
/// Examples:
///   match = VERIFY FACE saved_face AGAINST new_photo
///   result = VERIFY FACE "https://example.com/id.jpg" AGAINST camera_capture
///
/// Returns: Verification result with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyFaceKeyword {
    pub face1: FaceSource,
    pub face2: FaceSource,
    pub options: VerificationOptions,
}

/// ANALYZE FACE keyword - Analyze face attributes in detail
///
/// Syntax:
///   analysis = ANALYZE FACE image_url
///   analysis = ANALYZE FACE face_id WITH ATTRIBUTES attributes_list
///
/// Examples:
///   analysis = ANALYZE FACE photo WITH ATTRIBUTES ["age", "emotion", "gender"]
///   result = ANALYZE FACE captured_image
///
/// Returns: Detailed face analysis including emotions, age, gender, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeFaceKeyword {
    pub source: FaceSource,
    pub attributes: Vec<FaceAttributeType>,
    pub options: AnalysisOptions,
}

/// FIND SIMILAR FACES keyword - Find similar faces in a collection
///
/// Syntax:
///   similar = FIND SIMILAR FACES TO face IN collection
///
/// Examples:
///   matches = FIND SIMILAR FACES TO suspect_photo IN employee_database
///
/// Returns: Array of similar faces with similarity scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindSimilarFacesKeyword {
    pub target_face: FaceSource,
    pub collection_name: String,
    pub max_results: usize,
    pub min_confidence: f32,
}

/// GROUP FACES keyword - Group faces by similarity
///
/// Syntax:
///   groups = GROUP FACES face_list
///
/// Examples:
///   groups = GROUP FACES detected_faces
///
/// Returns: Groups of similar faces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFacesKeyword {
    pub faces: Vec<FaceSource>,
    pub options: GroupingOptions,
}

// ============================================================================
// Supporting Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    Url(String),
    Base64(String),
    FilePath(String),
    Variable(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FaceSource {
    Image(ImageSource),
    FaceId(Uuid),
    DetectedFace(Box<DetectedFace>),
    Embedding(Vec<f32>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionOptions {
    #[serde(default = "default_true")]
    pub return_face_id: bool,
    #[serde(default)]
    pub return_landmarks: bool,
    #[serde(default)]
    pub return_attributes: bool,
    #[serde(default)]
    pub return_embedding: bool,
    #[serde(default)]
    pub detection_model: Option<String>,
    #[serde(default)]
    pub recognition_model: Option<String>,
    #[serde(default = "default_max_faces")]
    pub max_faces: usize,
    #[serde(default = "default_min_face_size")]
    pub min_face_size: u32,
}

fn default_true() -> bool {
    true
}

fn default_max_faces() -> usize {
    100
}

fn default_min_face_size() -> u32 {
    36
}

impl Default for DetectionOptions {
    fn default() -> Self {
        Self {
            return_face_id: true,
            return_landmarks: false,
            return_attributes: false,
            return_embedding: false,
            detection_model: None,
            recognition_model: None,
            max_faces: 100,
            min_face_size: 36,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationOptions {
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
    #[serde(default)]
    pub recognition_model: Option<String>,
}

fn default_confidence_threshold() -> f32 {
    0.6
}

impl Default for VerificationOptions {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.6,
            recognition_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    #[serde(default = "default_true")]
    pub return_landmarks: bool,
    #[serde(default)]
    pub detection_model: Option<String>,
    #[serde(default)]
    pub recognition_model: Option<String>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            return_landmarks: true,
            detection_model: None,
            recognition_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingOptions {
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

fn default_similarity_threshold() -> f32 {
    0.5
}

impl Default for GroupingOptions {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FaceAttributeType {
    Age,
    Gender,
    Emotion,
    Smile,
    Glasses,
    FacialHair,
    HeadPose,
    Blur,
    Exposure,
    Noise,
    Occlusion,
    Accessories,
    Hair,
    Makeup,
    QualityForRecognition,
}

// ============================================================================
// Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetectionResult {
    pub success: bool,
    pub faces: Vec<DetectedFace>,
    pub face_count: usize,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

impl FaceDetectionResult {
    pub fn success(faces: Vec<DetectedFace>, processing_time_ms: u64) -> Self {
        let face_count = faces.len();
        Self {
            success: true,
            faces,
            face_count,
            image_width: None,
            image_height: None,
            processing_time_ms,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            faces: Vec::new(),
            face_count: 0,
            image_width: None,
            image_height: None,
            processing_time_ms: 0,
            error: Some(message),
        }
    }

    pub fn with_image_size(mut self, width: u32, height: u32) -> Self {
        self.image_width = Some(width);
        self.image_height = Some(height);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceVerificationResult {
    pub success: bool,
    pub is_match: bool,
    pub confidence: f64,
    pub threshold: f64,
    pub face1_id: Option<Uuid>,
    pub face2_id: Option<Uuid>,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

impl FaceVerificationResult {
    pub fn match_found(confidence: f64, threshold: f64, processing_time_ms: u64) -> Self {
        Self {
            success: true,
            is_match: confidence >= threshold,
            confidence,
            threshold,
            face1_id: None,
            face2_id: None,
            processing_time_ms,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            is_match: false,
            confidence: 0.0,
            threshold: 0.0,
            face1_id: None,
            face2_id: None,
            processing_time_ms: 0,
            error: Some(message),
        }
    }

    pub fn with_face_ids(mut self, face1_id: Uuid, face2_id: Uuid) -> Self {
        self.face1_id = Some(face1_id);
        self.face2_id = Some(face2_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceAnalysisResult {
    pub success: bool,
    pub face: Option<DetectedFace>,
    pub attributes: Option<FaceAttributes>,
    pub dominant_emotion: Option<String>,
    pub estimated_age: Option<f32>,
    pub gender: Option<String>,
    pub smile_intensity: Option<f32>,
    pub quality_score: Option<f32>,
    pub processing_time_ms: u64,
    pub error: Option<String>,
}

impl FaceAnalysisResult {
    pub fn success(face: DetectedFace, processing_time_ms: u64) -> Self {
        let attributes = face.attributes.clone();
        let dominant_emotion = attributes.as_ref()
            .and_then(|a| a.emotion.as_ref())
            .map(|e| e.dominant_emotion().to_string());
        let estimated_age = attributes.as_ref().and_then(|a| a.age);
        let gender = attributes.as_ref()
            .and_then(|a| a.gender)
            .map(|g| format!("{:?}", g).to_lowercase());
        let smile_intensity = attributes.as_ref().and_then(|a| a.smile);

        Self {
            success: true,
            face: Some(face),
            attributes,
            dominant_emotion,
            estimated_age,
            gender,
            smile_intensity,
            quality_score: None,
            processing_time_ms,
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            face: None,
            attributes: None,
            dominant_emotion: None,
            estimated_age: None,
            gender: None,
            smile_intensity: None,
            quality_score: None,
            processing_time_ms: 0,
            error: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFaceResult {
    pub face_id: Uuid,
    pub confidence: f64,
    pub person_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceGroup {
    pub group_id: Uuid,
    pub face_ids: Vec<Uuid>,
    pub representative_face_id: Option<Uuid>,
    pub confidence: f64,
}

// ============================================================================
// Face API Service
// ============================================================================

pub struct FaceApiService {
    config: FaceApiConfig,
    client: reqwest::Client,
}

impl FaceApiService {
    pub fn new(config: FaceApiConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Detect faces in an image
    pub async fn detect_faces(
        &self,
        image: &ImageSource,
        options: &DetectionOptions,
    ) -> Result<FaceDetectionResult, FaceApiError> {
        let start = std::time::Instant::now();

        match self.config.provider {
            FaceApiProvider::AzureFaceApi => {
                self.detect_faces_azure(image, options).await
            }
            FaceApiProvider::AwsRekognition => {
                self.detect_faces_aws(image, options).await
            }
            FaceApiProvider::OpenCv => {
                self.detect_faces_opencv(image, options).await
            }
            FaceApiProvider::InsightFace => {
                self.detect_faces_insightface(image, options).await
            }
        }
        .map(|mut result| {
            result.processing_time_ms = start.elapsed().as_millis() as u64;
            result
        })
    }

    /// Verify if two faces are the same person
    pub async fn verify_faces(
        &self,
        face1: &FaceSource,
        face2: &FaceSource,
        options: &VerificationOptions,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        let start = std::time::Instant::now();

        match self.config.provider {
            FaceApiProvider::AzureFaceApi => {
                self.verify_faces_azure(face1, face2, options).await
            }
            FaceApiProvider::AwsRekognition => {
                self.verify_faces_aws(face1, face2, options).await
            }
            FaceApiProvider::OpenCv => {
                self.verify_faces_opencv(face1, face2, options).await
            }
            FaceApiProvider::InsightFace => {
                self.verify_faces_insightface(face1, face2, options).await
            }
        }
        .map(|mut result| {
            result.processing_time_ms = start.elapsed().as_millis() as u64;
            result
        })
    }

    /// Analyze face attributes
    pub async fn analyze_face(
        &self,
        source: &FaceSource,
        attributes: &[FaceAttributeType],
        options: &AnalysisOptions,
    ) -> Result<FaceAnalysisResult, FaceApiError> {
        let start = std::time::Instant::now();

        match self.config.provider {
            FaceApiProvider::AzureFaceApi => {
                self.analyze_face_azure(source, attributes, options).await
            }
            FaceApiProvider::AwsRekognition => {
                self.analyze_face_aws(source, attributes, options).await
            }
            FaceApiProvider::OpenCv => {
                self.analyze_face_opencv(source, attributes, options).await
            }
            FaceApiProvider::InsightFace => {
                self.analyze_face_insightface(source, attributes, options).await
            }
        }
        .map(|mut result| {
            result.processing_time_ms = start.elapsed().as_millis() as u64;
            result
        })
    }

    // ========================================================================
    // Azure Face API Implementation
    // ========================================================================

    async fn detect_faces_azure(
        &self,
        image: &ImageSource,
        options: &DetectionOptions,
    ) -> Result<FaceDetectionResult, FaceApiError> {
        let endpoint = self.config.endpoint.as_ref()
            .ok_or(FaceApiError::ConfigError("Azure endpoint not configured".to_string()))?;
        let api_key = self.config.api_key.as_ref()
            .ok_or(FaceApiError::ConfigError("Azure API key not configured".to_string()))?;

        let mut return_params = vec!["faceId"];
        if options.return_landmarks {
            return_params.push("faceLandmarks");
        }

        let mut attributes = Vec::new();
        if options.return_attributes {
            attributes.extend_from_slice(&[
                "age", "gender", "smile", "glasses", "emotion",
                "facialHair", "headPose", "blur", "exposure", "noise", "occlusion"
            ]);
        }

        let url = format!(
            "{}/face/v1.0/detect?returnFaceId={}&returnFaceLandmarks={}&returnFaceAttributes={}",
            endpoint,
            options.return_face_id,
            options.return_landmarks,
            attributes.join(",")
        );

        let request = match image {
            ImageSource::Url(image_url) => {
                self.client
                    .post(&url)
                    .header("Ocp-Apim-Subscription-Key", api_key)
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({ "url": image_url }))
            }
            ImageSource::Base64(data) => {
                let bytes = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    data,
                ).map_err(|e| FaceApiError::InvalidInput(e.to_string()))?;

                self.client
                    .post(&url)
                    .header("Ocp-Apim-Subscription-Key", api_key)
                    .header("Content-Type", "application/octet-stream")
                    .body(bytes)
            }
            ImageSource::Binary(bytes) => {
                self.client
                    .post(&url)
                    .header("Ocp-Apim-Subscription-Key", api_key)
                    .header("Content-Type", "application/octet-stream")
                    .body(bytes.clone())
            }
            _ => return Err(FaceApiError::InvalidInput("Unsupported image source for Azure".to_string())),
        };

        let response = request.send().await
            .map_err(|e| FaceApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(FaceApiError::ApiError(error_text));
        }

        let azure_faces: Vec<AzureFaceResponse> = response.json().await
            .map_err(|e| FaceApiError::ParseError(e.to_string()))?;

        let faces = azure_faces
            .into_iter()
            .map(|af| af.into_detected_face())
            .collect();

        Ok(FaceDetectionResult::success(faces, 0))
    }

    async fn verify_faces_azure(
        &self,
        face1: &FaceSource,
        face2: &FaceSource,
        options: &VerificationOptions,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        let endpoint = self.config.endpoint.as_ref()
            .ok_or(FaceApiError::ConfigError("Azure endpoint not configured".to_string()))?;
        let api_key = self.config.api_key.as_ref()
            .ok_or(FaceApiError::ConfigError("Azure API key not configured".to_string()))?;

        // Get face IDs (may need to detect first)
        let face1_id = self.get_or_detect_face_id(face1).await?;
        let face2_id = self.get_or_detect_face_id(face2).await?;

        let url = format!("{}/face/v1.0/verify", endpoint);

        let response = self.client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "faceId1": face1_id.to_string(),
                "faceId2": face2_id.to_string()
            }))
            .send()
            .await
            .map_err(|e| FaceApiError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(FaceApiError::ApiError(error_text));
        }

        let result: AzureVerifyResponse = response.json().await
            .map_err(|e| FaceApiError::ParseError(e.to_string()))?;

        Ok(FaceVerificationResult::match_found(
            result.confidence,
            options.confidence_threshold as f64,
            0,
        ).with_face_ids(face1_id, face2_id))
    }

    async fn analyze_face_azure(
        &self,
        source: &FaceSource,
        attributes: &[FaceAttributeType],
        options: &AnalysisOptions,
    ) -> Result<FaceAnalysisResult, FaceApiError> {
        let detect_options = DetectionOptions {
            return_face_id: true,
            return_landmarks: options.return_landmarks,
            return_attributes: !attributes.is_empty(),
            ..Default::default()
        };

        let image = match source {
            FaceSource::Image(img) => img.clone(),
            FaceSource::DetectedFace(face) => {
                return Ok(FaceAnalysisResult::success(*face.clone(), 0));
            }
            _ => return Err(FaceApiError::InvalidInput("Cannot analyze from face ID alone".to_string())),
        };

        let result = self.detect_faces_azure(&image, &detect_options).await?;

        if let Some(face) = result.faces.into_iter().next() {
            Ok(FaceAnalysisResult::success(face, 0))
        } else {
            Err(FaceApiError::NoFaceFound)
        }
    }

    // ========================================================================
    // AWS Rekognition Implementation (Stub)
    // ========================================================================

    async fn detect_faces_aws(
        &self,
        _image: &ImageSource,
        _options: &DetectionOptions,
    ) -> Result<FaceDetectionResult, FaceApiError> {
        // TODO: Implement AWS Rekognition
        Err(FaceApiError::NotImplemented("AWS Rekognition".to_string()))
    }

    async fn verify_faces_aws(
        &self,
        _face1: &FaceSource,
        _face2: &FaceSource,
        _options: &VerificationOptions,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("AWS Rekognition".to_string()))
    }

    async fn analyze_face_aws(
        &self,
        _source: &FaceSource,
        _attributes: &[FaceAttributeType],
        _options: &AnalysisOptions,
    ) -> Result<FaceAnalysisResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("AWS Rekognition".to_string()))
    }

    // ========================================================================
    // OpenCV Implementation (Stub)
    // ========================================================================

    async fn detect_faces_opencv(
        &self,
        _image: &ImageSource,
        _options: &DetectionOptions,
    ) -> Result<FaceDetectionResult, FaceApiError> {
        // TODO: Implement local OpenCV detection
        Err(FaceApiError::NotImplemented("OpenCV".to_string()))
    }

    async fn verify_faces_opencv(
        &self,
        _face1: &FaceSource,
        _face2: &FaceSource,
        _options: &VerificationOptions,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("OpenCV".to_string()))
    }

    async fn analyze_face_opencv(
        &self,
        _source: &FaceSource,
        _attributes: &[FaceAttributeType],
        _options: &AnalysisOptions,
    ) -> Result<FaceAnalysisResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("OpenCV".to_string()))
    }

    // ========================================================================
    // InsightFace Implementation (Stub)
    // ========================================================================

    async fn detect_faces_insightface(
        &self,
        _image: &ImageSource,
        _options: &DetectionOptions,
    ) -> Result<FaceDetectionResult, FaceApiError> {
        // TODO: Implement InsightFace
        Err(FaceApiError::NotImplemented("InsightFace".to_string()))
    }

    async fn verify_faces_insightface(
        &self,
        _face1: &FaceSource,
        _face2: &FaceSource,
        _options: &VerificationOptions,
    ) -> Result<FaceVerificationResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("InsightFace".to_string()))
    }

    async fn analyze_face_insightface(
        &self,
        _source: &FaceSource,
        _attributes: &[FaceAttributeType],
        _options: &AnalysisOptions,
    ) -> Result<FaceAnalysisResult, FaceApiError> {
        Err(FaceApiError::NotImplemented("InsightFace".to_string()))
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    async fn get_or_detect_face_id(&self, source: &FaceSource) -> Result<Uuid, FaceApiError> {
        match source {
            FaceSource::FaceId(id) => Ok(*id),
            FaceSource::DetectedFace(face) => Ok(face.id),
            FaceSource::Image(image) => {
                let result = self.detect_faces(image, &DetectionOptions::default()).await?;
                result.faces.first()
                    .map(|f| f.id)
                    .ok_or(FaceApiError::NoFaceFound)
            }
            FaceSource::Embedding(_) => {
                Err(FaceApiError::InvalidInput("Cannot get face ID from embedding".to_string()))
            }
        }
    }
}

// ============================================================================
// Azure API Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureFaceResponse {
    face_id: Option<String>,
    face_rectangle: AzureFaceRectangle,
    face_landmarks: Option<AzureFaceLandmarks>,
    face_attributes: Option<AzureFaceAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureFaceRectangle {
    top: f32,
    left: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureFaceLandmarks {
    pupil_left: Option<AzurePoint>,
    pupil_right: Option<AzurePoint>,
    nose_tip: Option<AzurePoint>,
    mouth_left: Option<AzurePoint>,
    mouth_right: Option<AzurePoint>,
    eyebrow_left_outer: Option<AzurePoint>,
    eyebrow_left_inner: Option<AzurePoint>,
    eyebrow_right_outer: Option<AzurePoint>,
    eyebrow_right_inner: Option<AzurePoint>,
}

#[derive(Debug, Clone, Deserialize)]
struct AzurePoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureFaceAttributes {
    age: Option<f32>,
    gender: Option<String>,
    smile: Option<f32>,
    glasses: Option<String>,
    emotion: Option<AzureEmotion>,
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
#[serde(rename_all = "camelCase")]
struct AzureVerifyResponse {
    confidence: f64,
}

impl AzureFaceResponse {
    fn into_detected_face(self) -> DetectedFace {
        use crate::botmodels::{FaceLandmarks, Point2D, GlassesType};

        let face_id = self.face_id
            .and_then(|id| Uuid::parse_str(&id).ok())
            .unwrap_or_else(Uuid::new_v4);

        let landmarks = self.face_landmarks.map(|lm| {
            FaceLandmarks {
                left_eye: lm.pupil_left.map(|p| Point2D { x: p.x, y: p.y })
                    .unwrap_or(Point2D { x: 0.0, y: 0.0 }),
                right_eye: lm.pupil_right.map(|p| Point2D { x: p.x, y: p.y })
                    .unwrap_or(Point2D { x: 0.0, y: 0.0 }),
                nose_tip: lm.nose_tip.map(|p| Point2D { x: p.x, y: p.y })
                    .unwrap_or(Point2D { x: 0.0, y: 0.0 }),
                mouth_left: lm.mouth_left.map(|p| Point2D { x: p.x, y: p.y })
                    .unwrap_or(Point2D { x: 0.0, y: 0.0 }),
                mouth_right: lm.mouth_right.map(|p| Point2D { x: p.x, y: p.y })
                    .unwrap_or(Point2D { x: 0.0, y: 0.0 }),
                left_eyebrow_left: lm.eyebrow_left_outer.map(|p| Point2D { x: p.x, y: p.y }),
                left_eyebrow_right: lm.eyebrow_left_inner.map(|p| Point2D { x: p.x, y: p.y }),
                right_eyebrow_left: lm.eyebrow_right_inner.map(|p| Point2D { x: p.x, y: p.y }),
                right_eyebrow_right: lm.eyebrow_right_outer.map(|p| Point2D { x: p.x, y: p.y }),
            }
        });

        let attributes = self.face_attributes.map(|attrs| {
            let gender = attrs.gender.as_ref().map(|g| {
                match g.to_lowercase().as_str() {
                    "male" => Gender::Male,
                    "female" => Gender::Female,
                    _ => Gender::Unknown,
                }
            });

            let emotion = attrs.emotion.map(|e| EmotionScores {
                anger: e.anger,
                contempt: e.contempt,
                disgust: e.disgust,
                fear: e.fear,
                happiness: e.happiness,
                neutral: e.neutral,
                sadness: e.sadness,
                surprise: e.surprise,
            });

            let glasses = attrs.glasses.as_ref().map(|g| {
                match g.to_lowercase().as_str() {
                    "noглasses" | "noglasses" => GlassesType::NoGlasses,
                    "readingglasses" => GlassesType::ReadingGlasses,
                    "sunglasses" => GlassesType::Sunglasses,
                    "swimminggoggles" => GlassesType::SwimmingGoggles,
                    _ => GlassesType::NoGlasses,
                }
            });

            FaceAttributes {
                age: attrs.age,
                gender,
                emotion,
                glasses,
                facial_hair: None,
                head_pose: None,
                smile: attrs.smile,
                blur: None,
                exposure: None,
                noise: None,
                occlusion: None,
            }
        });

        DetectedFace {
            id: face_id,
            bounding_box: BoundingBox {
                left: self.face_rectangle.left,
                top: self.face_rectangle.top,
                width: self.face_rectangle.width,
                height: self.face_rectangle.height,
            },
            confidence: 1.0,
            landmarks,
            attributes,
            embedding: None,
        }
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum FaceApiError {
    ConfigError(String),
    NetworkError(String),
    ApiError(String),
    ParseError(String),
    InvalidInput(String),
    NoFaceFound,
    NotImplemented(String),
    RateLimited,
    Unauthorized,
}

impl std::fmt::Display for FaceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ApiError(msg) => write!(f, "API error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::NoFaceFound => write!(f, "No face found in image"),
            Self::NotImplemented(provider) => write!(f, "{} provider not implemented", provider),
            Self::RateLimited => write!(f, "Rate limit exceeded"),
            Self::Unauthorized => write!(f, "Unauthorized - check API credentials"),
        }
    }
}

impl std::error::Error for FaceApiError {}

// ============================================================================
// BASIC Keyword Executor
// ============================================================================

/// Execute DETECT FACES keyword
pub async fn execute_detect_faces(
    service: &FaceApiService,
    image_url: &str,
    options: Option<DetectionOptions>,
) -> Result<FaceDetectionResult, FaceApiError> {
    let image = ImageSource::Url(image_url.to_string());
    let opts = options.unwrap_or_default();
    service.detect_faces(&image, &opts).await
}

/// Execute VERIFY FACE keyword
pub async fn execute_verify_face(
    service: &FaceApiService,
    face1_url: &str,
    face2_url: &str,
    options: Option<VerificationOptions>,
) -> Result<FaceVerificationResult, FaceApiError> {
    let face1 = FaceSource::Image(ImageSource::Url(face1_url.to_string()));
    let face2 = FaceSource::Image(ImageSource::Url(face2_url.to_string()));
    let opts = options.unwrap_or_default();
    service.verify_faces(&face1, &face2, &opts).await
}

/// Execute ANALYZE FACE keyword
pub async fn execute_analyze_face(
    service: &FaceApiService,
    image_url: &str,
    attributes: Option<Vec<FaceAttributeType>>,
    options: Option<AnalysisOptions>,
) -> Result<FaceAnalysisResult, FaceApiError> {
    let source = FaceSource::Image(ImageSource::Url(image_url.to_string()));
    let attrs = attributes.unwrap_or_else(|| vec![
        FaceAttributeType::Age,
        FaceAttributeType::Gender,
        FaceAttributeType::Emotion,
        FaceAttributeType::Smile,
    ]);
    let opts = options.unwrap_or_default();
    service.analyze_face(&source, &attrs, &opts).await
}

/// Convert detection result to BASIC-friendly format
pub fn detection_to_basic_value(result: &FaceDetectionResult) -> serde_json::Value {
    serde_json::json!({
        "success": result.success,
        "face_count": result.face_count,
        "faces": result.faces.iter().map(|f| {
            serde_json::json!({
                "id": f.id.to_string(),
                "bounds": {
                    "left": f.bounding_box.left,
                    "top": f.bounding_box.top,
                    "width": f.bounding_box.width,
                    "height": f.bounding_box.height
                },
                "confidence": f.confidence,
                "age": f.attributes.as_ref().and_then(|a| a.age),
                "gender": f.attributes.as_ref().and_then(|a| a.gender).map(|g| format!("{:?}", g).to_lowercase()),
                "emotion": f.attributes.as_ref().and_then(|a| a.emotion.as_ref()).map(|e| e.dominant_emotion()),
                "smile": f.attributes.as_ref().and_then(|a| a.smile)
            })
        }).collect::<Vec<_>>(),
        "processing_time_ms": result.processing_time_ms,
        "error": result.error
    })
}

/// Convert verification result to BASIC-friendly format
pub fn verification_to_basic_value(result: &FaceVerificationResult) -> serde_json::Value {
    serde_json::json!({
        "success": result.success,
        "is_match": result.is_match,
        "confidence": result.confidence,
        "threshold": result.threshold,
        "processing_time_ms": result.processing_time_ms,
        "error": result.error
    })
}

/// Convert analysis result to BASIC-friendly format
pub fn analysis_to_basic_value(result: &FaceAnalysisResult) -> serde_json::Value {
    serde_json::json!({
        "success": result.success,
        "age": result.estimated_age,
        "gender": result.gender,
        "emotion": result.dominant_emotion,
        "smile": result.smile_intensity,
        "quality": result.quality_score,
        "processing_time_ms": result.processing_time_ms,
        "error": result.error
    })
}
