//! AWS provider adapter (#950 slice 1).
//!
//! [`AwsAdapter`] implements the thirteen actions advertised by the
//! integration catalog (`AWS_IMPLEMENTED_ACTIONS`) on top of the SigV4
//! executor in `client`. Credentials are parsed from the Vault envelope and
//! never leave this module: action outcomes carry only derived data.

mod actions;
mod client;
mod xml;

/// Catalog action keys implemented by this adapter, mirroring
/// `botserver/src/apps/integration_catalog/actions/aws.rs` exactly.
pub const AWS_IMPLEMENTED_ACTIONS: &[&str] = &[
    "sts.caller_identity.get",
    "s3.objects.list",
    "s3.objects.search",
    "s3.objects.get",
    "s3.objects.put",
    "s3.objects.delete",
    "ec2.instances.describe",
    "ec2.instances.start",
    "ec2.instances.stop",
    "cloudwatch.metrics.query",
    "cloudwatch.logs.search",
    "lambda.functions.list",
    "lambda.functions.invoke",
];

/// Adapter executing live AWS API calls for the integration control plane.
pub struct AwsAdapter;

impl super::ProviderAdapter for AwsAdapter {
    fn provider(&self) -> &'static str {
        "aws"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        AWS_IMPLEMENTED_ACTIONS
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
            let creds = client::AwsCreds::parse(credentials)?;
            actions::invoke(action, &creds, params).await
        })
    }
}
