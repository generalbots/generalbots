use super::*;

/// #1638 — Drive search across the caller's buckets, matching file names and
/// (bounded) text extracted from the file bodies.
pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    let current_bucket =
        resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid.as_str()), None, &user)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let query = params.query.unwrap_or_default();
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // #1638 — "search everywhere" means every bucket this caller may address,
    // never another tenant's. Without `all_buckets` the old single-bucket
    // behaviour is preserved for callers that ask for it explicitly.
    let mut buckets: Vec<String> = if params.all_buckets.unwrap_or(false) {
        super::cached_quota::allowed_buckets_for(&state, &user)
    } else {
        vec![current_bucket.clone()]
    };
    buckets.retain(|b| !b.trim().is_empty());
    let mut seen_buckets = std::collections::HashSet::new();
    buckets.retain(|bucket| seen_buckets.insert(bucket.clone()));

    let search_content = params.content.unwrap_or(true);
    let mut filtered: Vec<FileListItem> = Vec::new();
    let mut content_scanned = 0usize;

    for bucket in &buckets {
        let keys = match drive.list_objects(bucket, Some(&prefix)).await {
            Ok(keys) => keys,
            Err(e) => {
                warn!("search: listing {bucket} failed: {e}");
                continue;
            }
        };

        let mut sizes: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        if let Ok(objects) = drive.list_objects_with_metadata(bucket, Some(&prefix)).await {
            for object in objects {
                sizes.insert(object.key, object.size);
            }
        }

        // Walk the keys themselves: build_file_list_items collapses nested
        // objects into their top-level folder, so a file inside a folder was
        // invisible to search.
        for key in keys {
            if key.ends_with('/') {
                continue;
            }
            let Some(name) = key.rsplit('/').next().filter(|name| !name.is_empty()) else {
                continue;
            };
            let name_match = name.to_lowercase().contains(&query_lower);
            let mut content_match = false;
            if !name_match && search_content && content_scanned < SEARCH_MAX_CONTENT_FILES {
                content_scanned += 1;
                content_match =
                    object_contains_query(drive.as_ref(), bucket, &key, name, &query_lower).await;
            }
            if !name_match && !content_match {
                continue;
            }
            filtered.push(FileListItem {
                name: name.to_string(),
                path: key.clone(),
                is_dir: false,
                size: sizes.get(&key).copied().unwrap_or(0),
                modified: None,
                is_kb: key.contains(".gbkb"),
                is_public: false,
                bucket: (buckets.len() > 1).then(|| bucket.clone()),
            });
        }
    }

    Ok(Json(filtered))
}

/// Upper bounds for the content scan: a search must not turn into a full
/// bucket download. The newest/biggest documents beyond these limits are
/// still findable by name.
const SEARCH_MAX_CONTENT_FILES: usize = 120;
const SEARCH_MAX_CONTENT_BYTES: usize = 4 * 1024 * 1024;

fn is_text_like(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "csv"
            | "tsv"
            | "json"
            | "jsonl"
            | "html"
            | "htm"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "log"
            | "rs"
            | "js"
            | "ts"
            | "py"
            | "sh"
            | "bas"
            | "sql"
            | "css"
    )
}

#[cfg(feature = "drive")]
fn extract_pdf(path: &std::path::Path) -> String {
    crate::vectordb::extract_pdf_text_sync(path).unwrap_or_default()
}
#[cfg(not(feature = "drive"))]
fn extract_pdf(path: &std::path::Path) -> String {
    let _ = path;
    String::new()
}

#[cfg(feature = "drive")]
fn extract_docx(path: &std::path::Path) -> String {
    crate::vectordb::extract_docx_text_sync(path).unwrap_or_default()
}
#[cfg(not(feature = "drive"))]
fn extract_docx(path: &std::path::Path) -> String {
    let _ = path;
    String::new()
}

#[cfg(feature = "sheet")]
fn extract_xlsx(path: &std::path::Path) -> String {
    crate::vectordb::extract_xlsx_text_sync(path).unwrap_or_default()
}
#[cfg(not(feature = "sheet"))]
fn extract_xlsx(path: &std::path::Path) -> String {
    let _ = path;
    String::new()
}

/// True when the object's body contains the query. Text formats are read
/// directly; PDF/DOCX/XLSX go through the same extractors the indexer uses,
/// via a short-lived temp file. Any failure is a non-match, never an error
/// that aborts the whole search.
async fn object_contains_query(
    drive: &dyn botlib::traits::DriveRepository,
    bucket: &str,
    key: &str,
    name: &str,
    query_lower: &str,
) -> bool {
    let ext = name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    let office = matches!(ext.as_str(), "pdf" | "docx" | "xlsx");
    if !is_text_like(&ext) && !office {
        return false;
    }
    let bytes = match drive.get_object(bucket, key).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("search: reading {bucket}/{key} failed: {e}");
            return false;
        }
    };
    if bytes.is_empty() || bytes.len() > SEARCH_MAX_CONTENT_BYTES {
        return false;
    }
    if is_text_like(&ext) {
        return String::from_utf8_lossy(&bytes)
            .to_lowercase()
            .contains(query_lower);
    }

    let tmp = std::env::temp_dir().join(format!("gb-search-{}.{ext}", uuid::Uuid::new_v4()));
    if tokio::fs::write(&tmp, &bytes).await.is_err() {
        return false;
    }
    let path = tmp.clone();
    let text = tokio::task::spawn_blocking(move || {
        let text = match ext.as_str() {
            "pdf" => extract_pdf(&path),
            "docx" => extract_docx(&path),
            "xlsx" => extract_xlsx(&path),
            _ => String::new(),
        };
        let _ = std::fs::remove_file(&path);
        text
    })
    .await
    .unwrap_or_default();
    text.to_lowercase().contains(query_lower)
}

#[cfg(test)]
mod search_content_tests {
    use super::is_text_like;

    #[test]
    fn text_formats_are_scanned() {
        for ext in ["txt", "md", "csv", "json", "html", "rs", "bas"] {
            assert!(is_text_like(ext), "{ext} should be treated as text");
        }
    }

    #[test]
    fn binary_formats_are_not_read_as_text() {
        for ext in ["png", "jpg", "zip", "mp4", "exe"] {
            assert!(!is_text_like(ext), "{ext} must not be read as text");
        }
    }
}
