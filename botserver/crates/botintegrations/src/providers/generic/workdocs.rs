use super::helpers::{json, resource_id, s, s_req};
use super::{ActionSpec, AuthStyle, Origin, ProviderSpec, Risk};
const SMARTSHEET_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "smartsheet.spreadsheets.list",
        method: "GET",
        path: "/2.0/sheets",
        summary: "Listed Smartsheet sheets.",
        path_params: &[],
        query: &[("pageSize", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "smartsheet.values.get",
        method: "GET",
        path: "/2.0/sheets/{resource_id}",
        summary: "Read a Smartsheet sheet.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
    ActionSpec {
        key: "smartsheet.rows.append",
        method: "POST",
        path: "/2.0/sheets/{resource_id}/rows",
        summary: "Appended rows to a sheet.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("sheet_id"), json("data", true)],
    },
];

const SMARTSHEET_KEYS: &[&str] = &[
    "smartsheet.spreadsheets.list",
    "smartsheet.values.get",
    "smartsheet.rows.append",
];

pub const SMARTSHEET_SPEC: ProviderSpec = ProviderSpec {
    slug: "smartsheet",
    origin: Origin::Static("https://api.smartsheet.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: SMARTSHEET_ACTIONS,
    action_keys: SMARTSHEET_KEYS,
};

const CLICKUP_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "clickup.work_items.list",
        method: "GET",
        path: "/list/{list_id}/task",
        summary: "Listed ClickUp tasks in a list.",
        path_params: &["list_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("list_id")],
    },
    ActionSpec {
        key: "clickup.work_items.create",
        method: "POST",
        path: "/list/{list_id}/task",
        summary: "Created a ClickUp task.",
        path_params: &["list_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("list_id"), json("data", true)],
    },
    ActionSpec {
        key: "clickup.work_items.update",
        method: "PUT",
        path: "/task/{resource_id}",
        summary: "Updated a ClickUp task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("task_id"), json("data", true)],
    },
    ActionSpec {
        key: "clickup.work_items.delete",
        method: "DELETE",
        path: "/task/{resource_id}",
        summary: "Deleted a ClickUp task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const CLICKUP_KEYS: &[&str] = &[
    "clickup.work_items.list",
    "clickup.work_items.create",
    "clickup.work_items.update",
    "clickup.work_items.delete",
];

pub const CLICKUP_SPEC: ProviderSpec = ProviderSpec {
    slug: "clickup",
    origin: Origin::Static("https://api.clickup.com/api/v2"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: CLICKUP_ACTIONS,
    action_keys: CLICKUP_KEYS,
};

const TYPEFORM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "typeform.forms.list",
        method: "GET",
        path: "/forms",
        summary: "Listed Typeform forms.",
        path_params: &[],
        query: &[("page_size", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "typeform.responses.list",
        method: "GET",
        path: "/forms/{form_id}/responses",
        summary: "Listed form responses.",
        path_params: &["form_id"],
        query: &[("page_size", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("form_id"), s("limit")],
    },
    ActionSpec {
        key: "typeform.forms.create",
        method: "POST",
        path: "/forms",
        summary: "Created a Typeform form.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];

const TYPEFORM_KEYS: &[&str] = &[
    "typeform.forms.list",
    "typeform.responses.list",
    "typeform.forms.create",
];

pub const TYPEFORM_SPEC: ProviderSpec = ProviderSpec {
    slug: "typeform",
    origin: Origin::Static("https://api.typeform.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: TYPEFORM_ACTIONS,
    action_keys: TYPEFORM_KEYS,
};

const DROPBOX_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "dropbox.files.list",
        method: "POST",
        path: "/2/files/list_folder",
        summary: "Listed Dropbox folder contents.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", false)],
    },
    ActionSpec {
        key: "dropbox.files.search",
        method: "POST",
        path: "/2/files/search_v2",
        summary: "Searched Dropbox files.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "dropbox.files.get",
        method: "POST",
        path: "/2/files/get_metadata",
        summary: "Read Dropbox file metadata.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "dropbox.files.delete",
        method: "POST",
        path: "/2/files/delete_v2",
        summary: "Deleted a Dropbox file or folder.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json("data", true)],
    },
];

const DROPBOX_KEYS: &[&str] = &[
    "dropbox.files.list",
    "dropbox.files.search",
    "dropbox.files.get",
    "dropbox.files.delete",
];

pub const DROPBOX_SPEC: ProviderSpec = ProviderSpec {
    slug: "dropbox",
    origin: Origin::Static("https://api.dropboxapi.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: DROPBOX_ACTIONS,
    action_keys: DROPBOX_KEYS,
};
