use std::sync::Arc;
use log::{info, error};
use crate::security::{
    AuthConfig, AuthMiddlewareState, AuthProviderBuilder, ApiKeyAuthProvider,
    JwtConfig, JwtKey, JwtManager, RbacConfig, RbacManager,
    build_default_route_permissions, create_cors_layer,
};
use botcore::shared::state::AppState;

pub struct SecurityComponents {
    pub cors: tower_http::cors::CorsLayer,
    pub auth_config: Arc<AuthConfig>,
    pub jwt_manager: Option<Arc<JwtManager>>,
    pub rbac_manager: Arc<RbacManager>,
    pub auth_provider_registry: Arc<dyn std::any::Any + Send + Sync>,
    pub auth_middleware_state: AuthMiddlewareState,
}

pub async fn setup_security(app_state: &Arc<AppState>) -> SecurityComponents {
    use diesel::prelude::*;
    use diesel::QueryDsl;
    use botcore::shared::models::schema::bot_configuration::dsl::*;

    if let Ok(mut conn) = app_state.conn.get() {
        if let Ok(origins_str) = bot_configuration
            .filter(config_key.eq("cors-allowed-origins"))
            .select(config_value)
            .first::<String>(&mut conn)
        {
            let origins: Vec<String> = origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !origins.is_empty() {
                info!("Loaded {} CORS allowed origins from config", origins.len());
                crate::security::set_cors_allowed_origins(origins);
            }
        }
    }

    let cors = create_cors_layer();

    let auth_config = {
        let mut cfg = AuthConfig::from_env();
        // ⚠️ Remove catch-all "/" from public paths — CRITICAL SECURITY.
        // The default AuthConfig includes "/" which makes every endpoint
        // publicly accessible, including /api/files/buckets that enumerates
        // all MinIO buckets. Only explicitly listed paths are public.
        cfg.public_paths.retain(|p| p != "/");

        Arc::new(cfg
            .add_anonymous_path("/health")
            .add_anonymous_path("/healthz")
            .add_anonymous_path("/api/health")
            .add_anonymous_path("/api/product")
            .add_anonymous_path("/api/manifest")
            .add_anonymous_path("/api/i18n")
            .add_anonymous_path("/api/auth")
            .add_anonymous_path("/api/auth/login")
            .add_anonymous_path("/api/auth/refresh")
            .add_anonymous_path("/api/auth/bootstrap")
            .add_anonymous_path("/api/setup/status")
            .add_anonymous_path("/api/bot/config")
            .add_anonymous_path("/api/suggestions")
            .add_anonymous_path("/api/client-errors")
            .add_anonymous_path("/ws")
            .add_anonymous_path("/auth")
            .add_anonymous_path("/webhook/whatsapp")
            .add_anonymous_path("/api/catalog")
            .add_anonymous_path("/api/bots/")
            .add_public_path("/static")
            .add_public_path("/favicon.ico")
            .add_public_path("/suite")
            .add_public_path("/themes")
            .add_public_path("/api/product")
        )
    };

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        info!("JWT_SECRET not set, using default development secret");
        "dev-secret-key-change-in-production-minimum-32-chars".to_string()
    });

    let jwt_config = JwtConfig::default();
    let jwt_key = JwtKey::from_secret(&jwt_secret);
    let jwt_manager = match JwtManager::new(jwt_config, jwt_key) {
        Ok(manager) => {
            info!("JWT Manager initialized successfully");
            Some(Arc::new(manager))
        }
        Err(e) => {
            error!("Failed to initialize JWT Manager: {e}");
            None
        }
    };

    let rbac_config = RbacConfig::default();
    let rbac_manager = Arc::new(RbacManager::new(rbac_config));

    let default_permissions = build_default_route_permissions();
    rbac_manager.register_routes(default_permissions).await;
    info!(
        "RBAC Manager initialized with {} default route permissions",
        rbac_manager.config().cache_ttl_seconds
    );

    let auth_provider_registry = {
        let mut builder = AuthProviderBuilder::new()
            .with_api_key_provider(Arc::new(ApiKeyAuthProvider::new()))
            .with_auth_config(Arc::clone(&auth_config));

        if let Some(ref manager) = jwt_manager {
            builder = builder.with_jwt_manager(Arc::clone(manager));
        }

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

    SecurityComponents {
        cors,
        auth_config,
        jwt_manager,
        rbac_manager,
        auth_provider_registry,
        auth_middleware_state,
    }
}
