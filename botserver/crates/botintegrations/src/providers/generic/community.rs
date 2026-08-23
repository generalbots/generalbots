//! Lifestyle & community adapters: Eight Sleep (#993), Moltbook (#1048)
//! and Arena (#1000). Signal and Qik are intentionally NOT adapted —
//! their catalog entries carry Status::Unsupported until an official
//! public API exists (#1051, #988).

use super::helpers::{resource_id, s};
use super::{ActionSpec, AuthStyle, Origin, ParamSpec, ProviderSpec, Risk};

// ── Eight Sleep ──────────────────────────────────────────────────

const EIGHT_SLEEP_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "eight_sleep.user.me",
        method: "GET",
        path: "/v2/users/me",
        summary: "Fetched the authenticated user profile.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "eight_sleep.trends.get",
        method: "GET",
        path: "/v2/users/{resource_id}/trends",
        summary: "Fetched sleep trends for a user.",
        path_params: &["resource_id"],
        query: &[("tz", "tz")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[
            ParamSpec {
                name: "resource_id",
                kind: super::ParamKind::Str,
                required: true,
            },
            s("tz"),
        ],
    },
];

const EIGHT_SLEEP_KEYS: &[&str] = &[
    "eight_sleep.user.me",
    "eight_sleep.trends.get",
];

pub const EIGHT_SLEEP_SPEC: ProviderSpec = ProviderSpec {
    slug: "eight_sleep",
    origin: Origin::Static("https://client-api.8slp.net/v1"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: EIGHT_SLEEP_ACTIONS,
    action_keys: EIGHT_SLEEP_KEYS,
};

// ── Moltbook ─────────────────────────────────────────────────────

const MOLTBOOK_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "moltbook.posts.list",
        method: "GET",
        path: "/posts",
        summary: "Listed community posts.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "moltbook.submolts.list",
        method: "GET",
        path: "/submolts",
        summary: "Listed communities.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
];

const MOLTBOOK_KEYS: &[&str] = &[
    "moltbook.posts.list",
    "moltbook.submolts.list",
];

pub const MOLTBOOK_SPEC: ProviderSpec = ProviderSpec {
    slug: "moltbook",
    origin: Origin::Static("https://www.moltbook.com/api/v1"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: MOLTBOOK_ACTIONS,
    action_keys: MOLTBOOK_KEYS,
};

// ── Arena (community/chat events) ────────────────────────────────

const ARENA_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "arena.events.list",
        method: "GET",
        path: "/v3/events",
        summary: "Listed live community events.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "arena.messages.list",
        method: "GET",
        path: "/v3/events/{resource_id}/messages",
        summary: "Listed messages of an event room.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: resource_id(),
    },
];

const ARENA_KEYS: &[&str] = &[
    "arena.events.list",
    "arena.messages.list",
];

pub const ARENA_SPEC: ProviderSpec = ProviderSpec {
    slug: "arena",
    origin: Origin::Static("https://api.arena.im"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: ARENA_ACTIONS,
    action_keys: ARENA_KEYS,
};
