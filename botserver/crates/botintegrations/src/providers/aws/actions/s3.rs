//! S3 object actions for the AWS adapter (#950): list, search, get, put and
//! delete against virtual-hosted endpoints with strict inline payload caps.

use base64::Engine;
use serde_json::{json, Value};

use super::super::client::{
    signed_request, AwsCreds, AwsResponse, RequestPlan, MAX_RESPONSE_BYTES,
};
use super::super::xml;
use super::{
    encode_key, invalid, number_or_null, optional_text, outcome, region_of, required_text, s3_host,
    validate_bucket, validate_key, S3_GET_CAP_BYTES, S3_PUT_CAP_BYTES,
};
use crate::providers::ActionOutcome;

const MAX_LIST_KEYS: usize = 100;
const SEARCH_MAX_MATCHES: usize = 500;
const SEARCH_MAX_PAGES: usize = 5;

fn parse_bool(raw: Option<&str>) -> bool {
    raw == Some("true")
}

async fn list_objects_page(
    bucket: &str,
    region: &str,
    prefix: Option<&str>,
    continuation: Option<&str>,
    creds: &AwsCreds,
) -> Result<AwsResponse, String> {
    let mut query = format!("list-type=2&max-keys={MAX_LIST_KEYS}");
    if let Some(prefix) = prefix {
        query.push_str("&prefix=");
        query.push_str(&urlencoding::encode(prefix));
    }
    if let Some(token) = continuation {
        query.push_str("&continuation-token=");
        query.push_str(&urlencoding::encode(token));
    }
    signed_request(
        RequestPlan {
            service: "s3",
            region,
            method: reqwest::Method::GET,
            host: s3_host(bucket, region),
            path_and_query: format!("/?{query}"),
            body: None,
            content_type: None,
            extra_headers: Vec::new(),
            response_cap: MAX_RESPONSE_BYTES,
        },
        creds,
    )
    .await
}

/// GET /?list-type=2 - single page of up to 100 keys.
pub(crate) async fn objects_list(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let bucket = required_text(params, "bucket", 255)?;
    validate_bucket(&bucket)?;
    let prefix = optional_text(params, "prefix", 1024)?;
    let region = region_of(params)?;

    let response = list_objects_page(&bucket, &region, prefix.as_deref(), None, creds).await?;
    response.require_success("s3.objects.list")?;

    let document = response.text();
    let contents =
        xml::collect_child_blocks(&document, "ListBucketResult", "Contents", MAX_LIST_KEYS);
    let objects: Vec<Value> = contents
        .iter()
        .map(|block| {
            json!({
                "key": block.get("Key").cloned().unwrap_or_default(),
                "size": number_or_null(block.get("Size")),
                "last_modified": block.get("LastModified").cloned().unwrap_or_default(),
            })
        })
        .collect();
    let key_count = objects.len();
    let is_truncated = parse_bool(xml::first_text_by_path(&document, &["IsTruncated"]).as_deref());
    Ok(outcome(
        format!("Listed {key_count} objects in s3://{bucket}"),
        json!({
            "bucket": bucket,
            "key_count": key_count,
            "objects": objects,
            "is_truncated": is_truncated,
        }),
    ))
}

/// Paginated key search: up to 5 pages / 500 matches filtered by case-
/// sensitive substring over the returned keys.
pub(crate) async fn objects_search(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let bucket = required_text(params, "bucket", 255)?;
    validate_bucket(&bucket)?;
    let query_text = required_text(params, "query", 256)?;
    let prefix = optional_text(params, "prefix", 1024)?;
    let region = region_of(params)?;

    let mut matches: Vec<Value> = Vec::new();
    let mut scanned = 0usize;
    let mut continuation: Option<String> = None;
    for _page in 0..SEARCH_MAX_PAGES {
        let response = list_objects_page(
            &bucket,
            &region,
            prefix.as_deref(),
            continuation.as_deref(),
            creds,
        )
        .await?;
        response.require_success("s3.objects.search")?;

        let document = response.text();
        for block in
            xml::collect_child_blocks(&document, "ListBucketResult", "Contents", MAX_LIST_KEYS)
        {
            let Some(key) = block.get("Key") else {
                continue;
            };
            scanned += 1;
            if key.contains(query_text.as_str()) {
                matches.push(json!({
                    "key": key.clone(),
                    "size": number_or_null(block.get("Size")),
                    "last_modified": block.get("LastModified").cloned().unwrap_or_default(),
                }));
            }
        }
        let is_truncated =
            parse_bool(xml::first_text_by_path(&document, &["IsTruncated"]).as_deref());
        continuation = xml::first_text_by_path(&document, &["NextContinuationToken"]);
        if matches.len() >= SEARCH_MAX_MATCHES || !is_truncated || continuation.is_none() {
            break;
        }
    }

    let truncated = matches.len() >= SEARCH_MAX_MATCHES;
    let match_count = matches.len();
    Ok(ActionOutcome {
        summary: format!(
            "Found {match_count} matching objects in s3://{bucket} (scanned {scanned} keys)"
        ),
        data: json!({
            "bucket": bucket,
            "match_count": match_count,
            "scanned": scanned,
            "matches": matches,
        }),
        truncated,
    })
}

/// GET of one object capped at 256 KiB; larger objects are rejected with a
/// pointer to drive-side tooling instead of being truncated.
pub(crate) async fn objects_get(creds: &AwsCreds, params: &Value) -> Result<ActionOutcome, String> {
    let bucket = required_text(params, "bucket", 255)?;
    validate_bucket(&bucket)?;
    let key = required_text(params, "key", 1024)?;
    validate_key(&key)?;
    let region = region_of(params)?;

    let fetched = signed_request(
        RequestPlan {
            service: "s3",
            region: &region,
            method: reqwest::Method::GET,
            host: s3_host(&bucket, &region),
            path_and_query: format!("/{}", encode_key(&key)),
            body: None,
            content_type: None,
            extra_headers: Vec::new(),
            response_cap: S3_GET_CAP_BYTES + 1,
        },
        creds,
    )
    .await;
    match fetched {
        Ok(response) => {
            response.require_success("s3.objects.get")?;
            let size = response.body.len();
            let content_base64 = base64::engine::general_purpose::STANDARD.encode(&response.body);
            Ok(outcome(
                format!("Read {size} bytes from s3://{bucket}/{key}"),
                json!({
                    "bucket": bucket,
                    "key": key,
                    "size": size,
                    "content_base64": content_base64,
                    "truncated": false,
                }),
            ))
        }
        Err(error) if error.starts_with(super::super::client::ERR_RESPONSE_CAP) => Err(invalid(
            "object exceeds the 256 KiB inline cap; use drive-side tooling for larger objects"
                .to_string(),
        )),
        Err(error) => Err(error),
    }
}

/// PUT of an inline base64 payload (<=128 KiB decoded). Approval gating for
/// write actions is enforced upstream by the catalog and HTTP surface.
pub(crate) async fn objects_put(creds: &AwsCreds, params: &Value) -> Result<ActionOutcome, String> {
    let bucket = required_text(params, "bucket", 255)?;
    validate_bucket(&bucket)?;
    let key = required_text(params, "key", 1024)?;
    validate_key(&key)?;
    let content_reference = required_text(params, "content_reference", 200_000)?;
    let region = region_of(params)?;

    let payload = decode_inline_payload(&content_reference)?;
    let size = payload.len();
    let response = signed_request(
        RequestPlan {
            service: "s3",
            region: &region,
            method: reqwest::Method::PUT,
            host: s3_host(&bucket, &region),
            path_and_query: format!("/{}", encode_key(&key)),
            body: Some(payload),
            content_type: Some("application/octet-stream"),
            extra_headers: Vec::new(),
            response_cap: 64 * 1024,
        },
        creds,
    )
    .await?;
    response.require_success("s3.objects.put")?;

    let etag = response.header("etag").map(str::to_string);
    let version_id = response.header("x-amz-version-id").map(str::to_string);
    Ok(outcome(
        format!("Wrote {size} bytes to s3://{bucket}/{key}"),
        json!({
            "bucket": bucket,
            "key": key,
            "size": size,
            "etag": etag,
            "version_id": version_id,
        }),
    ))
}

/// Strict inline-base64 decoder enforcing the documented 128 KiB cap.
fn decode_inline_payload(content_reference: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(content_reference.trim())
        .ok()
        .filter(|decoded| !decoded.is_empty())
        .filter(|decoded| decoded.len() <= S3_PUT_CAP_BYTES)
        .ok_or_else(|| invalid("content_reference must be inline base64 (<=128KiB)".to_string()))
}

/// DELETE of one object; S3 answers 204 on success.
pub(crate) async fn objects_delete(
    creds: &AwsCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let bucket = required_text(params, "bucket", 255)?;
    validate_bucket(&bucket)?;
    let key = required_text(params, "key", 1024)?;
    validate_key(&key)?;
    let region = region_of(params)?;

    let response = signed_request(
        RequestPlan {
            service: "s3",
            region: &region,
            method: reqwest::Method::DELETE,
            host: s3_host(&bucket, &region),
            path_and_query: format!("/{}", encode_key(&key)),
            body: None,
            content_type: None,
            extra_headers: Vec::new(),
            response_cap: 16 * 1024,
        },
        creds,
    )
    .await?;
    response.require_success("s3.objects.delete")?;

    Ok(outcome(
        format!("Deleted s3://{bucket}/{key}"),
        json!({ "bucket": bucket, "key": key, "deleted": true }),
    ))
}
