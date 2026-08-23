use super::helpers::{json, resource_id, s, s_req};
use super::{ActionSpec, AuthStyle, Origin, ParamKind, ParamSpec, ProviderSpec, Risk};
const DATABRICKS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "databricks.jobs.list",
        method: "GET",
        path: "/api/2.1/jobs/list",
        summary: "Listed Databricks jobs.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "databricks.queries.run",
        method: "POST",
        path: "/api/2.0/sql/statements",
        summary: "Executed a SQL statement.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json("data", true)],
    },
];

const DATABRICKS_KEYS: &[&str] = &["databricks.jobs.list", "databricks.queries.run"];

pub const DATABRICKS_SPEC: ProviderSpec = ProviderSpec {
    slug: "databricks",
    origin: Origin::FromField {
        field: "host_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: DATABRICKS_ACTIONS,
    action_keys: DATABRICKS_KEYS,
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
const HEX_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "hex.projects.list",
        method: "GET",
        path: "/api/v1/projects",
        summary: "Listed Hex projects.",
        path_params: &[],
        query: &[("pageSize", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "hex.runs.get",
        method: "GET",
        path: "/api/v1/projects/{project_id}/runs/{run_id}",
        summary: "Read a Hex run status.",
        path_params: &["project_id", "run_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("project_id"), s_req("run_id")],
    },
    ActionSpec {
        key: "hex.queries.run",
        method: "POST",
        path: "/api/v1/projects/{project_id}/runs",
        summary: "Started a Hex project run.",
        path_params: &["project_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("project_id"), json("data", true)],
    },
];

const HEX_KEYS: &[&str] = &[
    "hex.projects.list",
    "hex.runs.get",
    "hex.queries.run",
];

pub const HEX_SPEC: ProviderSpec = ProviderSpec {
    slug: "hex",
    origin: Origin::Static("https://app.hex.tech"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: HEX_ACTIONS,
    action_keys: HEX_KEYS,
};
const N8N_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "n8n.workflows.list",
        method: "GET",
        path: "/api/v1/workflows",
        summary: "Listed n8n workflows.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "n8n.runs.list",
        method: "GET",
        path: "/api/v1/executions",
        summary: "Listed n8n executions.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "n8n.workflows.get",
        method: "GET",
        path: "/api/v1/workflows/{resource_id}",
        summary: "Read an n8n workflow.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &resource_id(),
    },
];

const N8N_KEYS: &[&str] = &[
    "n8n.workflows.list",
    "n8n.runs.list",
    "n8n.workflows.get",
];

pub const N8N_SPEC: ProviderSpec = ProviderSpec {
    slug: "n8n",
    origin: Origin::FromField {
        field: "base_url",
        pattern: "{value}",
    },
    auth: AuthStyle::ApiKeyHeader {
        header: "X-N8N-API-KEY",
        field: "api_key",
    },
    actions: N8N_ACTIONS,
    action_keys: N8N_KEYS,
};

// ---------------------------------------------------------------------------
// Paddle Billing API - Bearer key; vendor-dashed subdomain for sandbox.
// ---------------------------------------------------------------------------
const POSTMARK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "postmark.messages.list",
        method: "GET",
        path: "/messages/outbound",
        summary: "Listed outbound messages.",
        path_params: &[],
        query: &[("count", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "postmark.domains.list",
        method: "GET",
        path: "/domains",
        summary: "Listed sender domains.",
        path_params: &[],
        query: &[("count", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "postmark.messages.send",
        method: "POST",
        path: "/email",
        summary: "Sent an email via Postmark.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];

const POSTMARK_KEYS: &[&str] = &[
    "postmark.messages.list",
    "postmark.domains.list",
    "postmark.messages.send",
];

pub const POSTMARK_SPEC: ProviderSpec = ProviderSpec {
    slug: "postmark",
    origin: Origin::Static("https://api.postmarkapp.com"),
    auth: AuthStyle::ApiKeyHeader {
        header: "X-Postmark-Server-Token",
        field: "api_key",
    },
    actions: POSTMARK_ACTIONS,
    action_keys: POSTMARK_KEYS,
};
const UPSTASH_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "upstash_redis.queries.run",
    method: "POST",
    path: "/",
    summary: "Executed a Redis command pipeline.",
    path_params: &[],
    query: &[],
    body_param: Some("data"),
    body_wrapper: None,
    risk: Risk::High,
    params: &[json("data", true)],
}];

const UPSTASH_KEYS: &[&str] = &["upstash_redis.queries.run"];

pub const UPSTASH_REDIS_SPEC: ProviderSpec = ProviderSpec {
    slug: "upstash_redis",
    origin: Origin::FromField {
        field: "base_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: UPSTASH_ACTIONS,
    action_keys: UPSTASH_KEYS,
};
