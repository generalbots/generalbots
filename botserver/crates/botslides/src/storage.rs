use crate::types::{Presentation, PresentationMetadata, Slide};
use crate::utils::{create_content_slide, create_default_theme, create_title_slide, create_blank_slide};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

type CacheEntry = (Vec<u8>, DateTime<Utc>);
type CacheMap = HashMap<String, CacheEntry>;

static PRESENTATION_CACHE: once_cell::sync::Lazy<RwLock<CacheMap>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL_SECS: i64 = 3600;

pub trait DriveOps: Clone + Send + Sync + 'static {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, String>> + Send;

    fn list_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> impl std::future::Future<Output = Result<Vec<String>, String>> + Send;

    fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

pub fn get_user_presentations_path(user_id: &str) -> String {
    format!("users/{user_id}/presentations")
}

pub fn get_current_user_id() -> String {
    "default-user".to_string()
}

pub fn generate_presentation_id() -> String {
    Uuid::new_v4().to_string()
}

pub async fn cache_presentation_bytes(pres_id: &str, bytes: Vec<u8>) {
    let mut cache = PRESENTATION_CACHE.write().await;
    cache.insert(pres_id.to_string(), (bytes, Utc::now()));
    let now = Utc::now();
    cache.retain(|_, (_, modified)| (now - *modified).num_seconds() < CACHE_TTL_SECS);
}

pub async fn get_cached_presentation_bytes(pres_id: &str) -> Option<Vec<u8>> {
    let cache = PRESENTATION_CACHE.read().await;
    cache.get(pres_id).map(|(bytes, _)| bytes.clone())
}

fn extract_id_from_path(path: &str) -> String {
    let raw = path.split('/').next_back().unwrap_or_default();
    raw.strip_suffix(".json")
        .or_else(|| raw.strip_suffix(".pptx"))
        .unwrap_or(raw)
        .to_string()
}

pub async fn save_presentation_to_drive<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation: &Presentation,
) -> Result<(), String> {
    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation.id
    );
    let content = serde_json::to_string_pretty(presentation)
        .map_err(|e| format!("Serialization error: {e}"))?;

    drive
        .put_object("gbo", &path, content.into_bytes(), "application/json")
        .await?;

    Ok(())
}

pub async fn load_presentation_from_drive<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &Option<String>,
) -> Result<Presentation, String> {
    let presentation_id = presentation_id
        .as_ref()
        .ok_or_else(|| "Presentation ID is required".to_string())?;

    load_presentation_by_id(drive, user_id, presentation_id).await
}

pub async fn load_presentation_by_id<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &str,
) -> Result<Presentation, String> {
    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let bytes = drive.get_object("gbo", &path).await?;

    let presentation: Presentation =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse presentation: {e}"))?;

    Ok(presentation)
}

pub async fn list_presentations_from_drive<D: DriveOps>(
    drive: &D,
    user_id: &str,
) -> Result<Vec<PresentationMetadata>, String> {
    let prefix = format!("{}/", get_user_presentations_path(user_id));

    let keys = drive.list_objects("gbo", &prefix).await?;

    let mut presentations = Vec::new();

    for key in &keys {
        if key.ends_with(".json") {
            let id = extract_id_from_path(key);
            if let Ok(presentation) = load_presentation_by_id(drive, user_id, &id).await {
                presentations.push(PresentationMetadata {
                    id: presentation.id,
                    name: presentation.name,
                    owner_id: presentation.owner_id,
                    slide_count: presentation.slides.len(),
                    created_at: presentation.created_at,
                    updated_at: presentation.updated_at,
                });
            }
        }
    }

    presentations.sort_by_key(|b| std::cmp::Reverse(b.updated_at));

    Ok(presentations)
}

pub async fn delete_presentation_from_drive<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &Option<String>,
) -> Result<(), String> {
    let presentation_id = presentation_id
        .as_ref()
        .ok_or_else(|| "Presentation ID is required".to_string())?;

    let json_path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );
    let pptx_path = format!(
        "{}/{}.pptx",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let _ = drive.delete_object("gbo", &json_path).await;
    let _ = drive.delete_object("gbo", &pptx_path).await;

    Ok(())
}

pub fn create_new_presentation() -> Presentation {
    let theme = create_default_theme();
    let id = generate_presentation_id();

    Presentation {
        id,
        name: "Untitled Presentation".to_string(),
        owner_id: get_current_user_id(),
        slides: vec![create_title_slide(&theme)],
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn create_slide_with_layout(layout: &str, theme: &crate::types::PresentationTheme) -> Slide {
    match layout {
        "title" => create_title_slide(theme),
        "content" => create_content_slide(theme),
        "blank" => create_blank_slide(theme),
        _ => create_content_slide(theme),
    }
}
