
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
    "sheet",
    #[cfg(feature = "slides")]
    "slides",
    #[cfg(feature = "meet")]
    "meet",
    #[cfg(feature = "research")]
    "research",
    #[cfg(feature = "people")]
    "people",
    #[cfg(feature = "people")]
    "crm", // Alias for people
    #[cfg(any(feature = "people", feature = "billing", feature = "chat"))]
    "admin", // Core admin panel
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
    "editor",
    #[cfg(feature = "attendant")]
    "attendant", 
    #[cfg(feature = "automation")]
    "tools",
    #[cfg(feature = "erp")]
    "erp",
    #[cfg(feature = "hr")]
    "hr",
    #[cfg(feature = "sales")]
    "sales",
    #[cfg(feature = "banking")]
    "banking",
    #[cfg(feature = "pos")]
    "pos",
    #[cfg(feature = "retail")]
    "retail",
    #[cfg(feature = "fraud")]
    "fraud",
    #[cfg(feature = "inventory")]
    "inventory",
    #[cfg(feature = "gl")]
    "gl",
    #[cfg(feature = "integrations")]
    "integrations",
    #[cfg(feature = "itsm")]
    "itsm",
    #[cfg(feature = "handoff")]
    "handoff",
    #[cfg(feature = "kyc")]
    "kyc",
    #[cfg(feature = "timeclock")]
    "timeclock",
    #[cfg(feature = "m365")]
    "m365",
    #[cfg(feature = "tax")]
    "tax",
    #[cfg(feature = "vision")]
    "vision",
    #[cfg(feature = "compliance")]
    "compliance",
    #[cfg(feature = "plan")]
    "plan",
    #[cfg(feature = "legal")]
    "legal",
    #[cfg(feature = "database")]
    "database",
    #[cfg(feature = "browser")]
    "browser",
    #[cfg(feature = "terminal")]
    "terminal",
    #[cfg(feature = "templates")]
    "templates",
    #[cfg(feature = "campaigns")]
    "campaigns",
    #[cfg(feature = "lists")]
    "lists",
    #[cfg(feature = "minutes")]
    "minutes",
    #[cfg(feature = "marketing")]
    "marketing",
    #[cfg(feature = "vibe")]
    "vibe",
    #[cfg(feature = "whatsapp")]
    "whatsapp",
    #[cfg(feature = "telegram")]
    "telegram",
    #[cfg(feature = "instagram")]
    "instagram",
    #[cfg(feature = "msteams")]
    "msteams",
    #[cfg(feature = "contacts")]
    "contacts",
    #[cfg(feature = "search")]
    "search",
    #[cfg(feature = "multimodal")]
    "multimodal",
    #[cfg(feature = "attendance")]
    "attendance",
    #[cfg(feature = "workspaces")]
    "workspaces",
    #[cfg(feature = "biometry")]
    "biometry",
    #[cfg(feature = "office365")]
    "office365",
];

/// Check if a feature is compiled into the binary
pub fn is_feature_compiled(name: &str) -> bool {
    COMPILED_FEATURES.contains(&name)
}
