//! CloudWatch and Lambda actions for the AWS adapter (#950).

use serde_json::{json, Value};

use super::super::client::{signed_request, AwsCreds, RequestPlan, MAX_RESPONSE_BYTES};
use super::super::xml;
use super::{
    invalid, optional_text, outcome, region_of, required_text, FORM_CONTENT_TYPE,
    LAMBDA_PAYLOAD_CAP_BYTES,
};
use crate::providers::ActionOutcome;

const METRICS_PERIOD_SECONDS: i64 = 300;
const METRICS_DEFAULT_WINDOW_SECONDS: i64 = 3600;
const LOGS_DEFAULT_WINDOW_SECONDS: i64 = 6 * 3600;
const LOGS_POLL_ATTEMPTS: u32 = 5;
const LOGS_POLL_INTERVAL_SECONDS: u64 = 1;
const JSON_CONTENT_TYPE: &str = "application/x-amz-json-1.1";

/// Parses the catalog `start`/`end` timestamps (RFC 3339) or applies the
/// documented default window ending now.
fn time_range(params: &Value, default_window_seconds: i64) -> Result<(i64, i64), String> {
    let end = match optional_text(params, "end", 40)? {
        Some(raw) => parse_epoch(&raw, "end")?,
        None => chrono::Utc::now().timestamp(),
    };
    let start = match optional_text(params, "start", 40)? {
        Some(raw) => {
            let start = parse_epoch(&raw, "start")?;
            if start >= end {
                return Err(invalid("start must be earlier than end".to_string()));
            }
            start
        }
        None => end - default_window_seconds,
    };
    Ok((start, end))
}

fn parse_epoch(raw: &str, field: &str) -> Result<i64, String> {
    chrono::DateTime::parse_from_rfc3339(raw)
        .map(|parsed| parsed.timestamp())
        .map_err(|_| invalid(format!("{field} must be an RFC 3339 timestamp")))
}

fn start_rfc3339(epoch_seconds: i64) -> String {
    chrono::DateTime::from_timestamp(epoch_seconds, 0)
        .map(|time| time.to_rfc3339())
        .unwrap_or_else(|| epoch_seconds.to_string())
}

/// POST Action=GetMetricData with one Average query over a 300s period.
pub(crate) async fn metrics_query(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let metric = required_text(params, "metric", 255)?;
    let region = region_of(params)?;
    let (start, end) = time_range(params, METRICS_DEFAULT_WINDOW_SECONDS)?;

    let body = format!(
        "Action=GetMetricData&Version=2010-08-01\
         &MetricDataQueries.member.1.Id=m1\
         &MetricDataQueries.member.1.MetricStat.MetricName={}\
         &MetricDataQueries.member.1.MetricStat.Period={METRICS_PERIOD_SECONDS}\
         &MetricDataQueries.member.1.MetricStat.Stat=Average\
         &MetricDataQueries.member.1.ReturnData=true\
         &StartTime={}\
         &EndTime={}",
        urlencoding::encode(&metric),
        urlencoding::encode(&start_rfc3339(start)),
        urlencoding::encode(&start_rfc3339(end)),
    );
    let response = signed_request(
        RequestPlan {
            service: "monitoring",
            region: &region,
            method: reqwest::Method::POST,
            host: format!("monitoring.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(body.into_bytes()),
            content_type: Some(FORM_CONTENT_TYPE),
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        },
        creds,
    )
    .await?;
    response.require_success("cloudwatch.metrics.query")?;

    let document = response.text();
    let timestamps = xml::collect_leaf_texts(&document, &["Timestamps", "member"], 100);
    let values = xml::collect_leaf_texts(&document, &["Values", "member"], 100);
    let status =
        xml::first_text_by_path(&document, &["StatusCode"]).unwrap_or_else(|| "Unknown".into());
    Ok(outcome(
        format!(
            "Metric {metric}: {} datapoints ({status})",
            timestamps.len()
        ),
        json!({
            "metric": metric,
            "status": status,
            "timestamps": timestamps,
            "values": values,
        }),
    ))
}

/// Logs Insights search: StartQuery followed by bounded GetQueryResults
/// polling until Complete (max 5 attempts, 1s apart).
pub(crate) async fn logs_search(creds: &AwsCreds, params: &Value) -> Result<ActionOutcome, String> {
    let log_group = required_text(params, "log_group", 512)?;
    let query_text = required_text(params, "query", 4096)?;
    let region = region_of(params)?;
    let (start, end) = time_range(params, LOGS_DEFAULT_WINDOW_SECONDS)?;

    let target = "CloudWatchLogs_20140328.StartQuery";
    let body = json!({
        "logGroupName": log_group,
        "queryString": query_text,
        "startTime": start,
        "endTime": end,
        "limit": 100,
    })
    .to_string();
    let response = signed_request(
        RequestPlan {
            service: "logs",
            region: &region,
            method: reqwest::Method::POST,
            host: format!("logs.{region}.amazonaws.com"),
            path_and_query: "/".to_string(),
            body: Some(body.into_bytes()),
            content_type: Some(JSON_CONTENT_TYPE),
            extra_headers: vec![("x-amz-target", target.to_string())],
            response_cap: 64 * 1024,
        },
        creds,
    )
    .await?;
    response.require_success("cloudwatch.logs.search")?;

    let started: Value = serde_json::from_slice(&response.body)
        .map_err(|_| invalid("logs service returned malformed JSON".to_string()))?;
    let query_id = started
        .get("queryId")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("logs service did not return a queryId".to_string()))?
        .to_string();

    let mut status = String::from("Running");
    let mut rows: Vec<Value> = Vec::new();
    for _attempt in 0..LOGS_POLL_ATTEMPTS {
        tokio::time::sleep(std::time::Duration::from_secs(LOGS_POLL_INTERVAL_SECONDS)).await;
        let poll_body = json!({ "queryId": query_id }).to_string();
        let poll = signed_request(
            RequestPlan {
                service: "logs",
                region: &region,
                method: reqwest::Method::POST,
                host: format!("logs.{region}.amazonaws.com"),
                path_and_query: "/".to_string(),
                body: Some(poll_body.into_bytes()),
                content_type: Some(JSON_CONTENT_TYPE),
                extra_headers: vec![(
                    "x-amz-target",
                    "CloudWatchLogs_20140328.GetQueryResults".to_string(),
                )],
                response_cap: MAX_RESPONSE_BYTES,
            },
            creds,
        )
        .await?;
        poll.require_success("cloudwatch.logs.search")?;
        let result: Value = serde_json::from_slice(&poll.body)
            .map_err(|_| invalid("logs results returned malformed JSON".to_string()))?;
        status = result
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .to_string();
        rows = result
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if status != "Running" && !status.is_empty() {
            break;
        }
    }
    let row_count = rows.len();
    Ok(outcome(
        format!("Logs search on {log_group}: {row_count} rows ({status})"),
        json!({
            "query_id": query_id,
            "status": status,
            "row_count": row_count,
            "rows": rows,
        }),
    ))
}

/// GET /2018-10-31/functions?MaxItems=50 - REST-JSON surface.
pub(crate) async fn lambda_functions_list(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let region = region_of(params)?;
    let response = signed_request(
        RequestPlan {
            service: "lambda",
            region: &region,
            method: reqwest::Method::GET,
            host: format!("lambda.{region}.amazonaws.com"),
            path_and_query: "/2018-10-31/functions?MaxItems=50".to_string(),
            body: None,
            content_type: None,
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        },
        creds,
    )
    .await?;
    response.require_success("lambda.functions.list")?;

    let document: Value = serde_json::from_slice(&response.body)
        .map_err(|_| invalid("lambda service returned malformed JSON".to_string()))?;
    let functions: Vec<Value> = document
        .get("Functions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|function| {
            json!({
                "name": function.get("FunctionName").cloned().unwrap_or(Value::Null),
                "runtime": function.get("Runtime").cloned().unwrap_or(Value::Null),
                "last_modified": function.get("LastModified").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();
    let count = functions.len();
    Ok(outcome(
        format!("Listed {count} Lambda functions in {region}"),
        json!({ "region": region, "function_count": count, "functions": functions }),
    ))
}

/// POST /2015-03-31/functions/{name}/invocations - payload is capped at
/// 64 KiB; only the status code and function-error flag are reported.
pub(crate) async fn lambda_functions_invoke(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let function_name = required_text(params, "function_name", 200)?;
    if function_name
        .chars()
        .any(|c| c.is_control() || matches!(c, '?' | '#'))
    {
        return Err(invalid(
            "function_name contains invalid characters".to_string(),
        ));
    }
    let region = region_of(params)?;
    let payload = params.get("payload").cloned().unwrap_or(Value::Null);
    let body = payload.to_string();
    if body.len() > LAMBDA_PAYLOAD_CAP_BYTES {
        return Err(invalid(format!(
            "payload must not exceed {LAMBDA_PAYLOAD_CAP_BYTES} bytes"
        )));
    }

    let encoded_name = urlencoding::encode(&function_name);
    let response = signed_request(
        RequestPlan {
            service: "lambda",
            region: &region,
            method: reqwest::Method::POST,
            host: format!("lambda.{region}.amazonaws.com"),
            path_and_query: format!("/2015-03-31/functions/{encoded_name}/invocations"),
            body: Some(body.into_bytes()),
            content_type: Some("application/json"),
            extra_headers: Vec::new(),
            response_cap: LAMBDA_PAYLOAD_CAP_BYTES + 1,
        },
        creds,
    )
    .await?;
    if !(200..300).contains(&response.status) {
        log::warn!(
            "AWS lambda.functions.invoke failed with status {}: provider body withheld",
            response.status
        );
        return Err(format!(
            "provider_request_failed: lambda.functions.invoke returned status {}",
            response.status
        ));
    }
    let function_error = response.header("x-amz-function-error").is_some();
    Ok(outcome(
        format!(
            "Invoked {function_name} (HTTP {}, function error: {function_error})",
            response.status
        ),
        json!({
            "function_name": function_name,
            "status_code": response.status,
            "function_error": function_error,
        }),
    ))
}
