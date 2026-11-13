//! GET bridge for the slides shell (#1707).
//!
//! `slides.html` and its modules load every form/modal/panel fragment with
//! `hx-get` (hx-vals become query parameters), while the handlers were
//! registered as POST with a JSON body — each of those calls answered
//! 404/405 and the app rendered as an empty shell. This bridge accepts GET
//! with query parameters, converts them to the `Json` payload the existing
//! handlers expect and dispatches by path. POST callers keep working: the
//! original routes stay registered unchanged.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;

use super::html_escape;
use crate::auth::{SlidesUser, SlidesUnauthorized};
use crate::storage::DriveOps;
use crate::types::Presentation;
use crate::SlidesState;

type Params = HashMap<String, String>;

/// Serves the Slides shell and its assets when the `/suite/slides/*path`
/// wildcard shadows the `/suite` static mount. Only files inside
/// `BOTUI_PATH/slides` are eligible; any path with `.`/`..`/empty segments or
/// a non-asset extension is rejected so a crafted path can never read outside
/// the UI directory (#1715).
async fn serve_ui_asset(path: &str) -> Option<Response> {
    let last = path.rsplit('/').next().unwrap_or(path);
    let (_, ext) = last.rsplit_once('.')?;
    let ext = ext.to_ascii_lowercase();
    if !matches!(
        ext.as_str(),
        "html" | "css" | "js" | "mjs" | "json" | "svg" | "png" | "jpg" | "jpeg" | "gif"
            | "webp" | "ico" | "woff" | "woff2" | "ttf" | "map" | "txt"
    ) {
        return None;
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    let ui = std::env::var("BOTUI_PATH").ok()?;
    let full = std::path::Path::new(&ui).join("slides").join(path);
    let bytes = tokio::fs::read(&full).await.ok()?;
    let mime = match ext.as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "text/plain; charset=utf-8",
    };
    Some(([(header::CONTENT_TYPE, mime)], bytes).into_response())
}

fn to_json(mut params: Params) -> Json<serde_json::Value> {
    // The shell toolbar and modals predate the typed API: they address the
    // deck as `id` and the slide as `slide`, while the fragment handlers read
    // `presentation_id` and `slide_index`. Alias both here so a toolbar action
    // targets the open deck instead of an empty id.
    if let Some(value) = params.get("id").cloned() {
        params.entry("presentation_id".to_string()).or_insert(value);
    }
    if let Some(value) = params.get("slide").cloned() {
        if value.chars().all(|character| character.is_ascii_digit()) {
            params.entry("slide_index".to_string()).or_insert(value);
        }
    }
    Json(serde_json::Value::Object(
        params
            .into_iter()
            .map(|(key, value)| (key, serde_json::Value::String(value)))
            .collect(),
    ))
}

/// The deck an empty session starts from. `id` must be a real, unique deck
/// id (never the literal `current`) so an unsaved blank deck cannot be
/// persisted under a shared sentinel and overwrite another deck.
pub(crate) fn blank_presentation(owner_id: &str, id: &str) -> Presentation {
    let theme = crate::utils::create_default_theme();
    Presentation {
        id: id.to_string(),
        name: "Untitled Presentation".to_string(),
        owner_id: owner_id.to_string(),
        slides: vec![crate::utils::create_blank_slide(&theme)],
        theme,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        source_bucket: None,
        source_path: None,
    }
}

/// Resolves the deck for a fragment load.
///
/// - empty / `new`: no id was asked for → a fresh blank deck with a new id.
/// - `current`: legacy sentinel, never a real stored deck → treat as "no id".
/// - `transient_new && id`: a client-minted id for an unsaved blank deck →
///   render a blank deck with that exact id (not persisted) so parallel
///   fragment loads share one identity.
/// - otherwise: a requested id must exist; a miss is `Err("deck not found")`
///   so a missing/deleted/foreign id never silently becomes a blank deck.
pub(super) async fn load_or_blank<D: DriveOps>(
    state: &Arc<SlidesState<D>>,
    user_id: &str,
    id: &str,
    transient_new: bool,
) -> Result<Presentation, String> {
    let requested = id.trim();
    if requested.is_empty() || requested == "new" || requested == "current" {
        return Ok(blank_presentation(user_id, &uuid::Uuid::new_v4().to_string()));
    }
    match state.drive.as_ref() {
        Some(drive) => match crate::storage::load_presentation_by_id(drive, user_id, requested).await {
            Ok(presentation) => Ok(presentation),
            Err(_) if transient_new => Ok(blank_presentation(user_id, requested)),
            Err(_) => Err("Deck not found".to_string()),
        },
        None if transient_new => Ok(blank_presentation(user_id, requested)),
        None => Err("Deck not found".to_string()),
    }
}

fn thumbnails_html(pres: &Presentation) -> Html<String> {
    let mut html = String::from(
        r#"<div style="display:flex;flex-direction:column;gap:8px;padding:8px;">"#,
    );
    if pres.slides.is_empty() {
        html.push_str(&super::empty_fragment("Nenhum slide. Adicione um slide para começar."));
    }
    let total = pres.slides.len();
    for (index, slide) in pres.slides.iter().enumerate() {
        let delete_button = if total > 1 {
            format!(
                r##"<button type="button" title="Delete slide" onclick="event.stopPropagation()" hx-post="/api/slides/slide/delete" hx-vals='{{"presentation_id":"{id}","slide_index":{index}}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) refreshSlideView()" style="background:#7f1d1d;color:#fecaca;border:1px solid #991b1b;border-radius:3px;cursor:pointer;font-size:11px;line-height:1;padding:2px 4px;">×</button>"##,
                id = html_escape(&pres.id),
                index = index
            )
        } else {
            String::new()
        };
        html.push_str(&format!(
            r#"<div class="sl-thumb" data-index="{index}" data-slide="{index}" style="position:relative;background:#0f172a;border:1px solid #334155;border-radius:6px;padding:6px;cursor:pointer;">
<div style="position:absolute;top:4px;right:4px;display:flex;gap:2px;z-index:2;">
<button type="button" title="Duplicate slide" onclick="event.stopPropagation()" hx-post="/api/slides/slide/duplicate" hx-vals='{{"presentation_id":"{id}","slide_index":{index}}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) refreshSlideView()" style="background:#1e3a8a;color:#bfdbfe;border:1px solid #1d4ed8;border-radius:3px;cursor:pointer;font-size:11px;line-height:1;padding:2px 4px;">⧉</button>
{delete}
</div>
<div style="font-size:10px;color:#94a3b8;margin-bottom:4px;">Slide {n}</div>
<div style="background:#fff;border-radius:4px;overflow:hidden;aspect-ratio:16/9;">{preview}</div>
</div>"#,
            index = index,
            n = index + 1,
            preview = super::sidebars::render_slide(slide),
            id = html_escape(&pres.id),
            delete = delete_button
        ));
    }
    html.push_str("</div>");
    Html(html)
}

/// GET /suite/slides/*path — see module docs.
pub async fn get_bridge<D: DriveOps + Send + Sync + 'static>(
    State(state): State<Arc<SlidesState<D>>>,
    user: SlidesUser,
    Path(path): Path<String>,
    Query(params): Query<Params>,
) -> Result<Response, SlidesUnauthorized> {
    // The wildcard route takes precedence over the /suite static mount, so the
    // app shell and its assets would otherwise be answered with a fragment
    // ("route not found") instead of the file (#1715).
    if let Some(asset) = serve_ui_asset(&path).await {
        return Ok(asset);
    }
    let user_id = user.owner_id();
    let query = params.clone();
    let json = to_json(params);
    let transient_new = query
        .get("new")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let html = match path.as_str() {
        "fragments/presentation-view" => {
            let id = query.get("id").cloned().unwrap_or_default();
            let requested_slide = query
                .get("slide_index")
                .or_else(|| query.get("slide"))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            match load_or_blank(&state, user_id, &id, transient_new).await {
                Ok(pres) => Html(super::sidebars::render_presentation_view(&pres, requested_slide)),
                Err(message) => Html(super::err_fragment(&message)),
            }
        }
        "fragments/slide-thumbnails" => {
            let id = query.get("id").cloned().unwrap_or_default();
            match load_or_blank(&state, user_id, &id, transient_new).await {
                Ok(pres) => thumbnails_html(&pres),
                Err(message) => Html(super::err_fragment(&message)),
            }
        }
        "fragments/presentation-list" => {
            let items = match state.drive.as_ref() {
                Some(drive) => {
                    crate::storage::list_presentations_from_drive(drive, user_id)
                    .await
                    .unwrap_or_default()
                }
                None => Vec::new(),
            };
            super::sidebars::handle_presentation_list_sidebar(Json(items)).await
        }
        // "+ Nova": mint a fresh unsaved deck with a unique id.
        "fragments/new" => {
            let deck = blank_presentation(user_id, &uuid::Uuid::new_v4().to_string());
            super::sidebars::handle_presentation_view(Json(deck)).await
        }
        "fragments/themes" => super::themes::handle_theme_picker(json).await,
        "fragments/sorter" => {
            // The sorter renders the deck's slides, so it needs the same
            // verified owner as the rest of the bridge.
            super::panels::handle_slide_sorter::<D>(State(state.clone()), user.clone(), json)
                .await?
        }
        "fragments/presenter" => super::panels::handle_presenter_panel(json).await,
        "modals/new-slide" => super::modals::handle_new_slide_modal(json).await,
        "forms/add-text" => super::toolbar::handle_add_text_form(json).await,
        "forms/add-image" => super::toolbar::handle_add_image_form(json).await,
        "forms/add-shape" => super::toolbar::handle_add_shape_form(json).await,
        "forms/add-chart" => super::toolbar::handle_add_chart_form(json).await,
        "forms/add-table" => super::toolbar::handle_add_table_form(json).await,
        "forms/notes" => super::toolbar::handle_notes_form(json).await,
        "forms/background" => super::toolbar::handle_background_form(json).await,
        "forms/transition" => super::toolbar::handle_transition_form(json).await,
        "forms/animation" => super::toolbar::handle_animation_form(json).await,
        "forms/master" => super::toolbar::handle_master_form(json).await,
        "forms/export" => super::toolbar::handle_export_form(json).await,
        _ => Html(super::err_fragment(&format!("route not found: {path}"))),
    };
    Ok(html.into_response())
}
