
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

const HUE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "philips_hue.devices.list",
        method: "GET",
        path: "/clip/v2/resource/device",
        summary: "Listed Hue devices.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "philips_hue.scenes.list",
        method: "GET",
        path: "/clip/v2/resource/scene",
        summary: "Listed Hue scenes.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "philips_hue.devices.update",
        method: "PUT",
        path: "/clip/v2/resource/light/{resource_id}",
        summary: "Updated a Hue light.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json("data", true)],
    },
    ActionSpec {
        key: "philips_hue.scenes.activate",
        method: "PUT",
        path: "/clip/v2/resource/scene/{resource_id}",
        summary: "Activated a Hue scene.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json("data", true)],
    },
];

const HUE_KEYS: &[&str] = &[
    "philips_hue.devices.list",
    "philips_hue.scenes.list",
    "philips_hue.devices.update",
    "philips_hue.scenes.activate",
];

pub const PHILIPS_HUE_SPEC: ProviderSpec = ProviderSpec {
    slug: "philips_hue",
    origin: Origin::FromField {
        field: "bridge_url",
        pattern: "{value}",
    },
    auth: AuthStyle::ApiKeyHeader {
        header: "hue-application-key",
        field: "api_key",
    },
    actions: HUE_ACTIONS,
    action_keys: HUE_KEYS,
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

const GRAFANA_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "grafana.alerts.list",
        method: "GET",
        path: "/api/prometheus/grafana/api/v1/alerts",
        summary: "Listed Grafana alerts.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "grafana.resources.search",
        method: "GET",
        path: "/api/search",
        summary: "Searched Grafana dashboards.",
        path_params: &[],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("query")],
    },
    ActionSpec {
        key: "grafana.metrics.query",
        method: "POST",
        path: "/api/ds/query",
        summary: "Queried Grafana datasources.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
];

const GRAFANA_KEYS: &[&str] = &[
    "grafana.alerts.list",
    "grafana.resources.search",
    "grafana.metrics.query",
];

pub const GRAFANA_SPEC: ProviderSpec = ProviderSpec {
    slug: "grafana",
    origin: Origin::FromField {
        field: "base_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: GRAFANA_ACTIONS,
    action_keys: GRAFANA_KEYS,
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
