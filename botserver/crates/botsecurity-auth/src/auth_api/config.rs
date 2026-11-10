//! Authentication configuration and the anonymous/public route model.
//!
//! # Why this is not a list of path prefixes (#2133)
//!
//! A prefix entry is a namespace grant: `/api/products/items` also opened
//! `POST /api/products/items`, `PUT /api/products/items/:id/stock` and every
//! route added there later. That turned two ordinary allowlist entries into
//! unauthenticated writes (#2127, #2128).
//!
//! The allowlist is therefore a list of [`PublicRoute`]s: an exact HTTP method
//! plus a path that must match the request exactly, or match a declared
//! parameter (`:id`) / a single declared trailing wildcard (`/*`). Anything
//! that is not listed is authenticated. A state-changing method (`POST`,
//! `PUT`, `PATCH`, `DELETE`) cannot be declared as a plain read route: it must
//! be flagged as a credential entry point or as a provider call whose handler
//! verifies a signature/shared secret.

use axum::http::Method;
use std::fmt;

/// Why a route is reachable without a session.
///
/// The variant is the explicit declaration the old prefix list never asked
/// for: a route is public because of *what it does*, not because of where it
/// lives in the URL tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicAccess {
    /// Read-only endpoint that returns non-tenant or caller-scoped data
    /// (`/health`, the public bot access check, the app catalog).
    AnonymousRead,
    /// Static asset tree served by the UI (`/suite`, `/static`, `/themes`).
    /// Only paths that look like a file are matched, so a server-rendered
    /// route registered under the same prefix stays authenticated.
    Asset,
    /// Read-only endpoint whose access is proven by an unguessable capability
    /// (public share token, terminal session id, tracking pixel token).
    Capability,
    /// The request itself carries the credential being verified: login,
    /// signup, refresh, logout, bootstrap and 2FA. These are the only write
    /// endpoints allowed without a signature, because no session exists yet.
    CredentialEntry,
    /// Inbound call from an external provider or a browser redirect. The
    /// handler verifies a signature, a shared secret or a signed state value
    /// before acting (`VerifiedWebhook` is a promise the route keeps).
    VerifiedWebhook,
}

impl PublicAccess {
    /// Whether this kind may only be reached with a read method.
    #[must_use]
    pub fn is_read_only(self) -> bool {
        matches!(
            self,
            Self::AnonymousRead | Self::Asset | Self::Capability
        )
    }

    /// Whether `method` may use a route declared with this access kind.
    #[must_use]
    pub fn allows_method(self, method: &Method) -> bool {
        match self {
            Self::AnonymousRead | Self::Capability => is_read_method(method),
            Self::Asset => method == Method::GET || method == Method::HEAD,
            Self::CredentialEntry | Self::VerifiedWebhook => true,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AnonymousRead => "anonymous read",
            Self::Asset => "static asset",
            Self::Capability => "capability token",
            Self::CredentialEntry => "credential entry",
            Self::VerifiedWebhook => "verified webhook",
        }
    }
}

fn is_read_method(method: &Method) -> bool {
    // OPTIONS is included because a CORS preflight never carries credentials
    // and changes no state.
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

/// A single public route: one HTTP method, one path shape, one reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicRoute {
    pub method: Method,
    pub pattern: String,
    pub access: PublicAccess,
}

impl PublicRoute {
    /// Declares a public route, rejecting declarations that would widen into a
    /// namespace or make a write reachable without proof.
    pub fn new(
        method: Method,
        pattern: impl Into<String>,
        access: PublicAccess,
    ) -> Result<Self, AuthConfigError> {
        let pattern = pattern.into();
        validate_pattern(&pattern)?;
        if access.is_read_only() && !is_read_method(&method) {
            return Err(AuthConfigError::WriteMethodNeedsProof {
                method: method.to_string(),
                pattern,
                access: access.label(),
            });
        }
        Ok(Self {
            method,
            pattern,
            access,
        })
    }

    /// A plain read-only endpoint (`GET`, plus `HEAD`/`OPTIONS` by HTTP
    /// semantics).
    pub fn read(method: Method, pattern: impl Into<String>) -> Result<Self, AuthConfigError> {
        Self::new(method, pattern, PublicAccess::AnonymousRead)
    }

    /// A static asset tree. The pattern must end in `/*` and only file-like
    /// paths are matched.
    pub fn asset(pattern: impl Into<String>) -> Result<Self, AuthConfigError> {
        let route = Self::new(Method::GET, pattern, PublicAccess::Asset)?;
        if !route.pattern.ends_with("/*") {
            return Err(AuthConfigError::InvalidPattern {
                pattern: route.pattern,
                reason: "an asset tree must end with '/*'",
            });
        }
        Ok(route)
    }

    /// A read-only route gated by an unguessable capability.
    pub fn capability(pattern: impl Into<String>) -> Result<Self, AuthConfigError> {
        Self::new(Method::GET, pattern, PublicAccess::Capability)
    }

    /// A login/signup/refresh/bootstrap endpoint.
    pub fn credential_entry(
        method: Method,
        pattern: impl Into<String>,
    ) -> Result<Self, AuthConfigError> {
        Self::new(method, pattern, PublicAccess::CredentialEntry)
    }

    /// A provider call or browser callback whose handler verifies a signature,
    /// shared secret or signed state.
    pub fn webhook(
        method: Method,
        pattern: impl Into<String>,
    ) -> Result<Self, AuthConfigError> {
        Self::new(method, pattern, PublicAccess::VerifiedWebhook)
    }

    /// Whether this declaration admits the request.
    ///
    /// A `GET` declaration also admits `HEAD`, as the HTTP server does.
    #[must_use]
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        if self.method != *method && !(self.method == Method::GET && method == Method::HEAD) {
            return false;
        }
        if self.access == PublicAccess::Asset && !looks_like_asset(path) {
            return false;
        }
        route_matches(&self.pattern, path)
    }
}

/// Validates that a pattern cannot act as a namespace grant.
fn validate_pattern(pattern: &str) -> Result<(), AuthConfigError> {
    if pattern.is_empty() || !pattern.starts_with('/') {
        return Err(AuthConfigError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: "a public route must start with '/'",
        });
    }
    if pattern == "/" {
        return Err(AuthConfigError::RootPattern);
    }
    if pattern.contains('?') || pattern.contains('#') {
        return Err(AuthConfigError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: "a public route is a path; query strings are never matched",
        });
    }
    if pattern.ends_with('/') {
        return Err(AuthConfigError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: "a trailing '/' would grant a namespace; declare the exact route",
        });
    }

    let segments: Vec<&str> = pattern.split('/').collect();
    let last_index = segments.len().saturating_sub(1);
    for (index, segment) in segments.iter().enumerate() {
        if segment.contains('*') && *segment != "*" {
            return Err(AuthConfigError::InvalidPattern {
                pattern: pattern.to_string(),
                reason: "only a whole trailing '/*' wildcard is supported",
            });
        }
        if *segment == "*" && index != last_index {
            return Err(AuthConfigError::InvalidPattern {
                pattern: pattern.to_string(),
                reason: "a wildcard may only appear as the final segment",
            });
        }
        if segment.starts_with(':') && segment.len() < 2 {
            return Err(AuthConfigError::InvalidPattern {
                pattern: pattern.to_string(),
                reason: "a parameter segment needs a name, e.g. ':id'",
            });
        }
    }
    Ok(())
}

/// Matches a path against an exact or parameterized route shape.
///
/// * a literal segment must be equal;
/// * a `:name` segment matches exactly one non-empty segment;
/// * a final `/*` matches the remaining non-empty path.
///
/// There is deliberately no "starts with" case: `/api/auth` does not match
/// `/api/auth/directory-config`.
fn route_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    let mut path_segments = path.split('/');

    for segment in pattern_segments {
        let Some(actual) = path_segments.next() else {
            return false;
        };
        if segment == "*" {
            return !actual.is_empty();
        }
        if segment.starts_with(':') {
            if actual.is_empty() {
                return false;
            }
            continue;
        }
        if segment != actual {
            return false;
        }
    }
    path_segments.next().is_none()
}

/// Whether a path names a file rather than a server-rendered route.
///
/// Asset trees are the one place a prefix is legitimate (a UI ships thousands
/// of files). Requiring a dotted final segment keeps the server routes that
/// live under the same prefix — e.g. `/suite/sheet/fragments/sheet-list` —
/// authenticated.
fn looks_like_asset(path: &str) -> bool {
    let last = path.rsplit('/').next().unwrap_or_default();
    !last.is_empty()
        && last != "."
        && last != ".."
        && !last.starts_with('.')
        && last.contains('.')
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthConfigError {
    #[error("refusing public route {pattern:?}: {reason}")]
    InvalidPattern { pattern: String, reason: &'static str },

    #[error("refusing '/' as a public route: it would make every path public")]
    RootPattern,

    #[error(
        "refusing {method} {pattern:?} as {access}: a state-changing method needs an \
         explicit credential-entry or verified-webhook declaration"
    )]
    WriteMethodNeedsProof {
        method: String,
        pattern: String,
        access: &'static str,
    },
}

impl fmt::Display for PublicAccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub jwt_secret: Option<String>,
    pub api_key_header: String,
    pub bearer_prefix: String,
    pub session_cookie_name: String,
    /// Exact (method, path) routes reachable without a session. Empty by
    /// default: every path is authenticated unless the deployment declares it.
    pub public_routes: Vec<PublicRoute>,
    pub bot_id_header: String,
    pub org_id_header: String,
    pub internal_token: Option<String>,
    pub internal_token_header: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            jwt_secret: None,
            api_key_header: "X-API-Key".to_string(),
            bearer_prefix: "Bearer ".to_string(),
            session_cookie_name: "session_id".to_string(),
            // Secure by default (#2133): no route is anonymous until the
            // deployment declares it with an explicit method and path.
            public_routes: Vec::new(),
            bot_id_header: "X-Bot-ID".to_string(),
            org_id_header: "X-Organization-ID".to_string(),
            internal_token: None,
            internal_token_header: "X-Internal-Token".to_string(),
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(secret) = std::env::var("VAULT_TOKEN") {
            if !secret.is_empty() {
                if let Some(provider) = botsecurity_core::get_vault_provider() {
                    if let Ok(jwt_secret) = provider.get_jwt_secret() {
                        config.jwt_secret = Some(jwt_secret);
                    }
                }
            }
        }

        let internal_token = botcoresecrets::internal_api_token();
        if !internal_token.is_empty() {
            config.internal_token = Some(internal_token);
        }

        config
    }

    pub fn with_jwt_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt_secret = Some(secret.into());
        self
    }

    pub fn with_require_auth(mut self, require: bool) -> Self {
        self.require_auth = require;
        self
    }

    /// Replaces the public-route declarations.
    #[must_use]
    pub fn with_public_routes(mut self, routes: Vec<PublicRoute>) -> Self {
        self.public_routes = routes;
        self
    }

    /// Adds one already-validated public route.
    #[must_use]
    pub fn with_public_route(mut self, route: PublicRoute) -> Self {
        self.public_routes.push(route);
        self
    }

    /// Whether `method path` was explicitly declared public.
    #[must_use]
    pub fn is_public_route(&self, method: &Method, path: &str) -> bool {
        self.public_routes
            .iter()
            .any(|route| route.matches(method, path))
    }
}
