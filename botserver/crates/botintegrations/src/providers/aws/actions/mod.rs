//! Action dispatch and shared validation helpers for the AWS adapter (#950).

mod manage;
mod observe;
mod s3;

use serde_json::Value;

use super::client::AwsCreds;
use crate::providers::ActionOutcome;

pub(crate) const DEFAULT_REGION: &str = "us-east-1";
/// Inline cap for `s3.objects.get` payloads.
pub(crate) const S3_GET_CAP_BYTES: usize = 256 * 1024;
/// Inline cap for `s3.objects.put` decoded payloads.
pub(crate) const S3_PUT_CAP_BYTES: usize = 128 * 1024;
/// Cap for Lambda invocation request/response payloads.
pub(crate) const LAMBDA_PAYLOAD_CAP_BYTES: usize = 64 * 1024;
const FORM_CONTENT_TYPE: &str = "application/x-www-form-urlencoded";

fn invalid(detail: String) -> String {
    format!("invalid_request: {detail}")
}

fn required_text(params: &Value, key: &str, max_len: usize) -> Result<String, String> {
    let value = params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid(format!("{key} is required")))?;
    if value.len() > max_len {
        return Err(invalid(format!(
            "{key} must be at most {max_len} characters"
        )));
    }
    Ok(value.to_string())
}

fn optional_text(params: &Value, key: &str, max_len: usize) -> Result<Option<String>, String> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.len() > max_len {
                return Err(invalid(format!(
                    "{key} must be at most {max_len} characters"
                )));
            }
            Ok(Some(trimmed.to_string()))
        }
        Some(_) => Err(invalid(format!("{key} must be a string"))),
    }
}

fn region_of(params: &Value) -> Result<String, String> {
    let region = optional_text(params, "region", 32)?.unwrap_or_else(|| DEFAULT_REGION.to_string());
    if !region
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(invalid("region contains invalid characters".to_string()));
    }
    Ok(region)
}

fn validate_bucket(bucket: &str) -> Result<(), String> {
    let ok = (3..=63).contains(&bucket.len())
        && bucket
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        && bucket
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        && bucket
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric());
    if ok {
        Ok(())
    } else {
        Err(invalid(
            "bucket must be a lowercase virtual-host compatible name (3-63 chars)".to_string(),
        ))
    }
}

fn validate_key(key: &str) -> Result<(), String> {
    let ok = (1..=1024).contains(&key.len()) && key.chars().all(|c| !c.is_control());
    if ok {
        Ok(())
    } else {
        Err(invalid(
            "key must be 1-1024 characters without control characters".to_string(),
        ))
    }
}

fn s3_host(bucket: &str, region: &str) -> String {
    format!("{bucket}.s3.{region}.amazonaws.com")
}

/// Percent-encodes an S3 object key while preserving `/` separators.
fn encode_key(key: &str) -> String {
    key.split('/')
        .map(urlencoding::encode)
        .collect::<Vec<_>>()
        .join("/")
}

fn outcome(summary: String, data: Value) -> ActionOutcome {
    ActionOutcome {
        summary,
        data,
        truncated: false,
    }
}

fn number_or_null(raw: Option<&String>) -> Value {
    match raw.and_then(|text| text.parse::<i64>().ok()) {
        Some(number) => Value::from(number),
        None => Value::Null,
    }
}

/// Entry point used by [`super::AwsAdapter`]; unknown keys are rejected here
/// before any parameter validation or network activity happens.
pub(crate) async fn invoke(
    action: &str,
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    match action {
        "sts.caller_identity.get" => manage::sts_caller_identity(creds, params).await,
        "s3.objects.list" => s3::objects_list(creds, params).await,
        "s3.objects.search" => s3::objects_search(creds, params).await,
        "s3.objects.get" => s3::objects_get(creds, params).await,
        "s3.objects.put" => s3::objects_put(creds, params).await,
        "s3.objects.delete" => s3::objects_delete(creds, params).await,
        "ec2.instances.describe" => manage::ec2_instances_describe(creds, params).await,
        "ec2.instances.start" => manage::ec2_instances_change_state(creds, params, true).await,
        "ec2.instances.stop" => manage::ec2_instances_change_state(creds, params, false).await,
        "cloudwatch.metrics.query" => observe::metrics_query(creds, params).await,
        "cloudwatch.logs.search" => observe::logs_search(creds, params).await,
        "lambda.functions.list" => observe::lambda_functions_list(creds, params).await,
        "lambda.functions.invoke" => observe::lambda_functions_invoke(creds, params).await,
        _ => Err(crate::providers::ERR_UNKNOWN_ACTION.to_string()),
    }
}
