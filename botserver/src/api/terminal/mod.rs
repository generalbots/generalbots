pub struct TerminalManager;
impl TerminalManager {
    pub fn new() -> Self { Self }
}
pub fn configure_terminal_routes() -> axum::Router { axum::Router::new() }
