use super::super::types::{ActionTemplate, Parameter, ParameterType};
use super::{destructive, param, read, write};

const REGION: &[Parameter] = &[param(
    "region",
    ParameterType::String,
    false,
    "AWS region; defaults to the configured region",
)];
const S3_LIST: &[Parameter] = &[
    param("bucket", ParameterType::String, true, "S3 bucket name"),
    param("prefix", ParameterType::String, false, "Object key prefix"),
    param("region", ParameterType::String, false, "AWS region"),
];
const S3_SEARCH: &[Parameter] = &[
    param("bucket", ParameterType::String, true, "S3 bucket name"),
    param(
        "query",
        ParameterType::String,
        true,
        "Object key search text",
    ),
    param(
        "prefix",
        ParameterType::String,
        false,
        "Optional object key prefix",
    ),
];
const S3_GET: &[Parameter] = &[
    param("bucket", ParameterType::String, true, "S3 bucket name"),
    param("key", ParameterType::String, true, "S3 object key"),
];
const S3_PUT: &[Parameter] = &[
    param("bucket", ParameterType::String, true, "S3 bucket name"),
    param("key", ParameterType::String, true, "S3 object key"),
    param(
        "content_reference",
        ParameterType::String,
        true,
        "Reference to content already available to the backend",
    ),
];
const INSTANCE: &[Parameter] = &[
    param(
        "instance_id",
        ParameterType::String,
        true,
        "EC2 instance identifier",
    ),
    param("region", ParameterType::String, false, "AWS region"),
];
const METRIC_QUERY: &[Parameter] = &[
    param(
        "metric",
        ParameterType::String,
        true,
        "CloudWatch metric or expression",
    ),
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const LOGS_SEARCH: &[Parameter] = &[
    param(
        "log_group",
        ParameterType::String,
        true,
        "CloudWatch Logs group",
    ),
    param("query", ParameterType::String, true, "Logs Insights query"),
    param("start", ParameterType::DateTime, false, "Range start"),
    param("end", ParameterType::DateTime, false, "Range end"),
];
const LAMBDA_INVOKE: &[Parameter] = &[
    param(
        "function_name",
        ParameterType::String,
        true,
        "Lambda function name or ARN",
    ),
    param("payload", ParameterType::Json, false, "Invocation payload"),
    param("region", ParameterType::String, false, "AWS region"),
];

pub(crate) const AWS_ACTIONS: &[ActionTemplate] = &[
    read(
        "sts.caller_identity.get",
        "get",
        "Get caller identity",
        "Call STS GetCallerIdentity to verify the configured principal.",
        &[],
    ),
    read(
        "s3.objects.list",
        "list",
        "List S3 objects",
        "List objects in an S3 bucket.",
        S3_LIST,
    ),
    read(
        "s3.objects.search",
        "search",
        "Search S3 objects",
        "Search S3 object keys by prefix and text.",
        S3_SEARCH,
    ),
    read(
        "s3.objects.get",
        "get",
        "Get S3 object",
        "Read an S3 object.",
        S3_GET,
    ),
    write(
        "s3.objects.put",
        "put",
        "Put S3 object",
        "Write an object to S3.",
        S3_PUT,
    ),
    destructive(
        "s3.objects.delete",
        "delete",
        "Delete S3 object",
        "Delete an object from S3.",
        S3_GET,
    ),
    read(
        "ec2.instances.describe",
        "describe",
        "Describe EC2 instances",
        "Describe EC2 instances in a region.",
        REGION,
    ),
    write(
        "ec2.instances.start",
        "start",
        "Start EC2 instance",
        "Start an EC2 instance.",
        INSTANCE,
    ),
    write(
        "ec2.instances.stop",
        "stop",
        "Stop EC2 instance",
        "Stop an EC2 instance.",
        INSTANCE,
    ),
    read(
        "cloudwatch.metrics.query",
        "query",
        "Query CloudWatch metrics",
        "Query CloudWatch metric data.",
        METRIC_QUERY,
    ),
    read(
        "cloudwatch.logs.search",
        "search",
        "Search CloudWatch logs",
        "Run a CloudWatch Logs Insights search.",
        LOGS_SEARCH,
    ),
    read(
        "lambda.functions.list",
        "list",
        "List Lambda functions",
        "List Lambda functions in a region.",
        REGION,
    ),
    write(
        "lambda.functions.invoke",
        "invoke",
        "Invoke Lambda function",
        "Invoke a Lambda function.",
        LAMBDA_INVOKE,
    ),
];
