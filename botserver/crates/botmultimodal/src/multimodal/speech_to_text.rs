use super::*;

use botsecurity_core::validation::{fetch_user_url, FetchPolicy};

/// Cap for audio/image payloads pulled from a caller-supplied URL.
const MEDIA_MAX_BYTES: usize = 32 * 1024 * 1024;

/// Fetches a caller-supplied media URL through the shared SSRF guard (#2131).
async fn fetch_media_bytes(
    url: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let policy = FetchPolicy::default()
        .with_timeout(Duration::from_secs(60))
        .with_max_bytes(max_bytes);
    let response = fetch_user_url(url, &policy).await.map_err(|e| {
        log::warn!(
            "multimodal: refused media URL ({}): {}",
            e.kind().as_str(),
            e.detail()
        );
        Box::new(e) as Box<dyn std::error::Error + Send + Sync>
    })?;
    if !response.is_success() {
        return Err(format!("media URL answered HTTP {}", response.status).into());
    }
    if response.truncated {
        return Err(format!("media exceeds the {max_bytes} byte cap").into());
    }
    Ok(response.body)
}

impl BotModelsClient {
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

        let audio_data = if audio_url_or_path.starts_with("http://")
            || audio_url_or_path.starts_with("https://")
        {
            fetch_media_bytes(audio_url_or_path, MEDIA_MAX_BYTES).await?
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
            log::error!("Speech to text failed: {}", error_text);
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

        let image_data = if image_url_or_path.starts_with("http://")
            || image_url_or_path.starts_with("https://")
        {
            fetch_media_bytes(image_url_or_path, MEDIA_MAX_BYTES).await?
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
            log::error!("Barcode scan failed: {}", error_text);
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
        // #2131 — the URL is caller-supplied: guarded fetch, size cap, and
        // nothing is written when the policy refuses the target.
        let bytes = fetch_media_bytes(url, MEDIA_MAX_BYTES).await?;
        tokio::fs::write(local_path, bytes).await?;
        Ok(())
    }
}
