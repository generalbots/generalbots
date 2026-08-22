//! STS and EC2 actions for the AWS adapter (#950).

use serde_json::{json, Value};

use super::super::client::{signed_request, AwsCreds, RequestPlan, MAX_RESPONSE_BYTES};
use super::super::xml;
use super::{invalid, outcome, region_of, required_text, FORM_CONTENT_TYPE};
use crate::providers::ActionOutcome;

/// POST Action=GetCallerIdentity - verifies the configured principal.
pub(crate) async fn sts_caller_identity(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let region = region_of(params)?;
    let response = signed_request(
        RequestPlan {
            service: "sts",
            region: &region,
            method: reqwest::Method::POST,
            host: format!("sts.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(b"Action=GetCallerIdentity&Version=2011-06-15".to_vec()),
            content_type: Some(FORM_CONTENT_TYPE),
            extra_headers: Vec::new(),
            response_cap: 64 * 1024,
        },
        creds,
    )
    .await?;
    response.require_success("sts.caller_identity.get")?;

    let document = response.text();
    let arn = xml::first_text_by_path(&document, &["Arn"]).unwrap_or_default();
    let account = xml::first_text_by_path(&document, &["Account"]).unwrap_or_default();
    let user_id = xml::first_text_by_path(&document, &["UserId"]).unwrap_or_default();
    if arn.is_empty() {
        return Err(invalid(
            "STS response did not contain a caller ARN".to_string(),
        ));
    }
    Ok(outcome(
        format!("Verified AWS principal {arn}"),
        json!({ "arn": arn, "account": account, "user_id": user_id }),
    ))
}

fn validate_instance_id(instance_id: &str) -> Result<(), String> {
    let ok = (3..=256).contains(&instance_id.len())
        && (instance_id.starts_with("i-") || instance_id.starts_with("arn:aws:ec2:"))
        && instance_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | '/' | '_'));
    if ok {
        Ok(())
    } else {
        Err(invalid(
            "instance_id must be an EC2 instance id or instance ARN".to_string(),
        ))
    }
}

/// POST Action=DescribeInstances with MaxResults=50; parses the minimal
/// instanceId/state pairs from the reservation set.
pub(crate) async fn ec2_instances_describe(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let region = region_of(params)?;
    let body = "Action=DescribeInstances&Version=2016-11-15&MaxResults=50";
    let response = signed_request(
        RequestPlan {
            service: "ec2",
            region: &region,
            method: reqwest::Method::POST,
            host: format!("ec2.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(body.as_bytes().to_vec()),
            content_type: Some(FORM_CONTENT_TYPE),
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        },
        creds,
    )
    .await?;
    response.require_success("ec2.instances.describe")?;

    let document = response.text();
    let ids = xml::collect_leaf_texts(&document, &["instanceId"], 50);
    let states = xml::collect_leaf_texts(&document, &["name"], 50);
    let instances: Vec<Value> = ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            json!({
                "instance_id": id,
                "state": states.get(index).cloned().unwrap_or_default(),
            })
        })
        .collect();
    let count = instances.len();
    let running = instances
        .iter()
        .filter(|item| item.get("state").and_then(Value::as_str) == Some("running"))
        .count();
    Ok(outcome(
        format!("Described {count} EC2 instances in {region} ({running} running)"),
        json!({ "region": region, "instance_count": count, "instances": instances }),
    ))
}

async fn ec2_change_state(
    creds: &AwsCreds,
    action: &'static str,
    verb: &str,
    instance_id: &str,
    region: &str,
) -> Result<ActionOutcome, String> {
    let encoded = urlencoding::encode(instance_id);
    let body = format!("Action={action}&Version=2016-11-15&InstanceId={encoded}");
    let response = signed_request(
        RequestPlan {
            service: "ec2",
            region,
            method: reqwest::Method::POST,
            host: format!("ec2.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(body.into_bytes()),
            content_type: Some(FORM_CONTENT_TYPE),
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        },
        creds,
    )
    .await?;
    response.require_success(action)?;

    let document = response.text();
    let ids = xml::collect_leaf_texts(&document, &["instanceId"], 10);
    let current_states = xml::collect_leaf_texts(&document, &["currentState", "name"], 10);
    let instances: Vec<Value> = ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            json!({
                "instance_id": id,
                "current_state": current_states.get(index).cloned().unwrap_or_default(),
            })
        })
        .collect();
    let state = instances
        .first()
        .and_then(|item| item.get("current_state"))
        .cloned()
        .unwrap_or(Value::Null);
    Ok(outcome(
        format!("{verb} request accepted for {instance_id} (now {state})"),
        json!({ "instance_id": instance_id, "instances": instances }),
    ))
}

/// POST Action=StartInstances / StopInstances for one instance.
pub(crate) async fn ec2_instances_change_state(
    creds: &AwsCreds,
    params: &Value,
    start: bool,
) -> Result<ActionOutcome, String> {
    let instance_id = required_text(params, "instance_id", 256)?;
    validate_instance_id(&instance_id)?;
    let region = region_of(params)?;
    if start {
        ec2_change_state(creds, "StartInstances", "Start", &instance_id, &region).await
    } else {
        ec2_change_state(creds, "StopInstances", "Stop", &instance_id, &region).await
    }
}
