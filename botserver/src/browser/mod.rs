pub use botbrowser::*;

use std::sync::Arc;
pub struct AppStateBrowserState(pub Arc<botcore::shared::state::AppState>);
impl botbrowser::api::BrowserState for AppStateBrowserState {}
