//! Face API BASIC Keyword Executors
//!
//! This module contains functions to execute Face API keywords from BASIC code.

use super::results::{FaceAnalysisResult, FaceDetectionResult, FaceVerificationResult};
use super::service::FaceApiService;
use super::types::{AnalysisOptions, DetectionOptions, FaceAttributeType, VerificationOptions};

// ============================================================================
// BASIC Keyword Executor
// ============================================================================

/// Execute DETECT FACES keyword
pub async fn execute_detect_faces(
    service: &FaceApiService,
    image_url: &str,
    options: Option<DetectionOptions>,
) -> Result<FaceDetectionResult, super::error::FaceApiError> {
    let image = super::types::ImageSource::Url(image_url.to_string());
    let opts = options.unwrap_or_default();
    service.detect_faces(&image, &opts).await
}

/// Execute VERIFY FACE keyword
pub async fn execute_verify_face(
    service: &FaceApiService,
    face1_url: &str,
    face2_url: &str,
    options: Option<VerificationOptions>,
) -> Result<FaceVerificationResult, super::error::FaceApiError> {
    let face1 = super::types::FaceSource::Image(super::types::ImageSource::Url(face1_url.to_string()));
    let face2 = super::types::FaceSource::Image(super::types::ImageSource::Url(face2_url.to_string()));
    let opts = options.unwrap_or_default();
    service.verify_faces(&face1, &face2, &opts).await
}

/// Execute ANALYZE FACE keyword
pub async fn execute_analyze_face(
    service: &FaceApiService,
    image_url: &str,
    attributes: Option<Vec<FaceAttributeType>>,
    options: Option<AnalysisOptions>,
) -> Result<FaceAnalysisResult, super::error::FaceApiError> {
    let source = super::types::FaceSource::Image(super::types::ImageSource::Url(image_url.to_string()));
    let attrs = attributes.unwrap_or_else(|| vec![
        FaceAttributeType::Age,
        FaceAttributeType::Gender,
        FaceAttributeType::Emotion,
        FaceAttributeType::Smile,
    ]);
    let opts: AnalysisOptions = options.unwrap_or_default();
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
