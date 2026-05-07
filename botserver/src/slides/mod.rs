use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botslides::{
    Animation, ChartData, ChartDataset, Collaborator, CollaborationCursor,
    CollaborationSelection, DriveOps as SlidesDriveOps, ElementContent, ElementStyle,
    GradientStop, GradientStyle, MediaElement, Presentation, PresentationMetadata,
    PresentationTheme, PresenterSession, PresenterViewSettings, SaveResponse, ShadowStyle,
    Slide, SlideBackground, SlideElement, SlideMessage, SlideTransition, TableCell, TableData,
    ThemeColors, ThemeFonts, TransitionConfig, TransitionSound,
    configure_slides_routes as crate_configure_slides_routes,
    handle_get_collaborators, handle_get_mentions, handle_get_presence,
    handle_get_selections, handle_get_typing, handle_slides_websocket,
    ooxml, SlidesState, DbPool as SlidesDbPool, GetDefaultBotFn as SlidesGetDefaultBotFn,
};

#[derive(Clone)]
struct NoDriveSlides;

impl botslides::DriveOps for NoDriveSlides {
    async fn put_object(
        &self, _bucket: &str, _key: &str, _body: Vec<u8>, _content_type: &str,
    ) -> Result<(), String> {
        Err("Drive not configured".to_string())
    }
    async fn get_object(&self, _bucket: &str, _key: &str) -> Result<Vec<u8>, String> {
        Err("Drive not configured".to_string())
    }
    async fn list_objects(&self, _bucket: &str, _prefix: &str) -> Result<Vec<String>, String> {
        Err("Drive not configured".to_string())
    }
    async fn delete_object(&self, _bucket: &str, _key: &str) -> Result<(), String> {
        Err("Drive not configured".to_string())
    }
}

pub fn configure_slides_routes(app_state: Arc<AppState>) -> Router {
    let crate_state = SlidesState {
        drive: None,
        conn: Arc::new(app_state.conn.clone()),
        get_default_bot: Arc::new(|conn| get_default_bot(conn)),
    };
    crate_configure_slides_routes::<NoDriveSlides>()
        .with_state(Arc::new(crate_state))
}
