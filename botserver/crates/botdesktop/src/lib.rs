pub mod models;
pub mod proxy;
pub mod routes;
pub mod schema;
pub mod session_manager;
pub mod types;

// Re-exports for convenience
pub use models::{ConnectionConfig, DesktopSession, SessionStatus};
pub use routes::{router, AppState};
pub use session_manager::{SessionError, SessionManager};
pub use types::{
    ApiResponse, ConnectionCreateRequest, ConnectionSummary, HealthCheckRequest, HealthCheckResponse,
    WsProxyMessage,
};
