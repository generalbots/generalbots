#[cfg(feature = "embed-ui")]
use rust_embed::RustEmbed;
#[cfg(not(feature = "embed-ui"))]
use log::{error, info};
use std::path::PathBuf;

#[cfg(feature = "embed-ui")]
#[derive(RustEmbed)]
#[folder = "ui"]
pub struct Assets;

pub const SUITE_DIRS: &[&str] = &[
    "js",
    "css",
    "public",
    "assets",
    "partials",
    // Core & Support
    "settings",
    "about",
    // Core Apps (feature-gated to match botui Cargo.toml features)
    #[cfg(feature = "drive")]
    "drive",
    #[cfg(feature = "chat")]
    "chat",
    "mail",
    #[cfg(feature = "tasks")]
    "tasks",
    #[cfg(feature = "calendar")]
    "calendar",
    #[cfg(feature = "meet")]
    "meet",
    // Document Apps
    #[cfg(feature = "paper")]
    "paper",
    #[cfg(feature = "sheet")]
    "sheet",
    #[cfg(feature = "slides")]
    "slides",
    #[cfg(feature = "docs")]
    "docs",
    // Research & Learning
    #[cfg(feature = "research")]
    "research",
    #[cfg(feature = "sources")]
    "sources",
    #[cfg(feature = "learn")]
    "learn",
    // Analytics
    #[cfg(feature = "analytics")]
    "analytics",
    #[cfg(feature = "dashboards")]
    "dashboards",
    #[cfg(feature = "monitoring")]
    "monitoring",
    #[cfg(feature = "monitoring")]
    "governance",
    // Admin & Tools
    #[cfg(feature = "admin")]
    "admin",
    #[cfg(feature = "attendant")]
    "attendant",
    #[cfg(feature = "tools")]
    "tools",
    // Media
    #[cfg(feature = "video")]
    "video",
    #[cfg(feature = "player")]
    "player",
    #[cfg(feature = "canvas")]
    "canvas",
    // Social
    #[cfg(feature = "social")]
    "social",
    #[cfg(feature = "people")]
    "people",
    #[cfg(feature = "people")]
    "crm",
    #[cfg(feature = "tickets")]
    "tickets",
    // Business
    #[cfg(feature = "billing")]
    "billing",
    #[cfg(feature = "products")]
    "products",
    // Development
    #[cfg(feature = "designer")]
    "designer",
    #[cfg(feature = "workspace")]
    "workspace",
    #[cfg(feature = "project")]
    "project",
    #[cfg(feature = "goals")]
    "goals",
    "vibe",
    // Additional Apps (static HTML - always included)
    "banking", "biometry", "brazil", "browser", "campaigns",
    "compliance", "database", "desktop", "email", "handoff", "hr",
    "integrations", "erp", "itsm", "kyc", "lists", "o365", "minutes",
    "plan", "plugins", "pos", "retail", "sales", "tax",
    "templates", "templates-app", "terminal", "timeclock", "vision",
    // AI OS apps (issues #1170-fe, #1178-fe)
    "automations", "memory",
];

pub const ROOT_FILES: &[&str] = &[
    "designer.html",
    "designer.css",
    "designer.js",
    "editor.html",
    "editor.css",
    "editor.js",
    "home.html",
    "base.html",
    "base-layout.html",
    "base-layout.css",
    "desktop.html",
    "default.gbui",
    "single.gbui",
];

pub fn get_ui_root() -> PathBuf {
    #[cfg(feature = "embed-ui")]
    {
        PathBuf::from("ui")
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let candidates = [
            "ui",
            "botui/ui",
            "../botui/ui",
            "../../botui/ui",
            "../../../botui/ui",
        ];

        for path_str in candidates {
            let path = PathBuf::from(path_str);
            if path.exists() {
                info!("Found UI root at: {:?}", path);
                return path;
            }
        }

        let default = PathBuf::from("ui");
        error!(
            "Could not find 'ui' directory in candidates: {:?}. Defaulting to 'ui' (CWD: {:?})",
            candidates,
            std::env::current_dir()
        );
        default
    }
}
