use super::*;

/// Shell forms send hidden/number inputs as strings ("0"), while the API
/// declares usize/f64. Accept both shapes instead of rejecting the request.
fn deserialize_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Option<serde_json::Value> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Number(number)) => number.as_u64().unwrap_or(0) as usize,
        Some(serde_json::Value::String(text)) => text.trim().parse().unwrap_or(0),
        _ => 0,
    })
}

fn deserialize_opt_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Option<serde_json::Value> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Number(number)) => number.as_u64().map(|value| value as usize),
        Some(serde_json::Value::String(text)) => text.trim().parse().ok(),
        _ => None,
    })
}

fn deserialize_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Option<serde_json::Value> as serde::Deserialize>::deserialize(deserializer)?;
    Ok(match value {
        Some(serde_json::Value::Number(number)) => number.as_f64(),
        Some(serde_json::Value::String(text)) => text.trim().parse().ok(),
        _ => None,
    })
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePresentationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional so partial payloads work: the title field posts
    /// `{id, action}` only and must not wipe the stored deck.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slides: Option<Vec<Slide>>,
    #[serde(default)]
    pub theme: Option<PresentationTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSlideRequest {
    pub presentation_id: String,
    pub layout: String,
    #[serde(default, deserialize_with = "deserialize_opt_usize")]
    pub position: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSlideRequest {
    pub presentation_id: String,
    pub slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSlideRequest {
    pub presentation_id: String,
    pub slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderSlidesRequest {
    pub presentation_id: String,
    pub slide_order: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element: SlideElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element: SlideElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteElementRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyThemeRequest {
    pub presentation_id: String,
    #[serde(default)]
    pub theme: Option<PresentationTheme>,
    /// The picker sends the preset name only; the handler resolves it to a
    /// theme rather than rejecting the payload.
    #[serde(default)]
    pub theme_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSlideNotesRequest {
    pub presentation_id: String,
    #[serde(default, deserialize_with = "deserialize_usize")]
    pub slide_index: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    // The toolbar posts `presentation_id`; JSON API callers use `id`.
    #[serde(alias = "presentation_id")]
    pub id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlidesAiRequest {
    // The AI modal posts `prompt`; the compact API uses `command`.
    #[serde(alias = "prompt")]
    pub command: String,
    #[serde(default)]
    pub slide_index: Option<usize>,
    #[serde(default)]
    pub presentation_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SlidesAiResponse {
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFromDriveRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCursorRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSelectionRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCursorsResponse {
    pub cursors: Vec<CollaborationCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSelectionsResponse {
    pub selections: Vec<CollaborationSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTransitionRequest {
    pub presentation_id: String,
    #[serde(default, deserialize_with = "deserialize_usize")]
    pub slide_index: usize,
    #[serde(default)]
    pub transition: Option<TransitionConfig>,
    // The toolbar modal posts flat transition fields.
    #[serde(default)]
    pub transition_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_f64")]
    pub duration: Option<f64>,
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTransitionToAllRequest {
    pub presentation_id: String,
    pub transition: TransitionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTransitionRequest {
    pub presentation_id: String,
    pub slide_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMediaRequest {
    pub presentation_id: String,
    #[serde(default)]
    pub slide_index: usize,
    #[serde(default)]
    pub media: Option<MediaElement>,
    /// The image modal offers an external URL; accept it as a flat field and
    /// build the MediaElement server-side (file upload has no storage yet).
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMediaRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub media_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoplay: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_playback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMediaRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMediaResponse {
    pub media: Vec<MediaElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartPresenterRequest {
    pub presentation_id: String,
    // The "Apresentar" button posts only the presentation id; the viewer
    // settings are optional defaults, not a requirement to start.
    #[serde(default)]
    pub settings: PresenterViewSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresenterRequest {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_slide: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<PresenterViewSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndPresenterRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenterSessionResponse {
    pub session: PresenterSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenterNotesResponse {
    pub slide_index: usize,
    pub notes: Option<String>,
    pub next_slide_notes: Option<String>,
    pub next_slide_thumbnail: Option<String>,
}
