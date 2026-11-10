use super::*;
use crate::configure_routes::authorize_publisher;
use crate::render_routes::render_app;
use botsecurity_auth::auth_api::types::{AuthenticatedUser, Role};

fn published_app(name: &str, config: WebAppConfig) -> WebApp {
    WebApp {
        id: Uuid::new_v4(),
        name: name.to_string(),
        slug: slugify(name),
        description: None,
        template: WebAppTemplate::Blank,
        status: WebAppStatus::Published,
        config,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[test]
pub(super) fn test_app_name_is_escaped() {
    let app = published_app("</title><script>alert(1)</script>", WebAppConfig::default());

    let html = render_html(&app, "");

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;/title&gt;&lt;script&gt;"));
}

#[test]
pub(super) fn test_description_and_meta_values_are_escaped() {
    let mut config = WebAppConfig::default();
    config
        .meta_tags
        .insert("author".to_string(), "\"><script>alert(1)</script>".to_string());

    let mut app = published_app("Meta", config);
    app.description = Some("\" onload=\"alert(1)".to_string());

    let html = render_html(&app, "");

    assert!(!html.contains("\"><script>"));
    assert!(html.contains("&quot;&gt;&lt;script&gt;"));
    assert!(html.contains("&quot; onload=&quot;alert(1)"));
}

#[test]
pub(super) fn test_meta_name_is_escaped() {
    let mut config = WebAppConfig::default();
    config
        .meta_tags
        .insert("\"x\" onload=\"alert(1)".to_string(), "v".to_string());

    let html = render_html(&published_app("Meta name", config), "");

    assert!(!html.contains("\"x\" onload=\"alert(1)\""));
    assert!(html.contains("&quot;x&quot; onload=&quot;alert(1)"));
}

#[test]
pub(super) fn test_unsafe_script_and_style_urls_are_dropped() {
    let mut config = WebAppConfig::default();
    config.scripts = vec![
        "javascript:alert(1)".to_string(),
        " data:text/javascript,alert(1)".to_string(),
        "//evil.example.com/x.js".to_string(),
        "https://cdn.example.com/app.js".to_string(),
    ];
    config.styles = vec![
        "javascript:alert(1)".to_string(),
        "https://cdn.example.com/app.css".to_string(),
    ];

    let html = render_html(&published_app("Urls", config), "");

    assert!(!html.contains("javascript:"));
    assert!(!html.contains("data:text/javascript"));
    assert!(!html.contains("//evil.example.com"));
    assert!(html.contains(r#"<script src="https://cdn.example.com/app.js">"#));
    assert!(html.contains(r#"href="https://cdn.example.com/app.css""#));
}

#[test]
pub(super) fn test_safe_resource_url_rejects_dangerous_schemes() {
    assert!(safe_resource_url("javascript:alert(1)").is_none());
    assert!(safe_resource_url("JavaScript:alert(1)").is_none());
    assert!(safe_resource_url("data:text/html,<script>").is_none());
    assert!(safe_resource_url("vbscript:msgbox(1)").is_none());
    assert!(safe_resource_url("//evil.example.com/a.js").is_none());
    assert!(safe_resource_url("https://cdn.example.com/a.js").is_some());
    assert!(safe_resource_url("/suite/app.js").is_some());
    assert!(safe_resource_url("assets/app.js").is_some());
}

#[test]
pub(super) fn test_published_document_carries_sandbox_csp() {
    let response = document_response("<html></html>".to_string(), "no-store");
    let headers = response.headers();

    let csp = headers
        .get("content-security-policy")
        .and_then(|value| value.to_str().ok())
        .expect("published document must carry a CSP header");

    assert!(csp.contains("sandbox allow-scripts allow-forms"));
    assert!(!csp.contains("allow-same-origin"));
    assert!(csp.contains("default-src 'none'"));
    assert!(csp.contains("connect-src 'none'"));
    assert_eq!(
        headers
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );
}

#[test]
pub(super) fn test_embed_shell_uses_sandboxed_iframe() {
    let html = embed_shell("/render/x\"><script>alert(1)</script>", "T\"><b>");

    assert!(html.contains(r#"sandbox="allow-scripts allow-forms""#));
    assert!(!html.contains("allow-same-origin"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("\"><b>"));
    assert!(html.contains("&quot;&gt;&lt;b&gt;"));
}

#[test]
pub(super) fn test_render_route_response_carries_csp_header() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("test runtime");

    let response = runtime.block_on(async {
        let state = Arc::new(WebaState::new());
        let app = published_app("Served", WebAppConfig::default());
        state.apps.write().await.insert(app.id, app);

        render_app(State(state), Path("served".to_string()))
            .await
            .expect("render_app must serve a published app")
    });

    let csp = response
        .headers()
        .get("content-security-policy")
        .and_then(|value| value.to_str().ok())
        .expect("the render route must send the CSP header");

    assert!(csp.contains("sandbox allow-scripts allow-forms"));
    assert!(csp.contains("default-src 'none'"));
}

#[test]
pub(super) fn test_publish_requires_admin() {
    let anonymous = AuthenticatedUser::anonymous();
    assert!(authorize_publisher(&anonymous).is_err());

    let user = AuthenticatedUser::new(Uuid::new_v4(), "someone".to_string());
    assert!(authorize_publisher(&user).is_err());

    let admin =
        AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);
    assert!(authorize_publisher(&admin).is_ok());

    let desktop = AuthenticatedUser::desktop_local();
    assert!(authorize_publisher(&desktop).is_ok());
}
