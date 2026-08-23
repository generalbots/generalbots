
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

const COINBASE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "coinbase.accounts.list",
        method: "GET",
        path: "/v2/accounts",
        summary: "Listed Coinbase accounts.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "coinbase.transactions.list",
        method: "GET",
        path: "/v2/accounts/{account_id}/transactions",
        summary: "Listed account transactions.",
        path_params: &["account_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("account_id"), s("limit")],
    },
    ActionSpec {
        key: "coinbase.quotes.get",
        method: "GET",
        path: "/v2/prices/{currency_pair}/spot",
        summary: "Read spot price quote.",
        path_params: &["currency_pair"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("currency_pair")],
    },
];

const COINBASE_KEYS: &[&str] = &[
    "coinbase.accounts.list",
    "coinbase.transactions.list",
    "coinbase.quotes.get",
];

pub const COINBASE_SPEC: ProviderSpec = ProviderSpec {
    slug: "coinbase",
    origin: Origin::Static("https://api.coinbase.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: COINBASE_ACTIONS,
    action_keys: COINBASE_KEYS,
};

const VERCEL_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "vercel.projects.list",
        method: "GET",
        path: "/v2/projects",
        summary: "Listed Vercel projects.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "vercel.deployments.list",
        method: "GET",
        path: "/v6/deployments",
        summary: "Listed Vercel deployments.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "vercel.deployments.get",
        method: "GET",
        path: "/v13/deployments/{resource_id}",
        summary: "Read a Vercel deployment.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
    ActionSpec {
        key: "vercel.deployments.cancel",
        method: "PATCH",
        path: "/v12/deployments/{resource_id}/cancel",
        summary: "Cancelled a Vercel deployment.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::High,
        params: RESOURCE_ID,
    },
];

const VERCEL_KEYS: &[&str] = &[
    "vercel.projects.list",
    "vercel.deployments.list",
    "vercel.deployments.get",
    "vercel.deployments.cancel",
];

pub const VERCEL_SPEC: ProviderSpec = ProviderSpec {
    slug: "vercel",
    origin: Origin::Static("https://api.vercel.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: VERCEL_ACTIONS,
    action_keys: VERCEL_KEYS,
};

const SUPABASE_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "supabase.projects.list",
    method: "GET",
    path: "/v1/projects",
    summary: "Listed Supabase projects.",
    path_params: &[],
    query: &[],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[],
}];

const SUPABASE_KEYS: &[&str] = &["supabase.projects.list"];

pub const SUPABASE_SPEC: ProviderSpec = ProviderSpec {
    slug: "supabase",
    origin: Origin::Static("https://api.supabase.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: SUPABASE_ACTIONS,
    action_keys: SUPABASE_KEYS,
};

const WISE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "wise.accounts.list",
        method: "GET",
        path: "/v1/profiles/{profile_id}",
        summary: "Read Wise profile and accounts.",
        path_params: &["profile_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("profile_id")],
    },
    ActionSpec {
        key: "wise.balances.get",
        method: "GET",
        path: "/v4/profiles/{profile_id}/balances",
        summary: "Listed Wise balances.",
        path_params: &["profile_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("profile_id")],
    },
    ActionSpec {
        key: "wise.transfers.create",
        method: "POST",
        path: "/v1/transfers",
        summary: "Created a Wise transfer.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::High,
        params: &[json("data", true)],
    },
];

const WISE_KEYS: &[&str] = &[
    "wise.accounts.list",
    "wise.balances.get",
    "wise.transfers.create",
];

pub const WISE_SPEC: ProviderSpec = ProviderSpec {
    slug: "wise",
    origin: Origin::Static("https://api.wise.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: WISE_ACTIONS,
    action_keys: WISE_KEYS,
};

const MOONPAY_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "moonpay.accounts.list",
        method: "GET",
        path: "/v1/accounts/me",
        summary: "Read the MoonPay account.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "moonpay.transactions.list",
        method: "GET",
        path: "/v1/transactions",
        summary: "Listed MoonPay transactions.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "moonpay.quotes.get",
        method: "GET",
        path: "/v3/currencies/{currency_code}/quote",
        summary: "Read a buy quote.",
        path_params: &["currency_code"],
        query: &[("baseCurrencyAmount", "amount")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("currency_code"), s("amount")],
    },
];

const MOONPAY_KEYS: &[&str] = &[
    "moonpay.accounts.list",
    "moonpay.transactions.list",
    "moonpay.quotes.get",
];

pub const MOONPAY_SPEC: ProviderSpec = ProviderSpec {
    slug: "moonpay",
    origin: Origin::Static("https://api.moonpay.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: MOONPAY_ACTIONS,
    action_keys: MOONPAY_KEYS,
};

const BEEHIIV_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "beehiiv.content.list",
        method: "GET",
        path: "/public/v1/publications/{publication_id}/posts",
        summary: "Listed beehiiv posts.",
        path_params: &["publication_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("publication_id"), s("limit")],
    },
    ActionSpec {
        key: "beehiiv.content.search",
        method: "GET",
        path: "/public/v1/publications/{publication_id}/posts",
        summary: "Searched beehiiv posts.",
        path_params: &["publication_id"],
        query: &[("query", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("publication_id"), s_req("query")],
    },
    ActionSpec {
        key: "beehiiv.subscriptions.list",
        method: "GET",
        path: "/public/v1/publications/{publication_id}/subscriptions",
        summary: "Listed beehiiv subscriptions.",
        path_params: &["publication_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("publication_id"), s("limit")],
    },
];

const BEEHIIV_KEYS: &[&str] = &[
    "beehiiv.content.list",
    "beehiiv.content.search",
    "beehiiv.subscriptions.list",
];

pub const BEEHIIV_SPEC: ProviderSpec = ProviderSpec {
    slug: "beehiiv",
    origin: Origin::Static("https://api.beehiiv.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: BEEHIIV_ACTIONS,
    action_keys: BEEHIIV_KEYS,
};

const AHREFS_ACTIONS: &[ActionSpec] = &[ActionSpec {
    key: "ahrefs.keywords.search",
    method: "GET",
    path: "/v3/site-explorer/organic-keywords",
    summary: "Fetched organic keywords for a target.",
    path_params: &[],
    query: &[("target", "target"), ("country", "country"), ("limit", "limit")],
    body_param: None,
    body_wrapper: None,
    risk: Risk::Low,
    params: &[s_req("target"), s("country"), s("limit")],
}];

const AHREFS_KEYS: &[&str] = &["ahrefs.keywords.search"];

pub const AHREFS_SPEC: ProviderSpec = ProviderSpec {
    slug: "ahrefs",
    origin: Origin::Static("https://api.ahrefs.com"),
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: AHREFS_ACTIONS,
    action_keys: AHREFS_KEYS,
};

const CRUNCHBASE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "crunchbase.contacts.search",
        method: "GET",
        path: "/data/entities/organizations",
        summary: "Searched Crunchbase organizations.",
        path_params: &[],
        query: &[("q", "query"), ("field_ids", "fields")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query"), s("fields")],
    },
    ActionSpec {
        key: "crunchbase.deals.list",
        method: "GET",
        path: "/data/entities/funding_rounds",
        summary: "Listed Crunchbase funding rounds.",
        path_params: &[],
        query: &[("field_ids", "fields")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("fields")],
    },
];

const CRUNCHBASE_KEYS: &[&str] = &[
    "crunchbase.contacts.search",
    "crunchbase.deals.list",
];

pub const CRUNCHBASE_SPEC: ProviderSpec = ProviderSpec {
    slug: "crunchbase",
    origin: Origin::Static("https://api.crunchbase.com/api/v4"),
    auth: AuthStyle::QueryPairs {
        pairs: &[("user_key", "user_key")],
    },
    actions: CRUNCHBASE_ACTIONS,
    action_keys: CRUNCHBASE_KEYS,
};
