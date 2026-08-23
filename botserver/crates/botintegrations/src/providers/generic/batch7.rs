
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

const fn json(name: &'static str, required: bool) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Json,
        required,
    }
}

const AMPLITUDE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "amplitude.annotations.create",
        method: "POST",
        path: "/api/2/annotations",
        summary: "Created a chart annotation.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "amplitude.events.search",
        method: "GET",
        path: "/api/2/taxonomy/event",
        summary: "Read event taxonomy details.",
        path_params: &[],
        query: &[("event_type", "event_type")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("event_type")],
    },
];

const AMPLITUDE_KEYS: &[&str] = &[
    "amplitude.annotations.create",
    "amplitude.events.search",
];

pub const AMPLITUDE_SPEC: ProviderSpec = ProviderSpec {
    slug: "amplitude",
    origin: Origin::Static("https://amplitude.com"),
    auth: AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: Some("secret_key"),
    },
    actions: AMPLITUDE_ACTIONS,
    action_keys: AMPLITUDE_KEYS,
};

const ALGOLIA_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "algolia.indexes.list",
        method: "GET",
        path: "/1/indexes",
        summary: "Listed Algolia indexes.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "algolia.records.search",
        method: "POST",
        path: "/1/indexes/{index_id}/query",
        summary: "Searched an Algolia index.",
        path_params: &["index_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("index_id"), json("data", true)],
    },
    ActionSpec {
        key: "algolia.records.upsert",
        method: "PUT",
        path: "/1/indexes/{index_id}/objects/{object_id}",
        summary: "Upserted an Algolia object.",
        path_params: &["index_id", "object_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("index_id"), s_req("object_id"), json("data", true)],
    },
    ActionSpec {
        key: "algolia.records.delete",
        method: "DELETE",
        path: "/1/indexes/{index_id}/objects/{resource_id}",
        summary: "Deleted an Algolia object.",
        path_params: &["index_id", "resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &[
            s_req("index_id"),
            ParamSpec {
                name: "resource_id",
                kind: ParamKind::Str,
                required: true,
            },
        ],
    },
];

const ALGOLIA_KEYS: &[&str] = &[
    "algolia.indexes.list",
    "algolia.records.search",
    "algolia.records.upsert",
    "algolia.records.delete",
];

pub const ALGOLIA_SPEC: ProviderSpec = ProviderSpec {
    slug: "algolia",
    origin: Origin::FromField {
        field: "application_id",
        pattern: "https://{value}-dsn.algolia.net",
    },
    auth: AuthStyle::ApiKeyHeaders {
        pairs: &[
            ("X-Algolia-Application-Id", "application_id"),
            ("X-Algolia-API-Key", "api_key"),
        ],
    },
    actions: ALGOLIA_ACTIONS,
    action_keys: ALGOLIA_KEYS,
};

const DEVIN_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "devin.runs.list",
        method: "GET",
        path: "/v1/devins",
        summary: "Listed Devin sessions.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "devin.runs.get",
        method: "GET",
        path: "/v1/devins/{resource_id}",
        summary: "Read a Devin session.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[ParamSpec {
            name: "resource_id",
            kind: ParamKind::Str,
            required: true,
        }],
    },
    ActionSpec {
        key: "devin.models.run",
        method: "POST",
        path: "/v1/devins",
        summary: "Started a Devin session.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json("data", true)],
    },
];

const DEVIN_KEYS: &[&str] = &["devin.runs.list", "devin.runs.get", "devin.models.run"];

pub const DEVIN_SPEC: ProviderSpec = ProviderSpec {
    slug: "devin",
    origin: Origin::Static("https://api.devin.ai"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: DEVIN_ACTIONS,
    action_keys: DEVIN_KEYS,
};

const CURSOR_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "cursor.teams.members",
    method: "GET",
    path: "/v0/teams/members",
    summary: "Listed Cursor team members.",
    path_params: &[],
    query: &[],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[],
}];

const CURSOR_KEYS: &[&str] = &["cursor.teams.members"];

pub const CURSOR_SPEC: ProviderSpec = ProviderSpec {
    slug: "cursor",
    origin: Origin::Static("https://api.cursor.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: CURSOR_ACTIONS,
    action_keys: CURSOR_KEYS,
};

const SPLUNK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "splunk.metrics.query",
        method: "POST",
        path: "/services/search/jobs?output_mode=json",
        summary: "Started a Splunk search job.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "splunk.issues.list",
        method: "GET",
        path: "/services/server/info?output_mode=json",
        summary: "Read Splunk server info and health.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
];

const SPLUNK_KEYS: &[&str] = &["splunk.metrics.query", "splunk.issues.list"];

pub const SPLUNK_SPEC: ProviderSpec = ProviderSpec {
    slug: "splunk",
    origin: Origin::FromField {
        field: "base_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: SPLUNK_ACTIONS,
    action_keys: SPLUNK_KEYS,
};

const LOOPS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "loops_so.contacts.search",
        method: "GET",
        path: "/contacts/search",
        summary: "Searched Loops contacts by email.",
        path_params: &[],
        query: &[("email", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "loops_so.contacts.create",
        method: "POST",
        path: "/contacts/create",
        summary: "Created a Loops contact.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "loops_so.campaigns.send",
        method: "POST",
        path: "/transactional/send",
        summary: "Sent a Loops transactional email.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];

const LOOPS_KEYS: &[&str] = &[
    "loops_so.contacts.search",
    "loops_so.contacts.create",
    "loops_so.campaigns.send",
];

pub const LOOPS_SO_SPEC: ProviderSpec = ProviderSpec {
    slug: "loops_so",
    origin: Origin::Static("https://app.loops.so/api/v1"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: LOOPS_ACTIONS,
    action_keys: LOOPS_KEYS,
};
