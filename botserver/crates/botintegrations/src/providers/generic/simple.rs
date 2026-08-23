//! Batch 1 generic provider specifications (#939 wave): Zendesk, Trello,
//! Mailchimp, Mercury, Whop and Luma. Every entry mirrors the action keys
//! advertised by the integration catalog issues and targets each provider's
//! current official REST API.

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

const fn json_req(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Json,
        required: true,
    }
}

const RESOURCE_ID: &[ParamSpec] = &[ParamSpec {
    name: "resource_id",
    kind: ParamKind::Str,
    required: true,
}];

// ---------------------------------------------------------------------------
// Zendesk Support API v2 - Basic {email}/token over a tenant subdomain.
// ---------------------------------------------------------------------------

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
        params: RESOURCE_ID,
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
        params: RESOURCE_ID,
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
        params: RESOURCE_ID,
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

const MAILCHIMP_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "mailchimp.campaigns.list",
        method: "GET",
        path: "/campaigns",
        summary: "Listed Mailchimp campaigns.",
        path_params: &[],
        query: &[("count", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "mailchimp.contacts.search",
        method: "GET",
        path: "/search-members",
        summary: "Searched Mailchimp contacts.",
        path_params: &[],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "mailchimp.campaigns.reports.get",
        method: "GET",
        path: "/reports/{campaign_id}",
        summary: "Read campaign report.",
        path_params: &["campaign_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("campaign_id")],
    },
    ActionSpec {
        key: "mailchimp.campaigns.create",
        method: "POST",
        path: "/campaigns",
        summary: "Created a Mailchimp campaign.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json_req("data")],
    },
    ActionSpec {
        key: "mailchimp.campaigns.send",
        method: "POST",
        path: "/campaigns/{campaign_id}/actions/send",
        summary: "Sent a Mailchimp campaign.",
        path_params: &["campaign_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &[s_req("campaign_id")],
    },
    ActionSpec {
        key: "mailchimp.campaigns.delete",
        method: "DELETE",
        path: "/campaigns/{campaign_id}",
        summary: "Deleted a Mailchimp campaign.",
        path_params: &["campaign_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: &[s_req("campaign_id")],
    },
];

const MAILCHIMP_KEYS: &[&str] = &[
    "mailchimp.campaigns.list",
    "mailchimp.contacts.search",
    "mailchimp.campaigns.reports.get",
    "mailchimp.campaigns.create",
    "mailchimp.campaigns.send",
    "mailchimp.campaigns.delete",
];

pub const MAILCHIMP_SPEC: ProviderSpec = ProviderSpec {
    slug: "mailchimp",
    origin: Origin::MailchimpDataCenter,
    auth: AuthStyle::BasicTemplate {
        user_template: "anystring",
        password_field: "api_key",
    },
    actions: MAILCHIMP_ACTIONS,
    action_keys: MAILCHIMP_KEYS,
};

// ---------------------------------------------------------------------------
// Mercury Platform API - Bearer token.
// ---------------------------------------------------------------------------

const MERCURY_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "mercury.accounts.list",
        method: "GET",
        path: "/api/v1/accounts",
        summary: "Listed Mercury accounts.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "mercury.transactions.list",
        method: "GET",
        path: "/api/v1/account/{account_id}/transactions",
        summary: "Listed Mercury transactions.",
        path_params: &["account_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("account_id"), s("limit")],
    },
    ActionSpec {
        key: "mercury.transactions.search",
        method: "GET",
        path: "/api/v1/account/{account_id}/transactions",
        summary: "Searched Mercury transactions.",
        path_params: &["account_id"],
        query: &[("search", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("account_id"), s_req("query")],
    },
    ActionSpec {
        key: "mercury.balances.get",
        method: "GET",
        path: "/api/v1/account/{account_id}",
        summary: "Read Mercury account balance.",
        path_params: &["account_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("account_id")],
    },
    ActionSpec {
        key: "mercury.transfers.create",
        method: "POST",
        path: "/api/v1/transfers",
        summary: "Created a Mercury transfer.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json_req("data")],
    },
];

const MERCURY_KEYS: &[&str] = &[
    "mercury.accounts.list",
    "mercury.transactions.list",
    "mercury.transactions.search",
    "mercury.balances.get",
    "mercury.transfers.create",
];

pub const MERCURY_SPEC: ProviderSpec = ProviderSpec {
    slug: "mercury",
    origin: Origin::Static("https://api.mercury.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: MERCURY_ACTIONS,
    action_keys: MERCURY_KEYS,
};

// ---------------------------------------------------------------------------
// Whop REST API v2 - Bearer token.
// ---------------------------------------------------------------------------

const WHOP_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "whop.products.list",
        method: "GET",
        path: "/products",
        summary: "Listed Whop products.",
        path_params: &[],
        query: &[("per", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "whop.products.search",
        method: "GET",
        path: "/products",
        summary: "Searched Whop products.",
        path_params: &[],
        query: &[("search", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "whop.orders.list",
        method: "GET",
        path: "/orders",
        summary: "Listed Whop orders.",
        path_params: &[],
        query: &[("per", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "whop.products.create",
        method: "POST",
        path: "/products",
        summary: "Created a Whop product.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json_req("data")],
    },
    ActionSpec {
        key: "whop.products.update",
        method: "PATCH",
        path: "/products/{resource_id}",
        summary: "Updated a Whop product.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json_req("data")],
    },
    ActionSpec {
        key: "whop.products.delete",
        method: "DELETE",
        path: "/products/{resource_id}",
        summary: "Deleted a Whop product.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: RESOURCE_ID,
    },
];

const WHOP_KEYS: &[&str] = &[
    "whop.products.list",
    "whop.products.search",
    "whop.orders.list",
    "whop.products.create",
    "whop.products.update",
    "whop.products.delete",
];

pub const WHOP_SPEC: ProviderSpec = ProviderSpec {
    slug: "whop",
    origin: Origin::Static("https://api.whop.com/api/v2"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: WHOP_ACTIONS,
    action_keys: WHOP_KEYS,
};
