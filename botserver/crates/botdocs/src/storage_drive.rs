use chrono::{DateTime, Utc};

use crate::state::DocState;
use crate::storage_core::{
    cache_document_bytes, count_words, get_cached_document_bytes,
    get_user_docs_path, html_to_paragraphs, remove_from_cache,
};
use crate::storage_docx::convert_html_to_docx;
use crate::types::{Document, DocumentMetadata};

pub async fn save_document_as_docx(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<Vec<u8>, String> {
    let docx_bytes = if let Some(original_bytes) = get_cached_document_bytes(doc_id).await {
        let paragraphs = html_to_paragraphs(content);
        crate::ooxml::update_docx_text(&original_bytes, &paragraphs)
            .unwrap_or_else(|_| convert_html_to_docx(title, content).unwrap_or_default())
    } else {
        convert_html_to_docx(title, content)?
    };

    let base_path = get_user_docs_path(user_identifier);
    let docx_path = format!("{base_path}/{doc_id}.docx");

    state
        .drive
        .put_object(
            &state.bucket_name,
            &docx_path,
            docx_bytes.clone(),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        )
        .await
        .map_err(|e| format!("Failed to save DOCX: {e}"))?;

    cache_document_bytes(doc_id, docx_bytes.clone()).await;

    Ok(docx_bytes)
}

/// Document fields persisted beside the HTML body. The editor's title+content
/// autosave must never erase them, so every write merges the previous
/// sidecar and a full `Document` save overlays its own values.
const DOC_META_EXTRAS: [&str; 8] = [
    "collaborators",
    "track_changes",
    "track_changes_enabled",
    "comments",
    "footnotes",
    "endnotes",
    "styles",
    "toc",
];

async fn read_doc_meta(state: &DocState, meta_path: &str) -> serde_json::Value {
    match state
        .drive
        .get_object(&state.bucket_name, meta_path)
        .await
    {
        Ok(bytes) => String::from_utf8(bytes)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(serde_json::Value::Null),
        Err(_) => serde_json::Value::Null,
    }
}

fn meta_field<T: serde::de::DeserializeOwned>(meta: &serde_json::Value, key: &str) -> Option<T> {
    meta.get(key)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

pub async fn save_document_to_drive(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
) -> Result<String, String> {
    save_document_with_extras(state, user_identifier, doc_id, title, content, None).await
}

async fn save_document_with_extras(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
    title: &str,
    content: &str,
    document: Option<&Document>,
) -> Result<String, String> {
    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{base_path}/{doc_id}.html");
    let meta_path = format!("{base_path}/{doc_id}.meta.json");

    state
        .drive
        .put_object(
            &state.bucket_name,
            &doc_path,
            content.as_bytes().to_vec(),
            Some("text/html"),
        )
        .await
        .map_err(|e| format!("Failed to save document: {e}"))?;

    // Read the previous sidecar first: `created_at` and the structured
    // collections belong to the document, not to the latest keystroke.
    let existing = read_doc_meta(state, &meta_path).await;
    let created_at = existing
        .get("created_at")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let version = existing
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        .saturating_add(1);

    let mut metadata = serde_json::json!({
        "id": doc_id,
        "title": title,
        "created_at": created_at,
        "updated_at": Utc::now().to_rfc3339(),
        "word_count": count_words(content),
        "version": version,
    });

    let doc_value = document.and_then(|doc| serde_json::to_value(doc).ok());
    for key in DOC_META_EXTRAS {
        if let Some(value) = doc_value
            .as_ref()
            .and_then(|value| value.get(key))
        {
            metadata[key] = value.clone();
            continue;
        }
        if let Some(value) = existing.get(key) {
            metadata[key] = value.clone();
        }
    }

    state
        .drive
        .put_object(
            &state.bucket_name,
            &meta_path,
            metadata.to_string().into_bytes(),
            Some("application/json"),
        )
        .await
        .map_err(|e| format!("Failed to save metadata: {e}"))?;

    Ok(doc_path)
}

pub async fn save_document(
    state: &DocState,
    user_identifier: &str,
    doc: &Document,
) -> Result<String, String> {
    save_document_with_extras(
        state,
        user_identifier,
        &doc.id,
        &doc.title,
        &doc.content,
        Some(doc),
    )
    .await
}

pub async fn load_document_from_drive(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
) -> Result<Option<Document>, String> {
    let base_path = get_user_docs_path(user_identifier);
    let doc_path = format!("{base_path}/{doc_id}.html");
    let meta_path = format!("{base_path}/{doc_id}.meta.json");

    let content = match state
        .drive
        .get_object(&state.bucket_name, &doc_path)
        .await
    {
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| e.to_string())?,
        Err(_) => return Ok(None),
    };

    let meta = read_doc_meta(state, &meta_path).await;
    let title = meta
        .get("title")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Untitled")
        .to_string();
    let created_at = meta
        .get("created_at")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let updated_at = meta
        .get("updated_at")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    Ok(Some(Document {
        id: doc_id.to_string(),
        title,
        content,
        owner_id: user_identifier.to_string(),
        storage_path: doc_path,
        created_at,
        updated_at,
        collaborators: meta_field(&meta, "collaborators").unwrap_or_default(),
        version: meta
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1),
        track_changes: meta_field(&meta, "track_changes"),
        comments: meta_field(&meta, "comments"),
        footnotes: meta_field(&meta, "footnotes"),
        endnotes: meta_field(&meta, "endnotes"),
        styles: meta_field(&meta, "styles"),
        toc: meta_field(&meta, "toc"),
        track_changes_enabled: meta
            .get("track_changes_enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    }))
}

pub async fn list_documents_from_drive(
    state: &DocState,
    user_identifier: &str,
) -> Result<Vec<DocumentMetadata>, String> {
    let base_path = get_user_docs_path(user_identifier);
    let prefix = format!("{base_path}/");
    let mut documents = Vec::new();

    let objects = state
        .drive
        .list_objects(&state.bucket_name, Some(&prefix))
        .await
        .map_err(|e| format!("Failed to list documents: {e}"))?;

    for key in &objects {
        if key.ends_with(".meta.json") {
            if let Ok(bytes) = state
                .drive
                .get_object(&state.bucket_name, key)
                .await
            {
                if let Ok(meta_str) = String::from_utf8(bytes) {
                    if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_str) {
                        let doc_meta = DocumentMetadata {
                            id: meta["id"].as_str().unwrap_or_default().to_string(),
                            title: meta["title"]
                                .as_str()
                                .unwrap_or("Untitled")
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
                            storage_type: "html".to_string(),
                        };
                        documents.push(doc_meta);
                    }
                }
            }
        }
    }

    documents.sort_by_key(|a| std::cmp::Reverse(a.updated_at));
    Ok(documents)
}

pub async fn delete_document_from_drive(
    state: &DocState,
    user_identifier: &str,
    doc_id: &str,
) -> Result<(), String> {
    let base_path = get_user_docs_path(user_identifier);

    for ext in &[".html", ".docx", ".meta.json"] {
        let path = format!("{base_path}/{doc_id}{ext}");
        let _ = state
            .drive
            .delete_object(&state.bucket_name, &path)
            .await;
    }

    remove_from_cache(doc_id).await;

    Ok(())
}
