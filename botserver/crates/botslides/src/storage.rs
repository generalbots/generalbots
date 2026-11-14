//! Deck persistence.
//!
//! **Storage boundary (#2137).** A deck is one object at
//! `users/{owner}/presentations/{id}.json` (plus `{id}.pptx` when the deck was
//! imported from a package) inside the deployment's deck bucket (`gbo`).
//! `{owner}` is the verified user id from [`crate::auth::SlidesUser`] — never a
//! shared default and never a client-supplied value — so one user's prefix is
//! unreachable from another user's request. The user prefix is the isolation
//! boundary here; a per-tenant bucket (instead of the shared `gbo`) is a
//! follow-up that needs a data migration, so this change keeps the bucket and
//! removes the shared owner.
//!
//! **Legacy data.** Decks written before #2137 live under
//! `users/default-user/presentations/`. No verified identity resolves to
//! `default-user`, so that prefix is quarantined in place: still on Drive, no
//! longer served to any caller. Move it to the install owner (or to
//! `legacy/`) with `mc` before deleting it.

use crate::types::{DeckSlideInput, Presentation, PresentationMetadata, PresentationTheme, Slide};
use crate::utils::{
    create_blank_slide, create_content_slide, create_default_theme, create_title_slide,
};
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

/// #2041 — the uploaded PPTX is kept next to the deck JSON so a save can patch
/// the original package instead of regenerating it.
pub fn get_presentation_pptx_path(user_id: &str, presentation_id: &str) -> String {
    format!(
        "{}/{}.pptx",
        get_user_presentations_path(user_id),
        presentation_id
    )
}

pub async fn save_presentation_pptx<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let path = get_presentation_pptx_path(user_id, presentation_id);
    drive
        .put_object(
            "gbo",
            &path,
            bytes.to_vec(),
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        )
        .await
}

/// Returns the preserved original package, or `None` for decks imported before
/// #2041 (JSON only). A read error is treated as "not preserved" and logged at
/// debug level: the export falls back to regeneration, which is the old
/// behaviour.
pub async fn load_presentation_pptx<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &str,
) -> Option<Vec<u8>> {
    let path = get_presentation_pptx_path(user_id, presentation_id);
    match drive.get_object("gbo", &path).await {
        Ok(bytes) => Some(bytes),
        Err(error) => {
            log::debug!("slides: no preserved PPTX at {path}: {error}");
            None
        }
    }
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
    // Empty ids turn into a degenerate S3 GET (`presentations/.json`) whose raw
    // storage error would leak to the client — validate first.
    if presentation_id.trim().is_empty() {
        return Err("Presentation ID is required".to_string());
    }
    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let bytes = drive.get_object("gbo", &path).await.map_err(|error| {
        if error.contains("404") {
            "Presentation not found".to_string()
        } else {
            log::error!("presentation load failed: {error}");
            "Failed to load presentation".to_string()
        }
    })?;

    let presentation: Presentation =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse presentation: {e}"))?;

    // The path already scopes the read to `user_id`; the stored owner is a
    // second, independent check so a deck copied (or forged) into the wrong
    // prefix is still never served (#2137).
    if presentation.owner_id != user_id {
        log::warn!(
            "slides: deck {presentation_id} is owned by '{}', refusing read for '{user_id}'",
            presentation.owner_id
        );
        return Err("Presentation not found".to_string());
    }

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

/// Deletes a deck from the caller's own namespace.
///
/// A missing (or foreign) id answers `Err("Presentation not found")` so the
/// handler renders `404`: delete used to report success for any id, which
/// hid typos and made a cross-user probe look like a successful delete.
pub async fn delete_presentation_from_drive<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation_id: &str,
) -> Result<(), String> {
    load_presentation_by_id(drive, user_id, presentation_id).await?;

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

/// Builds an empty deck owned by `owner_id`.
pub fn create_new_presentation(owner_id: &str) -> Presentation {
    let theme = create_default_theme();
    let id = generate_presentation_id();

    Presentation {
        id,
        name: "Untitled Presentation".to_string(),
        owner_id: owner_id.to_string(),
        slides: vec![create_title_slide(&theme)],
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_bucket: None,
        source_path: None,
    }
}

/// Escapes text placed in the generated slide HTML.
fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A content slide whose heading and body come from the request.
fn slide_from_input(input: &DeckSlideInput, theme: &PresentationTheme) -> Slide {
    let mut slide = create_content_slide(theme);
    if let Some(title) = input.title.as_deref() {
        if let Some(element) = slide.elements.get_mut(0) {
            element.content.text = Some(title.to_string());
            element.content.html = Some(format!("<h2>{}</h2>", escape_text(title)));
        }
    }
    if let Some(body) = input.body.as_deref() {
        if let Some(element) = slide.elements.get_mut(1) {
            element.content.text = Some(body.to_string());
            element.content.html = Some(format!("<p>{}</p>", escape_text(body)));
        }
    }
    slide
}

/// A cover slide built from the first `{title, body}` entry so a deck created
/// in one call opens with a proper intro instead of a generic content page.
fn title_slide_from_input(input: &DeckSlideInput, theme: &PresentationTheme) -> Slide {
    let mut slide = create_title_slide(theme);
    if let Some(title) = input.title.as_deref() {
        if let Some(element) = slide.elements.get_mut(0) {
            element.content.text = Some(title.to_string());
            element.content.html = Some(format!("<h1>{}</h1>", escape_text(title)));
        }
    }
    if let Some(body) = input.body.as_deref() {
        if let Some(element) = slide.elements.get_mut(1) {
            element.content.text = Some(body.to_string());
            element.content.html = Some(format!("<p>{}</p>", escape_text(body)));
        }
    } else if let Some(element) = slide.elements.get_mut(1) {
        // Drop the placeholder subtitle when the caller only supplied a title.
        element.content.text = Some(String::new());
        element.content.html = Some(String::new());
    }
    slide
}

/// Builds a deck from a title and optional `{title, body}` slides without
/// touching Drive, so the created artifact is testable on its own (#2143).
pub fn create_presentation_with(
    owner_id: &str,
    title: Option<&str>,
    slides: Option<&[DeckSlideInput]>,
) -> Presentation {
    let mut deck = create_new_presentation(owner_id);
    if let Some(title) = title.map(str::trim).filter(|title| !title.is_empty()) {
        deck.name = title.to_string();
    }
    if let Some(inputs) = slides.filter(|inputs| !inputs.is_empty()) {
        deck.slides = inputs
            .iter()
            .enumerate()
            .map(|(index, input)| {
                if index == 0 {
                    title_slide_from_input(input, &deck.theme)
                } else {
                    slide_from_input(input, &deck.theme)
                }
            })
            .collect();
    }
    deck.updated_at = Utc::now();
    deck
}

/// Builds a new slide for the layout name the insert dialog sends. Every
/// layout the dialog offers is built by name (`crate::layouts::LAYOUTS`);
/// `two_column` used to fall through to a single-column content slide, so the
/// new slide looked unchanged (UAT SLIDE-02).
pub fn create_slide_with_layout(layout: &str, theme: &crate::types::PresentationTheme) -> Slide {
    match layout {
        "title" => create_title_slide(theme),
        "content" => create_content_slide(theme),
        "two_column" | "two-column" | "twocolumn" => crate::layouts::create_two_column_slide(theme),
        "section" => crate::layouts::create_section_slide(theme),
        "image" => crate::layouts::create_image_slide(theme),
        "blank" => create_blank_slide(theme),
        _ => create_content_slide(theme),
    }
}
