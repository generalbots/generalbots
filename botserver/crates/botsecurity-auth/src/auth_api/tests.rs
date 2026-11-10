use super::super::types::*;
use super::{AuthConfig, AuthError, PublicRoute};
use axum::http::{Method, StatusCode};

#[test]
fn test_role_permissions() {
    assert!(!Role::Anonymous.has_permission(&Permission::Read));
    assert!(Role::User.has_permission(&Permission::Read));
    assert!(Role::User.has_permission(&Permission::AccessApi));
    assert!(!Role::User.has_permission(&Permission::Write));

    assert!(Role::Admin.has_permission(&Permission::ManageUsers));
    assert!(Role::Admin.has_permission(&Permission::Delete));

    assert!(Role::SuperAdmin.has_permission(&Permission::ManageSecrets));
}

#[test]
fn test_role_hierarchy() {
    assert!(Role::SuperAdmin.is_at_least(&Role::Admin));
    assert!(Role::Admin.is_at_least(&Role::Moderator));
    assert!(Role::BotOwner.is_at_least(&Role::BotOperator));
    assert!(Role::BotOperator.is_at_least(&Role::BotViewer));
    assert!(!Role::User.is_at_least(&Role::Admin));
}

#[test]
fn test_authenticated_user_builder() {
    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "testuser".to_string())
        .with_email("test@example.com")
        .with_role(Role::Admin)
        .with_metadata("key", "value");

    assert_eq!(user.email, Some("test@example.com".to_string()));
    assert!(user.has_role(&Role::Admin));
    assert_eq!(user.metadata.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_user_permissions() {
    let admin =
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);

    assert!(admin.has_permission(&Permission::ManageUsers));
    assert!(admin.has_permission(&Permission::Delete));
    assert!(admin.is_admin());

    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string());
    assert!(user.has_permission(&Permission::Read));
    assert!(!user.has_permission(&Permission::ManageUsers));
    assert!(!user.is_admin());
}

#[test]
fn test_anonymous_user() {
    let anon = AuthenticatedUser::anonymous();
    assert!(!anon.is_authenticated());
    assert!(anon.has_role(&Role::Anonymous));
    assert!(!anon.has_permission(&Permission::Read));
}

#[test]
fn test_desktop_local_user() {
    let local = AuthenticatedUser::desktop_local();
    // The local owner is a real principal, not an anonymous visitor.
    assert_eq!(local.username, "local");
    assert!(local.has_role(&Role::User));
    assert!(!local.has_role(&Role::Anonymous));
    assert_eq!(
        local.metadata.get("desktop_local").map(String::as_str),
        Some("true")
    );
    // No directory account exists, so the nil id keeps Drive's user scope on
    // the historical `default` folder instead of inventing a new bucket.
    assert_eq!(local.user_id, uuid::Uuid::nil());
    assert!(!local.is_authenticated());
}

#[test]
fn test_service_user() {
    let service = AuthenticatedUser::service("scheduler");
    assert!(service.has_role(&Role::Service));
    assert!(service.has_permission(&Permission::ExecuteTasks));
}

#[test]
fn test_bot_user() {
    let bot_id = uuid::Uuid::new_v4();
    let bot = AuthenticatedUser::bot_user(bot_id, "test-bot");
    assert!(bot.is_bot());
    assert!(bot.has_permission(&Permission::SendMessages));
    assert_eq!(bot.current_bot_id, Some(bot_id));
}

#[test]
fn test_auth_config_paths() {
    let config = AuthConfig::default()
        .with_public_route(PublicRoute::read(Method::GET, "/health").expect("valid route"))
        .with_public_route(PublicRoute::asset("/static/*").expect("valid asset tree"));

    assert!(config.is_public_route(&Method::GET, "/health"));
    assert!(!config.is_public_route(&Method::POST, "/health"));
    assert!(!config.is_public_route(&Method::GET, "/api/users"));

    assert!(config.is_public_route(&Method::GET, "/static/css/style.css"));
    assert!(!config.is_public_route(&Method::GET, "/api/private"));
}

#[test]
fn test_auth_config_parameterized_paths() {
    let config = AuthConfig::default()
        .with_public_route(PublicRoute::read(Method::GET, "/api/bots/:bot_name/access").expect("valid route"));

    // The parameter scopes exactly one route shape.
    assert!(config.is_public_route(&Method::GET, "/api/bots/foo/access"));
    assert!(config.is_public_route(&Method::GET, "/api/bots/default/access"));
    // Sibling routes under /api/bots must NOT be caught by the pattern.
    assert!(!config.is_public_route(&Method::GET, "/api/bots/list"));
    assert!(!config.is_public_route(&Method::GET, "/api/bots/config"));
    assert!(!config.is_public_route(&Method::GET, "/api/bots"));
    assert!(!config.is_public_route(&Method::GET, "/api/bots/foo/config"));
    assert!(!config.is_public_route(&Method::GET, "/api/bots/a/access/extra"));
}

#[test]
fn a_bare_prefix_is_never_a_public_route() {
    // #2133 — the exact route only. A sibling added later stays authenticated.
    let config = AuthConfig::default()
        .with_public_route(PublicRoute::read(Method::GET, "/api/auth/login").expect("valid route"));

    assert!(config.is_public_route(&Method::GET, "/api/auth/login"));
    assert!(!config.is_public_route(&Method::GET, "/api/auth/login/extra"));
    assert!(!config.is_public_route(&Method::GET, "/api/auth/directory-config"));
    assert!(!config.is_public_route(&Method::GET, "/api/auth"));
}

#[test]
fn a_trailing_slash_declaration_is_refused() {
    // A trailing slash used to be a silent no-op (the matcher produced a `//`
    // prefix). It is now rejected instead of being kept as dead policy.
    assert!(PublicRoute::read(Method::GET, "/api/bots/").is_err());
}

#[test]
fn a_write_method_cannot_be_declared_as_a_read_route() {
    for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
        assert!(
            PublicRoute::read(method.clone(), "/api/products/items").is_err(),
            "{method} must not be declarable as an anonymous read route"
        );
    }
    // ... unless the route is explicitly a credential entry or a webhook.
    assert!(PublicRoute::credential_entry(Method::POST, "/api/auth/login").is_ok());
    assert!(PublicRoute::webhook(Method::POST, "/api/whatsapp/webhook").is_ok());
}

#[test]
fn a_write_request_never_matches_a_read_declaration() {
    let config = AuthConfig::default()
        .with_public_route(PublicRoute::read(Method::GET, "/api/products/items").expect("valid route"));

    assert!(config.is_public_route(&Method::GET, "/api/products/items"));
    assert!(!config.is_public_route(&Method::POST, "/api/products/items"));
    assert!(!config.is_public_route(&Method::PUT, "/api/products/items"));
    assert!(!config.is_public_route(&Method::DELETE, "/api/products/items"));
}

#[test]
fn an_asset_tree_never_exposes_a_server_route() {
    let config = AuthConfig::default()
        .with_public_route(PublicRoute::asset("/suite/*").expect("valid asset tree"));

    assert!(config.is_public_route(&Method::GET, "/suite/desktop.html"));
    assert!(config.is_public_route(&Method::GET, "/suite/sheet/sheet.js"));
    // Server-rendered fragments under the same prefix stay authenticated.
    assert!(!config.is_public_route(&Method::GET, "/suite/sheet/fragments/list"));
    assert!(!config.is_public_route(&Method::GET, "/suite/dashboards"));
}

#[test]
fn test_auth_error_responses() {
    assert_eq!(
        AuthError::MissingToken.status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        AuthError::InsufficientPermissions.status_code(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        AuthError::RateLimited.status_code(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        AuthError::BotAccessDenied.status_code(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn test_bot_access() {
    let bot_id = uuid::Uuid::new_v4();
    let other_bot_id = uuid::Uuid::new_v4();

    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string())
        .with_bot_access(BotAccess::viewer(bot_id));

    assert!(user.can_access_bot(&bot_id));
    assert!(user.can_view_bot(&bot_id));
    assert!(!user.can_operate_bot(&bot_id));
    assert!(!user.can_manage_bot(&bot_id));
    assert!(!user.can_access_bot(&other_bot_id));

    let admin =
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);

    assert!(admin.can_access_bot(&bot_id));
    assert!(admin.can_access_bot(&other_bot_id));
}

#[test]
fn test_bot_owner_access() {
    let bot_id = uuid::Uuid::new_v4();

    let owner = AuthenticatedUser::new(uuid::Uuid::new_v4(), "owner".to_string())
        .with_bot_access(BotAccess::owner(bot_id));

    assert!(owner.can_access_bot(&bot_id));
    assert!(owner.can_view_bot(&bot_id));
    assert!(owner.can_operate_bot(&bot_id));
    assert!(owner.can_manage_bot(&bot_id));
}

#[test]
fn test_bot_operator_access() {
    let bot_id = uuid::Uuid::new_v4();

    let operator = AuthenticatedUser::new(uuid::Uuid::new_v4(), "operator".to_string())
        .with_bot_access(BotAccess::operator(bot_id));

    assert!(operator.can_access_bot(&bot_id));
    assert!(operator.can_view_bot(&bot_id));
    assert!(operator.can_operate_bot(&bot_id));
    assert!(!operator.can_manage_bot(&bot_id));
}

#[test]
fn test_bot_permission_check() {
    let bot_id = uuid::Uuid::new_v4();

    let operator = AuthenticatedUser::new(uuid::Uuid::new_v4(), "operator".to_string())
        .with_bot_access(BotAccess::operator(bot_id));

    assert!(operator.has_bot_permission(&bot_id, &Permission::SendMessages));
    assert!(operator.has_bot_permission(&bot_id, &Permission::ViewAnalytics));
    assert!(!operator.has_bot_permission(&bot_id, &Permission::ManageBots));
}

#[test]
fn test_bot_access_expiry() {
    let bot_id = uuid::Uuid::new_v4();
    let past_time = chrono::Utc::now().timestamp() - 3600;

    let expired_access = BotAccess::viewer(bot_id).with_expiry(past_time);
    assert!(expired_access.is_expired());
    assert!(!expired_access.is_valid());

    let future_time = chrono::Utc::now().timestamp() + 3600;
    let valid_access = BotAccess::viewer(bot_id).with_expiry(future_time);
    assert!(!valid_access.is_expired());
    assert!(valid_access.is_valid());
}

#[test]
fn test_accessible_bot_ids() {
    let bot1 = uuid::Uuid::new_v4();
    let bot2 = uuid::Uuid::new_v4();

    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string())
        .with_bot_access(BotAccess::owner(bot1))
        .with_bot_access(BotAccess::viewer(bot2));

    let accessible = user.accessible_bot_ids();
    assert_eq!(accessible.len(), 2);
    assert!(accessible.contains(&bot1));
    assert!(accessible.contains(&bot2));

    let owned = user.owned_bot_ids();
    assert_eq!(owned.len(), 1);
    assert!(owned.contains(&bot1));
}

#[test]
fn test_organization_access() {
    let org_id = uuid::Uuid::new_v4();
    let other_org_id = uuid::Uuid::new_v4();

    let user =
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string()).with_organization(org_id);

    assert!(user.can_access_organization(&org_id));
    assert!(!user.can_access_organization(&other_org_id));
}

#[test]
fn test_has_any_permission() {
    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string());

    assert!(user.has_any_permission(&[Permission::Read, Permission::Write]));
    assert!(!user.has_any_permission(&[Permission::Delete, Permission::Admin]));
}

#[test]
fn test_has_all_permissions() {
    let admin =
        AuthenticatedUser::new(uuid::Uuid::new_v4(), "admin".to_string()).with_role(Role::Admin);

    assert!(admin.has_all_permissions(&[Permission::Read, Permission::Write, Permission::Delete]));
    assert!(!admin.has_all_permissions(&[Permission::ManageSecrets]));
}

#[test]
fn test_highest_role() {
    let user = AuthenticatedUser::new(uuid::Uuid::new_v4(), "user".to_string())
        .with_role(Role::Admin)
        .with_role(Role::Moderator);

    assert_eq!(user.highest_role(), &Role::Admin);
}

#[test]
fn forged_bearer_jwt_is_rejected() {
    let user_id = uuid::Uuid::new_v4();
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        serde_json::json!({"sub": user_id, "exp": 4_000_000_000u64}).to_string(),
    );
    let config = AuthConfig::default().with_jwt_secret("unit-test-secret");
    let token = format!("{header}.{payload}.not-a-signature");
    let result = super::auth_api_utils::validate_bearer_token_sync(&token, &config);
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[test]
fn signed_bearer_jwt_is_accepted() {
    let user_id = uuid::Uuid::new_v4();
    let config = AuthConfig::default().with_jwt_secret("unit-test-secret");
    let claims = serde_json::json!({
        "sub": user_id,
        "email": "user@example.com",
        "roles": ["Admin"],
        "exp": 4_000_000_000u64,
    });
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"unit-test-secret"),
    )
    .expect("test JWT encodes");
    let user = super::auth_api_utils::validate_bearer_token_sync(&token, &config)
        .expect("signed JWT verifies");
    assert_eq!(user.user_id, user_id);
    assert_eq!(user.email.as_deref(), Some("user@example.com"));
    assert!(user.roles.contains(&Role::Admin));
}

#[test]
fn unknown_session_token_is_rejected() {
    let result = super::auth_api_utils::validate_session_sync("gb_not-in-the-session-store");
    assert!(matches!(result, Err(AuthError::SessionExpired)));
}

/// #2122 — `X-User-ID` is caller-supplied, so it must never authenticate a
/// request. Only a verified API key, bearer/JWT or session may do that.
#[test]
fn an_x_user_id_header_alone_is_rejected() {
    let config = AuthConfig::default();
    let request = axum::http::Request::builder()
        .uri("/api/crm/contacts")
        .header("X-User-ID", "6ba7b810-9dad-11d1-80b4-00c04fd430c8")
        .body(axum::body::Body::empty())
        .expect("request builds");

    let error = super::auth_api_utils::extract_user_from_request(&request, &config)
        .expect_err("X-User-ID must not authenticate");
    assert!(matches!(error, AuthError::MissingToken));
    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);

    // The structured view used by the provider chain must not surface it either.
    let extracted = super::auth_api_utils::ExtractedAuthData::from_request(&request, &config);
    assert!(extracted.api_key.is_none());
    assert!(extracted.bearer_token.is_none());
    assert!(extracted.query_token.is_none());
    assert!(extracted.session_id.is_none());
}

/// #2122 — the provider chain returns `MissingToken` (→ 401) for a request
/// whose only "credential" is the `X-User-ID` header.
#[tokio::test]
async fn the_provider_chain_rejects_an_x_user_id_header_alone() {
    let config = AuthConfig::default();
    let registry = crate::auth_provider::AuthProviderRegistry::new();
    let request = axum::http::Request::builder()
        .uri("/api/crm/contacts")
        .header("X-User-ID", "6ba7b810-9dad-11d1-80b4-00c04fd430c8")
        .body(axum::body::Body::empty())
        .expect("request builds");

    let extracted = super::auth_api_utils::ExtractedAuthData::from_request(&request, &config);
    let error = super::authenticate_with_extracted_data(extracted, &config, &registry)
        .await
        .expect_err("X-User-ID must not authenticate");
    assert!(matches!(error, AuthError::MissingToken));
    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
}
