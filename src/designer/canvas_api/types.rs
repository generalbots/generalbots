use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub width: f64,
    pub height: f64,
    pub background_color: String,
    pub grid_enabled: bool,
    pub grid_size: i32,
    pub snap_to_grid: bool,
    pub zoom_level: f64,
    pub elements: Vec<CanvasElement>,
    pub layers: Vec<Layer>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasElement {
    pub id: Uuid,
    pub element_type: ElementType,
    pub layer_id: Uuid,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub opacity: f64,
    pub visible: bool,
    pub locked: bool,
    pub name: Option<String>,
    pub style: ElementStyle,
    pub properties: ElementProperties,
    pub z_index: i32,
    pub parent_id: Option<Uuid>,
    pub children: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Polygon,
    Path,
    Text,
    Image,
    Icon,
    Group,
    Frame,
    Component,
    Html,
    Svg,
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rectangle => write!(f, "rectangle"),
            Self::Ellipse => write!(f, "ellipse"),
            Self::Line => write!(f, "line"),
            Self::Arrow => write!(f, "arrow"),
            Self::Polygon => write!(f, "polygon"),
            Self::Path => write!(f, "path"),
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::Icon => write!(f, "icon"),
            Self::Group => write!(f, "group"),
            Self::Frame => write!(f, "frame"),
            Self::Component => write!(f, "component"),
            Self::Html => write!(f, "html"),
            Self::Svg => write!(f, "svg"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementStyle {
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
    pub shadow: Option<ShadowStyle>,
    pub blur: Option<f64>,
    pub border_radius: Option<BorderRadius>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillStyle {
    pub fill_type: FillType,
    pub color: Option<String>,
    pub gradient: Option<Gradient>,
    pub pattern: Option<PatternFill>,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FillType {
    Solid,
    LinearGradient,
    RadialGradient,
    Pattern,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub angle: f64,
    pub center_x: Option<f64>,
    pub center_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFill {
    pub pattern_type: String,
    pub scale: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: String,
    pub width: f64,
    pub dash_array: Option<Vec<f64>>,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowStyle {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub spread: f64,
    pub inset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderRadius {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl BorderRadius {
    pub fn uniform(radius: f64) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementProperties {
    pub text_content: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub text_align: Option<TextAlign>,
    pub vertical_align: Option<VerticalAlign>,
    pub line_height: Option<f64>,
    pub letter_spacing: Option<f64>,
    pub text_decoration: Option<String>,
    pub text_color: Option<String>,
    pub image_url: Option<String>,
    pub image_fit: Option<ImageFit>,
    pub icon_name: Option<String>,
    pub icon_set: Option<String>,
    pub html_content: Option<String>,
    pub svg_content: Option<String>,
    pub path_data: Option<String>,
    pub points: Option<Vec<Point>>,
    pub arrow_start: Option<ArrowHead>,
    pub arrow_end: Option<ArrowHead>,
    pub component_id: Option<Uuid>,
    pub component_props: Option<HashMap<String, serde_json::Value>>,
    pub constraints: Option<Constraints>,
    pub auto_layout: Option<AutoLayout>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowHead {
    None,
    Triangle,
    Circle,
    Diamond,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub horizontal: ConstraintType,
    pub vertical: ConstraintType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    Fixed,
    Min,
    Max,
    Center,
    Scale,
    Stretch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLayout {
    pub direction: LayoutDirection,
    pub spacing: f64,
    pub padding_top: f64,
    pub padding_right: f64,
    pub padding_bottom: f64,
    pub padding_left: f64,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f64,
    pub blend_mode: BlendMode,
    pub z_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub canvas_data: serde_json::Value,
    pub is_system: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLibraryItem {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub url: Option<String>,
    pub svg_content: Option<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub is_system: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Icon,
    Image,
    Illustration,
    Shape,
    Component,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCanvasRequest {
    pub name: String,
    pub description: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub template_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCanvasRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub background_color: Option<String>,
    pub grid_enabled: Option<bool>,
    pub grid_size: Option<i32>,
    pub snap_to_grid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddElementRequest {
    pub element_type: ElementType,
    pub layer_id: Option<Uuid>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub style: Option<ElementStyle>,
    pub properties: Option<ElementProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateElementRequest {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotation: Option<f64>,
    pub scale_x: Option<f64>,
    pub scale_y: Option<f64>,
    pub opacity: Option<f64>,
    pub visible: Option<bool>,
    pub locked: Option<bool>,
    pub name: Option<String>,
    pub style: Option<ElementStyle>,
    pub properties: Option<ElementProperties>,
    pub z_index: Option<i32>,
    pub layer_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveElementRequest {
    pub delta_x: f64,
    pub delta_y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeElementRequest {
    pub width: f64,
    pub height: f64,
    pub anchor: ResizeAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupElementsRequest {
    pub element_ids: Vec<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignElementsRequest {
    pub element_ids: Vec<Uuid>,
    pub alignment: Alignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Left,
    CenterHorizontal,
    Right,
    Top,
    CenterVertical,
    Bottom,
    DistributeHorizontal,
    DistributeVertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLayerRequest {
    pub name: String,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLayerRequest {
    pub name: Option<String>,
    pub visible: Option<bool>,
    pub locked: Option<bool>,
    pub opacity: Option<f64>,
    pub blend_mode: Option<BlendMode>,
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub quality: Option<i32>,
    pub scale: Option<f64>,
    pub background: Option<bool>,
    pub element_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Png,
    Jpg,
    Svg,
    Pdf,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub format: ExportFormat,
    pub data: String,
    pub content_type: String,
    pub filename: String,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignRequest {
    pub prompt: String,
    pub context: Option<AiDesignContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignContext {
    pub selected_elements: Option<Vec<Uuid>>,
    pub canvas_state: Option<serde_json::Value>,
    pub style_preferences: Option<StylePreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreferences {
    pub color_palette: Option<Vec<String>>,
    pub font_families: Option<Vec<String>>,
    pub design_style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDesignResponse {
    pub success: bool,
    pub elements_created: Vec<CanvasElement>,
    pub elements_modified: Vec<Uuid>,
    pub message: String,
    pub html_preview: Option<String>,
    pub svg_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEvent {
    pub event_type: CanvasEventType,
    pub canvas_id: Uuid,
    pub user_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasEventType {
    ElementAdded,
    ElementUpdated,
    ElementDeleted,
    ElementMoved,
    ElementResized,
    ElementsGrouped,
    ElementsUngrouped,
    LayerAdded,
    LayerUpdated,
    LayerDeleted,
    CanvasUpdated,
    SelectionChanged,
    CursorMoved,
    UndoPerformed,
    RedoPerformed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedoState {
    pub canvas_id: Uuid,
    pub undo_stack: Vec<CanvasSnapshot>,
    pub redo_stack: Vec<CanvasSnapshot>,
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    pub id: Uuid,
    pub elements: Vec<CanvasElement>,
    pub layers: Vec<Layer>,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_display() {
        assert_eq!(ElementType::Rectangle.to_string(), "rectangle");
        assert_eq!(ElementType::Ellipse.to_string(), "ellipse");
        assert_eq!(ElementType::Text.to_string(), "text");
    }

    #[test]
    fn test_border_radius_uniform() {
        let radius = BorderRadius::uniform(10.0);
        assert_eq!(radius.top_left, 10.0);
        assert_eq!(radius.top_right, 10.0);
        assert_eq!(radius.bottom_right, 10.0);
        assert_eq!(radius.bottom_left, 10.0);
    }

    #[test]
    fn test_blend_mode_default() {
        let mode = BlendMode::default();
        assert_eq!(mode, BlendMode::Normal);
    }

    #[test]
    fn test_element_style_default() {
        let style = ElementStyle::default();
        assert!(style.fill.is_none());
        assert!(style.stroke.is_none());
        assert!(style.blur.is_none());
    }
}
