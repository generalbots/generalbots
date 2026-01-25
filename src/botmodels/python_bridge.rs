use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonFaceDetection {
    pub face_id: String,
    pub bounding_box: PythonBoundingBox,
    pub confidence: f64,
    pub landmarks: Option<PythonFaceLandmarks>,
    pub attributes: Option<PythonFaceAttributes>,
    pub embedding: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonFaceLandmarks {
    pub left_eye: PythonPoint,
    pub right_eye: PythonPoint,
    pub nose: PythonPoint,
    pub left_mouth: PythonPoint,
    pub right_mouth: PythonPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonFaceAttributes {
    pub age: Option<f64>,
    pub gender: Option<String>,
    pub emotion: Option<String>,
    pub glasses: Option<bool>,
    pub beard: Option<bool>,
    pub mask: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonVerificationResult {
    pub is_match: bool,
    pub confidence: f64,
    pub distance: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum PythonCommand {
    #[serde(rename = "detect")]
    Detect {
        image_base64: String,
        detect_landmarks: bool,
        detect_attributes: bool,
        model: String,
    },
    #[serde(rename = "verify")]
    Verify {
        image1_base64: String,
        image2_base64: String,
        model: String,
    },
    #[serde(rename = "extract_embedding")]
    ExtractEmbedding {
        image_base64: String,
        model: String,
    },
    #[serde(rename = "analyze")]
    Analyze {
        image_base64: String,
        attributes: Vec<String>,
    },
    #[serde(rename = "health")]
    Health,
    #[serde(rename = "shutdown")]
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum PythonResponse {
    #[serde(rename = "success")]
    Success { data: serde_json::Value },
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum PythonModel {
    #[default]
    MediaPipe,
    DeepFace,
    FaceRecognition,
    InsightFace,
    Dlib,
    OpenCV,
}

impl PythonModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MediaPipe => "mediapipe",
            Self::DeepFace => "deepface",
            Self::FaceRecognition => "face_recognition",
            Self::InsightFace => "insightface",
            Self::Dlib => "dlib",
            Self::OpenCV => "opencv",
        }
    }
}

impl std::str::FromStr for PythonModel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mediapipe" => Ok(Self::MediaPipe),
            "deepface" => Ok(Self::DeepFace),
            "face_recognition" => Ok(Self::FaceRecognition),
            "insightface" => Ok(Self::InsightFace),
            "dlib" => Ok(Self::Dlib),
            "opencv" => Ok(Self::OpenCV),
            _ => Err(()),
        }
    }
}

impl PythonModel {
}


#[derive(Debug, Clone)]
pub struct PythonBridgeConfig {
    pub python_path: String,
    pub script_path: String,
    pub model: PythonModel,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub gpu_enabled: bool,
}

impl Default for PythonBridgeConfig {
    fn default() -> Self {
        Self {
            python_path: "python3".to_string(),
            script_path: "scripts/face_detection.py".to_string(),
            model: PythonModel::MediaPipe,
            timeout_seconds: 30,
            max_retries: 3,
            gpu_enabled: false,
        }
    }
}

struct PythonProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

pub struct PythonFaceBridge {
    config: PythonBridgeConfig,
    process: Arc<Mutex<Option<PythonProcess>>>,
    is_healthy: Arc<RwLock<bool>>,
    embeddings_cache: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl PythonFaceBridge {
    pub fn new(config: PythonBridgeConfig) -> Self {
        Self {
            config,
            process: Arc::new(Mutex::new(None)),
            is_healthy: Arc::new(RwLock::new(false)),
            embeddings_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> Result<(), PythonBridgeError> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Ok(());
        }

        let mut child = std::process::Command::new(&self.config.python_path)
            .arg(&self.config.script_path)
            .arg("--model")
            .arg(self.config.model.as_str())
            .arg(if self.config.gpu_enabled { "--gpu" } else { "--cpu" })
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PythonBridgeError::ProcessSpawnFailed(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| PythonBridgeError::ProcessSpawnFailed("Failed to capture stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| PythonBridgeError::ProcessSpawnFailed("Failed to capture stdout".to_string()))?;

        *process_guard = Some(PythonProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        });

        drop(process_guard);

        let health_result = self.health_check().await;
        let mut is_healthy = self.is_healthy.write().await;
        *is_healthy = health_result.is_ok();

        if health_result.is_err() {
            return Err(PythonBridgeError::HealthCheckFailed);
        }

        log::info!("Python face detection bridge started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), PythonBridgeError> {
        let mut process_guard = self.process.lock().await;

        if let Some(mut process) = process_guard.take() {
            let shutdown_cmd = PythonCommand::Shutdown;
            let cmd_json = serde_json::to_string(&shutdown_cmd)
                .map_err(|e| PythonBridgeError::SerializationError(e.to_string()))?;

            let _ = writeln!(process.stdin, "{cmd_json}");
            let _ = process.stdin.flush();

            std::thread::sleep(std::time::Duration::from_millis(500));

            let _ = process.child.kill();
            let _ = process.child.wait();
        }

        let mut is_healthy = self.is_healthy.write().await;
        *is_healthy = false;

        log::info!("Python face detection bridge stopped");
        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool, PythonBridgeError> {
        let response = self.send_command(PythonCommand::Health).await?;

        match response {
            PythonResponse::Success { .. } => Ok(true),
            PythonResponse::Error { message, .. } => {
                log::warn!("Python bridge health check failed: {message}");
                Err(PythonBridgeError::HealthCheckFailed)
            }
        }
    }

    pub async fn detect_faces(
        &self,
        image_data: &[u8],
        detect_landmarks: bool,
        detect_attributes: bool,
    ) -> Result<Vec<PythonFaceDetection>, PythonBridgeError> {
        let image_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            image_data,
        );

        let command = PythonCommand::Detect {
            image_base64,
            detect_landmarks,
            detect_attributes,
            model: self.config.model.as_str().to_string(),
        };

        let response = self.send_command(command).await?;

        match response {
            PythonResponse::Success { data } => {
                let faces: Vec<PythonFaceDetection> = serde_json::from_value(data)
                    .map_err(|e| PythonBridgeError::DeserializationError(e.to_string()))?;
                Ok(faces)
            }
            PythonResponse::Error { message, .. } => {
                Err(PythonBridgeError::DetectionFailed(message))
            }
        }
    }

    pub async fn verify_faces(
        &self,
        image1_data: &[u8],
        image2_data: &[u8],
    ) -> Result<PythonVerificationResult, PythonBridgeError> {
        let image1_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            image1_data,
        );
        let image2_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            image2_data,
        );

        let command = PythonCommand::Verify {
            image1_base64,
            image2_base64,
            model: self.config.model.as_str().to_string(),
        };

        let response = self.send_command(command).await?;

        match response {
            PythonResponse::Success { data } => {
                let result: PythonVerificationResult = serde_json::from_value(data)
                    .map_err(|e| PythonBridgeError::DeserializationError(e.to_string()))?;
                Ok(result)
            }
            PythonResponse::Error { message, .. } => {
                Err(PythonBridgeError::VerificationFailed(message))
            }
        }
    }

    pub async fn extract_embedding(
        &self,
        image_data: &[u8],
    ) -> Result<Vec<f64>, PythonBridgeError> {
        let image_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            image_data,
        );

        let command = PythonCommand::ExtractEmbedding {
            image_base64,
            model: self.config.model.as_str().to_string(),
        };

        let response = self.send_command(command).await?;

        match response {
            PythonResponse::Success { data } => {
                let embedding: Vec<f64> = serde_json::from_value(data)
                    .map_err(|e| PythonBridgeError::DeserializationError(e.to_string()))?;
                Ok(embedding)
            }
            PythonResponse::Error { message, .. } => {
                Err(PythonBridgeError::EmbeddingFailed(message))
            }
        }
    }

    pub async fn analyze_face(
        &self,
        image_data: &[u8],
        attributes: Vec<String>,
    ) -> Result<PythonFaceAttributes, PythonBridgeError> {
        let image_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            image_data,
        );

        let command = PythonCommand::Analyze {
            image_base64,
            attributes,
        };

        let response = self.send_command(command).await?;

        match response {
            PythonResponse::Success { data } => {
                let attrs: PythonFaceAttributes = serde_json::from_value(data)
                    .map_err(|e| PythonBridgeError::DeserializationError(e.to_string()))?;
                Ok(attrs)
            }
            PythonResponse::Error { message, .. } => {
                Err(PythonBridgeError::AnalysisFailed(message))
            }
        }
    }

    pub async fn cache_embedding(&self, person_id: &str, embedding: Vec<f64>) {
        let mut cache = self.embeddings_cache.write().await;
        cache.insert(person_id.to_string(), embedding);
    }

    pub async fn get_cached_embedding(&self, person_id: &str) -> Option<Vec<f64>> {
        let cache = self.embeddings_cache.read().await;
        cache.get(person_id).cloned()
    }

    pub async fn find_matching_face(
        &self,
        image_data: &[u8],
        threshold: f64,
    ) -> Result<Option<(String, f64)>, PythonBridgeError> {
        let query_embedding = self.extract_embedding(image_data).await?;
        let cache = self.embeddings_cache.read().await;

        let mut best_match: Option<(String, f64)> = None;

        for (person_id, cached_embedding) in cache.iter() {
            let distance = self.cosine_distance(&query_embedding, cached_embedding);
            let similarity = 1.0 - distance;

            if similarity >= threshold {
                if let Some((_, best_similarity)) = &best_match {
                    if similarity > *best_similarity {
                        best_match = Some((person_id.clone(), similarity));
                    }
                } else {
                    best_match = Some((person_id.clone(), similarity));
                }
            }
        }

        Ok(best_match)
    }

    fn cosine_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 1.0;
        }

        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (norm_a * norm_b))
    }

    async fn send_command(&self, command: PythonCommand) -> Result<PythonResponse, PythonBridgeError> {
        let mut process_guard = self.process.lock().await;

        let process = process_guard
            .as_mut()
            .ok_or(PythonBridgeError::ProcessNotRunning)?;

        let cmd_json = serde_json::to_string(&command)
            .map_err(|e| PythonBridgeError::SerializationError(e.to_string()))?;

        writeln!(process.stdin, "{cmd_json}")
            .map_err(|e| PythonBridgeError::CommunicationError(e.to_string()))?;

        process.stdin.flush()
            .map_err(|e| PythonBridgeError::CommunicationError(e.to_string()))?;

        let mut response_line = String::new();
        process.stdout.read_line(&mut response_line)
            .map_err(|e| PythonBridgeError::CommunicationError(e.to_string()))?;

        let response: PythonResponse = serde_json::from_str(&response_line)
            .map_err(|e| PythonBridgeError::DeserializationError(e.to_string()))?;

        Ok(response)
    }

    pub async fn is_running(&self) -> bool {
        let is_healthy = self.is_healthy.read().await;
        *is_healthy
    }

    pub fn get_model(&self) -> &PythonModel {
        &self.config.model
    }

    pub async fn set_model(&mut self, model: PythonModel) -> Result<(), PythonBridgeError> {
        self.stop().await?;
        self.config.model = model;
        self.start().await?;
        Ok(())
    }
}

impl Drop for PythonFaceBridge {
    fn drop(&mut self) {
        if let Ok(mut process_guard) = self.process.try_lock() {
            if let Some(mut process) = process_guard.take() {
                let _ = process.child.kill();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PythonBridgeError {
    ProcessSpawnFailed(String),
    ProcessNotRunning,
    CommunicationError(String),
    SerializationError(String),
    DeserializationError(String),
    HealthCheckFailed,
    DetectionFailed(String),
    VerificationFailed(String),
    EmbeddingFailed(String),
    AnalysisFailed(String),
    Timeout,
    ModelNotSupported(String),
}

impl std::fmt::Display for PythonBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessSpawnFailed(msg) => write!(f, "Failed to spawn Python process: {msg}"),
            Self::ProcessNotRunning => write!(f, "Python process is not running"),
            Self::CommunicationError(msg) => write!(f, "Communication error: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {msg}"),
            Self::HealthCheckFailed => write!(f, "Python bridge health check failed"),
            Self::DetectionFailed(msg) => write!(f, "Face detection failed: {msg}"),
            Self::VerificationFailed(msg) => write!(f, "Face verification failed: {msg}"),
            Self::EmbeddingFailed(msg) => write!(f, "Embedding extraction failed: {msg}"),
            Self::AnalysisFailed(msg) => write!(f, "Face analysis failed: {msg}"),
            Self::Timeout => write!(f, "Operation timed out"),
            Self::ModelNotSupported(model) => write!(f, "Model not supported: {model}"),
        }
    }
}

impl std::error::Error for PythonBridgeError {}

pub async fn create_python_bridge(model: Option<PythonModel>) -> Result<PythonFaceBridge, PythonBridgeError> {
    let config = PythonBridgeConfig {
        model: model.unwrap_or_default(),
        ..Default::default()
    };

    let bridge = PythonFaceBridge::new(config);
    bridge.start().await?;
    Ok(bridge)
}

pub async fn detect_faces_python(
    image_data: &[u8],
    detect_landmarks: bool,
    detect_attributes: bool,
) -> Result<Vec<PythonFaceDetection>, PythonBridgeError> {
    let bridge = create_python_bridge(None).await?;
    let result = bridge.detect_faces(image_data, detect_landmarks, detect_attributes).await;
    let _ = bridge.stop().await;
    result
}

pub async fn verify_faces_python(
    image1_data: &[u8],
    image2_data: &[u8],
) -> Result<PythonVerificationResult, PythonBridgeError> {
    let bridge = create_python_bridge(None).await?;
    let result = bridge.verify_faces(image1_data, image2_data).await;
    let _ = bridge.stop().await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_model_from_str() {
        assert_eq!("mediapipe".parse::<PythonModel>(), Ok(PythonModel::MediaPipe));
        assert_eq!("deepface".parse::<PythonModel>(), Ok(PythonModel::DeepFace));
        assert!("unknown".parse::<PythonModel>().is_err());
    }

    #[test]
    fn test_python_model_as_str() {
        assert_eq!(PythonModel::MediaPipe.as_str(), "mediapipe");
        assert_eq!(PythonModel::DeepFace.as_str(), "deepface");
    }

    #[test]
    fn test_cosine_distance() {
        let bridge = PythonFaceBridge::new(PythonBridgeConfig::default());

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((bridge.cosine_distance(&a, &b) - 0.0).abs() < 0.0001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((bridge.cosine_distance(&c, &d) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_error_display() {
        let error = PythonBridgeError::ProcessNotRunning;
        assert!(error.to_string().contains("not running"));

        let error = PythonBridgeError::DetectionFailed("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }

    #[test]
    fn test_command_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let cmd = PythonCommand::Health;
        let json = serde_json::to_string(&cmd)?;
        assert!(json.contains("health"));
        Ok(())
    }
}
