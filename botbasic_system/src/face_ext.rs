use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point2D { pub x: f64, pub y: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox { pub left: f64, pub top: f64, pub width: f64, pub height: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gender { Male, Female, Unspecified }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlassesType { None, Reading, Sunglasses, Swimming }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionScores { pub happiness: f64, pub sadness: f64, pub surprise: f64, pub anger: f64, pub neutral: f64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceLandmarks { pub points: Vec<Point2D> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceAttributes { pub age: f64, pub gender: Gender, pub glasses: GlassesType, pub emotions: EmotionScores, pub landmarks: Option<FaceLandmarks> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFace { pub face_id: String, pub bounding_box: BoundingBox, pub attributes: Option<FaceAttributes> }

#[derive(Debug, Clone)]
pub enum FaceApiProvider { Azure, Aws, Local }

#[derive(Debug, Clone)]
pub struct FaceApiConfig { pub provider: FaceApiProvider, pub endpoint: String, pub api_key: String }

pub struct FaceApiService { config: FaceApiConfig }

impl FaceApiService {
    pub fn new(config: FaceApiConfig) -> Self { Self { config } }
}
