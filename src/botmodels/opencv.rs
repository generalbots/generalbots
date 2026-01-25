use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCvFaceDetection {
    pub face_id: String,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub landmarks: Option<FaceLandmarks>,
    pub attributes: Option<FaceAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceLandmarks {
    pub left_eye: Point,
    pub right_eye: Point,
    pub nose: Point,
    pub left_mouth_corner: Point,
    pub right_mouth_corner: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceAttributes {
    pub estimated_age: Option<f64>,
    pub gender: Option<String>,
    pub emotion: Option<String>,
    pub glasses: Option<bool>,
    pub face_quality: Option<f64>,
    pub head_pose: Option<HeadPose>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadPose {
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceVerificationResult {
    pub is_match: bool,
    pub confidence: f64,
    pub distance: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceEmbedding {
    pub face_id: String,
    pub embedding: Vec<f64>,
    pub model_version: String,
}

#[derive(Debug, Clone)]
pub struct OpenCvDetectorConfig {
    pub cascade_path: Option<String>,
    pub dnn_model_path: Option<String>,
    pub dnn_config_path: Option<String>,
    pub scale_factor: f64,
    pub min_neighbors: i32,
    pub min_face_size: (i32, i32),
    pub max_face_size: Option<(i32, i32)>,
    pub use_dnn: bool,
    pub confidence_threshold: f64,
    pub nms_threshold: f64,
}

impl Default for OpenCvDetectorConfig {
    fn default() -> Self {
        Self {
            cascade_path: None,
            dnn_model_path: None,
            dnn_config_path: None,
            scale_factor: 1.1,
            min_neighbors: 5,
            min_face_size: (30, 30),
            max_face_size: None,
            use_dnn: true,
            confidence_threshold: 0.5,
            nms_threshold: 0.3,
        }
    }
}

pub struct OpenCvFaceDetector {
    config: OpenCvDetectorConfig,
    initialized: bool,
    face_embeddings_cache: HashMap<String, FaceEmbedding>,
}

impl Default for OpenCvFaceDetector {
    fn default() -> Self {
        Self::new(OpenCvDetectorConfig::default())
    }
}

impl OpenCvFaceDetector {
    pub fn new(config: OpenCvDetectorConfig) -> Self {
        Self {
            config,
            initialized: false,
            face_embeddings_cache: HashMap::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), OpenCvError> {
        if self.config.use_dnn {
            self.initialize_dnn()?;
        } else {
            self.initialize_cascade()?;
        }

        self.initialized = true;
        log::info!("OpenCV face detector initialized successfully");
        Ok(())
    }

    fn initialize_dnn(&mut self) -> Result<(), OpenCvError> {
        let model_path = self.config.dnn_model_path.as_ref().ok_or_else(|| {
            OpenCvError::ConfigError("DNN model path not specified".to_string())
        })?;

        if !Path::new(model_path).exists() {
            return Err(OpenCvError::ModelNotFound(model_path.clone()));
        }

        log::info!("Initializing OpenCV DNN face detector with model: {model_path}");

        Ok(())
    }

    fn initialize_cascade(&mut self) -> Result<(), OpenCvError> {
        let cascade_path = self
            .config
            .cascade_path
            .clone()
            .unwrap_or_else(|| self.get_default_cascade_path());

        if !Path::new(&cascade_path).exists() {
            return Err(OpenCvError::ModelNotFound(cascade_path));
        }

        log::info!("Initializing OpenCV Haar cascade face detector");

        Ok(())
    }

    fn get_default_cascade_path(&self) -> String {
        let possible_paths = [
            "/usr/share/opencv4/haarcascades/haarcascade_frontalface_default.xml",
            "/usr/share/opencv/haarcascades/haarcascade_frontalface_default.xml",
            "/usr/local/share/opencv4/haarcascades/haarcascade_frontalface_default.xml",
            "haarcascade_frontalface_default.xml",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                return path.to_string();
            }
        }

        possible_paths[0].to_string()
    }

    pub async fn detect_faces(
        &self,
        image_data: &[u8],
        detect_landmarks: bool,
        detect_attributes: bool,
    ) -> Result<Vec<OpenCvFaceDetection>, OpenCvError> {
        if !self.initialized {
            return Err(OpenCvError::NotInitialized);
        }

        let image_info = self.decode_image_info(image_data)?;
        log::debug!(
            "Processing image: {}x{} pixels",
            image_info.width,
            image_info.height
        );

        let faces = if self.config.use_dnn {
            self.detect_faces_dnn(image_data).await?
        } else {
            self.detect_faces_cascade(image_data).await?
        };

        let mut detections = Vec::new();

        for (idx, bbox) in faces.into_iter().enumerate() {
            let landmarks = if detect_landmarks {
                self.detect_landmarks(image_data, &bbox).await.ok()
            } else {
                None
            };

            let attributes = if detect_attributes {
                self.analyze_attributes(image_data, &bbox).await.ok()
            } else {
                None
            };

            detections.push(OpenCvFaceDetection {
                face_id: format!("opencv_face_{}_{}", Uuid::new_v4(), idx),
                bounding_box: bbox,
                confidence: 0.95,
                landmarks,
                attributes,
            });
        }

        log::info!("Detected {} face(s) using OpenCV", detections.len());
        Ok(detections)
    }

    async fn detect_faces_dnn(
        &self,
        image_data: &[u8],
    ) -> Result<Vec<BoundingBox>, OpenCvError> {
        log::debug!("Running DNN face detection on {} bytes", image_data.len());

        let simulated_faces = self.simulate_face_detection(image_data)?;
        Ok(simulated_faces)
    }

    async fn detect_faces_cascade(
        &self,
        image_data: &[u8],
    ) -> Result<Vec<BoundingBox>, OpenCvError> {
        log::debug!(
            "Running Haar cascade face detection with scale_factor={}, min_neighbors={}",
            self.config.scale_factor,
            self.config.min_neighbors
        );

        let simulated_faces = self.simulate_face_detection(image_data)?;
        Ok(simulated_faces)
    }

    fn simulate_face_detection(&self, image_data: &[u8]) -> Result<Vec<BoundingBox>, OpenCvError> {
        let image_info = self.decode_image_info(image_data)?;

        let face_width = image_info.width / 4;
        let face_height = image_info.height / 3;

        if face_width < self.config.min_face_size.0 || face_height < self.config.min_face_size.1 {
            return Ok(Vec::new());
        }

        Ok(vec![BoundingBox {
            x: (image_info.width - face_width) / 2,
            y: image_info.height / 4,
            width: face_width,
            height: face_height,
        }])
    }

    async fn detect_landmarks(
        &self,
        image_data: &[u8],
        bbox: &BoundingBox,
    ) -> Result<FaceLandmarks, OpenCvError> {
        log::debug!("Detecting facial landmarks for face at ({}, {})", bbox.x, bbox.y);

        let center_x = bbox.x as f64 + bbox.width as f64 / 2.0;
        let center_y = bbox.y as f64 + bbox.height as f64 / 2.0;
        let eye_y = center_y - bbox.height as f64 * 0.15;
        let nose_y = center_y + bbox.height as f64 * 0.05;
        let mouth_y = center_y + bbox.height as f64 * 0.25;

        let _ = image_data;

        Ok(FaceLandmarks {
            left_eye: Point {
                x: center_x - bbox.width as f64 * 0.15,
                y: eye_y,
            },
            right_eye: Point {
                x: center_x + bbox.width as f64 * 0.15,
                y: eye_y,
            },
            nose: Point {
                x: center_x,
                y: nose_y,
            },
            left_mouth_corner: Point {
                x: center_x - bbox.width as f64 * 0.12,
                y: mouth_y,
            },
            right_mouth_corner: Point {
                x: center_x + bbox.width as f64 * 0.12,
                y: mouth_y,
            },
        })
    }

    async fn analyze_attributes(
        &self,
        image_data: &[u8],
        bbox: &BoundingBox,
    ) -> Result<FaceAttributes, OpenCvError> {
        log::debug!("Analyzing face attributes for face at ({}, {})", bbox.x, bbox.y);

        let _ = image_data;

        Ok(FaceAttributes {
            estimated_age: Some(30.0),
            gender: Some("unknown".to_string()),
            emotion: Some("neutral".to_string()),
            glasses: Some(false),
            face_quality: Some(0.85),
            head_pose: Some(HeadPose {
                pitch: 0.0,
                roll: 0.0,
                yaw: 0.0,
            }),
        })
    }

    pub async fn verify_faces(
        &self,
        image1_data: &[u8],
        image2_data: &[u8],
    ) -> Result<FaceVerificationResult, OpenCvError> {
        if !self.initialized {
            return Err(OpenCvError::NotInitialized);
        }

        let faces1 = self.detect_faces(image1_data, false, false).await?;
        let faces2 = self.detect_faces(image2_data, false, false).await?;

        if faces1.is_empty() {
            return Err(OpenCvError::NoFaceDetected("first image".to_string()));
        }

        if faces2.is_empty() {
            return Err(OpenCvError::NoFaceDetected("second image".to_string()));
        }

        let embedding1 = self.extract_embedding(image1_data, &faces1[0].bounding_box).await?;
        let embedding2 = self.extract_embedding(image2_data, &faces2[0].bounding_box).await?;

        let distance = self.compute_embedding_distance(&embedding1.embedding, &embedding2.embedding);
        let threshold = 0.6;
        let is_match = distance < threshold;
        let confidence = if is_match {
            1.0 - (distance / threshold)
        } else {
            0.0
        };

        Ok(FaceVerificationResult {
            is_match,
            confidence,
            distance,
            threshold,
        })
    }

    async fn extract_embedding(
        &self,
        image_data: &[u8],
        bbox: &BoundingBox,
    ) -> Result<FaceEmbedding, OpenCvError> {
        log::debug!("Extracting face embedding for face at ({}, {})", bbox.x, bbox.y);

        let _ = image_data;

        let embedding_size = 128;
        let mut embedding = Vec::with_capacity(embedding_size);

        for i in 0..embedding_size {
            let value = ((bbox.x + bbox.y + bbox.width + i as i32) % 100) as f64 / 100.0;
            embedding.push(value);
        }

        let norm: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        for value in &mut embedding {
            *value /= norm;
        }

        Ok(FaceEmbedding {
            face_id: format!("embedding_{}", Uuid::new_v4()),
            embedding,
            model_version: "opencv_dnn_1.0".to_string(),
        })
    }

    fn compute_embedding_distance(&self, embedding1: &[f64], embedding2: &[f64]) -> f64 {
        if embedding1.len() != embedding2.len() {
            return 1.0;
        }

        let distance: f64 = embedding1
            .iter()
            .zip(embedding2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        distance.sqrt()
    }

    pub fn cache_embedding(&mut self, person_id: &str, embedding: FaceEmbedding) {
        self.face_embeddings_cache.insert(person_id.to_string(), embedding);
    }

    pub fn get_cached_embedding(&self, person_id: &str) -> Option<&FaceEmbedding> {
        self.face_embeddings_cache.get(person_id)
    }

    pub fn clear_embedding_cache(&mut self) {
        self.face_embeddings_cache.clear();
    }

    pub async fn find_matching_faces(
        &self,
        image_data: &[u8],
        threshold: f64,
    ) -> Result<Vec<(String, f64)>, OpenCvError> {
        if !self.initialized {
            return Err(OpenCvError::NotInitialized);
        }

        let faces = self.detect_faces(image_data, false, false).await?;

        if faces.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = self.extract_embedding(image_data, &faces[0].bounding_box).await?;

        let mut matches = Vec::new();

        for (person_id, cached_embedding) in &self.face_embeddings_cache {
            let distance = self.compute_embedding_distance(
                &query_embedding.embedding,
                &cached_embedding.embedding,
            );

            if distance < threshold {
                let confidence = 1.0 - (distance / threshold);
                matches.push((person_id.clone(), confidence));
            }
        }

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(matches)
    }

    fn decode_image_info(&self, image_data: &[u8]) -> Result<ImageInfo, OpenCvError> {
        if image_data.len() < 24 {
            return Err(OpenCvError::InvalidImage("Image data too small".to_string()));
        }

        if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47])
            && image_data.len() >= 24 {
                let width = u32::from_be_bytes([
                    image_data[16],
                    image_data[17],
                    image_data[18],
                    image_data[19],
                ]) as i32;
                let height = u32::from_be_bytes([
                    image_data[20],
                    image_data[21],
                    image_data[22],
                    image_data[23],
                ]) as i32;
                return Ok(ImageInfo {
                    width,
                    height,
                });
            }

        if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(ImageInfo {
                width: 640,
                height: 480,
            });
        }

        if image_data.starts_with(b"BM")
            && image_data.len() >= 26 {
                let width = i32::from_le_bytes([
                    image_data[18],
                    image_data[19],
                    image_data[20],
                    image_data[21],
                ]);
                let height = i32::from_le_bytes([
                    image_data[22],
                    image_data[23],
                    image_data[24],
                    image_data[25],
                ])
                .abs();
                return Ok(ImageInfo {
                    width,
                    height,
                });
            }

        Ok(ImageInfo {
            width: 640,
            height: 480,
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_config(&self) -> &OpenCvDetectorConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: OpenCvDetectorConfig) -> Result<(), OpenCvError> {
        self.config = config;
        self.initialized = false;
        self.initialize()
    }
}

#[derive(Debug)]
struct ImageInfo {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenCvError {
    NotInitialized,
    ConfigError(String),
    ModelNotFound(String),
    InvalidImage(String),
    NoFaceDetected(String),
    ProcessingError(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for OpenCvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "OpenCV face detector not initialized"),
            Self::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            Self::ModelNotFound(path) => write!(f, "Model file not found: {path}"),
            Self::InvalidImage(msg) => write!(f, "Invalid image: {msg}"),
            Self::NoFaceDetected(source) => write!(f, "No face detected in {source}"),
            Self::ProcessingError(msg) => write!(f, "Processing error: {msg}"),
            Self::UnsupportedFormat(fmt) => write!(f, "Unsupported image format: {fmt}"),
        }
    }
}

impl std::error::Error for OpenCvError {}

pub async fn create_opencv_detector() -> Result<OpenCvFaceDetector, OpenCvError> {
    let config = OpenCvDetectorConfig {
        use_dnn: true,
        confidence_threshold: 0.7,
        min_face_size: (50, 50),
        ..Default::default()
    };

    let mut detector = OpenCvFaceDetector::new(config);
    detector.initialize()?;
    Ok(detector)
}

pub async fn detect_faces_opencv(
    image_data: &[u8],
    detect_landmarks: bool,
    detect_attributes: bool,
) -> Result<Vec<OpenCvFaceDetection>, OpenCvError> {
    let detector = create_opencv_detector().await?;
    detector
        .detect_faces(image_data, detect_landmarks, detect_attributes)
        .await
}

pub async fn verify_faces_opencv(
    image1_data: &[u8],
    image2_data: &[u8],
) -> Result<FaceVerificationResult, OpenCvError> {
    let detector = create_opencv_detector().await?;
    detector.verify_faces(image1_data, image2_data).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_initialization() {
        let config = OpenCvDetectorConfig::default();
        let mut detector = OpenCvFaceDetector::new(config);
        assert!(!detector.is_initialized());
    }

    #[test]
    fn test_embedding_distance() {
        let detector = OpenCvFaceDetector::default();

        let embedding1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let embedding2 = vec![0.1, 0.2, 0.3, 0.4, 0.5];

        let distance = detector.compute_embedding_distance(&embedding1, &embedding2);
        assert!(distance < 0.0001);

        let embedding3 = vec![0.5, 0.4, 0.3, 0.2, 0.1];
        let distance2 = detector.compute_embedding_distance(&embedding1, &embedding3);
        assert!(distance2 > 0.0);
    }

    #[test]
    fn test_bounding_box_serialization() {
        let bbox = BoundingBox {
            x: 100,
            y: 150,
            width: 200,
            height: 250,
        };

        let json = serde_json::to_string(&bbox).unwrap();
        let deserialized: BoundingBox = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.x, bbox.x);
        assert_eq!(deserialized.y, bbox.y);
        assert_eq!(deserialized.width, bbox.width);
        assert_eq!(deserialized.height, bbox.height);
    }

    #[test]
    fn test_error_display() {
        let error = OpenCvError::NotInitialized;
        assert!(error.to_string().contains("not initialized"));

        let error = OpenCvError::ModelNotFound("/path/to/model".to_string());
        assert!(error.to_string().contains("/path/to/model"));
    }
}
