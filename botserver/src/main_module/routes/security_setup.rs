use crate::security::auth::{AuthConfig, AuthMiddlewareState};
use crate::security::auth_provider::{ApiKeyAuthProvider, AuthProviderBuilder};
use crate::security::cors::create_cors_layer;
use crate::security::jwt::{JwtConfig, JwtKey, JwtManager};
use crate::security::rbac_middleware::{build_default_route_permissions, RbacConfig, RbacManager};
use botcore::shared::state::AppState;
use log::{error, info};
use std::sync::Arc;

pub struct SecurityComponents {
    pub cors: tower_http::cors::CorsLayer,
    pub auth_config: Arc<AuthConfig>,
    pub jwt_manager: Option<Arc<JwtManager>>,
    pub rbac_manager: Arc<RbacManager>,
    pub auth_provider_registry: Arc<dyn std::any::Any + Send + Sync>,
    pub auth_middleware_state: AuthMiddlewareState,
    /// `no_security=true` in the product file: single-user desktop install.
    /// Callers use it to skip checks that only make sense with an account
    /// session (CSRF double-submit), never to relax a normal deployment.
    pub no_security: bool,
}

/// Builds the auth/JWT/RBAC stack.
///
/// #2140 — returns an error when the SaaS JWT secret cannot be resolved (a
/// production deployment with no persisted secret). The caller must refuse to
/// start: serving with a guessable key would let anyone mint a valid token.
pub async fn setup_security(app_state: &Arc<AppState>) -> Result<SecurityComponents, String> {
    use botcore::config::ConfigManager;

    let cfg = ConfigManager::new(app_state.conn.clone());
    let origins_str = cfg
        .get_config(&uuid::Uuid::nil(), "cors-allowed-origins", None)
        .unwrap_or_default();
    if !origins_str.is_empty() {
        let origins: Vec<String> = origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !origins.is_empty() {
            info!("Loaded {} CORS allowed origins from config", origins.len());
            crate::security::cors::set_cors_allowed_origins(origins);
        }
    }

    let cors = create_cors_layer();

    // Read the product file directly: security setup runs before the lazy
    // global is touched on some boots, and the desktop edition must not come
    // up with an auth gate it cannot satisfy.
    let no_security = botcore::product::ProductConfig::load()
        .map(|product| product.no_security)
        .unwrap_or_else(|_| {
            botcore::product::PRODUCT_CONFIG
                .read()
                .map(|product| product.no_security)
                .unwrap_or(false)
        });

    let auth_config = {
        let mut cfg = AuthConfig::from_env();

        // Wire no_security from .product into the auth middleware: when
        // the product config sets no_security=true, failed auth falls
        // through as the single local desktop user instead of returning 401.
        if no_security {
            info!("no_security=true in .product — auth middleware will use the local desktop identity");
            cfg.require_auth = false;
        }

        // #2133 — anonymous access is an exact (method, path) declaration, not
        // a path prefix. Anything not declared in `platform_public_routes()` is
        // authenticated, and a state-changing method cannot be declared as a
        // plain read route. The old prefix chain made every route added under
        // `/api/auth`, `/api/products/items` and `/api/sheet` public by
        // accident (#2127, #2128).
        let public_routes = crate::security::auth::platform_public_routes();
        let private_routes = crate::security::auth::platform_private_routes();
        info!(
            "Anonymous allowlist: {} explicit (method, path) routes, {} always-authenticated exceptions",
            public_routes.len(),
            private_routes.len()
        );
        Arc::new(
            cfg.with_public_routes(public_routes)
                .with_private_routes(private_routes),
        )
    };

    // #2140 — one resolver for the whole workspace. Resolution fails closed in
    // production when no secret can be obtained, so the server refuses to start
    // instead of minting and verifying with a public constant.
    let jwt_secret = botsecurity_core::verified_jwt::resolve_saas_jwt_secret()
        .map_err(|error| error.to_string())?;

    // SECURITY: the agent VM API reads the token payload itself; without a
    // signature check any handcrafted JWT reached another user's VM. Hand the
    // signing secret to that crate (process-wide, no crate cycle).
    std::env::set_var("GB_AGENT_JWT_SECRET", &jwt_secret);

    let jwt_config = JwtConfig::default();
    let jwt_key = JwtKey::from_secret(&jwt_secret);
    let jwt_manager = match JwtManager::new(jwt_config, jwt_key) {
        Ok(manager) => {
            #[cfg(feature = "cache")]
            let manager = {
                // Persist the token blacklist in the shared cache so revoked
                // tokens stay rejected across process restarts (#901). Falls
                // back to in-memory when no cache client is available.
                match &app_state.cache {
                    Some(client) => {
                        let store = Arc::new(
                            crate::security::redis_blacklist_store::RedisBlacklistStore::new(
                                Arc::clone(client),
                            ),
                        );
                        info!("JWT Manager using Redis-backed token blacklist");
                        manager.with_blacklist_store(store)
                    }
                    None => {
                        info!("JWT Manager using in-memory token blacklist (no cache client)");
                        manager
                    }
                }
            };

            info!("JWT Manager initialized successfully");
            Some(Arc::new(manager))
        }
        Err(e) => {
            error!("Failed to initialize JWT Manager: {e}");
            None
        }
    };

    let mut rbac_config = RbacConfig::default();
    if no_security {
        // Desktop-local profile: no accounts, so RBAC has no principal to
        // check against. The product file is the explicit switch.
        rbac_config.allow_all = true;
        info!("no_security=true in .product — RBAC allows the local owner");
    }
    let rbac_manager = Arc::new(RbacManager::new(rbac_config));

    let default_permissions = build_default_route_permissions();
    let default_permission_count = default_permissions.len();
    rbac_manager.register_routes(default_permissions).await;
    info!(
        "RBAC Manager initialized with {} default route permissions",
        default_permission_count
    );

    let auth_provider_registry = {
        let mut builder = AuthProviderBuilder::new()
            .with_api_key_provider(Arc::new(ApiKeyAuthProvider::new()))
            .with_auth_config(Arc::clone(&auth_config));

        if let Some(ref manager) = jwt_manager {
            builder = builder.with_jwt_manager(Arc::clone(manager));
        }

        // Cloud login JWTs (handle_login/signup, minted with the persisted
        // saas_jwt_secret) carry claim-light payloads (sub/email/org/branch,
        // no iss/aud). The generic local-jwt provider rejects them, which used
        // to degrade every cloud-authenticated suite request to anonymous.
        builder = builder.with_provider(Arc::new(
            crate::security::saas_jwt_auth::SaasJwtAuthProvider::new(jwt_secret.clone()),
        ));

        let zitadel_configured = std::env::var("ZITADEL_ISSUER_URL").is_ok()
            && std::env::var("ZITADEL_CLIENT_ID").is_ok();

        if zitadel_configured {
            info!("Zitadel environment variables detected - external IdP authentication available");
        }

        Arc::new(builder.build().await)
    };

    info!(
        "Auth provider registry initialized with {} providers",
        auth_provider_registry.provider_count().await
    );

    let auth_middleware_state = AuthMiddlewareState::new(
        Arc::clone(&auth_config),
        Arc::clone(&auth_provider_registry),
    );

    Ok(SecurityComponents {
        cors,
        auth_config,
        jwt_manager,
        rbac_manager,
        auth_provider_registry,
        auth_middleware_state,
        no_security,
    })
}
