#[cfg(not(feature = "embed-ui"))]
use log::{error, info};
#[cfg(feature = "embed-ui")]
use rust_embed::RustEmbed;
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
    // #2567 — the shared identity layer (tokens.css, fonts, accent, theme)
    // consumed by the suite's Default theme and the assistant module.
    "identity",
    // Core & Support
    "settings",
    "about",
    // Chat-first shell (product shell=assistant). Must be embedded so the
    // hand-off from desktop.html can serve it in release builds.
    "assistant",
    // Core Apps — always served. RustEmbed embeds the whole `ui/` tree, and
    // the launcher is JS-driven (APPS_REGISTRY), so feature-gating this list
    // only made every app whose Cargo feature was off 404 in release builds
    // while working in dev (ServeDir). Keep every suite dir here.
    "drive",
    "chat",
    "mail",
    "tasks",
    "calendar",
    "meet",
    // Document Apps
    "paper",
    "sheet",
    "slides",
    "docs",
    // Research & Learning
    "research",
    "learn",
    // Analytics
    "analytics",
    "dashboards",
    "monitoring",
    "governance",
    // Admin & Tools
    "admin",
    "attendant",
    "tools",
    // Media
    "video",
    "player",
    "canvas",
    // Social
    "social",
    "people",
    "crm",
    "tickets",
    // Business
    "billing",
    "products",
    // Development
    "designer",
    "workspace",
    "project",
    "goals",
    "vibe",
    // Additional Apps (static HTML - always included)
    "banking",
    "biometry",
    "brazil",
    "browser",
    "campaigns",
    "events",
    "approvals",
    "compliance",
    "database",
    "desktop",
    "email",
    "handoff",
    "hr",
    "integrations",
    "erp",
    "itsm",
    "kyc",
    "lists",
    "o365",
    "minutes",
    "plan",
    "plugins",
    "pos",
    "retail",
    "sales",
    "tax",
    "templates",
    "templates-app",
    "terminal",
    "timeclock",
    "vision",
    // AI OS apps (issues #1170-fe, #1178-fe)
    "automations",
    "memory",
    // AI OS utility apps (embedded registry apps, always included)
    "jukebox",
    "calculator",
    "notepad",
    "photos",
    "timer",
    "weather",
    "recycle",
    "snapshot",
    "clock",
    "notes",
    "store",
    "concierge",
    "fraud",
    // Desktop widget partials (widget-registry.js loads /suite/widgets/*)
    "widgets",
    // Split assets introduced by #1731
    "desktop-inline",
    "editor",
    "editor-partials",
    "designer-partials",
    "base-layout",
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

// Site-root static assets (embedded at the `ui/` folder root, served at
// `/` scope rather than `/suite/`): PWA manifest and offline shell worker.
pub const SITE_ROOT_FILES: &[&str] = &["manifest.webmanifest", "sw.js", "logo.svg"];

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
