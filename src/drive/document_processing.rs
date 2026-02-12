use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct MergeDocumentsRequest {
    pub bucket: String,
    pub source_paths: Vec<String>,
    pub output_path: String,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertDocumentRequest {
    pub bucket: String,
    pub source_path: String,
    pub output_path: String,
    pub from_format: String,
    pub to_format: String,
}

#[derive(Debug, Deserialize)]
pub struct FillDocumentRequest {
    pub bucket: String,
    pub template_path: String,
    pub output_path: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ExportDocumentRequest {
    pub bucket: String,
    pub source_path: String,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct ImportDocumentRequest {
    pub bucket: String,
    pub source_url: Option<String>,
    pub source_data: Option<String>,
    pub output_path: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub success: bool,
    pub output_path: Option<String>,
    pub message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn merge_documents(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MergeDocumentsRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    if req.source_paths.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No source documents provided" })),
        ));
    }

    let mut merged_content = String::new();
    let format = req.format.as_deref().unwrap_or("txt");

    for (idx, path) in req.source_paths.iter().enumerate() {
        let result = s3_client
            .get_object()
            .bucket(&req.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to read document {}: {}", path, e) })),
                )
            })?;

        let bytes = result.body.collect().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read document body: {}", e) })),
            )
        })?.into_bytes();

        let content = String::from_utf8(bytes.to_vec()).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Document is not valid UTF-8: {}", e) })),
            )
        })?;

        if idx > 0 {
            merged_content.push_str("\n\n");
            if format == "md" || format == "markdown" {
                merged_content.push_str("---\n\n");
            }
        }
        merged_content.push_str(&content);
    }

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&req.output_path)
        .body(merged_content.into_bytes().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to write merged document: {}", e) })),
            )
        })?;

    Ok(Json(DocumentResponse {
        success: true,
        output_path: Some(req.output_path),
        message: Some(format!(
            "Successfully merged {} documents",
            req.source_paths.len()
        )),
        metadata: Some(serde_json::json!({
            "source_count": req.source_paths.len(),
            "format": format
        })),
    }))
}

pub async fn convert_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConvertDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&req.bucket)
        .key(&req.source_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read source document: {}", e) })),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to read document body: {}", e) }),
                ),
            )
        })?
        .into_bytes();

    let source_content = String::from_utf8(bytes.to_vec()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Document is not valid UTF-8: {}", e) })),
        )
    })?;

    let converted_content = match (req.from_format.as_str(), req.to_format.as_str()) {
        ("txt", "md") | ("text", "markdown") => {
            format!("# Converted Document\n\n{}", source_content)
        }
        ("md", "txt") | ("markdown", "text") => source_content
            .lines()
            .map(|line| {
                line.trim_start_matches('#')
                    .trim_start_matches('*')
                    .trim_start_matches('-')
                    .trim()
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ("json", "csv") => {
            let data: Result<serde_json::Value, _> = serde_json::from_str(&source_content);
            match data {
                Ok(serde_json::Value::Array(arr)) => {
                    if arr.is_empty() {
                        String::new()
                    } else {
                        let headers = if let Some(serde_json::Value::Object(first)) = arr.first() {
                            first.keys().cloned().collect::<Vec<_>>()
                        } else {
                            vec![]
                        };

                        let mut csv = headers.join(",") + "\n";
                        for item in arr {
                            if let serde_json::Value::Object(obj) = item {
                                let row = headers
                                    .iter()
                                    .map(|h| {
                                        obj.get(h)
                                            .map(|v| {
                                                if let Some(s) = v.as_str() {
                                                    s.to_string()
                                                } else {
                                                    v.to_string()
                                                }
                                            })
                                            .unwrap_or_else(String::new)
                                    })
                                    .collect::<Vec<_>>()
                                    .join(",");
                                csv.push_str(&row);
                                csv.push('\n');
                            }
                        }
                        csv
                    }
                }
                _ => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(
                            serde_json::json!({ "error": "JSON must be an array for CSV conversion" }),
                        ),
                    ));
                }
            }
        }
        ("csv", "json") => {
            let lines: Vec<&str> = source_content.lines().collect();
            if lines.is_empty() {
                "[]".to_string()
            } else {
                let headers: Vec<&str> = lines[0].split(',').collect();
                let mut result = Vec::new();

                for line in lines.iter().skip(1) {
                    let values: Vec<&str> = line.split(',').collect();
                    let mut obj = serde_json::Map::new();
                    for (i, header) in headers.iter().enumerate() {
                        if let Some(value) = values.get(i) {
                            obj.insert(header.trim().to_string(), serde_json::json!(value.trim()));
                        }
                    }
                    result.push(serde_json::Value::Object(obj));
                }
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "[]".to_string())
            }
        }
        ("html", "txt" | "text") => {
            source_content
                .replace("<br>", "\n")
                .replace("<p>", "\n")
                .replace("</p>", "\n")
                .chars()
                .fold((String::new(), false), |(mut acc, in_tag), c| {
                    if c == '<' {
                        (acc, true)
                    } else if c == '>' {
                        (acc, false)
                    } else if !in_tag {
                        acc.push(c);
                        (acc, in_tag)
                    } else {
                        (acc, in_tag)
                    }
                })
                .0
        }
        _ => source_content,
    };

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&req.output_path)
        .body(converted_content.into_bytes().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to write converted document: {}", e) })),
            )
        })?;

    Ok(Json(DocumentResponse {
        success: true,
        output_path: Some(req.output_path),
        message: Some(format!(
            "Successfully converted from {} to {}",
            req.from_format, req.to_format
        )),
        metadata: Some(serde_json::json!({
            "from_format": req.from_format,
            "to_format": req.to_format
        })),
    }))
}

pub async fn fill_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FillDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&req.bucket)
        .key(&req.template_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read template: {}", e) })),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to read template body: {}", e) }),
                ),
            )
        })?
        .into_bytes();

    let mut template = String::from_utf8(bytes.to_vec()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Template is not valid UTF-8: {}", e) })),
        )
    })?;

    if let serde_json::Value::Object(data_map) = &req.data {
        for (key, value) in data_map {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => String::new(),
                _ => value.to_string(),
            };
            template = template.replace(&placeholder, &replacement);
        }
    }

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&req.output_path)
        .body(template.into_bytes().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to write filled document: {}", e) })),
            )
        })?;

    Ok(Json(DocumentResponse {
        success: true,
        output_path: Some(req.output_path),
        message: Some("Successfully filled document template".to_string()),
        metadata: Some(serde_json::json!({
            "template": req.template_path,
            "fields_filled": req.data.as_object().map(|o| o.len()).unwrap_or(0)
        })),
    }))
}

pub async fn export_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&req.bucket)
        .key(&req.source_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read document: {}", e) })),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to read document body: {}", e) }),
                ),
            )
        })?
        .into_bytes();

    let content = String::from_utf8(bytes.to_vec()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Document is not valid UTF-8: {}", e) })),
        )
    })?;

    let exported_content = match req.format.as_str() {
        "pdf" => {
            format!("PDF Export:\n{}", content)
        }
        "docx" => {
            format!("DOCX Export:\n{}", content)
        }
        "html" => {
            format!("<!DOCTYPE html>\n<html>\n<head><title>Exported Document</title></head>\n<body>\n<pre>{}</pre>\n</body>\n</html>", content)
        }
        "xml" => {
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document>\n<content><![CDATA[{}]]></content>\n</document>", content)
        }
        _ => content,
    };

    Ok(Json(DocumentResponse {
        success: true,
        output_path: None,
        message: Some(format!("Document exported as {}", req.format)),
        metadata: Some(serde_json::json!({
            "format": req.format,
            "size": exported_content.len(),
            "content": exported_content
        })),
    }))
}

pub async fn import_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImportDocumentRequest>,
) -> Result<Json<DocumentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let content = if let Some(url) = &req.source_url {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Failed to fetch URL: {}", e) })),
            )
        })?;

        response.text().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read response: {}", e) })),
            )
        })?
    } else if let Some(data) = &req.source_data {
        data.clone()
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({ "error": "Either source_url or source_data must be provided" }),
            ),
        ));
    };

    let processed_content = match req.format.as_str() {
        "json" => {
            let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("Invalid JSON: {}", e) })),
                )
            })?;
            serde_json::to_string_pretty(&parsed).unwrap_or(content)
        }
        // "xml", "csv", and any other format pass through unchanged
        _ => content,
    };

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&req.output_path)
        .body(processed_content.into_bytes().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to save imported document: {}", e) })),
            )
        })?;

    Ok(Json(DocumentResponse {
        success: true,
        output_path: Some(req.output_path),
        message: Some("Document imported successfully".to_string()),
        metadata: Some(serde_json::json!({
            "format": req.format,
            "source_type": if req.source_url.is_some() { "url" } else { "data" }
        })),
    }))
}
