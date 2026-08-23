use super::helpers::{json, json_req, resource_id, s, s_req};
use super::{ActionSpec, AuthStyle, Origin, ParamKind, ParamSpec, ProviderSpec, Risk};
const APOLLO_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "apollo.contacts.list",
        method: "GET",
        path: "/v1/contacts",
        summary: "Listed Apollo contacts.",
        path_params: &[],
        query: &[("page", "page")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("page")],
    },
    ActionSpec {
        key: "apollo.contacts.search",
        method: "GET",
        path: "/v1/contacts/search",
        summary: "Searched Apollo contacts.",
        path_params: &[],
        query: &[("q_keywords", "query"), ("page", "page")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query"), s("page")],
    },
    ActionSpec {
        key: "apollo.deals.list",
        method: "POST",
        path: "/v1/opportunities/search",
        summary: "Listed Apollo opportunities.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[ParamSpec {
            name: "data",
            kind: ParamKind::Json,
            required: false,
        }],
    },
    ActionSpec {
        key: "apollo.contacts.create",
        method: "POST",
        path: "/v1/contacts",
        summary: "Created an Apollo contact.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[ParamSpec {
            name: "data",
            kind: ParamKind::Json,
            required: true,
        }],
    },
    ActionSpec {
        key: "apollo.contacts.update",
        method: "PUT",
        path: "/v1/contacts/{resource_id}",
        summary: "Updated an Apollo contact.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"),
            ParamSpec {
                name: "data",
                kind: ParamKind::Json,
                required: true,
            }],
    },
];


const APOLLO_KEYS: &[&str] = &[
    "apollo.contacts.list",
    "apollo.contacts.search",
    "apollo.deals.list",
    "apollo.contacts.create",
    "apollo.contacts.update",
];

pub const APOLLO_SPEC: ProviderSpec = ProviderSpec {
    slug: "apollo",
    origin: Origin::Static("https://api.apollo.io"),
    auth: AuthStyle::ApiKeyHeader {
        header: "X-Api-Key",
        field: "api_key",
    },
    actions: APOLLO_ACTIONS,
    action_keys: APOLLO_KEYS,
};

// ---------------------------------------------------------------------------
// Lemlist API v2 - Basic :{apiKey} (empty team id) or teamId:apiKey.
// Verified subset: campaign listing and per-campaign statistics.
// ---------------------------------------------------------------------------
const CANNY_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "canny.tickets.list",
        method: "POST",
        path: "/posts/list",
        summary: "Listed Canny posts.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit"), s("status")],
    },
    ActionSpec {
        key: "canny.tickets.search",
        method: "POST",
        path: "/posts/list",
        summary: "Searched Canny posts.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "canny.tickets.get",
        method: "POST",
        path: "/posts/get",
        summary: "Read a Canny post.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("id")],
    },
    ActionSpec {
        key: "canny.tickets.create",
        method: "POST",
        path: "/posts/create",
        summary: "Created a Canny post.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("boardID"), s_req("title"), s_req("details")],
    },
    ActionSpec {
        key: "canny.tickets.update",
        method: "POST",
        path: "/posts/update",
        summary: "Updated a Canny post.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("id"), s("title"), s("details")],
    },
    ActionSpec {
        key: "canny.tickets.delete",
        method: "POST",
        path: "/posts/delete",
        summary: "Deleted a Canny post.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const CANNY_KEYS: &[&str] = &[
    "canny.tickets.list",
    "canny.tickets.search",
    "canny.tickets.get",
    "canny.tickets.create",
    "canny.tickets.update",
    "canny.tickets.delete",
];

pub const CANNY_SPEC: ProviderSpec = ProviderSpec {
    slug: "canny",
    origin: Origin::Static("https://canny.io/api/v1"),
    auth: AuthStyle::BodyField { field: "api_key" },
    actions: CANNY_ACTIONS,
    action_keys: CANNY_KEYS,
};

// ---------------------------------------------------------------------------
// Apollo.io REST API v1 - X-Api-Key header. Verified subset of the catalog:
// contacts and opportunities CRUD via /v1/contacts and /v1/opportunities.
// ---------------------------------------------------------------------------
const STREAK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "streak.deals.list",
        method: "GET",
        path: "/v1/pipelines/{pipeline_key}/boxes",
        summary: "Listed pipeline boxes (deals).",
        path_params: &["pipeline_key"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("pipeline_key")],
    },
    ActionSpec {
        key: "streak.deals.get",
        method: "GET",
        path: "/v1/pipelines/{pipeline_key}/boxes/{resource_id}",
        summary: "Read a pipeline box.",
        path_params: &["pipeline_key", "resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("pipeline_key"), s_req("box_key")],
    },
];

const STREAK_KEYS: &[&str] = &["streak.deals.list", "streak.deals.get"];

pub const STREAK_SPEC: ProviderSpec = ProviderSpec {
    slug: "streak",
    origin: Origin::Static("https://www.streak.com/api"),
    auth: AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: None,
    },
    actions: STREAK_ACTIONS,
    action_keys: STREAK_KEYS,
};
const ZENDESK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "zendesk.tickets.list",
        method: "GET",
        path: "/tickets.json",
        summary: "Listed support tickets.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "zendesk.tickets.search",
        method: "GET",
        path: "/search.json",
        summary: "Searched support tickets.",
        path_params: &[],
        query: &[("query", "query"), ("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query"), s("limit")],
    },
    ActionSpec {
        key: "zendesk.tickets.get",
        method: "GET",
        path: "/tickets/{resource_id}.json",
        summary: "Read support ticket.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
    ActionSpec {
        key: "zendesk.tickets.create",
        method: "POST",
        path: "/tickets.json",
        summary: "Created support ticket.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: Some("ticket"),
        risk: Risk::Medium,
        params: &[json_req("data")],
    },
    ActionSpec {
        key: "zendesk.tickets.update",
        method: "PUT",
        path: "/tickets/{resource_id}.json",
        summary: "Updated support ticket.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: Some("ticket"),
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json_req("data")],
    },
    ActionSpec {
        key: "zendesk.tickets.delete",
        method: "DELETE",
        path: "/tickets/{resource_id}.json",
        summary: "Deleted support ticket.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const ZENDESK_KEYS: &[&str] = &[
    "zendesk.tickets.list",
    "zendesk.tickets.search",
    "zendesk.tickets.get",
    "zendesk.tickets.create",
    "zendesk.tickets.update",
    "zendesk.tickets.delete",
];

pub const ZENDESK_SPEC: ProviderSpec = ProviderSpec {
    slug: "zendesk",
    origin: Origin::ZendeskSubdomain,
    auth: AuthStyle::BasicTemplate {
        user_template: "{email}/token",
        password_field: "token",
    },
    actions: ZENDESK_ACTIONS,
    action_keys: ZENDESK_KEYS,
};

// ---------------------------------------------------------------------------
// Trello REST API - key/token query authentication.
// ---------------------------------------------------------------------------

// HubSpot CRM API v3 - Bearer token (private app or OAuth).

const HUBSPOT_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "hubspot.contacts.list",
        method: "GET",
        path: "/crm/v3/objects/contacts",
        summary: "Listed HubSpot contacts.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "hubspot.contacts.search",
        method: "POST",
        path: "/crm/v3/objects/contacts/search",
        summary: "Searched HubSpot contacts.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "hubspot.deals.list",
        method: "GET",
        path: "/crm/v3/objects/deals",
        summary: "Listed HubSpot deals.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "hubspot.contacts.create",
        method: "POST",
        path: "/crm/v3/objects/contacts",
        summary: "Created a HubSpot contact.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "hubspot.contacts.update",
        method: "PATCH",
        path: "/crm/v3/objects/contacts/{resource_id}",
        summary: "Updated a HubSpot contact.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json("data", true)],
    },
    ActionSpec {
        key: "hubspot.contacts.delete",
        method: "DELETE",
        path: "/crm/v3/objects/contacts/{resource_id}",
        summary: "Deleted a HubSpot contact.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &resource_id(),
    },
];

const HUBSPOT_KEYS: &[&str] = &[
    "hubspot.contacts.list",
    "hubspot.contacts.search",
    "hubspot.deals.list",
    "hubspot.contacts.create",
    "hubspot.contacts.update",
    "hubspot.contacts.delete",
];

pub const HUBSPOT_SPEC: ProviderSpec = ProviderSpec {
    slug: "hubspot",
    origin: Origin::Static("https://api.hubapi.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: HUBSPOT_ACTIONS,
    action_keys: HUBSPOT_KEYS,
};

const CLICKFUNNELS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "clickfunnels.campaigns.list",
        method: "GET",
        path: "/workspaces/{workspace_id}/funnels",
        summary: "Listed ClickFunnels funnels.",
        path_params: &["workspace_id"],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("workspace_id"), s("limit")],
    },
    ActionSpec {
        key: "clickfunnels.contacts.list",
        method: "GET",
        path: "/workspaces/{workspace_id}/contacts",
        summary: "Listed ClickFunnels contacts.",
        path_params: &["workspace_id"],
        query: &[("per_page", "limit"), ("filter[email]", "email")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("workspace_id"), s("limit"), s("email")],
    },
];

const CLICKFUNNELS_KEYS: &[&str] = &[
    "clickfunnels.campaigns.list",
    "clickfunnels.contacts.list",
];

pub const CLICKFUNNELS_SPEC: ProviderSpec = ProviderSpec {
    slug: "clickfunnels",
    origin: Origin::Static("https://api.myclickfunnels.com/api/v2"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: CLICKFUNNELS_ACTIONS,
    action_keys: CLICKFUNNELS_KEYS,
};
