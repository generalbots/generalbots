//! Stripe provider adapter (#950 slice 2).
//!
//! [`StripeAdapter`] implements the ten actions advertised by the
//! integration catalog (`STRIPE_IMPLEMENTED_ACTIONS`) on top of the shared
//! REST executor in `rest_client` with the Stripe-specific form-encoded
//! request policy in `client`. Credentials are parsed from the Vault
//! envelope and never leave this module: action outcomes carry only derived
//! data, and secret-bearing response fields are stripped by the shared
//! redactor.

mod actions;
mod client;

/// Catalog action keys implemented by this adapter, mirroring the
/// `STRIPE_ACTIONS` profile in
/// `botserver/src/apps/integration_catalog/actions/transactions.rs` exactly.
pub const STRIPE_IMPLEMENTED_ACTIONS: &[&str] = &[
    "balance.retrieve",
    "customers.list",
    "customers.search",
    "customers.create",
    "payment_intents.list",
    "payment_intents.get",
    "payment_intents.create",
    "prices.list",
    "refunds.create",
    "subscriptions.list",
];

/// Adapter executing live Stripe API calls for the integration control
/// plane.
pub struct StripeAdapter;

fn param(name: &str, kind: &str, required: bool) -> super::LlmSafeParam {
    super::LlmSafeParam {
        name: name.to_string(),
        kind: kind.to_string(),
        required,
    }
}

fn action(
    key: &str,
    summary: &str,
    params: Vec<super::LlmSafeParam>,
    risk: &str,
    requires_approval: bool,
) -> super::LlmSafeAction {
    super::LlmSafeAction {
        name: key.to_string(),
        summary: summary.to_string(),
        params,
        risk: risk.to_string(),
        requires_approval,
    }
}

impl StripeAdapter {
    /// Chat-surface action metadata mirroring
    /// `botserver/src/apps/integration_catalog/actions/transactions.rs`
    /// exactly - same keys as [`STRIPE_IMPLEMENTED_ACTIONS`] and the same
    /// risk/approval mapping (read -> low/no-approval, write ->
    /// medium/with-approval, destructive -> high/with-approval). Only actions
    /// executable from chat are declared here, so this table is the single
    /// truth behind [`StripeAdapter::safe_action_catalog`].
    fn chat_action_metadata() -> Vec<super::LlmSafeAction> {
        let string = |name: &str, required: bool| param(name, "string", required);
        let integer = |name: &str| param(name, "integer", false);
        vec![
            action(
                "balance.retrieve",
                "Read the Stripe account balance.",
                vec![],
                "low",
                false,
            ),
            action(
                "customers.list",
                "List payment customers.",
                vec![integer("limit")],
                "low",
                false,
            ),
            action(
                "customers.search",
                "Search payment customers.",
                vec![string("query", true), integer("limit")],
                "low",
                false,
            ),
            action(
                "customers.create",
                "Create a payment customer.",
                vec![param("data", "json", true)],
                "medium",
                true,
            ),
            action(
                "payment_intents.list",
                "List payments.",
                vec![integer("limit")],
                "low",
                false,
            ),
            action(
                "payment_intents.get",
                "Read payment details.",
                vec![string("resource_id", true)],
                "low",
                false,
            ),
            action(
                "payment_intents.create",
                "Create or authorize a payment.",
                vec![param("data", "json", true)],
                "medium",
                true,
            ),
            action(
                "prices.list",
                "List active prices.",
                vec![integer("limit")],
                "low",
                false,
            ),
            action(
                "refunds.create",
                "Refund a payment.",
                vec![
                    string("payment_intent", true),
                    integer("amount"),
                    string("reason", false),
                ],
                "high",
                true,
            ),
            action(
                "subscriptions.list",
                "List subscriptions.",
                vec![integer("limit")],
                "low",
                false,
            ),
        ]
    }
}

impl super::ProviderAdapter for StripeAdapter {
    fn provider(&self) -> &'static str {
        "stripe"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        STRIPE_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<super::LlmSafeAction> {
        Self::chat_action_metadata()
    }

    fn invoke<'a>(
        &'a self,
        action: &'a str,
        credentials: &'a serde_json::Value,
        params: &'a serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<super::ActionOutcome, String>> + Send + 'a>,
    > {
        Box::pin(async move {
            let creds = client::StripeCreds::parse(credentials)?;
            actions::invoke(action, &creds, params).await
        })
    }
}
