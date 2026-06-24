pub use botvibe::*;

pub fn configure_vibe_routes(_app_state: &std::sync::Arc<botcore::shared::state::AppState>) -> axum::Router {
    axum::Router::new()
}
