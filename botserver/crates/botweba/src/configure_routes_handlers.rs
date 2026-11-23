use super::*;

pub(super) async fn list_apps(
    State(state): State<Arc<WebaState>>,
    Query(query): Query<ListQuery>,
) -> Json<Vec<WebApp>> {
    let apps = state.apps.read().await;
    let mut result: Vec<WebApp> = apps.values().cloned().collect();

    if let Some(status) = query.status {
        result.retain(|app| {
            matches!(
                (&app.status, status.as_str()),
                (WebAppStatus::Draft, "draft")
                    | (WebAppStatus::Published, "published")
                    | (WebAppStatus::Archived, "archived")
            )
        });
    }

    result.sort_by_key(|a| std::cmp::Reverse(a.updated_at));

    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let result: Vec<WebApp> = result.into_iter().skip(offset).take(limit).collect();

    Json(result)
}

pub(super) async fn create_app(
    State(state): State<Arc<WebaState>>,
    Json(req): Json<CreateAppRequest>,
) -> Json<WebApp> {
    let now = chrono::Utc::now();
    let id = Uuid::new_v4();
    let slug = slugify(&req.name);

    let app = WebApp {
        id,
        name: req.name,
        slug,
        description: req.description,
        template: req.template.unwrap_or_default(),
        status: WebAppStatus::Draft,
        config: WebAppConfig::default(),
        created_at: now,
        updated_at: now,
    };

    state.apps.write().await.insert(id, app.clone());
    state.persist().await;

    Json(app)
}

pub(super) async fn get_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebApp>, StatusCode> {
    state
        .apps
        .read()
        .await
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub(super) async fn update_app(
    State(state): State<Arc<WebaState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAppRequest>,
) -> Result<Json<WebApp>, StatusCode> {
    // Flipping an app to `published` through the update endpoint must not be a
    // way around the publish gate (#2124).
    if req
        .status
        .as_ref()
        .is_some_and(|status| matches!(status, WebAppStatus::Published))
    {
        authorize_publisher(&user)?;
    }

    let mut apps = state.apps.write().await;
    let app = apps.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = req.name {
        app.name.clone_from(&name);
        app.slug = slugify(&name);
    }
    if let Some(description) = req.description {
        app.description = Some(description);
    }
    if let Some(status) = req.status {
        app.status = status;
    }
    if let Some(config) = req.config {
        app.config = config;
    }
    app.updated_at = chrono::Utc::now();
    let result = app.clone();
    drop(apps);
    state.persist().await;

    Ok(Json(result))
}

pub(super) async fn delete_app(
    State(state): State<Arc<WebaState>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let mut apps = state.apps.write().await;
    let mut pages = state.pages.write().await;
    pages.retain(|_, page| page.app_id != id);
    let removed = apps.remove(&id).is_some();
    drop(pages);
    drop(apps);
    state.published.write().await.remove(&id);
    if removed {
        state.persist().await;
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub(super) async fn list_pages(
    State(state): State<Arc<WebaState>>,
    Path(app_id): Path<Uuid>,
) -> Json<Vec<WebAppPage>> {
    let pages = state.pages.read().await;
    let result: Vec<WebAppPage> = pages
        .values()
        .filter(|p| p.app_id == app_id)
        .cloned()
        .collect();
    Json(result)
}

pub(super) async fn create_page(
    State(state): State<Arc<WebaState>>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<WebAppPage>, StatusCode> {
    let apps = state.apps.read().await;
    if !apps.contains_key(&app_id) {
        return Err(StatusCode::NOT_FOUND);
    }
    drop(apps);

    let now = chrono::Utc::now();
    let id = Uuid::new_v4();

    let page = WebAppPage {
        id,
        app_id,
        path: req.path,
        title: req.title,
        content: req.content,
        layout: req.layout,
        is_index: req.is_index,
        created_at: now,
        updated_at: now,
    };

    state.pages.write().await.insert(id, page.clone());
    state.persist().await;

    Ok(Json(page))
}

pub(super) async fn get_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WebAppPage>, StatusCode> {
    state
        .pages
        .read()
        .await
        .get(&page_id)
        .filter(|p| p.app_id == app_id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub(super) async fn update_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreatePageRequest>,
) -> Result<Json<WebAppPage>, StatusCode> {
    let mut pages = state.pages.write().await;
    let page = pages
        .get_mut(&page_id)
        .filter(|p| p.app_id == app_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    page.path = req.path;
    page.title = req.title;
    page.content = req.content;
    page.layout = req.layout;
    page.is_index = req.is_index;
    page.updated_at = chrono::Utc::now();
    let result = page.clone();
    drop(pages);
    state.persist().await;

    Ok(Json(result))
}

pub(super) async fn delete_page(
    State(state): State<Arc<WebaState>>,
    Path((app_id, page_id)): Path<(Uuid, Uuid)>,
) -> StatusCode {
    let mut pages = state.pages.write().await;
    let exists = pages
        .get(&page_id)
        .is_some_and(|page| page.app_id == app_id);
    if exists {
        pages.remove(&page_id);
    }
    drop(pages);
    if exists {
        state.persist().await;
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub(super) async fn publish_app(
    State(state): State<Arc<WebaState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<WebApp>, StatusCode> {
    authorize_publisher(&user)?;
    state.publish(id).await.ok_or(StatusCode::NOT_FOUND)?;
    let app = state
        .apps
        .read()
        .await
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(app))
}

pub(super) async fn unpublish_app(
    State(state): State<Arc<WebaState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    authorize_publisher(&user)?;
    if state.unpublish(id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub(super) async fn submit_form(
    State(state): State<Arc<WebaState>>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Json(request): Json<FormSubmitRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    submit_form_inner(state, slug, headers, request).await
}

/// Plain-HTML form endpoint: the published page CSP sets `connect-src 'none'`,
/// so a real form posts here without any JavaScript. Errors keep the JSON
/// contract (the browser shows them on the redirect target).
pub(super) async fn submit_form_html(
    State(state): State<Arc<WebaState>>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    axum::Form(request): axum::Form<FormSubmitRequest>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    drop(submit_form_inner(state, slug.clone(), headers, request).await?);
    Ok((
        StatusCode::SEE_OTHER,
        [(
            axum::http::header::LOCATION,
            format!("/render/{slug}?submitted=1"),
        )],
    )
        .into_response())
}

async fn submit_form_inner(
    state: Arc<WebaState>,
    slug: String,
    headers: HeaderMap,
    request: FormSubmitRequest,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Some(reason) = forms::validate(&request) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": reason })),
        ));
    }
    // Honeypot: silently accept so a bot learns nothing.
    if request.website.as_ref().is_some_and(|value| !value.trim().is_empty()) {
        return Ok(Json(serde_json::json!({ "status": "accepted" })));
    }

    let caller = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|value| value.to_str().ok()))
        .unwrap_or("local");
    if !state.rate_limiter.allow(&format!("{slug}:{caller}")) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({ "error": "rate_limited" })),
        ));
    }

    let site_id = state
        .live_site(&slug)
        .await
        .map(|site| site.app_id)
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "site_not_found" })),
        ))?;
    // #2888 — the tenant that owns this site/form is the only branch the lead
    // may be written to. Any verified claim from the visitor must agree with
    // it; a nil owning branch is a misconfigured site, never a valid tenant.
    let site_branch = match super::resolve_form_branch(
        state.branch_id,
        botsecurity_core::scope::branch_from_claim(&headers),
    ) {
        Ok(branch) => branch,
        Err(super::FormTenantError::UnboundSite) => {
            return Err((
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "error": "site_not_bound" })),
            ));
        }
        Err(super::FormTenantError::BranchMismatch) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "tenant_mismatch" })),
            ));
        }
    };
    let email = forms::normalize_email(&request.email);
    let existing = state
        .submissions
        .read()
        .await
        .iter()
        .find(|item| item.slug == slug && item.email == email)
        .cloned();
    let submission = FormSubmission {
        id: existing.as_ref().map_or_else(Uuid::new_v4, |item| item.id),
        site_id,
        slug: slug.clone(),
        email,
        name: request.name.clone(),
        message: request.message.clone(),
        fields: request.fields.clone(),
        source: format!("site:{slug}"),
        created_at: existing.as_ref().map_or_else(chrono::Utc::now, |item| item.created_at),
        updated_at: chrono::Utc::now(),
    };
    let crm_submission = submission.clone();
    state.record_submission(submission).await;

    if let Err(error) = forms::sync_crm(site_branch, &slug, &crm_submission).await {
        let sanitized = log_and_sanitize_json(&error, "weba::configure_routes::submit_form", None);
        log::error!("WEBA CRM sync: {sanitized}");
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": "crm_sync_failed" })),
        ));
    }

    Ok(Json(serde_json::json!({ "status": "accepted" })))
}
