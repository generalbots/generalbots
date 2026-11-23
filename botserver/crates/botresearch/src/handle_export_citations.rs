use super::*;

pub async fn handle_export_citations<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();

    if query.trim().is_empty() {
        return Html(
            "<script>alert('No search results to export. Run a search first.');</script>"
                .to_string(),
        );
    }

    let conn = state.db_pool().clone();
    let scope = bot_scope_clause(&state.resolve_scope(&headers));
    let search_pattern = format!("%{}%", query.to_lowercase());

    let docs = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Vec::new();
            }
        };

        diesel::sql_query(format!(
            "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE (LOWER(file_path) LIKE $1 OR metadata::text ILIKE $1){} ORDER BY file_path ASC LIMIT 20",
            scope
        ))
        .bind::<diesel::sql_types::Text, _>(&search_pattern)
        .load::<KbDocumentRow>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    Html(render_citations_html(&docs))
}

/// Build a JavaScript string literal that is safe to embed in an inline HTML
/// `<script>` element.
///
/// `serde_json::to_string` emits a valid JS/JSON string literal (escaping
/// quotes, backslashes and control characters), but leaves `<`, `>`, `&` and
/// U+2028/U+2029 as raw characters. In an inline script those are dangerous:
/// `</script>` terminates the element and lets DB-sourced text escape into the
/// surrounding HTML document, while U+2028/U+2029 are JavaScript line
/// terminators. Replacing each with a `\uXXXX` escape keeps both parsers from
/// being tricked while leaving the runtime string value unchanged.
fn js_string_literal(s: &str) -> String {
    let literal = match serde_json::to_string(s) {
        Ok(literal) => literal,
        Err(_) => "\"\"".to_string(),
    };
    literal
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Render the citation-export response. Kept separate from the handler so the
/// escaping of DB-sourced fields can be exercised by unit tests.
fn render_citations_html(docs: &[KbDocumentRow]) -> String {
    let mut bibtex = String::new();
    for (i, doc) in docs.iter().enumerate() {
        bibtex.push_str(&format!("@misc{{research_{},\n", i));
        bibtex.push_str(&format!("  title = {{{}}},\n", doc.file_path));
        bibtex.push_str(&format!("  note = {{{}}},\n", doc.collection_name));
        bibtex.push_str("}\n\n");
    }

    let bibtex_js = js_string_literal(&bibtex);
    let count = docs.len();
    format!(
        "<script>navigator.clipboard.writeText({bibtex_js}).then(function(){{alert('BibTeX for {count} results copied to clipboard');}});</script>"
    )
}
pub async fn handle_sources<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let category = params.q.unwrap_or_default();
    let conn = state.db_pool().clone();
    let scope = bot_scope_clause(&state.resolve_scope(&headers));

    let sources: Vec<String> = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        if category.is_empty() || category == "all" {
            diesel::sql_query(format!(
                "SELECT DISTINCT collection_name FROM kb_documents WHERE 1 = 1{} ORDER BY collection_name ASC LIMIT 50",
                scope
            ))
            .load::<CollectionNameRow>(&mut db_conn)
            .map(|rows| rows.into_iter().map(|r| r.name).collect())
            .unwrap_or_default()
        } else {
            diesel::sql_query(format!(
                "SELECT DISTINCT collection_name FROM kb_documents WHERE collection_name ILIKE $1{} ORDER BY collection_name ASC LIMIT 50",
                scope
            ))
            .bind::<diesel::sql_types::Text, _>(&format!("%{category}%"))
            .load::<CollectionNameRow>(&mut db_conn)
            .map(|rows| rows.into_iter().map(|r| r.name).collect())
            .unwrap_or_default()
        }
    })
    .await
    .unwrap_or_default();

    Json(serde_json::json!({ "sources": sources }))
}
pub async fn handle_collections_save<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(scope) = state.resolve_scope(&headers) else {
        return Json(serde_json::json!({ "ok": false, "error": "tenant_scope_unavailable" }));
    };
    let bot_id = scope.bot_id;
    let pool = state.db_pool().clone();
    let collection_id = payload
        .get("collection_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let res = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO research_collection_items (id, bot_id, collection_id, title, content, url, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(collection_id)
        .bind::<diesel::sql_types::Text, _>(&title)
        .bind::<diesel::sql_types::Text, _>(&content)
        .bind::<diesel::sql_types::Text, _>(&url)
        .execute(&mut conn)
        .map_err(|e| e.to_string())
    })
    .await;

    match res {
        Ok(Ok(_)) => Json(serde_json::json!({ "ok": true, "saved": true })),
        Ok(Err(e)) => {
            log::error!("research collection save failed: {e}");
            Json(serde_json::json!({ "ok": false, "error": "save_failed" }))
        }
        Err(e) => {
            log::error!("research collection save join failed: {e}");
            Json(serde_json::json!({ "ok": false, "error": "internal" }))
        }
    }
}
pub async fn handle_share<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let Some(scope) = state.resolve_scope(&headers) else {
        return Json(serde_json::json!({ "ok": false, "error": "tenant_scope_unavailable" }));
    };
    let bot_id = scope.bot_id;
    let pool = state.db_pool().clone();
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Research result")
        .to_string();
    let url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let channel = payload
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("link")
        .to_string();
    let id = uuid::Uuid::new_v4();
    let inserted = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        diesel::sql_query(
            "INSERT INTO research_shares (id, bot_id, title, url, channel, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(&title)
        .bind::<diesel::sql_types::Text, _>(&url)
        .bind::<diesel::sql_types::Text, _>(&channel)
        .execute(&mut conn)
        .map_err(|e| e.to_string())
    })
    .await;

    match inserted {
        Ok(Ok(_)) => {
            let share_url = format!("/suite/research?share={}", id);
            Json(serde_json::json!({ "ok": true, "share_url": share_url }))
        }
        Ok(Err(e)) => {
            log::error!("research share failed: {e}");
            Json(serde_json::json!({ "ok": false, "error": "share_failed" }))
        }
        Err(e) => {
            log::error!("research share join failed: {e}");
            Json(serde_json::json!({ "ok": false, "error": "internal" }))
        }
    }
}
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn malicious_doc(file_path: &str, collection_name: &str) -> KbDocumentRow {
        KbDocumentRow {
            id: "test-id".to_string(),
            file_path: file_path.to_string(),
            collection_name: collection_name.to_string(),
            metadata: None,
        }
    }

    #[test]
    fn closing_script_tag_cannot_break_out() {
        let html =
            render_citations_html(&[malicious_doc("x</script><img onerror=alert(1)>", "notes")]);
        assert!(
            !html.contains("</script><img"),
            "raw breakout sequence leaked: {html}"
        );
        assert!(
            !html[..html.len() - "</script>".len()].contains("</script>"),
            "an inner </script> closed the element early: {html}"
        );
        assert!(html.contains("\\u003c/script\\u003e"));
    }

    #[test]
    fn img_onerror_cannot_break_out() {
        let literal = js_string_literal("<img src=x onerror=alert(1)>");
        assert!(!literal.contains('<'), "raw '<' leaked: {literal}");
        assert!(!literal.contains('>'), "raw '>' leaked: {literal}");
        assert!(literal.contains("\\u003cimg src=x onerror=alert(1)\\u003e"));
    }

    #[test]
    fn quotes_are_valid_js_string_literal() {
        let literal = js_string_literal("say \"hi\" and 'bye' & <tag>");
        assert!(literal.starts_with('"') && literal.ends_with('"'));
        assert!(
            literal.contains("\\\""),
            "quotes were not escaped: {literal}"
        );
        assert!(
            literal.contains("\\u0026"),
            "ampersand not escaped: {literal}"
        );
        assert!(!literal.contains('<') && !literal.contains('>'));
    }

    #[test]
    fn line_separators_are_escaped() {
        let literal = js_string_literal("a\u{2028}b\u{2029}c");
        assert!(
            !literal.contains('\u{2028}'),
            "raw U+2028 leaked: {literal}"
        );
        assert!(
            !literal.contains('\u{2029}'),
            "raw U+2029 leaked: {literal}"
        );
        assert!(literal.contains("\\u2028"));
        assert!(literal.contains("\\u2029"));
    }

    #[test]
    fn full_pipeline_from_db_fields_is_safe() {
        let docs = [
            malicious_doc("../../</script><svg onload=alert(1)>", "col\"&<x>\u{2028}"),
            malicious_doc("normal.txt", "safe"),
        ];
        let html = render_citations_html(&docs);
        let script_body_end = html.rfind("</script>").unwrap_or(html.len());
        let script_body = &html[..script_body_end];
        // The wrapper's own `<script>` opening tag is expected; what matters
        // is that no DB-sourced literal `<`, `>` or `</script>` survives.
        for dangerous in ["</script>", "<svg", "<img"] {
            assert!(
                !script_body.contains(dangerous),
                "DB text broke out of script body via {dangerous}: {html}"
            );
        }
        assert!(
            !script_body.contains("&"),
            "raw ampersand in script body: {html}"
        );
        assert!(!script_body.contains('\u{2028}'));
        assert!(!script_body.contains('\u{2029}'));
    }
}
