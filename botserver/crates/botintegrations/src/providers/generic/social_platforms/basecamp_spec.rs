use super::*;

const BASECAMP_KEYS: &[&str] = &[
    "basecamp.work_items.list",
    "basecamp.todolists.list",
    "basecamp.work_items.create",
    "basecamp.todos.create",
];
pub const BASECAMP_SPEC: ProviderSpec = ProviderSpec {
    slug: "basecamp",
    origin: Origin::FromField {
        field: "base_url",
        pattern: "{value}",
    },
    auth: AuthStyle::Bearer {
        token_field: "token",
    },
    actions: BASECAMP_ACTIONS,
    action_keys: BASECAMP_KEYS,
};
