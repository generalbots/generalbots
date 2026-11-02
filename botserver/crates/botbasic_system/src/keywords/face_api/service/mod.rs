//! Face API Service
//!
//! This module contains the main FaceApiService implementation with support for
//! multiple providers: Azure Face API, AWS Rekognition, OpenCV, and InsightFace.

pub(crate) use super::azure::{AzureFaceResponse, AzureVerifyResponse};
pub(crate) use super::error::FaceApiError;
pub(crate) use super::results::{FaceAnalysisResult, FaceDetectionResult, FaceVerificationResult};
pub(crate) use super::types::{AnalysisOptions, DetectionOptions, FaceAttributeType, FaceSource, ImageSource, VerificationOptions};
pub(crate) use botmodelsbridge::face_api::{BoundingBox, DetectedFace, EmotionScores, FaceApiConfig, FaceApiProvider, FaceAttributes, FaceLandmarks, Gender, GlassesType, Point2D};

pub struct FaceApiService {
    config: FaceApiConfig,
    client: reqwest::Client,
    face_cache: Arc<RwLock<HashMap<Uuid, DetectedFace>>>,
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

mod cosine_similarity;
mod detect_faces;
mod verify_faces_aws;
mod verify_faces_insightface;

use cosine_similarity::*;
