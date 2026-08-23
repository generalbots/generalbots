//! Batch 2 generic provider specifications (#939 wave): Canny, Apollo,
//! Lemlist and Luma. Action keys mirror the integration catalog; providers
//! where the official API covers only part of the catalog verbs implement
//! that verified subset - the rest stay advertised as planned.

use super::{ActionSpec, AuthStyle, Origin, ParamKind, ParamSpec, ProviderSpec, Risk};

const fn s(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Str,
        required: false,
    }
}

const fn s_req(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Str,
        required: true,
    }
}

const RESOURCE_ID: &[ParamSpec] = &[ParamSpec {
    name: "resource_id",
    kind: ParamKind::Str,
    required: true,
}];

// ---------------------------------------------------------------------------
// Canny REST API - every call is a POST with api_key in the JSON body.
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
        params: RESOURCE_ID,
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

const LEMLIST_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "lemlist.campaigns.list",
        method: "GET",
        path: "/campaigns",
        summary: "Listed Lemlist campaigns.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "lemlist.campaigns.reports.get",
        method: "GET",
        path: "/campaigns/{campaign_id}/stats",
        summary: "Read Lemlist campaign statistics.",
        path_params: &["campaign_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("campaign_id")],
    },
];

const LEMLIST_KEYS: &[&str] = &[
    "lemlist.campaigns.list",
    "lemlist.campaigns.reports.get",
];

pub const LEMLIST_SPEC: ProviderSpec = ProviderSpec {
    slug: "lemlist",
    origin: Origin::Static("https://api.lemlist.com/api"),
    auth: AuthStyle::BasicTemplate {
        user_template: "",
        password_field: "api_key",
    },
    actions: LEMLIST_ACTIONS,
    action_keys: LEMLIST_KEYS,
};

// ---------------------------------------------------------------------------
// Luma public API v1 - x-luma-api-key header. Verified subset of the catalog
// (public API has no delete operation).
// ---------------------------------------------------------------------------

const LUMA_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "luma.events.list",
        method: "GET",
        path: "/public/v1/event/list",
        summary: "Listed Luma events.",
        path_params: &[],
        query: &[("pagination_limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "luma.events.search",
        method: "GET",
        path: "/public/v1/event/search",
        summary: "Searched Luma events.",
        path_params: &[],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "luma.events.get",
        method: "GET",
        path: "/public/v1/event",
        summary: "Read a Luma event.",
        path_params: &[],
        query: &[("api_id", "event_id")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("event_id")],
    },
    ActionSpec {
        key: "luma.events.create",
        method: "POST",
        path: "/public/v1/event/create",
        summary: "Created a Luma event.",
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
];

const LUMA_KEYS: &[&str] = &[
    "luma.events.list",
    "luma.events.search",
    "luma.events.get",
    "luma.events.create",
];

pub const LUMA_SPEC: ProviderSpec = ProviderSpec {
    slug: "luma",
    origin: Origin::Static("https://api.lu.ma"),
    auth: AuthStyle::ApiKeyHeader {
        header: "x-luma-api-key",
        field: "api_key",
    },
    actions: LUMA_ACTIONS,
    action_keys: LUMA_KEYS,
};
