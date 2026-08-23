use super::helpers::{json, resource_id, s, s_req};
use super::{ActionSpec, AuthStyle, Origin, ProviderSpec, Risk};
const CAL_COM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "cal_com.events.list",
        method: "GET",
        path: "/v2/event-types",
        summary: "Listed Cal.com event types.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "cal_com.bookings.list",
        method: "GET",
        path: "/v2/bookings",
        summary: "Listed Cal.com bookings.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
];

const CAL_COM_KEYS: &[&str] = &["cal_com.events.list", "cal_com.bookings.list"];

pub const CAL_COM_SPEC: ProviderSpec = ProviderSpec {
    slug: "cal_com",
    origin: Origin::Static("https://api.cal.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: CAL_COM_ACTIONS,
    action_keys: CAL_COM_KEYS,
};
const JOTFORM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "jotform.forms.list",
        method: "GET",
        path: "/user/forms",
        summary: "Listed Jotform forms.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "jotform.responses.list",
        method: "GET",
        path: "/form/{resource_id}/submissions",
        summary: "Listed form submissions.",
        path_params: &["resource_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("form_id"), s("limit")],
    },
];

const JOTFORM_KEYS: &[&str] = &["jotform.forms.list", "jotform.responses.list"];

pub const JOTFORM_SPEC: ProviderSpec = ProviderSpec {
    slug: "jotform",
    origin: Origin::Static("https://api.jotform.com"),
    auth: AuthStyle::QueryPairs {
        pairs: &[("apiKey", "api_key")],
    },
    actions: JOTFORM_ACTIONS,
    action_keys: JOTFORM_KEYS,
};
const MOTION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "motion.tasks.list",
        method: "GET",
        path: "/tasks",
        summary: "Listed Motion tasks.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "motion.tasks.create",
        method: "POST",
        path: "/tasks",
        summary: "Created a Motion task.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "motion.tasks.update",
        method: "PATCH",
        path: "/tasks/{resource_id}",
        summary: "Updated a Motion task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json("data", true)],
    },
];

const MOTION_KEYS: &[&str] = &[
    "motion.tasks.list",
    "motion.tasks.create",
    "motion.tasks.update",
];

pub const MOTION_SPEC: ProviderSpec = ProviderSpec {
    slug: "motion",
    origin: Origin::Static("https://api.usemotion.com/v1"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: MOTION_ACTIONS,
    action_keys: MOTION_KEYS,
};
const TRELLO_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "trello.work_items.list",
        method: "GET",
        path: "/1/members/me/cards",
        summary: "Listed Trello cards.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "trello.work_items.search",
        method: "GET",
        path: "/1/search",
        summary: "Searched Trello boards and cards.",
        path_params: &[],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "trello.work_items.create",
        method: "POST",
        path: "/1/cards",
        summary: "Created a Trello card.",
        path_params: &[],
        query: &[
            ("idList", "id_list"),
            ("name", "name"),
            ("desc", "description"),
        ],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("id_list"), s_req("name"), s("description")],
    },
    ActionSpec {
        key: "trello.work_items.update",
        method: "PUT",
        path: "/1/cards/{resource_id}",
        summary: "Updated a Trello card.",
        path_params: &["resource_id"],
        query: &[
            ("name", "name"),
            ("desc", "description"),
            ("closed", "closed"),
        ],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), s("name"), s("description"), s("closed")],
    },
    ActionSpec {
        key: "trello.work_items.delete",
        method: "DELETE",
        path: "/1/cards/{resource_id}",
        summary: "Deleted a Trello card.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const TRELLO_KEYS: &[&str] = &[
    "trello.work_items.list",
    "trello.work_items.search",
    "trello.work_items.create",
    "trello.work_items.update",
    "trello.work_items.delete",
];

pub const TRELLO_SPEC: ProviderSpec = ProviderSpec {
    slug: "trello",
    origin: Origin::Static("https://api.trello.com"),
    auth: AuthStyle::QueryPairs {
        pairs: &[("key", "key"), ("token", "token")],
    },
    actions: TRELLO_ACTIONS,
    action_keys: TRELLO_KEYS,
};

// ---------------------------------------------------------------------------
// Mailchimp Marketing API 3.0 - Basic anystring:{key}, host picked from the
// data center suffix of the key.
// ---------------------------------------------------------------------------

const TODOIST_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "todoist.tasks.list",
        method: "GET",
        path: "/tasks",
        summary: "Listed Todoist tasks.",
        path_params: &[],
        query: &[("filter", "filter")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("filter")],
    },
    ActionSpec {
        key: "todoist.tasks.search",
        method: "GET",
        path: "/tasks",
        summary: "Searched Todoist tasks by filter.",
        path_params: &[],
        query: &[("filter", "query"), ("lang", "lang")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "todoist.tasks.create",
        method: "POST",
        path: "/tasks",
        summary: "Created a Todoist task.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "todoist.tasks.update",
        method: "POST",
        path: "/tasks/{resource_id}",
        summary: "Updated a Todoist task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("task_id"), json("data", true)],
    },
    ActionSpec {
        key: "todoist.tasks.complete",
        method: "POST",
        path: "/tasks/{resource_id}/close",
        summary: "Completed a Todoist task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Medium,
        params: &resource_id(),
    },
    ActionSpec {
        key: "todoist.tasks.delete",
        method: "DELETE",
        path: "/tasks/{resource_id}",
        summary: "Deleted a Todoist task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const TODOIST_KEYS: &[&str] = &[
    "todoist.tasks.list",
    "todoist.tasks.search",
    "todoist.tasks.create",
    "todoist.tasks.update",
    "todoist.tasks.complete",
    "todoist.tasks.delete",
];

pub const TODOIST_SPEC: ProviderSpec = ProviderSpec {
    slug: "todoist",
    origin: Origin::Static("https://api.todoist.com/rest/v2"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: TODOIST_ACTIONS,
    action_keys: TODOIST_KEYS,
};

const NOTION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "notion.pages.search",
        method: "POST",
        path: "/search",
        summary: "Searched Notion pages and databases.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", false)],
    },
    ActionSpec {
        key: "notion.pages.get",
        method: "GET",
        path: "/pages/{page_id}",
        summary: "Read a Notion page.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
    ActionSpec {
        key: "notion.pages.create",
        method: "POST",
        path: "/pages",
        summary: "Created a Notion page.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "notion.pages.update",
        method: "PATCH",
        path: "/pages/{resource_id}",
        summary: "Updated Notion page properties.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("page_id"), json("data", true)],
    },
    ActionSpec {
        key: "notion.pages.archive",
        method: "PATCH",
        path: "/pages/{resource_id}",
        summary: "Archived a Notion page.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[s_req("page_id"), json("data", true)],
    },
];

const NOTION_KEYS: &[&str] = &[
    "notion.pages.search",
    "notion.pages.get",
    "notion.pages.create",
    "notion.pages.update",
    "notion.pages.archive",
];

pub const NOTION_SPEC: ProviderSpec = ProviderSpec {
    slug: "notion",
    origin: Origin::Static("https://api.notion.com/v1"),
    auth: AuthStyle::BearerHeaders {
        token_field: "token",
        headers: &[("Notion-Version", "2022-06-28")],
    },
    actions: NOTION_ACTIONS,
    action_keys: NOTION_KEYS,
};

const CALENDLY_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "calendly.events.list",
        method: "GET",
        path: "/scheduled_events",
        summary: "Listed Calendly scheduled events.",
        path_params: &[],
        query: &[("count", "limit"), ("user", "user_uri")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit"), s("user_uri")],
    },
    ActionSpec {
        key: "calendly.users.me",
        method: "GET",
        path: "/users/me",
        summary: "Read the Calendly current user.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
];

const CALENDLY_KEYS: &[&str] = &["calendly.events.list", "calendly.users.me"];

pub const CALENDLY_SPEC: ProviderSpec = ProviderSpec {
    slug: "calendly",
    origin: Origin::Static("https://api.calendly.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: CALENDLY_ACTIONS,
    action_keys: CALENDLY_KEYS,
};

const GOOGLE_DRIVE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "drive.files.list",
        method: "GET",
        path: "/drive/v3/files",
        summary: "Listed Google Drive files.",
        path_params: &[],
        query: &[("pageSize", "limit"), ("q", "query"), ("pageToken", "cursor")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit"), s("query"), s("cursor")],
    },
    ActionSpec {
        key: "drive.files.get",
        method: "GET",
        path: "/drive/v3/files/{resource_id}",
        summary: "Read Drive file metadata.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
];

const GOOGLE_DRIVE_KEYS: &[&str] = &["drive.files.list", "drive.files.get"];

pub const GOOGLE_DRIVE_SPEC: ProviderSpec = ProviderSpec {
    slug: "google_drive",
    origin: Origin::Static("https://www.googleapis.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GOOGLE_DRIVE_ACTIONS,
    action_keys: GOOGLE_DRIVE_KEYS,
};

const GCAL_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "calendar.events.list",
        method: "GET",
        path: "/calendar/v3/calendars/primary/events",
        summary: "Listed calendar events.",
        path_params: &[],
        query: &[("maxResults", "limit"), ("timeMin", "time_min"), ("timeMax", "time_max")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit"), s("time_min"), s("time_max")],
    },
    ActionSpec {
        key: "calendar.events.create",
        method: "POST",
        path: "/calendar/v3/calendars/primary/events",
        summary: "Created a calendar event.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];

const GCAL_KEYS: &[&str] = &["calendar.events.list", "calendar.events.create"];

pub const GOOGLE_CALENDAR_SPEC: ProviderSpec = ProviderSpec {
    slug: "google_calendar",
    origin: Origin::Static("https://www.googleapis.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GCAL_ACTIONS,
    action_keys: GCAL_KEYS,
};

const GPHOTOS_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "google_photos.mediaitems.list",
    method: "GET",
    path: "/v1/mediaItems",
    summary: "Listed Google Photos media items.",
    path_params: &[],
    query: &[("pageSize", "limit")],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[s("limit")],
}];

const GPHOTOS_KEYS: &[&str] = &["google_photos.mediaitems.list"];

pub const GOOGLE_PHOTOS_SPEC: ProviderSpec = ProviderSpec {
    slug: "google_photos",
    origin: Origin::Static("https://photoslibrary.googleapis.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GPHOTOS_ACTIONS,
    action_keys: GPHOTOS_KEYS,
};

const GFORMS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "google_forms.forms.get",
        method: "GET",
        path: "/v1/forms/{resource_id}",
        summary: "Read a Google Form.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
    ActionSpec {
        key: "google_forms.responses.list",
        method: "GET",
        path: "/v1/forms/{form_id}/responses",
        summary: "Listed form responses.",
        path_params: &["form_id"],
        query: &[("pageSize", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("form_id"), s("limit")],
    },
];

const GFORMS_KEYS: &[&str] = &["google_forms.forms.get", "google_forms.responses.list"];

pub const GOOGLE_FORMS_SPEC: ProviderSpec = ProviderSpec {
    slug: "google_forms",
    origin: Origin::Static("https://forms.googleapis.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GFORMS_ACTIONS,
    action_keys: GFORMS_KEYS,
};

const OUTLOOK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "outlook.messages.list",
        method: "GET",
        path: "/v1.0/me/messages",
        summary: "Listed mailbox messages.",
        path_params: &[],
        query: &[("$top", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "outlook.messages.send",
        method: "POST",
        path: "/v1.0/me/sendMail",
        summary: "Sent a mailbox message.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json("data", true)],
    },
];

const OUTLOOK_KEYS: &[&str] = &["outlook.messages.list", "outlook.messages.send"];

pub const OUTLOOK_SPEC: ProviderSpec = ProviderSpec {
    slug: "outlook",
    origin: Origin::Static("https://graph.microsoft.com"),
    auth: AuthStyle::BearerHeaders {
        token_field: "token",
        headers: &[("ConsistencyLevel", "eventual")],
    },
    actions: OUTLOOK_ACTIONS,
    action_keys: OUTLOOK_KEYS,
};

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
        params: &resource_id(),
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
