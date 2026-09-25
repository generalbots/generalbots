//! `publish::site` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Published-app site domain: `{appname}.{published_domain()}` (#1261).
/// Honors `GB_PLATFORM_DOMAIN` (e.g. `generalbots.org`) so published vibe
/// apps sit on the same wildcard zone as bots; falls back to `SITE_DOMAIN`
/// then `gb.solutions` for legacy self-hosted deployments.
pub fn published_domain() -> String {
    std::env::var("GB_PLATFORM_DOMAIN")
        .ok()
        .filter(|d| !d.trim().is_empty())
        .or_else(|| std::env::var("SITE_DOMAIN").ok().filter(|d| !d.trim().is_empty()))
        .unwrap_or_else(|| "gb.solutions".to_string())
}

/// #1180 — public deliverable URLs: `https://host/r/{slug}` 302-redirects to
/// the project's published route (`{slug}.{domain}` or a bound custom
/// domain), so Vibe artifacts are shareable without any auth.
pub fn publish_router(pool: DbPool) -> Router {
    Router::new()
        .route("/r/:slug", get(resolve_slug).layer(Extension(pool)))
}

#[derive(diesel::QueryableByName)]
pub(crate) struct SlugRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) domain: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(crate) access: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub(crate) verified: bool,
}

pub(crate) async fn resolve_slug(
    Extension(pool): Extension<DbPool>,
    AxumPath(slug): AxumPath<String>,
) -> axum::response::Response {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Publish /r/ resolve: db pool {e}");
            return response_with_headers(
                StatusCode::INTERNAL_SERVER_ERROR,
                &[],
                "internal error",
            );
        }
    };
    let candidates = [
        format!("{slug}.{}", published_domain()),
        slug.clone(),
    ];
    let mut found: Option<(String, String)> = None;
    for domain in candidates {
        let row = diesel::sql_query(
            "SELECT domain, access, verified FROM project_domains WHERE domain = $1 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(&domain)
        .get_result::<SlugRow>(&mut conn);
        match row {
            Ok(r) if r.access != "private" && r.verified => {
                found = Some((r.domain, r.access));
                break;
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    match found {
        Some((domain, _access)) => {
            let url = format!("https://{domain}");
            response_with_headers(
                StatusCode::FOUND,
                &[
                    (header::LOCATION, url),
                    (header::CACHE_CONTROL, "no-store".to_string()),
                ],
                "redirecting",
            )
        }
        None => response_with_headers(StatusCode::NOT_FOUND, &[], "artifact not found"),
    }
}

/// Builds a response with headers, keeping the error path panic-free:
/// builder failures (invalid header name/value) are logged and fall back to
/// a bare 500 response.
pub(crate) fn response_with_headers(
    status: StatusCode,
    headers: &[(header::HeaderName, String)],
    body: &str,
) -> axum::response::Response {
    let mut builder = axum::response::Response::builder().status(status);
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    match builder.body(axum::body::Body::from(body.to_string())) {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Publish response build failed: {e}");
            axum::response::Response::new(axum::body::Body::from("internal error"))
        }
    }
}

/// Server-side stamp that authorizes writing a site project's PUBLIC slug
/// (`{slug}.{domain}`). Only the deploy pipeline and the admin ops paths set
/// it; an agent tool call can never supply it (the schema does not expose the
/// field and the server overwrites it), so an in-flight change always lands on
/// the project's `-test` twin instead of the live site.
pub const PUBLISH_PRODUCTION_STAMP: &str = "_deploy_approved";

/// `true` when the caller was sanctioned by the deploy pipeline to write the
/// public slug.
pub(crate) fn production_approved(args: &Value) -> bool {
    args.get(PUBLISH_PRODUCTION_STAMP)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn publish_project_schema() -> ToolSchema {
    ToolSchema::new("publish/project", "Publish a project to an environment: ensures the env VM (Incus container), deploys via the deployment API, optionally binds a custom domain, and records the deployment for history/rollback.")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "project_id": { "type": "string", "description": "UUID of the project to publish" },
                "env": { "type": "string", "enum": ["test", "development", "staging", "production"], "default": "production", "description": "Target environment. Website and python projects keep two websites: the working copy at {slug}-test.{domain} and the public page at {slug}.{domain}. A publish from this tool always lands on the test twin unless the Deploy pipeline is the caller, so ask for 'test' when the user wants to see a change before it goes public." },
                "domain": { "type": "string", "description": "Optional custom domain to bind" },
                "launcher": { "type": "boolean", "description": "When true, the published app auto-pins to the desktop launcher (desktop category) for workspace users (#1160)." },
                "widget": { "type": "boolean", "description": "When true, the published app is registered as a desktop widget (always-visible tile) instead of a windowed app (#1160)." },
                "on_behalf_of_user": { "type": "string", "description": "Internal — always overwritten by the server with the run's acting user for deploy RBAC; never supply it yourself." }
            },
            "required": ["project_id"]
        }))
        .with_approval()
        .with_use_cases(vec![VibeUseCase::SoftwareDevelopment])
}

/// Collect the project's source files from its workspace (the agent's actual
/// output) so the deployment API can push them to the ALM repo instead of an
/// empty app. The workspace dir is keyed by the ALM repo slug, falling back
/// to the raw project id.
pub(crate) fn collect_workspace_files(project: &Project) -> Result<Vec<Value>, String> {
    let candidates = [VmLifecycle::alm_repo(&project.name), project.id.to_string()];
    for key in candidates {
        let dir = harness::workspace_root().join(&key);
        if !dir.is_dir() {
            continue;
        }
        let mut out = Vec::new();
        let mut total_bytes = 0u64;
        match walk_workspace(&dir, &dir, &mut out, &mut total_bytes) {
            Ok(()) => {
                if !out.is_empty() {
                    log::info!(
                        "Vibe publish: packaged {} files ({total_bytes} bytes) from workspace '{key}'",
                        out.len()
                    );
                    return Ok(out);
                }
            }
            // A size-budget violation is an actionable error, not a reason to
            // fall through to an empty archive (#934).
            Err(e) => return Err(e),
        }
    }
    log::warn!(
        "Vibe publish: no workspace files found for project '{}' — deploying empty repo",
        project.name
    );
    Ok(Vec::new())
}
