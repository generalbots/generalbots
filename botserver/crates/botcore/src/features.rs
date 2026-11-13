/// List of features compiled into this binary.
///
/// #1811 — this list must describe what Cargo actually built. Every entry is
/// therefore gated by the same feature that compiles the code; there are no
/// unconditional `sheet`/`editor` entries. The `.product` files decide whether
/// a compiled app is always visible, preview or hidden, but they never change
/// what is compiled.
pub const COMPILED_FEATURES: &[&str] = &[
    // Core surfaces are always present.
    "about",
    "admin",
    "settings",
    // Apps whose feature is compiled.
    #[cfg(feature = "analytics")]
    "analytics",
    #[cfg(feature = "attendance")]
    "attendance",
    #[cfg(feature = "attendant")]
    "attendant",
    #[cfg(feature = "automations")]
    "automations",
    #[cfg(feature = "banking")]
    "banking",
    #[cfg(feature = "billing")]
    "billing",
    #[cfg(feature = "biometry")]
    "biometry",
    #[cfg(feature = "browser")]
    "browser",
    #[cfg(feature = "calendar")]
    "calendar",
    #[cfg(feature = "marketing")]
    "campaigns",
    #[cfg(feature = "canvas")]
    "canvas",
    #[cfg(feature = "chat")]
    "chat",
    #[cfg(feature = "compliance")]
    "compliance",
    #[cfg(feature = "consent")]
    "consent",
    #[cfg(any(feature = "people", feature = "contacts"))]
    "crm",
    #[cfg(feature = "database")]
    "database",
    #[cfg(feature = "designer")]
    "designer",
    #[cfg(feature = "desktop")]
    "desktop",
    #[cfg(feature = "docs")]
    "docs",
    #[cfg(feature = "drive")]
    "drive",
    #[cfg(feature = "editor")]
    "editor",
    #[cfg(feature = "erp")]
    "erp",
    #[cfg(feature = "fraud")]
    "fraud",
    #[cfg(feature = "gl")]
    "gl",
    #[cfg(feature = "goals")]
    "goals",
    #[cfg(feature = "handoff")]
    "handoff",
    #[cfg(feature = "hr")]
    "hr",
    #[cfg(feature = "integrations")]
    "integrations",
    #[cfg(feature = "inventory")]
    "inventory",
    #[cfg(feature = "itsm")]
    "itsm",
    #[cfg(feature = "kyc")]
    "kyc",
    #[cfg(feature = "learn")]
    "learn",
    #[cfg(feature = "marketing")]
    "lists",
    #[cfg(feature = "mail")]
    "mail",
    #[cfg(feature = "meet")]
    "meet",
    #[cfg(feature = "memory-os")]
    "memory",
    #[cfg(feature = "minutes")]
    "minutes",
    #[cfg(feature = "monitoring")]
    "monitoring",
    #[cfg(feature = "m365")]
    "o365",
    #[cfg(feature = "paper")]
    "paper",
    #[cfg(feature = "people")]
    "people",
    #[cfg(feature = "plan")]
    "plan",
    #[cfg(feature = "player")]
    "player",
    #[cfg(feature = "pos")]
    "pos",
    #[cfg(feature = "products")]
    "products",
    #[cfg(feature = "project")]
    "project",
    #[cfg(feature = "research")]
    "research",
    #[cfg(feature = "retail")]
    "retail",
    #[cfg(feature = "sales")]
    "sales",
    #[cfg(feature = "search")]
    "search",
    #[cfg(feature = "sheet")]
    "sheet",
    #[cfg(feature = "slides")]
    "slides",
    #[cfg(feature = "social")]
    "social",
    #[cfg(feature = "sources")]
    "sources",
    #[cfg(feature = "tasks")]
    "tasks",
    #[cfg(feature = "tax")]
    "tax",
    #[cfg(feature = "templates")]
    "templates",
    #[cfg(feature = "terminal")]
    "terminal",
    #[cfg(feature = "tickets")]
    "tickets",
    #[cfg(feature = "timeclock")]
    "timeclock",
    #[cfg(feature = "vibe")]
    "vibe",
    #[cfg(feature = "video")]
    "video",
    #[cfg(feature = "vision")]
    "vision",
    #[cfg(feature = "workspaces")]
    "workspace",
    // Apps with no dedicated Cargo feature: the UI and routes are always part
    // of the binary, so their ids must be reported even in a slim build.
    "calculator",
    "clock-widget",
    "concierge",
    "jukebox",
    "notes",
    "notepad",
    "photos",
    "plugins",
    "recycle",
    "store",
    "timer",
    "weather",
    // Infrastructure features (useful to the About page and diagnostics).
    #[cfg(feature = "agent-vm")]
    "agent-vm",
    #[cfg(feature = "automation")]
    "automation",
    #[cfg(feature = "cache")]
    "cache",
    #[cfg(feature = "directory")]
    "directory",
    #[cfg(feature = "jemalloc")]
    "jemalloc",
    #[cfg(feature = "kb-extraction")]
    "kb-extraction",
    #[cfg(feature = "llm")]
    "llm",
    #[cfg(feature = "multimodal")]
    "multimodal",
    #[cfg(feature = "office365")]
    "office365",
    #[cfg(feature = "progress-bars")]
    "progress-bars",
    #[cfg(feature = "rbac")]
    "rbac",
    #[cfg(feature = "security")]
    "security",
    #[cfg(feature = "timeseries")]
    "timeseries",
    #[cfg(feature = "vectordb")]
    "vectordb",
    #[cfg(feature = "weba")]
    "weba",
    #[cfg(feature = "whatsapp")]
    "whatsapp",
    #[cfg(feature = "telegram")]
    "telegram",
    #[cfg(feature = "instagram")]
    "instagram",
    #[cfg(feature = "msteams")]
    "msteams",
];

/// Check if a feature is compiled into the binary
pub fn is_feature_compiled(name: &str) -> bool {
    COMPILED_FEATURES.contains(&name)
}

#[cfg(test)]
mod manifest_tests {
    /// #1811 — the manifest reports exactly the features the command line
    /// requested. This runs with a deliberately small feature set in CI, so the
    /// old "desktop means chat+drive" contract is no longer encoded here.
    #[test]
    fn manifest_reports_only_requested_features() {
        assert_eq!(
            super::is_feature_compiled("chat"),
            cfg!(feature = "chat"),
            "chat is reported exactly when compiled"
        );
        assert_eq!(
            super::is_feature_compiled("drive"),
            cfg!(feature = "drive"),
            "drive is reported exactly when compiled"
        );
        assert_eq!(
            super::is_feature_compiled("sheet"),
            cfg!(feature = "sheet"),
            "sheet is never reported unconditionally"
        );
        assert_eq!(
            super::is_feature_compiled("editor"),
            cfg!(feature = "editor"),
            "editor is never reported unconditionally"
        );
    }
}
