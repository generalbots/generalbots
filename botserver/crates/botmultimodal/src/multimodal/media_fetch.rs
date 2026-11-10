//! Guarded download of caller-supplied media URLs (#2131).
//!
//! Speech, barcode, image-description and video-description all accept an
//! `http(s)` URL from the caller (bot script, LLM tool call, API request) and
//! used to `client.get()` it directly. Every one of those goes through the
//! shared SSRF guard now, with a byte cap so a huge response cannot exhaust
//! memory before it reaches the model.

use botsecurity_core::validation::{fetch_user_url, FetchPolicy};

/// Cap for audio/image/video payloads pulled from a caller-supplied URL.
pub(crate) const MEDIA_MAX_BYTES: usize = 32 * 1024 * 1024;

pub(crate) async fn fetch_media_bytes(
    url: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let policy = FetchPolicy::default()
        .with_timeout(std::time::Duration::from_secs(60))
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
