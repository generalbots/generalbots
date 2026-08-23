//! Batch 3 generic provider specifications (#939 wave): Readwise,
//! Cloudflare, Hugging Face, Resend, Sentry, PostHog, n8n and Paddle.
//! Action keys mirror the integration catalog; each table implements the
//! subset backed by the provider's current official REST API.

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

const RESOURCE_ID: &[ParamSpec] = &[ParamSpec {
    name: "resource_id",
    kind: ParamKind::Str,
    required: true,
}];

// ---------------------------------------------------------------------------
// Readwise Reader API - Bearer token.
// ---------------------------------------------------------------------------

const READWISE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "readwise.bookmarks.list",
        method: "GET",
        path: "/api/v3/list/",
        summary: "Listed saved documents.",
        path_params: &[],
        query: &[("pageCursor", "cursor"), ("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("cursor"), s("limit")],
    },
    ActionSpec {
        key: "readwise.bookmarks.search",
        method: "GET",
        path: "/api/v3/list/",
        summary: "Searched saved documents.",
        path_params: &[],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "readwise.bookmarks.get",
        method: "GET",
        path: "/api/v3/list/{resource_id}",
        summary: "Read a saved document.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
];

const READWISE_KEYS: &[&str] = &[
    "readwise.bookmarks.list",
    "readwise.bookmarks.search",
    "readwise.bookmarks.get",
];

pub const READWISE_SPEC: ProviderSpec = ProviderSpec {
    slug: "readwise",
    origin: Origin::Static("https://readwise.io"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: READWISE_ACTIONS,
    action_keys: READWISE_KEYS,
};

// ---------------------------------------------------------------------------
// Cloudflare API v4 - Bearer token; resources are zones.
// ---------------------------------------------------------------------------

const CLOUDFLARE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "cloudflare.resources.list",
        method: "GET",
        path: "/client/v4/zones",
        summary: "Listed Cloudflare zones.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "cloudflare.resources.search",
        method: "GET",
        path: "/client/v4/zones",
        summary: "Searched Cloudflare zones.",
        path_params: &[],
        query: &[("name", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "cloudflare.resources.get",
        method: "GET",
        path: "/client/v4/zones/{resource_id}",
        summary: "Read a Cloudflare zone.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
];

const CLOUDFLARE_KEYS: &[&str] = &[
    "cloudflare.resources.list",
    "cloudflare.resources.search",
    "cloudflare.resources.get",
];

pub const CLOUDFLARE_SPEC: ProviderSpec = ProviderSpec {
    slug: "cloudflare",
    origin: Origin::Static("https://api.cloudflare.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: CLOUDFLARE_ACTIONS,
    action_keys: CLOUDFLARE_KEYS,
};

// ---------------------------------------------------------------------------
// Hugging Face Hub API - Bearer token. Verified subset: model listing and
// search (inference runs require per-endpoint routing and stay planned).
// ---------------------------------------------------------------------------

const HF_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "hugging_face.models.list",
        method: "GET",
        path: "/api/models",
        summary: "Listed Hugging Face models.",
        path_params: &[],
        query: &[("limit", "limit"), ("sort", "sort")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit"), s("sort")],
    },
    ActionSpec {
        key: "hugging_face.models.search",
        method: "GET",
        path: "/api/models",
        summary: "Searched Hugging Face models.",
        path_params: &[],
        query: &[("search", "query"), ("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query"), s("limit")],
    },
];

const HF_KEYS: &[&str] = &["hugging_face.models.list", "hugging_face.models.search"];

pub const HUGGING_FACE_SPEC: ProviderSpec = ProviderSpec {
    slug: "hugging_face",
    origin: Origin::Static("https://huggingface.co"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: HF_ACTIONS,
    action_keys: HF_KEYS,
};

// ---------------------------------------------------------------------------
// Resend REST API - Bearer api key.
// ---------------------------------------------------------------------------

const RESEND_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "resend.messages.list",
        method: "GET",
        path: "/emails",
        summary: "Listed sent emails.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "resend.messages.get",
        method: "GET",
        path: "/emails/{resource_id}",
        summary: "Read a sent email.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
    ActionSpec {
        key: "resend.domains.list",
        method: "GET",
        path: "/domains",
        summary: "Listed sending domains.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "resend.messages.send",
        method: "POST",
        path: "/emails",
        summary: "Sent an email via Resend.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
];

const RESEND_KEYS: &[&str] = &[
    "resend.messages.list",
    "resend.messages.get",
    "resend.domains.list",
    "resend.messages.send",
];

pub const RESEND_SPEC: ProviderSpec = ProviderSpec {
    slug: "resend",
    origin: Origin::Static("https://api.resend.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: RESEND_ACTIONS,
    action_keys: RESEND_KEYS,
};

// ---------------------------------------------------------------------------
// Sentry Web API - Bearer token.
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
        params: RESOURCE_ID,
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

const PADDLE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "paddle.payments.list",
        method: "GET",
        path: "/billing/transactions",
        summary: "Listed Paddle transactions.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "paddle.payments.get",
        method: "GET",
        path: "/billing/transactions/{resource_id}",
        summary: "Read a Paddle transaction.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
    ActionSpec {
        key: "paddle.customers.search",
        method: "GET",
        path: "/billing/customers",
        summary: "Searched Paddle customers.",
        path_params: &[],
        query: &[("email", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
];

const PADDLE_KEYS: &[&str] = &[
    "paddle.payments.list",
    "paddle.payments.get",
    "paddle.customers.search",
];

pub const PADDLE_SPEC: ProviderSpec = ProviderSpec {
    slug: "paddle",
    origin: Origin::Static("https://api.paddle.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: PADDLE_ACTIONS,
    action_keys: PADDLE_KEYS,
};
