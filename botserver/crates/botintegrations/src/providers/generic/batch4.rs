
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

const BIGCOMMERCE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "bigcommerce.products.list",
        method: "GET",
        path: "/catalog/products",
        summary: "Listed BigCommerce products.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "bigcommerce.products.search",
        method: "GET",
        path: "/catalog/products",
        summary: "Searched BigCommerce products.",
        path_params: &[],
        query: &[("keyword", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "bigcommerce.orders.list",
        method: "GET",
        path: "/orders",
        summary: "Listed BigCommerce orders.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
];

const BIGCOMMERCE_KEYS: &[&str] = &[
    "bigcommerce.products.list",
    "bigcommerce.products.search",
    "bigcommerce.orders.list",
];

pub const BIGCOMMERCE_SPEC: ProviderSpec = ProviderSpec {
    slug: "bigcommerce",
    origin: Origin::FromField {
        field: "store_hash",
        pattern: "https://api.bigcommerce.com/stores/{value}/v3",
    },
    auth: AuthStyle::ApiKeyHeader {
        header: "X-Auth-Token",
        field: "token",
    },
    actions: BIGCOMMERCE_ACTIONS,
    action_keys: BIGCOMMERCE_KEYS,
};

const WOOCOMMERCE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "woocommerce.products.list",
        method: "GET",
        path: "/products",
        summary: "Listed WooCommerce products.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "woocommerce.products.search",
        method: "GET",
        path: "/products",
        summary: "Searched WooCommerce products.",
        path_params: &[],
        query: &[("search", "query")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query")],
    },
    ActionSpec {
        key: "woocommerce.orders.list",
        method: "GET",
        path: "/orders",
        summary: "Listed WooCommerce orders.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
];

const WOOCOMMERCE_KEYS: &[&str] = &[
    "woocommerce.products.list",
    "woocommerce.products.search",
    "woocommerce.orders.list",
];

pub const WOOCOMMERCE_SPEC: ProviderSpec = ProviderSpec {
    slug: "woocommerce",
    origin: Origin::FromField {
        field: "url",
        pattern: "{value}/wp-json/wc/v3",
    },
    auth: AuthStyle::BasicJoin {
        first_field: "consumer_key",
        separator: ':',
        second_field: Some("consumer_secret"),
    },
    actions: WOOCOMMERCE_ACTIONS,
    action_keys: WOOCOMMERCE_KEYS,
};

const JOTFORM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "jotform.forms.list",
        method: "GET",
        path: "/user/forms",
        summary: "Listed Jotform forms.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "jotform.responses.list",
        method: "GET",
        path: "/form/{resource_id}/submissions",
        summary: "Listed form submissions.",
        path_params: &["resource_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("form_id"), s("limit")],
    },
];

const JOTFORM_KEYS: &[&str] = &["jotform.forms.list", "jotform.responses.list"];

pub const JOTFORM_SPEC: ProviderSpec = ProviderSpec {
    slug: "jotform",
    origin: Origin::Static("https://api.jotform.com"),
    auth: AuthStyle::QueryPairs {
        pairs: &[("apiKey", "api_key")],
    },
    actions: JOTFORM_ACTIONS,
    action_keys: JOTFORM_KEYS,
};

const LAST_FM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "last_fm.library.list",
        method: "GET",
        path: "/2.0/?method=user.gettopartists",
        summary: "Listed top artists for a listener.",
        path_params: &[],
        query: &[("user", "user"), ("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("user"), s("limit")],
    },
    ActionSpec {
        key: "last_fm.library.search",
        method: "GET",
        path: "/2.0/?method=artist.search",
        summary: "Searched Last.fm artists.",
        path_params: &[],
        query: &[("artist", "query"), ("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("query"), s("limit")],
    },
];

const LAST_FM_KEYS: &[&str] = &["last_fm.library.list", "last_fm.library.search"];

pub const LAST_FM_SPEC: ProviderSpec = ProviderSpec {
    slug: "last_fm",
    origin: Origin::Static("https://ws.audioscrobbler.com"),
    auth: AuthStyle::QueryPairs {
        pairs: &[("api_key", "api_key"), ("format", "format_fixed")],
    },
    actions: LAST_FM_ACTIONS,
    action_keys: LAST_FM_KEYS,
};

const NEWSCATCHER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "newscatcher.articles.search",
        method: "POST",
        path: "/v1/search",
        summary: "Searched news articles.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Low,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "newscatcher.sources.list",
        method: "GET",
        path: "/v1/sources",
        summary: "Listed news sources.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
];

const NEWSCATCHER_KEYS: &[&str] = &["newscatcher.articles.search", "newscatcher.sources.list"];

pub const NEWSCATCHER_SPEC: ProviderSpec = ProviderSpec {
    slug: "newscatcher",
    origin: Origin::Static("https://v3-api.newscatcherapi.com"),
    auth: AuthStyle::ApiKeyHeader {
        header: "x-api-token",
        field: "api_key",
    },
    actions: NEWSCATCHER_ACTIONS,
    action_keys: NEWSCATCHER_KEYS,
};

const CAL_COM_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "cal_com.events.list",
        method: "GET",
        path: "/v2/event-types",
        summary: "Listed Cal.com event types.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "cal_com.bookings.list",
        method: "GET",
        path: "/v2/bookings",
        summary: "Listed Cal.com bookings.",
        path_params: &[],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
];

const CAL_COM_KEYS: &[&str] = &["cal_com.events.list", "cal_com.bookings.list"];

pub const CAL_COM_SPEC: ProviderSpec = ProviderSpec {
    slug: "cal_com",
    origin: Origin::Static("https://api.cal.com"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: CAL_COM_ACTIONS,
    action_keys: CAL_COM_KEYS,
};

const MOTION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "motion.tasks.list",
        method: "GET",
        path: "/tasks",
        summary: "Listed Motion tasks.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "motion.tasks.create",
        method: "POST",
        path: "/tasks",
        summary: "Created a Motion task.",
        path_params: &[],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[json("data", true)],
    },
    ActionSpec {
        key: "motion.tasks.update",
        method: "PATCH",
        path: "/tasks/{resource_id}",
        summary: "Updated a Motion task.",
        path_params: &["resource_id"],
        query: &[],
        body_param: Some("data"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[s_req("resource_id"), json("data", true)],
    },
];

const MOTION_KEYS: &[&str] = &[
    "motion.tasks.list",
    "motion.tasks.create",
    "motion.tasks.update",
];

pub const MOTION_SPEC: ProviderSpec = ProviderSpec {
    slug: "motion",
    origin: Origin::Static("https://api.usemotion.com/v1"),
    auth: AuthStyle::Bearer {
        token_field: "api_key",
    },
    actions: MOTION_ACTIONS,
    action_keys: MOTION_KEYS,
};

const SQUARESPACE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "squarespace.products.list",
        method: "GET",
        path: "/1.0/commerce/products",
        summary: "Listed Squarespace products.",
        path_params: &[],
        query: &[("cursor", "cursor")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("cursor")],
    },
    ActionSpec {
        key: "squarespace.orders.list",
        method: "GET",
        path: "/1.0/commerce/orders",
        summary: "Listed Squarespace orders.",
        path_params: &[],
        query: &[("cursor", "cursor")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("cursor")],
    },
    ActionSpec {
        key: "squarespace.orders.get",
        method: "GET",
        path: "/1.0/commerce/orders/{resource_id}",
        summary: "Read a Squarespace order.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
];

const SQUARESPACE_KEYS: &[&str] = &[
    "squarespace.products.list",
    "squarespace.orders.list",
    "squarespace.orders.get",
];

pub const SQUARESPACE_SPEC: ProviderSpec = ProviderSpec {
    slug: "squarespace",
    origin: Origin::Static("https://api.squarespace.com"),
    auth: AuthStyle::BasicJoin {
        first_field: "token",
        separator: ':',
        second_field: None,
    },
    actions: SQUARESPACE_ACTIONS,
    action_keys: SQUARESPACE_KEYS,
};

const DHL_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "dhl.shipments.track",
        method: "GET",
        path: "/track/shipments",
        summary: "Tracked DHL shipments.",
        path_params: &[],
        query: &[("trackingNumber", "tracking_number")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s_req("tracking_number")],
    },
];

const DHL_KEYS: &[&str] = &["dhl.shipments.track"];

pub const DHL_SPEC: ProviderSpec = ProviderSpec {
    slug: "dhl",
    origin: Origin::Static("https://api-eu.dhl.com"),
    auth: AuthStyle::ApiKeyHeader {
        header: "DHL-API-Key",
        field: "api_key",
    },
    actions: DHL_ACTIONS,
    action_keys: DHL_KEYS,
};

const PHANTOMBUSTER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "phantombuster.workflows.list",
        method: "GET",
        path: "/workflows/fetch-all",
        summary: "Listed PhantomBuster workflows.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
    ActionSpec {
        key: "phantombuster.runs.list",
        method: "GET",
        path: "/workflows/fetch-containers",
        summary: "Listed PhantomBuster container runs.",
        path_params: &[],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[],
    },
];

const PHANTOMBUSTER_KEYS: &[&str] = &[
    "phantombuster.workflows.list",
    "phantombuster.runs.list",
];

pub const PHANTOMBUSTER_SPEC: ProviderSpec = ProviderSpec {
    slug: "phantombuster",
    origin: Origin::Static("https://api.phantombuster.com/api/v2"),
    auth: AuthStyle::ApiKeyHeader {
        header: "X-Phantombuster-Key",
        field: "api_key",
    },
    actions: PHANTOMBUSTER_ACTIONS,
    action_keys: PHANTOMBUSTER_KEYS,
};

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

const GREENHOUSE_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        key: "greenhouse.candidates.list",
        method: "GET",
        path: "/v1/candidates",
        summary: "Listed Greenhouse candidates.",
        path_params: &[],
        query: &[("per_page", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[s("limit")],
    },
    ActionSpec {
        key: "greenhouse.candidates.get",
        method: "GET",
        path: "/v1/candidates/{resource_id}",
        summary: "Read a Greenhouse candidate.",
        path_params: &["resource_id"],
        query: &[],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: RESOURCE_ID,
    },
];

const GREENHOUSE_KEYS: &[&str] = &[
    "greenhouse.candidates.list",
    "greenhouse.candidates.get",
];

pub const GREENHOUSE_SPEC: ProviderSpec = ProviderSpec {
    slug: "greenhouse",
    origin: Origin::Static("https://harvest.greenhouse.io"),
    auth: AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: None,
    },
    actions: GREENHOUSE_ACTIONS,
    action_keys: GREENHOUSE_KEYS,
};
