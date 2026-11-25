use crate::auth::DocUser;
use crate::export_isolation::{html_export_headers, render_html_export, text_export_headers};
use crate::handlers_api::admission::admit;
use crate::state::DocState;
use crate::storage::load_document_from_drive;
use crate::storage_core::strip_html;
use crate::storage_docx_source::DocxFidelity;
use crate::types::ExportQuery;
use crate::utils::html_to_markdown;
use crate::utils_ooxml::html_to_odt_zip;
use crate::utils_pdf::html_to_pdf;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use botcore::WorkClass;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

// Pure DOCX rendering lives in its own file so this handler module stays below
// the 450-line limit (#2452 follow-up). No behaviour moved with it.
#[path = "export_docx.rs"]
mod export_docx;
use export_docx::html_to_docx_full;

// #2452 — the role gate lives beside the handlers but in its own file so the
// handler module stays below the 450-line limit. It reuses the `decide()` ACL
// and the `resource_permissions` role source of the AST read.
#[path = "export_acl.rs"]
mod export_acl;
use export_acl::{acl_denied, denial_code, export_decision};

/// #2459 — emit export duration under a sanitized format token.
async fn record_export(state: &DocState, format: &str, started: Instant) {
    let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
    botcore::shared::analytics::track_office_export(&state.metrics_collector, format, duration_ms)
        .await;
}

pub async fn handle_export_pdf(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    // #2530 — hold the export gate across the renderer; RAII releases it.
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs pdf export admitted: {}", permit.class().as_str());
    let pdf_bytes = html_to_pdf(&doc.content);
    record_export(&state, "pdf", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "pdf",
        Some(client_ip.as_str()),
    );

    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        pdf_bytes,
    ))
}

pub async fn handle_export_docx(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    // #2530 — hold the export gate across the package rebuild; RAII releases it.
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs docx export admitted: {}", permit.class().as_str());
    // #2042 — export honours the preserved package, like save does. Rebuilding
    // from HTML drops styles, headers/footers, tables and images.
    let user_id = user.id().to_string();
    // The richer builder below stays as the last-resort rebuild; the preserving
    // path is preferred because it keeps the original package.
    let (docx_bytes, fidelity) = match crate::storage::docx_bytes_with_fidelity(
        &state,
        &user_id,
        &doc.id,
        &doc.title,
        &doc.content,
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            log::warn!("docs export: preserving path failed: {error}");
            let rebuilt = html_to_docx_full(&doc.title, &doc.content).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::export", None)})),
                    )
                })?;
            (rebuilt, DocxFidelity::Regenerated)
        }
    };

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
    );
    headers.insert(
        "x-gb-docx-fidelity",
        axum::http::HeaderValue::from_static(fidelity.as_str()),
    );
    record_export(&state, "docx", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "docx",
        Some(client_ip.as_str()),
    );
    Ok((headers, docx_bytes))
}

pub async fn handle_export_odt(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    // #2530 — hold the export gate across the ODT builder; RAII releases it.
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs odt export admitted: {}", permit.class().as_str());
    let odt_bytes = html_to_odt_zip(&doc.title, &doc.content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::export", None)}))))?;

    record_export(&state, "odt", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "odt",
        Some(client_ip.as_str()),
    );
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "application/vnd.oasis.opendocument.text",
        )],
        odt_bytes,
    ))
}

async fn load_export_document(
    state: &Arc<DocState>,
    user: &DocUser,
    query: &ExportQuery,
) -> Result<crate::types::Document, (StatusCode, Json<serde_json::Value>)> {
    // #2452 — authorize BEFORE reading or rendering any bytes. Previously the
    // only "denial" was the namespace lookup below, which returned a generic
    // 404 for a foreign caller and never consulted the object role. The gate
    // resolves the verified caller's effective role from the shared
    // `resource_permissions` store and runs the existing `decide()` policy with
    // `Action::Export`; resolution failures fail closed.
    let decision = export_decision(state, user, &query.id);
    if !decision.is_allowed() {
        log::warn!(
            "docs export: ACL denied ({})",
            denial_code(decision.reason())
        );
        return Err(acl_denied(decision.reason()));
    }

    let user_id = user.id();

    match load_document_from_drive(state, user_id, &query.id).await {
        // #2624 — every mounted export format flows through this loader;
        // redact here so a stored secret never leaves in pdf/docx/html/odt.
        Ok(Some(mut d)) => {
            let dlp_user = uuid::Uuid::parse_str(user_id).ok();
            d.content = botsecurity_core::dlp::redact_text(
                &d.content,
                botsecurity_core::dlp::ScanDirection::Outbound,
                dlp_user,
            )
            .await;
            Ok(d)
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::export", None)}),
            ),
        )),
    }
}

pub async fn handle_export_md(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    // #2530 — same gate for every export format, not just the heavy ones.
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs md export admitted: {}", permit.class().as_str());
    let markdown = html_to_markdown(&doc.content);

    // A download, not a page: `attachment` keeps the browser from rendering it
    // on the suite origin (#2124 follow-up).
    record_export(&state, "md", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "md",
        Some(client_ip.as_str()),
    );
    Ok((
        text_export_headers(&doc.title, "md", "text/markdown; charset=utf-8"),
        markdown,
    ))
}

/// Serves the standalone HTML document as a download.
///
/// The title is escaped and the response carries the sandbox policy, so the
/// document can never run as the suite user even if a proxy drops
/// `Content-Disposition` (#2124 follow-up). See [`crate::export_isolation`].
pub async fn handle_export_html(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs html export admitted: {}", permit.class().as_str());
    record_export(&state, "html", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "html",
        Some(client_ip.as_str()),
    );

    Ok((
        html_export_headers(&doc.title),
        render_html_export(&doc.title, &doc.content),
    ))
}

pub async fn handle_export_txt(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(query): Query<ExportQuery>,
    // #2452 — botserver inserts ConnectInfo on both its listeners.
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let started = Instant::now();
    let doc = load_export_document(&state, &user, &query).await?;
    let tenant = user.id().to_string();
    let permit = admit(
        WorkClass::OfficeExport,
        doc.content.len() as u64,
        Some(&tenant),
    )
    .await?;
    log::debug!("docs txt export admitted: {}", permit.class().as_str());
    let plain_text = strip_html(&doc.content);
    record_export(&state, "txt", started).await;
    let client_ip = addr.ip().to_string();
    crate::access_audit::record_export_access_with_ip(
        &state,
        &user,
        &doc.id,
        "txt",
        Some(client_ip.as_str()),
    );

    Ok((
        text_export_headers(&doc.title, "txt", "text/plain; charset=utf-8"),
        plain_text,
    ))
}
