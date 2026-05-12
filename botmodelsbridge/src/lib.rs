pub mod face_api;
pub mod insightface;
pub mod opencv;
pub mod python_bridge;
pub mod rekognition;

pub use face_api::{
    BoundingBox, DetectedFace, FaceApiConfig, FaceApiError, FaceApiProvider, FaceApiService,
    FaceAttributes, FaceDetectionRequest, FaceDetectionResponse, FaceLandmarks, FaceMatch,
    FaceSearchResult, FaceVerificationResult, Gender, GlassesType, Point2D,
    BlurLevel, EmotionScores, ExposureLevel, FacialHair, HeadPose, NoiseLevel, Occlusion,
    create_azure_config, create_aws_config, create_opencv_config, create_insightface_config,
};
pub use python_bridge::{
    PythonBridgeConfig, PythonBridgeError, PythonCommand, PythonFaceBridge, PythonFaceDetection,
    PythonModel, PythonResponse, create_python_bridge, detect_faces_python, verify_faces_python,
};
pub use insightface::{
    InsightFaceConfig, InsightFaceError, InsightFaceModel, InsightFaceService,
};
pub use opencv::{
    OpenCvDetectorConfig, OpenCvError, OpenCvFaceDetector,
};
pub use rekognition::{
    RekognitionConfig, RekognitionError, RekognitionService,
};
