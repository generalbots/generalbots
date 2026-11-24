//! Boot-time KB backfill for Docs (#2499).
//!
//! The in-memory KB index is process-local and starts empty on every server
//! restart, while documents live in Drive. A fresh index therefore makes every
//! previously saved document invisible to chat/search until it is saved again.
//! This module replays persisted documents from Drive into the shared index at
//! boot. It is strictly best effort: listing/reading failures are logged and
//! never propagate into startup.

use crate::state::DocState;
use crate::storage::load_document_from_drive;

/// Drive prefix that holds every Docs namespace (`users/{owner}/docs/...`).
const USERS_PREFIX: &str = "users/";

/// Re-index every persisted document into `state`'s in-memory KB index.
///
/// Returns the number of documents successfully projected. A failed Drive
/// listing or an unreadable document is logged once and skipped — the server
/// must always finish booting with a partial (or empty) index rather than not
/// boot at all.
pub async fn reindex_persisted_documents(state: &DocState) -> usize {
    let objects = match state
        .drive
        .list_objects(&state.bucket_name, Some(USERS_PREFIX))
        .await
    {
        Ok(objects) => objects,
        Err(error) => {
            log::warn!(
                "docs: KB backfill could not list persisted documents in {}: {error}",
                state.bucket_name
            );
            return 0;
        }
    };

    let mut indexed = 0usize;
    let mut failed = 0usize;
    for key in objects {
        let Some((owner, doc_id)) = parse_document_key(&key) else {
            continue;
        };
        match load_document_from_drive(state, owner, doc_id).await {
            Ok(Some(document)) => {
                state.project_to_kb(
                    &document.owner_id,
                    &document.id,
                    &document.title,
                    &document.content,
                );
                indexed += 1;
            }
            Ok(None) => {}
            Err(_) => failed += 1,
        }
    }

    if failed > 0 {
        log::warn!(
            "docs: KB backfill skipped {failed} persisted document(s) in {} that could not be read",
            state.bucket_name
        );
    }
    if indexed > 0 {
        log::info!(
            "docs: KB backfill re-indexed {indexed} persisted document(s) from {}",
            state.bucket_name
        );
    }
    indexed
}

/// Extract `(owner, doc_id)` from a persisted document HTML key.
///
/// Document keys have the shape `users/{owner}/docs/{doc_id}.html`. Sidecars
/// (`.meta.json`, `.docx`, …) and any other object are rejected, so each
/// document is projected exactly once.
fn parse_document_key(key: &str) -> Option<(&str, &str)> {
    let rest = key.strip_prefix(USERS_PREFIX)?;
    let mut segments = rest.split('/');
    let owner = segments.next()?;
    if segments.next()? != "docs" {
        return None;
    }
    let file = segments.next()?;
    if segments.next().is_some() {
        return None;
    }
    let doc_id = file.strip_suffix(".html")?;
    if owner.is_empty() || doc_id.is_empty() {
        return None;
    }
    Some((owner, doc_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use botcore::kb::permissions::UserContext;
    use botcore::shared::analytics::MetricsCollector;
    use botlib::traits::{
        BoxFutureBool, BoxFutureDriveList, BoxFutureOptionDriveMeta, BoxFutureUnit,
        BoxFutureVecDriveObject, BoxFutureVecString, BoxFutureVecU8, DriveListResult,
        DriveRepository,
    };
    use uuid::Uuid;

    const TENANT: &str = "11111111-1111-1111-1111-111111111111";
    const OWNER: &str = "33333333-3333-3333-3333-333333333333";
    const DOC_ID: &str = "doc-backfill-1";
    const HTML: &str = "<h1>Handbook</h1><h2>Termination</h2><p>7.3 Notice</p>\
                         <p>The ZANZIBAR_BACKFILL clause body is here.</p>";

    #[derive(Debug)]
    struct BackfillDrive;

    macro_rules! noop_drive_methods {
        ($($method:ident($($arg:ident : $ty:ty),*) -> $ret:ty => $value:expr;)*) => {
            $(
                fn $method(&self, $($arg: $ty),*) -> $ret {
                    let _ = ($($arg,)*);
                    Box::pin(async { Ok($value) })
                }
            )*
        };
    }

    impl DriveRepository for BackfillDrive {
        noop_drive_methods! {
            put_object(bucket: &str, key: &str, data: Vec<u8>, content_type: Option<&str>) -> BoxFutureUnit => ();
            delete_object(bucket: &str, key: &str) -> BoxFutureUnit => ();
            copy_object(bucket: &str, from_key: &str, to_key: &str) -> BoxFutureUnit => ();
            list_objects_with_metadata(bucket: &str, prefix: Option<&str>) -> BoxFutureVecDriveObject => Vec::new();
            list_common_prefixes(bucket: &str, delimiter: &str) -> BoxFutureVecString => Vec::new();
            list_all_buckets() -> BoxFutureVecString => Vec::new();
            object_exists(bucket: &str, key: &str) -> BoxFutureBool => false;
            get_object_metadata(bucket: &str, key: &str) -> BoxFutureOptionDriveMeta => None;
            create_bucket_if_not_exists(bucket: &str) -> BoxFutureUnit => ();
            delete_objects(bucket: &str, keys: Vec<String>) -> BoxFutureUnit => ();
            head_bucket(bucket: &str) -> BoxFutureBool => false;
            list_objects_v2(bucket: &str, prefix: &str, delimiter: Option<&str>) -> BoxFutureDriveList => DriveListResult { objects: Vec::new(), common_prefixes: Vec::new() };
            upload_file(bucket: &str, key: &str, file_path: &str, content_type: Option<&str>) -> BoxFutureUnit => ();
            download_file(bucket: &str, key: &str, file_path: &str) -> BoxFutureUnit => ();
        }

        fn list_objects(&self, _bucket: &str, prefix: Option<&str>) -> BoxFutureVecString {
            let prefix = prefix.map(str::to_string).unwrap_or_else(String::new);
            Box::pin(async move {
                if prefix == USERS_PREFIX {
                    Ok(vec![
                        format!("users/{OWNER}/docs/{DOC_ID}.html"),
                        format!("users/{OWNER}/docs/{DOC_ID}.meta.json"),
                        format!("users/{OWNER}/docs/{DOC_ID}.docx"),
                        "users/elsewhere/notes.txt".to_string(),
                    ])
                } else {
                    Ok(Vec::new())
                }
            })
        }

        fn get_object(&self, _bucket: &str, key: &str) -> BoxFutureVecU8 {
            let key = key.to_string();
            Box::pin(async move {
                if key == format!("users/{OWNER}/docs/{DOC_ID}.html") {
                    Ok(HTML.as_bytes().to_vec())
                } else if key == format!("users/{OWNER}/docs/{DOC_ID}.meta.json") {
                    Ok(
                        format!(r#"{{"id":"{DOC_ID}","title":"Handbook","version":2}}"#)
                            .into_bytes(),
                    )
                } else {
                    Err("not found".to_string())
                }
            })
        }
    }

    fn state() -> std::sync::Arc<DocState> {
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://unused:unused@127.0.0.1:1/unused",
        );
        let pool = diesel::r2d2::Pool::builder()
            .max_size(1)
            .build_unchecked(manager);
        std::sync::Arc::new(DocState::with_in_memory_kb_index(
            std::sync::Arc::new(pool),
            std::sync::Arc::new(BackfillDrive),
            TENANT.to_string(),
            std::sync::Arc::new(MetricsCollector::default()),
        ))
    }

    #[test]
    fn parse_document_key_accepts_only_html_documents() {
        assert_eq!(
            parse_document_key("users/owner-1/docs/doc-1.html"),
            Some(("owner-1", "doc-1"))
        );
        assert!(parse_document_key("users/owner-1/docs/doc-1.meta.json").is_none());
        assert!(parse_document_key("users/owner-1/notes/doc-1.html").is_none());
        assert!(parse_document_key("users/owner-1/docs/doc-1.html/extra").is_none());
        assert!(parse_document_key("other/owner-1/docs/doc-1.html").is_none());
    }

    #[tokio::test]
    async fn backfill_reindexes_persisted_documents_into_kb() {
        let state = state();
        let owner = UserContext::authenticated(
            Uuid::parse_str(OWNER).unwrap_or_else(|error| {
                log::warn!("docs backfill: invalid owner UUID {OWNER}: {error}");
                Uuid::nil()
            }),
            None,
            Some(Uuid::parse_str(TENANT).unwrap_or_else(|error| {
                log::warn!("docs backfill: invalid tenant UUID {TENANT}: {error}");
                Uuid::nil()
            })),
        );

        // The in-memory index starts empty on a fresh boot.
        assert!(state.search_text("ZANZIBAR_BACKFILL", &owner, 5).is_empty());

        // Only the .html document is projected; sidecars and other objects are
        // ignored, so exactly one document is counted.
        let indexed = reindex_persisted_documents(&state).await;
        assert_eq!(indexed, 1);

        let hits = state.search_text("ZANZIBAR_BACKFILL", &owner, 5);
        assert_eq!(hits.len(), 1, "backfilled document must be searchable");
        assert_eq!(hits[0].source.locator, "7.3");
    }
}
