use botlib::http_client::create_tls_client;
use log::{error, info, trace};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait ConfigProvider {
    fn get_config(&self, bot_id: &Uuid, key: &str, default: Option<&str>) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct BotModelsConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub use_https: bool,
}

impl BotModelsConfig {
    pub fn from_provider(provider: &dyn ConfigProvider, bot_id: &Uuid) -> Self {
        let enabled = provider
            .get_config(bot_id, "botmodels-enabled", Some("false"))
            .unwrap_or_default()
            .to_lowercase()
            == "true";

        let host = provider
            .get_config(bot_id, "botmodels-host", Some("0.0.0.0"))
            .unwrap_or("0.0.0.0".to_string());

        let port = provider
            .get_config(bot_id, "botmodels-port", Some("8085"))
            .unwrap_or("8085".to_string())
            .parse()
            .unwrap_or(8085);

        let api_key = provider
            .get_config(bot_id, "botmodels-api-key", Some(""))
            .unwrap_or_default();

        let use_https = provider
            .get_config(bot_id, "botmodels-https", Some("false"))
            .unwrap_or_default()
            .to_lowercase()
            == "true";

        Self {
            enabled,
            host,
            port,
            api_key,
            use_https,
        }
    }

    pub fn base_url(&self) -> String {
        let protocol = if self.use_https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }
}

#[derive(Debug, Clone)]
pub struct ImageGeneratorConfig {
    pub model: String,
    pub steps: u32,
    pub width: u32,
    pub height: u32,
    pub gpu_layers: u32,
    pub batch_size: u32,
}

impl ImageGeneratorConfig {
    pub fn from_provider(provider: &dyn ConfigProvider, bot_id: &Uuid) -> Self {
        Self {
            model: provider
                .get_config(bot_id, "image-generator-model", None)
                .unwrap_or_default(),
            steps: provider
                .get_config(bot_id, "image-generator-steps", Some("4"))
                .unwrap_or( "4".to_string())
                .parse()
                .unwrap_or(4),
            width: provider
                .get_config(bot_id, "image-generator-width", Some("512"))
                .unwrap_or( "512".to_string())
                .parse()
                .unwrap_or(512),
            height: provider
                .get_config(bot_id, "image-generator-height", Some("512"))
                .unwrap_or( "512".to_string())
                .parse()
                .unwrap_or(512),
            gpu_layers: provider
                .get_config(bot_id, "image-generator-gpu-layers", Some("20"))
                .unwrap_or( "20".to_string())
                .parse()
                .unwrap_or(20),
            batch_size: provider
                .get_config(bot_id, "image-generator-batch-size", Some("1"))
                .unwrap_or( "1".to_string())
                .parse()
                .unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoGeneratorConfig {
    pub model: String,
    pub frames: u32,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub gpu_layers: u32,
    pub batch_size: u32,
}

impl VideoGeneratorConfig {
    pub fn from_provider(provider: &dyn ConfigProvider, bot_id: &Uuid) -> Self {
        Self {
            model: provider
                .get_config(bot_id, "video-generator-model", None)
                .unwrap_or_default(),
            frames: provider
                .get_config(bot_id, "video-generator-frames", Some("24"))
                .unwrap_or( "24".to_string())
                .parse()
                .unwrap_or(24),
            fps: provider
                .get_config(bot_id, "video-generator-fps", Some("8"))
                .unwrap_or( "8".to_string())
                .parse()
                .unwrap_or(8),
            width: provider
                .get_config(bot_id, "video-generator-width", Some("320"))
                .unwrap_or( "320".to_string())
                .parse()
                .unwrap_or(320),
            height: provider
                .get_config(bot_id, "video-generator-height", Some("576"))
                .unwrap_or( "576".to_string())
                .parse()
                .unwrap_or(576),
            gpu_layers: provider
                .get_config(bot_id, "video-generator-gpu-layers", Some("15"))
                .unwrap_or( "15".to_string())
                .parse()
                .unwrap_or(15),
            batch_size: provider
                .get_config(bot_id, "video-generator-batch-size", Some("1"))
                .unwrap_or( "1".to_string())
                .parse()
                .unwrap_or(1),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImageGenerateRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct VideoGenerateRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_frames: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SpeechGenerateRequest {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerationResponse {
    pub status: String,
    pub file_path: Option<String>,
    pub generation_time: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DescribeResponse {
    pub description: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct VideoDescribeResponse {
    pub description: String,
    pub frame_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SpeechToTextResponse {
    pub text: String,
    pub language: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug)]
pub struct BotModelsClient {
    client: Client,
    config: BotModelsConfig,
    image_config: ImageGeneratorConfig,
    video_config: VideoGeneratorConfig,
}

impl BotModelsClient {
    pub fn new(
        config: BotModelsConfig,
        image_config: ImageGeneratorConfig,
        video_config: VideoGeneratorConfig,
    ) -> Self {
        let client = create_tls_client(Some(300));

        Self {
            client,
            config,
            image_config,
            video_config,
        }
    }

    pub fn from_provider_all(provider: &dyn ConfigProvider, bot_id: &Uuid) -> Self {
        let config = BotModelsConfig::from_provider(provider, bot_id);
        let image_config = ImageGeneratorConfig::from_provider(provider, bot_id);
        let video_config = VideoGeneratorConfig::from_provider(provider, bot_id);
        Self::new(config, image_config, video_config)
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn generate_image(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/image/generate", self.config.base_url());
        trace!("Generating image at {}: {}", url, prompt);

        let request = ImageGenerateRequest {
            prompt: prompt.to_string(),
            steps: Some(self.image_config.steps),
            width: Some(self.image_config.width),
            height: Some(self.image_config.height),
            guidance_scale: Some(7.5),
            seed: None,
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Image generation failed: {}", error_text);
            return Err(format!("Image generation failed: {}", error_text).into());
        }

        let result: GenerationResponse = response.json().await?;

        if result.status == "completed" {
            if let Some(file_path) = result.file_path {
                let full_url = format!("{}{}", self.config.base_url(), file_path);
                info!("Image generated: {}", full_url);
                return Ok(full_url);
            }
        }

        Err(result
            .error
            .unwrap_or("Unknown error".to_string())
            .into())
    }

    pub async fn generate_video(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/video/generate", self.config.base_url());
        trace!("Generating video at {}: {}", url, prompt);

        let request = VideoGenerateRequest {
            prompt: prompt.to_string(),
            num_frames: Some(self.video_config.frames),
            fps: Some(self.video_config.fps),
            steps: Some(50),
            seed: None,
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Video generation failed: {}", error_text);
            return Err(format!("Video generation failed: {}", error_text).into());
        }

        let result: GenerationResponse = response.json().await?;

        if result.status == "completed" {
            if let Some(file_path) = result.file_path {
                let full_url = format!("{}{}", self.config.base_url(), file_path);
                info!("Video generated: {}", full_url);
                return Ok(full_url);
            }
        }

        Err(result
            .error
            .unwrap_or("Unknown error".to_string())
            .into())
    }

    pub async fn generate_audio(
        &self,
        text: &str,
        voice: Option<&str>,
        language: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/speech/generate", self.config.base_url());
        trace!("Generating audio at {}: {}", url, text);

        let request = SpeechGenerateRequest {
            prompt: text.to_string(),
            voice: voice.map(String::from),
            language: language.map(String::from),
        };

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Audio generation failed: {}", error_text);
            return Err(format!("Audio generation failed: {}", error_text).into());
        }

        let result: GenerationResponse = response.json().await?;

        if result.status == "completed" {
            if let Some(file_path) = result.file_path {
                let full_url = format!("{}{}", self.config.base_url(), file_path);
                info!("Audio generated: {}", full_url);
                return Ok(full_url);
            }
        }

        Err(result
            .error
            .unwrap_or("Unknown error".to_string())
            .into())
    }

    pub async fn describe_image(
        &self,
        image_url_or_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/vision/describe", self.config.base_url());
        trace!("Describing image at {}: {}", url, image_url_or_path);

        let image_data = if image_url_or_path.starts_with("http") {
            let response = self.client.get(image_url_or_path).send().await?;
            response.bytes().await?.to_vec()
        } else {
            tokio::fs::read(image_url_or_path).await?
        };

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(image_data)
                .file_name("image.png")
                .mime_str("image/png")?,
        );

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Image description failed: {}", error_text);
            return Err(format!("Image description failed: {}", error_text).into());
        }

        let result: DescribeResponse = response.json().await?;
        info!("Image described: {}", result.description);
        Ok(result.description)
    }

    pub async fn describe_video(
        &self,
        video_url_or_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/vision/describe_video", self.config.base_url());
        trace!("Describing video at {}: {}", url, video_url_or_path);

        let video_data = if video_url_or_path.starts_with("http") {
            let response = self.client.get(video_url_or_path).send().await?;
            response.bytes().await?.to_vec()
        } else {
            tokio::fs::read(video_url_or_path).await?
        };

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(video_data)
                .file_name("video.mp4")
                .mime_str("video/mp4")?,
        );

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Video description failed: {}", error_text);
            return Err(format!("Video description failed: {}", error_text).into());
        }

        let result: VideoDescribeResponse = response.json().await?;
        info!("Video described: {}", result.description);
        Ok(result.description)
    }

    pub async fn speech_to_text(
        &self,
        audio_url_or_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/speech/totext", self.config.base_url());
        trace!(
            "Converting speech to text at {}: {}",
            url,
            audio_url_or_path
        );

        let audio_data = if audio_url_or_path.starts_with("http") {
            let response = self.client.get(audio_url_or_path).send().await?;
            response.bytes().await?.to_vec()
        } else {
            tokio::fs::read(audio_url_or_path).await?
        };

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(audio_data)
                .file_name("audio.wav")
                .mime_str("audio/wav")?,
        );

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Speech to text failed: {}", error_text);
            return Err(format!("Speech to text failed: {}", error_text).into());
        }

        let result: SpeechToTextResponse = response.json().await?;
        info!("Speech converted: {}", result.text);
        Ok(result.text)
    }

    pub async fn scan_barcode(
        &self,
        image_url_or_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Err("BotModels is not enabled".into());
        }

        let url = format!("{}/api/vision/barcode", self.config.base_url());
        trace!("Scanning barcode at {}: {}", url, image_url_or_path);

        let image_data = if image_url_or_path.starts_with("http") {
            let response = self.client.get(image_url_or_path).send().await?;
            response.bytes().await?.to_vec()
        } else {
            tokio::fs::read(image_url_or_path).await?
        };

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(image_data)
                .file_name("image.png")
                .mime_str("image/png")?,
        );

        let response = self
            .client
            .post(&url)
            .header("X-API-Key", &self.config.api_key)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Barcode scan failed: {}", error_text);
            return Err(format!("Barcode scan failed: {}", error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        info!("Barcode scanned: {:?}", result);
        Ok(result.to_string())
    }

    pub async fn health_check(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let url = format!("{}/api/health", self.config.base_url());
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn download_file(
        &self,
        url: &str,
        local_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(local_path, bytes).await?;
        Ok(())
    }
}
