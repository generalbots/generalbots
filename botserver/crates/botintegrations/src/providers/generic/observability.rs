use super::helpers::{json, s, s_req};
use super::{ActionSpec, AuthStyle, Origin, ProviderSpec, Risk};
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
const POSTHOG_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "posthog.events.list",
        method: "GET",
        path: "/api/projects/{project_id}/events/",
        summary: "Listed PostHog events.",
        path_params: &["project_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("project_id"), s("limit")],
    },
    ActionSpec {
        key: "posthog.events.search",
        method: "GET",
        path: "/api/projects/{project_id}/events/",
        summary: "Searched PostHog events.",
        path_params: &["project_id"],
        query: &[("search", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("project_id"), s_req("query")],
    },
];

const POSTHOG_KEYS: &[&str] = &["posthog.events.list", "posthog.events.search"];

pub const POSTHOG_SPEC: ProviderSpec = ProviderSpec {
    slug: "posthog",
    origin: Origin::Static("https://us.posthog.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: POSTHOG_ACTIONS,
    action_keys: POSTHOG_KEYS,
};

// ---------------------------------------------------------------------------
// n8n public API - X-N8N-API-KEY header.
// ---------------------------------------------------------------------------
const SENTRY_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "sentry.issues.list",
        method: "GET",
        path: "/api/0/organizations/{organization_id}/issues/",
        summary: "Listed organization issues.",
        path_params: &["organization_id"],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("organization_id"), s("query")],
    },
    ActionSpec {
        key: "sentry.logs.search",
        method: "GET",
        path: "/api/0/organizations/{organization_id}/events/",
        summary: "Searched organization events.",
        path_params: &["organization_id"],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("organization_id"), s_req("query")],
    },
];

const SENTRY_KEYS: &[&str] = &["sentry.issues.list", "sentry.logs.search"];

pub const SENTRY_SPEC: ProviderSpec = ProviderSpec {
    slug: "sentry",
    origin: Origin::Static("https://sentry.io"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: SENTRY_ACTIONS,
    action_keys: SENTRY_KEYS,
};

// ---------------------------------------------------------------------------
// PostHog API - Bearer personal API key, project scoped paths.
// ---------------------------------------------------------------------------
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
