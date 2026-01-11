use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideMessage {
    pub msg_type: String,
    pub presentation_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slide_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: String,
    pub name: String,
    pub color: String,
    pub current_slide: Option<usize>,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub slides: Vec<Slide>,
    pub theme: PresentationTheme,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: String,
    pub layout: String,
    pub elements: Vec<SlideElement>,
    pub background: SlideBackground,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<SlideTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideElement {
    pub id: String,
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub rotation: f64,
    pub content: ElementContent,
    pub style: ElementStyle,
    #[serde(default)]
    pub animations: Vec<Animation>,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_data: Option<ChartData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_data: Option<TableData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlideBackground {
    #[serde(default = "default_bg_type")]
    pub bg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient: Option<GradientStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_fit: Option<String>,
}

fn default_bg_type() -> String {
    "solid".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStyle {
    pub gradient_type: String,
    pub angle: f64,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub color: String,
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideTransition {
    pub transition_type: String,
    pub duration: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub animation_type: String,
    pub trigger: String,
    pub duration: f64,
    pub delay: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationTheme {
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub text_light: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: String,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<TableCell>>,
    pub col_widths: Vec<f64>,
    pub row_heights: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableCell {
    pub content: String,
    #[serde(default)]
    pub colspan: usize,
    #[serde(default)]
    pub rowspan: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ElementStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationMetadata {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub slide_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
