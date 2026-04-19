//! Face API BASIC Keywords
//!
//! Provides face detection, verification, and analysis capabilities through BASIC keywords.
//! Supports Azure Face API, AWS Rekognition, and local OpenCV fallback.

mod azure;
mod error;
mod executor;
mod results;
mod service;
mod types;

// Re-export all public types
pub use error::FaceApiError;
pub use executor::{
    analysis_to_basic_value,
    detection_to_basic_value,
    execute_analyze_face,
    execute_detect_faces,
    execute_verify_face,
    verification_to_basic_value,
};
pub use results::{
    FaceAnalysisResult,
    FaceDetectionResult,
    FaceGroup,
    SimilarFaceResult,
    FaceVerificationResult,
};
pub use service::FaceApiService;
pub use types::{
    AnalyzeFaceKeyword,
    AnalysisOptions,
    DetectFacesKeyword,
    DetectionOptions,
    FaceAttributeType,
    FaceSource,
    FindSimilarFacesKeyword,
    GroupFacesKeyword,
    GroupingOptions,
    ImageSource,
    VerifyFaceKeyword,
    VerificationOptions,
};
