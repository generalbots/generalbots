use aws_sdk_s3::primitives::ByteStream;
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::core::shared::state::AppState;

use super::models::{Document, DocumentMetadata};

fn get_user_papers_path(user_identifier: &str) -> String {
    let safe_id = user_identifier
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .to_lowercase();
    format!("users/{}/papers", safe_id)
}

pub async fn save_document_to_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
    is_named: bool,
) -> Result<String, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);
    let storage_type = if is_named { "named" } else { "current" };

    let (doc_path, metadata_path) = if is_named {
        let safe_title = title
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            .to_lowercase()
            .chars()
            .take(50)
            .collect::<String>();
        (
            format!("{}/{}/{}/document.md", base_path, storage_type, safe_title),
            Some(format!(
                "{}/{}/{}/metadata.json",
                base_path, storage_type, safe_title
            )),
        )
    } else {
        (
            format!("{}/{}/{}.md", base_path, storage_type, doc_id),
            None,
        )
    };

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&doc_path)
        .body(ByteStream::from(content.as_bytes().to_vec()))
        .content_type("text/markdown")
        .send()
        .await
        .map_err(|e| format!("Failed to save document: {}", e))?;

    if let Some(meta_path) = metadata_path {
        let metadata = serde_json::json!({
            "id": doc_id,
            "title": title,
            "created_at": Utc::now().to_rfc3339(),
            "updated_at": Utc::now().to_rfc3339(),
            "word_count": content.split_whitespace().count()
        });

        s3_client
            .put_object()
            .bucket(&state.bucket_name)
            .key(&meta_path)
            .body(ByteStream::from(metadata.to_string().into_bytes()))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to save metadata: {}", e))?;
    }

    Ok(doc_path)
}

pub async fn load_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<Option<Document>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);

    let current_path = format!("{}/current/{}.md", base_path, doc_id);

    if let Ok(result) = s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&current_path)
        .send()
        .await
    {
        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| e.to_string())?
            .into_bytes();
        let content = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;

        let title = content
            .lines()
            .next()
            .map(|l| l.trim_start_matches('#').trim())
            .unwrap_or("Untitled")
            .to_string();

        return Ok(Some(Document {
            id: doc_id.to_string(),
            title,
            content,
            owner_id: user_identifier.to_string(),
            storage_path: current_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }));
    }

    Ok(None)
}

pub async fn list_documents_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
) -> Result<Vec<DocumentMetadata>, String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);
    let mut documents = Vec::new();

    let current_prefix = format!("{}/current/", base_path);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&current_prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                if key.to_lowercase().ends_with(".md") {
                    let id = key
                        .trim_start_matches(&current_prefix)
                        .trim_end_matches(".md")
                        .to_string();

                    documents.push(DocumentMetadata {
                        id: id.clone(),
                        title: format!("Untitled ({})", &id[..8.min(id.len())]),
                        owner_id: user_identifier.to_string(),
                        created_at: Utc::now(),
                        updated_at: obj
                            .last_modified()
                            .map(|t| {
                                DateTime::from_timestamp(t.secs(), t.subsec_nanos())
                                    .unwrap_or_else(Utc::now)
                            })
                            .unwrap_or_else(Utc::now),
                        word_count: 0,
                        storage_type: "current".to_string(),
                    });
                }
            }
        }
    }

    let named_prefix = format!("{}/named/", base_path);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&named_prefix)
        .delimiter("/")
        .send()
        .await
    {
        for prefix in result.common_prefixes() {
            if let Some(folder) = prefix.prefix() {
                let folder_name = folder
                    .trim_start_matches(&named_prefix)
                    .trim_end_matches('/');

                let meta_key = format!("{}metadata.json", folder);
                if let Ok(meta_result) = s3_client
                    .get_object()
                    .bucket(&state.bucket_name)
                    .key(&meta_key)
                    .send()
                    .await
                {
                    if let Ok(bytes) = meta_result.body.collect().await {
                        if let Ok(meta_str) = String::from_utf8(bytes.into_bytes().to_vec()) {
                            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                                documents.push(DocumentMetadata {
                                    id: meta["id"].as_str().unwrap_or(folder_name).to_string(),
                                    title: meta["title"]
                                        .as_str()
                                        .unwrap_or(folder_name)
                                        .to_string(),
                                    owner_id: user_identifier.to_string(),
                                    created_at: meta["created_at"]
                                        .as_str()
                                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                        .map(|d| d.with_timezone(&Utc))
                                        .unwrap_or_else(Utc::now),
                                    updated_at: meta["updated_at"]
                                        .as_str()
                                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                        .map(|d| d.with_timezone(&Utc))
                                        .unwrap_or_else(Utc::now),
                                    word_count: meta["word_count"].as_u64().unwrap_or(0) as usize,
                                    storage_type: "named".to_string(),
                                });
                                continue;
                            }
                        }
                    }
                }

                documents.push(DocumentMetadata {
                    id: folder_name.to_string(),
                    title: folder_name.to_string(),
                    owner_id: user_identifier.to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    word_count: 0,
                    storage_type: "named".to_string(),
                });
            }
        }
    }

    documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(documents)
}

pub async fn delete_document_from_drive(
    state: &Arc<AppState>,
    user_identifier: &str,
    doc_id: &str,
) -> Result<(), String> {
    let s3_client = state.drive.as_ref().ok_or("S3 service not available")?;

    let base_path = get_user_papers_path(user_identifier);

    let current_path = format!("{}/current/{}.md", base_path, doc_id);
    let _ = s3_client
        .delete_object()
        .bucket(&state.bucket_name)
        .key(&current_path)
        .send()
        .await;

    let named_prefix = format!("{}/named/{}/", base_path, doc_id);
    if let Ok(result) = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&named_prefix)
        .send()
        .await
    {
        for obj in result.contents() {
            if let Some(key) = obj.key() {
                let _ = s3_client
                    .delete_object()
                    .bucket(&state.bucket_name)
                    .key(key)
                    .send()
                    .await;
            }
        }
    }

    Ok(())
}
