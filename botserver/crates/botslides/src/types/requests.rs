use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePresentationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub slides: Vec<Slide>,
    pub theme: PresentationTheme,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub theme: PresentationTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSlideNotesRequest {
    pub presentation_id: String,
    pub slide_index: usize,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
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
    pub slide_index: usize,
    pub transition: TransitionConfig,
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
    pub slide_index: usize,
    pub media: MediaElement,
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
