use super::*;

const OUTLOOK_CAL_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "outlook_calendar.events.list",
        method: "GET",
        path: "/v1.0/me/events",
        summary: "Listed Outlook calendar events.",
        path_params: &[],
        query: &[("$top", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "outlook_calendar.events.create",
        method: "POST",
        path: "/v1.0/me/events",
        summary: "Created an Outlook event.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];
const OUTLOOK_CAL_KEYS: &[&str] = &[
    "outlook_calendar.events.list",
    "outlook_calendar.events.create",
];
pub const OUTLOOK_CALENDAR_SPEC: ProviderSpec = ProviderSpec {
    slug: "outlook_calendar",
    origin: Origin::Static("https://graph.microsoft.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: OUTLOOK_CAL_ACTIONS,
    action_keys: OUTLOOK_CAL_KEYS,
};
const ONEDRIVE_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "onedrive.files.list",
    method: "GET",
    path: "/v1.0/me/drive/root/children",
    summary: "Listed OneDrive root children.",
    path_params: &[],
    query: &[("$top", "limit")],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[s("limit")],
}];
const ONEDRIVE_KEYS: &[&str] = &["onedrive.files.list"];
pub const ONEDRIVE_SPEC: ProviderSpec = ProviderSpec {
    slug: "onedrive",
    origin: Origin::Static("https://graph.microsoft.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: ONEDRIVE_ACTIONS,
    action_keys: ONEDRIVE_KEYS,
};
const SHAREPOINT_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "sharepoint.sites.search",
    method: "GET",
    path: "/v1.0/sites",
    summary: "Searched SharePoint sites.",
    path_params: &[],
    query: &[("search", "query")],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[s_req("query")],
}];
const SHAREPOINT_KEYS: &[&str] = &["sharepoint.sites.search"];
pub const SHAREPOINT_SPEC: ProviderSpec = ProviderSpec {
    slug: "sharepoint",
    origin: Origin::Static("https://graph.microsoft.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: SHAREPOINT_ACTIONS,
    action_keys: SHAREPOINT_KEYS,
};
const CONFLUENCE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "confluence.pages.search",
        method: "GET",
        path: "/api/v2/pages",
        summary: "Listed Confluence pages.",
        path_params: &[],
        query: &[("space-id", "space_id"), ("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("space_id"), s("limit")],
    },
    ActionSpec {
        key: "confluence.pages.get",
        method: "GET",
        path: "/api/v2/pages/{resource_id}",
        summary: "Read a Confluence page.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: resource_id(),
    },
];
const CONFLUENCE_KEYS: &[&str] = &["confluence.pages.search", "confluence.pages.get"];
pub const CONFLUENCE_SPEC: ProviderSpec = ProviderSpec {
    slug: "confluence",
    origin: Origin::FromField {
        field: "site_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: CONFLUENCE_ACTIONS,
    action_keys: CONFLUENCE_KEYS,
};

// Gmail (Google Workspace / consumer). Read a mailbox the way the user would
// talk about it: "what did I get today about X" (list with a query) and "open
// that message" (get). Sending stays out: outbound mail already has an
// approval path in SEND MAIL, and a silent send from chat is not something an
// integration read scope should allow.
const GMAIL_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "gmail.messages.list",
        method: "GET",
        path: "/gmail/v1/users/me/messages",
        summary: "Listed Gmail messages matching a query (Gmail search syntax).",
        path_params: &[],
        query: &[("q", "query"), ("maxResults", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("query"), s("limit")],
    },
    ActionSpec {
        key: "gmail.messages.get",
        method: "GET",
        path: "/gmail/v1/users/me/messages/{message_id}",
        summary: "Read one Gmail message (headers, snippet, body parts).",
        path_params: &["message_id"],
        query: &[("format", "format")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("message_id"), s("format")],
    },
];
const GMAIL_KEYS: &[&str] = &["gmail.messages.list", "gmail.messages.get"];
pub const GMAIL_SPEC: ProviderSpec = ProviderSpec {
    slug: "gmail",
    origin: Origin::Static("https://gmail.googleapis.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GMAIL_ACTIONS,
    action_keys: GMAIL_KEYS,
};
