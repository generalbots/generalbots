use super::*;

/// #2036 — deep links for the apps that had none (personal, system and store
/// surfaces). Kept in its own file so the main table stays under the size
/// limit.
pub static APP_DEEP_LINKS_EXTRA: &[(&str, &[DeepLinkParam])] = &[
    (
        "about",
        &[DeepLinkParam {
            key: "tab",
            description: "about tab (licenses or credits)",
            example: "licenses",
        }],
    ),
    (
        "automations",
        &[DeepLinkParam {
            key: "automation_id",
            description: "automation id",
            example: "sched-12",
        }],
    ),
    (
        "calculator",
        &[DeepLinkParam {
            key: "expression",
            description: "expression to prefill",
            example: "12*8",
        }],
    ),
    (
        "clock",
        &[DeepLinkParam {
            key: "timezone",
            description: "IANA timezone",
            example: "America%2FSao_Paulo",
        }],
    ),
    (
        "clock-widget",
        &[DeepLinkParam {
            key: "timezone",
            description: "IANA timezone",
            example: "UTC",
        }],
    ),
    (
        "concierge",
        &[DeepLinkParam {
            key: "goal",
            description: "goal for the concierge",
            example: "plan%20my%20week",
        }],
    ),
    (
        "desktop",
        &[DeepLinkParam {
            key: "app",
            description: "app id to open",
            example: "drive",
        }],
    ),
    (
        "memory",
        &[DeepLinkParam {
            key: "query",
            description: "memory search text",
            example: "supplier%20preferences",
        }],
    ),
    (
        "monitoring",
        &[DeepLinkParam {
            key: "card",
            description: "monitoring card id",
            example: "resources",
        }],
    ),
    (
        "notes",
        &[DeepLinkParam {
            key: "note_id",
            description: "note id",
            example: "note-7",
        }],
    ),
    (
        "o365",
        &[DeepLinkParam {
            key: "folder",
            description: "OneDrive folder path",
            example: "Documents",
        }],
    ),
    (
        "photos",
        &[DeepLinkParam {
            key: "photo_id",
            description: "photo id",
            example: "img-42",
        }],
    ),
    (
        "plugins",
        &[DeepLinkParam {
            key: "plugin_id",
            description: "plugin id",
            example: "pdf-tools",
        }],
    ),
    (
        "recycle",
        &[DeepLinkParam {
            key: "item_id",
            description: "recycle item id",
            example: "rec-9",
        }],
    ),
    (
        "settings",
        &[DeepLinkParam {
            key: "section",
            description: "settings section id",
            example: "notifications",
        }],
    ),
    (
        "store",
        &[DeepLinkParam {
            key: "product_id",
            description: "store product id",
            example: "vps-small",
        }],
    ),
    (
        "terminal",
        &[DeepLinkParam {
            key: "cwd",
            description: "working directory",
            example: "output",
        }],
    ),
    (
        "timer",
        &[DeepLinkParam {
            key: "duration",
            description: "duration to start",
            example: "25m",
        }],
    ),
    (
        "weather",
        &[DeepLinkParam {
            key: "city",
            description: "city name",
            example: "Sao%20Paulo",
        }],
    ),
];

/// Every declared deep-link table, core first.
pub fn all_deep_links() -> impl Iterator<Item = &'static (&'static str, &'static [DeepLinkParam])> {
    APP_DEEP_LINKS_CORE.iter().chain(APP_DEEP_LINKS_EXTRA.iter())
}
