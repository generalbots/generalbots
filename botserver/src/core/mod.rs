// Thin re-exports from botcore crate
#[cfg(feature = "automation")]
pub use botcore::automation;
pub use botcore::bootstrap;
pub use botcore::bot_database;
pub use botcore::config;
pub use botcore::config_reload;
pub use botcore::dns;
pub use botcore::features;
pub use botcore::i18n;
#[cfg(any(feature = "research", feature = "llm"))]
pub use botcore::kb;
pub use botcore::large_org_optimizer;
pub use botcore::manifest;
pub use botcore::middleware;
pub use botcore::organization;
pub use botcore::organization_invitations;
pub use botcore::organization_rbac;
pub use botcore::package_manager;
pub use botcore::performance;
pub use botcore::product;
pub use botcore::rate_limit;
pub use botcore::shared;
pub use botcore::urls;

// These remain as thin re-exports from their own crates
pub mod bot;
pub mod directory;
pub mod oauth;
pub mod secrets;
pub mod session;
pub mod state;
// Keep the incus module if needed
pub use botcore::incus;
