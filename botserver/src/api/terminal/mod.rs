pub struct TerminalManager;
impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self { Self }
}
pub fn configure_terminal_routes() -> axum::Router { axum::Router::new() }
