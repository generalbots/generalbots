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

impl AwsAdapter {
    /// Chat-surface action metadata mirroring
    /// `botserver/src/apps/integration_catalog/actions/aws.rs` exactly -
    /// same keys as [`AWS_IMPLEMENTED_ACTIONS`] and the same
    /// risk/approval mapping (read -> low/no-approval, write ->
    /// medium/with-approval, destructive -> high/with-approval). Only
    /// actions executable from chat are declared here, so this table is the
    /// single truth behind [`AwsAdapter::safe_action_catalog`].
    fn chat_action_metadata() -> Vec<super::LlmSafeAction> {
        let string = |name: &str, required: bool| param(name, "string", required);
        let date = |name: &str| param(name, "datetime", false);
        vec![
            action(
                "sts.caller_identity.get",
                "Call STS GetCallerIdentity to verify the configured principal.",
                vec![],
                "low",
                false,
            ),
            action(
                "s3.objects.list",
                "List objects in an S3 bucket.",
                vec![
                    string("bucket", true),
                    string("prefix", false),
                    string("region", false),
                ],
                "low",
                false,
            ),
            action(
                "s3.objects.search",
                "Search S3 object keys by prefix and text.",
                vec![
                    string("bucket", true),
                    string("query", true),
                    string("prefix", false),
                ],
                "low",
                false,
            ),
            action(
                "s3.objects.get",
                "Read an S3 object.",
                vec![string("bucket", true), string("key", true)],
                "low",
                false,
            ),
            action(
                "s3.objects.put",
                "Write an object to S3.",
                vec![
                    string("bucket", true),
                    string("key", true),
                    string("content_reference", true),
                ],
                "medium",
                true,
            ),
            action(
                "s3.objects.delete",
                "Delete an object from S3.",
                vec![string("bucket", true), string("key", true)],
                "high",
                true,
            ),
            action(
                "ec2.instances.describe",
                "Describe EC2 instances in a region.",
                vec![string("region", false)],
                "low",
                false,
            ),
            action(
                "ec2.instances.start",
                "Start an EC2 instance.",
                vec![string("instance_id", true), string("region", false)],
                "medium",
                true,
            ),
            action(
                "ec2.instances.stop",
                "Stop an EC2 instance.",
                vec![string("instance_id", true), string("region", false)],
                "medium",
                true,
            ),
            action(
                "cloudwatch.metrics.query",
                "Query CloudWatch metric data.",
                vec![string("metric", true), date("start"), date("end")],
                "low",
                false,
            ),
            action(
                "cloudwatch.logs.search",
                "Run a CloudWatch Logs Insights search.",
                vec![
                    string("log_group", true),
                    string("query", true),
                    date("start"),
                    date("end"),
                ],
                "low",
                false,
            ),
            action(
                "lambda.functions.list",
                "List Lambda functions in a region.",
                vec![string("region", false)],
                "low",
                false,
            ),
            action(
                "lambda.functions.invoke",
                "Invoke a Lambda function.",
                vec![
                    string("function_name", true),
                    param("payload", "json", false),
                    string("region", false),
                ],
                "medium",
                true,
            ),
        ]
    }
}

impl super::ProviderAdapter for AwsAdapter {
    fn provider(&self) -> &'static str {
        "aws"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        AWS_IMPLEMENTED_ACTIONS
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
            let creds = client::AwsCreds::parse(credentials)?;
            actions::invoke(action, &creds, params).await
        })
    }
}
