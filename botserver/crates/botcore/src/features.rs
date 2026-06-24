
/// List of features compiled into this binary
pub const COMPILED_FEATURES: &[&str] = &[
    #[cfg(feature = "chat")]
    "chat",
    #[cfg(feature = "mail")]
    "mail",
    #[cfg(feature = "mail")]
    "email", // Alias for mail
    #[cfg(feature = "calendar")]
    "calendar",
    #[cfg(feature = "drive")]
    "drive",
    #[cfg(feature = "tasks")]
    "tasks",
    #[cfg(feature = "docs")]
    "docs",
    #[cfg(feature = "paper")]
    "paper",
    #[cfg(feature = "sheet")]
    "sheet",
    #[cfg(feature = "slides")]
    "slides",
    #[cfg(feature = "meet")]
    "meet",
    #[cfg(feature = "research")]
    "research",
    #[cfg(feature = "people")]
    "people",
    #[cfg(feature = "social")]
    "social",
    #[cfg(feature = "analytics")]
    "analytics",
    #[cfg(feature = "monitoring")]
    "monitoring",
    #[cfg(feature = "settings")]
    "settings",
    #[cfg(feature = "automation")]
    "automation",
    #[cfg(feature = "cache")]
    "cache",
    #[cfg(feature = "directory")]
    "directory",
    // Add other app features as they are defined in Cargo.toml
    #[cfg(feature = "project")]
    "project",
    #[cfg(feature = "goals")]
    "goals",
    #[cfg(feature = "workspaces")]
    "workspaces",
    #[cfg(feature = "tickets")]
    "tickets",
    #[cfg(feature = "billing")]
    "billing",
    #[cfg(feature = "billing")]
    "products",
    #[cfg(feature = "video")]
    "video",
    #[cfg(feature = "player")]
    "player",
    #[cfg(feature = "canvas")]
    "canvas",
    #[cfg(feature = "learn")]
    "learn",
    #[cfg(feature = "sources")]
    "sources",
    #[cfg(feature = "dashboards")]
    "dashboards",
    #[cfg(feature = "designer")]
    "designer",
    #[cfg(feature = "editor")]
    "editor",
    #[cfg(feature = "attendant")]
    "attendant", 
    #[cfg(feature = "automation")]
    "tools",
];

/// Check if a feature is compiled into the binary
pub fn is_feature_compiled(name: &str) -> bool {
    COMPILED_FEATURES.contains(&name)
}
