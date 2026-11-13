//! Enterprise-grade application catalog.
//!
//! Single source of truth for every application in the suite. The frontend
//! launchers (start menu, apps menu) consume this catalog through
//! `GET /api/apps/catalog` instead of maintaining their own duplicate lists.

use serde::Serialize;

pub use ownership::{ownership_of, AppOwnership, AppStatus, OWNERSHIP, UNOWNED};

/// Launch surface of an application. Widgets additionally declare a
/// [`WidgetSpec`] so the desktop can render them as live tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppKind {
    /// Regular app opened as a desktop window (#1160).
    App,
    /// Always-visible desktop tile; also windowable and isolable.
    Widget,
}

/// Launcher split of the 7.0 app list (#2097). Every registry entry declares
/// exactly one value, so no launcher (start menu, sidebar rail, spotlight,
/// command palette) has to keep its own list of app ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LauncherVisibility {
    /// Shown in every launcher, Preview mode on or off.
    Always,
    /// Withheld from the launchers until Preview mode is on.
    Preview,
}

/// Compact render hints for widget-capable apps (#1160).
#[derive(Debug, Clone, Serialize)]
pub struct WidgetSpec {
    /// Full web page rendered in a sandboxed iframe (cross-origin safe).
    pub url: Option<String>,
    /// Minimum grid cells, e.g. (2, 2).
    pub size_w: u16,
    pub size_h: u16,
    /// Optional poll interval in seconds; `None` means static.
    pub refresh_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppDefinition {
    pub id: String,
    pub title: String,
    pub category: String,
    pub color: String,
    pub url: String,
    pub description: String,
    pub keywords: String,
    pub icon: String,
    /// Whether this app is a windowable app or a desktop widget.
    pub kind: AppKind,
    /// Whether launchers list this app always or only in Preview mode.
    pub launcher_visibility: LauncherVisibility,
    /// Owning team (#2120), e.g. `desktop` or `office`. Never empty.
    pub owner: &'static str,
    /// Support status declared by the supported matrix (#2120).
    pub status: AppStatus,
    /// Widget render spec for `kind == Widget` (and for apps that expose one).
    pub widget: Option<WidgetSpec>,
    /// Vibe-published apps may opt in to auto-pin on the launcher (#1160).
    pub launcher_default: bool,
}

pub const CATEGORIES: &[(&str, &str)] = &[
    ("ai", "AI & Assistants"),
    ("business", "Business"),
    ("office", "Office & Productivity"),
    ("dev", "Development"),
    ("system", "System & Tools"),
];

pub fn category_labels() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, label) in CATEGORIES {
        map.insert((*key).to_string(), serde_json::json!(label));
    }
    serde_json::Value::Object(map)
}
/// Identity and presentation of one catalog entry. Grouped so the `app` and
/// `widget_app` constructors keep short argument lists.
#[derive(Debug, Clone, Copy)]
pub struct AppSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub color: &'static str,
    pub url: &'static str,
    pub description: &'static str,
    pub keywords: &'static str,
    pub icon: &'static str,
}

fn definition(spec: AppSpec, kind: AppKind, widget: Option<WidgetSpec>) -> AppDefinition {
    let AppSpec {
        id,
        title,
        category,
        color,
        url,
        description,
        keywords,
        icon,
    } = spec;
    // #2120 — owner and status come from the matrix projection. A missing row
    // is never silent: the runtime logs it and the guard test fails the build.
    let ownership = ownership_of(id);
    if ownership.is_none() {
        log::error!("app registry: `{id}` has no row in the supported matrix; treated as preview-only (#2120)");
    }
    AppDefinition {
        id: id.to_string(),
        title: title.to_string(),
        category: category.to_string(),
        color: color.to_string(),
        url: url.to_string(),
        description: description.to_string(),
        keywords: keywords.to_string(),
        icon: icon.to_string(),
        kind,
        launcher_visibility: LauncherVisibility::Preview,
        owner: ownership.map(|entry| entry.owner).unwrap_or(UNOWNED),
        status: ownership.map(|entry| entry.status).unwrap_or(AppStatus::Preview),
        widget,
        launcher_default: false,
    }
}

/// A regular application entry.
fn app(spec: AppSpec) -> AppDefinition {
    definition(spec, AppKind::App, None)
}

/// A widget-capable entry: rendered as a live desktop tile through `spec`.
fn widget_app(spec: AppSpec, widget: WidgetSpec) -> AppDefinition {
    definition(spec, AppKind::Widget, Some(widget))
}

/// A regular application entry that every launcher always lists (#2097).
fn app_always(spec: AppSpec) -> AppDefinition {
    always_visible(app(spec))
}

/// A widget entry that every launcher always lists (#2097).
fn widget_app_always(spec: AppSpec, widget: WidgetSpec) -> AppDefinition {
    always_visible(widget_app(spec, widget))
}

/// Marks a published app to auto-pin on the launcher when the frontend first
/// sees it (Vibe `publish` with `launcher: true`, #1160).
pub fn with_launcher_default(mut definition: AppDefinition) -> AppDefinition {
    definition.launcher_default = true;
    definition
}

/// Marks an entry as always visible in the launchers (#2097). Entries built by
/// [`app`] and [`widget_app`] default to preview-only, so the "always visible"
/// list of the release lives here, next to the entry itself, instead of being
/// repeated in every `.product` file and frontend launcher.
pub fn always_visible(mut definition: AppDefinition) -> AppDefinition {
    definition.launcher_visibility = LauncherVisibility::Always;
    definition
}

impl AppDefinition {
    /// `true` when launchers must show this app even with Preview mode off.
    pub fn is_always_visible(&self) -> bool {
        self.launcher_visibility == LauncherVisibility::Always
    }
}

/// Launcher split of one registry id (#2097). `None` means the id is not a
/// registry app — core-only surfaces (`auth`, `admin`) and the dynamic
/// `bot-*` / `vibeapp-*` tiles appended by the catalog.
pub fn launcher_visibility_of(id: &str) -> Option<LauncherVisibility> {
    all_apps()
        .into_iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.launcher_visibility)
}

/// `true` when `id` is a registry app the launchers always list.
pub fn is_always_visible(id: &str) -> bool {
    launcher_visibility_of(id) == Some(LauncherVisibility::Always)
}

/// `true` when `id` is a registry app withheld until Preview mode is on.
pub fn is_preview_only(id: &str) -> bool {
    launcher_visibility_of(id) == Some(LauncherVisibility::Preview)
}

pub fn all_apps() -> Vec<AppDefinition> {
    let mut apps = entries::entries();
    apps.extend(entries_extra::entries());
    apps
}

mod entries;
mod entries_extra;
mod ownership;

#[cfg(test)]
mod ownership_guard_tests;

#[cfg(test)]
mod launcher_split_guard_tests {
    use super::{all_apps, LauncherVisibility};
    use std::collections::HashSet;

    /// #2097/#2234 — "Always visible" list of the 7.0 launcher, extended by
    /// #2234 so core office/business apps are supported and always available in
    /// normal (non-preview) sessions. The registry entries must match this.
    const ALWAYS_VISIBLE_7_0: &[&str] = &[
        "chat",
        "drive",
        "clock-widget",
        "jukebox",
        "player",
        "calculator",
        "browser",
        "notes",
        "notepad",
        "photos",
        "timer",
        "weather",
        "integrations",
        "settings",
        "docs",
        "sheet",
        "slides",
        "paper",
        "crm",
        "billing",
        "products",
        "tasks",
        "calendar",
        "mail",
    ];

    /// #2097 — apps the launcher lists only while Preview mode is on.
    const PREVIEW_7_0: &[&str] = &[
        "canvas",
        "goals",
        "analytics",
        "attendant",
        "automations",
        "banking",
        "campaigns",
        "compliance",
        "concierge",
        "designer",
        "fraud",
        "handoff",
        "hr",
        "kyc",
        "learn",
        "lists",
        "meet",
        "memory",
        "minutes",
        "monitoring",
        "o365",
        "people",
        "plan",
        "plugins",
        "pos",
        "project",
        "recycle",
        "retail",
        "social",
        "store",
        "tax",
        "templates",
        "tickets",
        "timeclock",
        "vibe",
        "video",
        "vision",
        "workspace",
    ];

    /// Registry entries that the issue does not name. They stay preview-only:
    /// the issue's "always visible" list is exhaustive, so anything it does
    /// not name can never be always visible. Listing them here (instead of
    /// letting them pass silently) makes a new unclassified registry entry
    /// fail the guard.
    const REGISTRY_ONLY: &[&str] = &[
        "about",
        "clock",
        "database",
        "desktop",
        "editor",
        "research",
        "terminal",
    ];

    fn registry_visibility() -> std::collections::HashMap<String, LauncherVisibility> {
        all_apps()
            .into_iter()
            .map(|app| (app.id, app.launcher_visibility))
            .collect()
    }

    /// The guard: the two 7.0 lists and the registry must agree, and every
    /// registry app must be classified exactly once (one enum value per
    /// entry), so no launcher can disagree with another about an app.
    #[test]
    fn every_registry_app_is_in_exactly_one_launcher_list() {
        let registry = registry_visibility();
        assert!(!registry.is_empty(), "the app registry is empty");

        let mut seen: HashSet<&str> = HashSet::new();
        let mut unknown: Vec<&str> = Vec::new();
        let mut misplaced: Vec<String> = Vec::new();

        for id in ALWAYS_VISIBLE_7_0 {
            assert!(seen.insert(*id), "{id} is listed twice in the 7.0 lists");
            match registry.get(*id) {
                None => unknown.push(*id),
                Some(LauncherVisibility::Always) => {}
                Some(LauncherVisibility::Preview) => {
                    misplaced.push(format!("{id} (issue: always, registry: preview)"))
                }
            }
        }
        for id in PREVIEW_7_0 {
            assert!(seen.insert(*id), "{id} is listed twice in the 7.0 lists");
            match registry.get(*id) {
                None => unknown.push(*id),
                Some(LauncherVisibility::Preview) => {}
                Some(LauncherVisibility::Always) => {
                    misplaced.push(format!("{id} (issue: preview, registry: always)"))
                }
            }
        }

        assert!(
            unknown.is_empty(),
            "issue #2097 names apps missing from the registry: {}",
            unknown.join(", ")
        );
        assert!(
            misplaced.is_empty(),
            "registry disagrees with issue #2097: {}",
            misplaced.join(", ")
        );

        let extras: HashSet<&str> = REGISTRY_ONLY.iter().copied().collect();
        let unclassified: Vec<&str> = registry
            .keys()
            .map(String::as_str)
            .filter(|id| !seen.contains(id) && !extras.contains(id))
            .collect();
        assert!(
            unclassified.is_empty(),
            "registry apps in neither 7.0 list nor REGISTRY_ONLY: {}",
            unclassified.join(", ")
        );
        for id in REGISTRY_ONLY {
            match registry.get(*id) {
                None => panic!("REGISTRY_ONLY names a missing app: {id}"),
                Some(LauncherVisibility::Preview) => {}
                Some(LauncherVisibility::Always) => {
                    panic!("{id} is registry-only, so it cannot be always visible")
                }
            }
        }
    }

    /// `.product` must not carry its own copy of the always-visible list: the
    /// `apps=` line has to match the registry flag (#2097). A new always-visible
    /// entry without a `.product` update fails here.
    #[test]
    fn product_apps_line_matches_the_registry_flag() {
        // Core surfaces are always available and must not be listed in apps=.
        const CORE_APPS: &[&str] = &["about", "settings", "auth"];
        for (label, raw) in [
            (".product", include_str!("../../../.product")),
            (
                ".product.desktop-local",
                include_str!("../../../.product.desktop-local"),
            ),
        ] {
            let line = raw
                .lines()
                .find(|line| line.starts_with("apps="))
                .unwrap_or_else(|| panic!("{label} has no apps= line"));
            let declared: HashSet<&str> = line
                .trim_start_matches("apps=")
                .split(',')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .collect();
            let expected: HashSet<&str> = ALWAYS_VISIBLE_7_0
                .iter()
                .copied()
                .filter(|id| !CORE_APPS.contains(id))
                .collect();
            let mut only_product: Vec<&&str> = declared.difference(&expected).collect();
            let mut only_registry: Vec<&&str> = expected.difference(&declared).collect();
            only_product.sort();
            only_registry.sort();
            assert!(
                only_product.is_empty() && only_registry.is_empty(),
                "{label} apps= disagrees with the registry flag — only in {label}: {only_product:?}, only in the registry: {only_registry:?}"
            );
        }
    }
}
