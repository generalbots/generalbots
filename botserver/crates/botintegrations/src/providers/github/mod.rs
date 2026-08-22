//! GitHub provider adapter (#950 slice 2).
//!
//! [`GithubAdapter`] implements the nine actions advertised by the
//! integration catalog (`GITHUB_IMPLEMENTED_ACTIONS`) on top of the shared
//! REST executor in `rest_client` with the GitHub-specific request policy in
//! `client`. Credentials are parsed from the Vault envelope and never leave
//! this module: action outcomes carry only derived data.

mod actions;
mod client;

/// Catalog action keys implemented by this adapter, mirroring the
/// `REPOSITORY_ACTIONS` profile in
/// `botserver/src/apps/integration_catalog/actions/developer.rs` exactly.
pub const GITHUB_IMPLEMENTED_ACTIONS: &[&str] = &[
    "repositories.list",
    "repositories.search",
    "repositories.get",
    "issues.list",
    "issues.search",
    "issues.create",
    "issues.update",
    "pull_requests.list",
    "actions.runs.list",
];

/// Adapter executing live GitHub REST API calls for the integration control
/// plane.
pub struct GithubAdapter;

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

impl GithubAdapter {
    /// Chat-surface action metadata mirroring
    /// `botserver/src/apps/integration_catalog/actions/developer.rs`
    /// exactly - same keys as [`GITHUB_IMPLEMENTED_ACTIONS`] and the same
    /// risk/approval mapping (read -> low/no-approval, write ->
    /// medium/with-approval). Only actions executable from chat are declared
    /// here, so this table is the single truth behind
    /// [`GithubAdapter::safe_action_catalog`].
    fn chat_action_metadata() -> Vec<super::LlmSafeAction> {
        let string = |name: &str, required: bool| param(name, "string", required);
        let integer = |name: &str| param(name, "integer", false);
        let repository = || string("repository", true);
        vec![
            action(
                "repositories.list",
                "List accessible repositories.",
                vec![integer("limit")],
                "low",
                false,
            ),
            action(
                "repositories.search",
                "Search repositories.",
                vec![string("query", true), integer("limit")],
                "low",
                false,
            ),
            action(
                "repositories.get",
                "Read repository metadata.",
                vec![repository()],
                "low",
                false,
            ),
            action(
                "issues.list",
                "List open repository issues.",
                vec![repository()],
                "low",
                false,
            ),
            action(
                "issues.search",
                "Search repository issues.",
                vec![string("repository", false), string("query", true)],
                "low",
                false,
            ),
            action(
                "issues.create",
                "Create a repository issue.",
                vec![param("data", "json", true)],
                "medium",
                true,
            ),
            action(
                "issues.update",
                "Update a repository issue.",
                vec![
                    string("resource_id", true),
                    param("changes", "json", true),
                    repository(),
                ],
                "medium",
                true,
            ),
            action(
                "pull_requests.list",
                "List repository pull requests.",
                vec![repository()],
                "low",
                false,
            ),
            action(
                "actions.runs.list",
                "List workflow runs for a repository.",
                vec![repository()],
                "low",
                false,
            ),
        ]
    }
}

impl super::ProviderAdapter for GithubAdapter {
    fn provider(&self) -> &'static str {
        "github"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        GITHUB_IMPLEMENTED_ACTIONS
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
            let creds = client::GithubCreds::parse(credentials)?;
            actions::invoke(action, &creds, params).await
        })
    }
}
