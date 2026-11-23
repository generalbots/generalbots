use crate::models::{ConnectBody, ItemRow};
use crate::{acl, audit, registry, schema, sync};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use botcore::shared::state::AppState;
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt, VerifiedJwt};
use diesel::prelude::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_SEARCH_LIMIT: i64 = 25;
const MAX_SEARCH_LIMIT: i64 = 100;

/// Authenticated admin principal, or an HTTP error (#2748).
///
/// No claim is read until [`verify_saas_jwt`] has checked the HS256 signature
/// (constant-time HMAC), `exp`/`nbf` and the subject. A token whose payload was
/// written by the caller — `alg:none`, tampered, foreign-key or expired — never
/// reaches `roles`, `sub` or `org_id`, so it cannot grant admin or name another
/// user's tenant. Roles are taken from the signed claims only; when the token
/// carries no role the caller is a plain user, never an admin.
fn verified_admin(headers: &HeaderMap) -> Result<VerifiedJwt, (StatusCode, String)> {
    let token = bearer_token(headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Bearer token required".to_string()))?;
    let claims = verify_saas_jwt(token).map_err(|reason| {
        tracing::warn!(target: "botconnectors", reason = %reason, "connector admin token rejected");
        (StatusCode::UNAUTHORIZED, "Valid JWT required".to_string())
    })?;
    if !claims.roles.iter().any(|role| role.eq_ignore_ascii_case("admin")) {
        return Err((StatusCode::FORBIDDEN, "Administrator access required".to_string()));
    }
    Ok(claims)
}

/// Caller identity from the verified principal. Suite subjects are the stable
/// UUID derived at login, matching the `user_id` used by the ACL table.
fn verified_user_id(claims: &VerifiedJwt) -> Result<Uuid, (StatusCode, String)> {
    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token subject".to_string()))
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn db_conn(
    state: &Arc<AppState>,
) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>, (StatusCode, String)>
{
    state.conn.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))
}

async fn list_connections(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, String)> {
    verified_admin(&headers)?;
    let mut conn = db_conn(&state)?;
    use schema::connector_connections::dsl::*;

    let rows: Vec<crate::models::ConnectionRow> = connector_connections
        .order(created_at.desc())
        .limit(200)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query connections: {e}")))?;

    let connections: Vec<Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id, "org_id": row.org_id, "kind": row.kind,
                "display_name": row.display_name, "status": row.status,
                "last_sync_at": row.last_sync_at,
                "has_credentials": !row.vault_token_ref.is_empty(),
                "created_at": row.created_at, "updated_at": row.updated_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "connections": connections })))
}

async fn create_connection(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<ConnectBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let admin = verified_admin(&headers)?;
    if registry::connector_for_kind(body.kind.trim()).is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Unsupported connector kind; available: {:?}", registry::registered_kinds()),
        ));
    }
    // Tenant scope comes from the verified token, never from the request body.
    let org = admin.org_id.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "Organization claim required".to_string())
    })?;
    if body.credentials.as_object().is_none() {
        return Err((StatusCode::BAD_REQUEST, "Credentials must be a JSON object".to_string()));
    }

    let connection_id = Uuid::new_v4();
    let vault_ref = format!("secret/gbo/connectors/{org}/{connection_id}");
    registry::store_credentials(&vault_ref, &body.credentials).await.map_err(|e| {
        tracing::error!("botconnectors: credential store failed on create: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to persist credentials".to_string())
    })?;

    let mut conn = db_conn(&state)?;
    use schema::connector_connections::dsl::*;
    diesel::insert_into(connector_connections)
        .values((
            id.eq(connection_id),
            org_id.eq(org),
            kind.eq(body.kind.trim()),
            display_name.eq(body.display_name.as_deref().filter(|d| !d.trim().is_empty())),
            vault_token_ref.eq(&vault_ref),
            status.eq("connected"),
            cursors.eq(serde_json::json!({})),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Create connection: {e}")))?;

    tracing::info!(target: "botconnectors", connection_id = %connection_id, kind = %body.kind, org_id = %org, "connector connection created");
    Ok(Json(serde_json::json!({
        "status": "created", "id": connection_id, "kind": body.kind.trim(),
    })))
}

async fn sync_connection_route(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(connection_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, String)> {
    verified_admin(&headers)?;
    let outcome = sync::sync_connection(&state, connection_id).await?;
    Ok(Json(serde_json::to_value(outcome)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Outcome encode failed".to_string()))?))
}

async fn delete_connection(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(connection_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, String)> {
    verified_admin(&headers)?;
    let mut conn = db_conn(&state)?;

    let purged = diesel::sql_query("DELETE FROM indexed_items WHERE connection_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Purge items: {e}")))?;

    let updated =
        diesel::sql_query(
            "UPDATE connector_connections SET status = 'disconnected', updated_at = NOW() WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Disconnect: {e}")))?;

    if updated == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Connector connection {connection_id} not found")));
    }

    if let Some(row) = load_vault_ref(&mut conn, connection_id) {
        registry::delete_credentials(&row).await;
    }

    Ok(Json(serde_json::json!({
        "status": "disconnected", "id": connection_id, "items_purged": purged as i64,
    })))
}

fn load_vault_ref(conn: &mut PgConnection, connection_id: Uuid) -> Option<String> {
    #[derive(diesel::QueryableByName)]
    struct RefRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        vault_token_ref: String,
    }
    diesel::sql_query("SELECT vault_token_ref FROM connector_connections WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .get_result::<RefRow>(conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.vault_token_ref)
}

async fn search_connectors(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let admin = verified_admin(&headers)?;
    let user_id = verified_user_id(&admin)?;

    let q = params.get("q").map(|s| s.trim().to_string()).unwrap_or_default();
    if q.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query parameter 'q' is required".to_string()));
    }
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .map(|l| l.clamp(1, MAX_SEARCH_LIMIT))
        .unwrap_or(DEFAULT_SEARCH_LIMIT);
    let sources: Option<Vec<String>> = match params.get("sources").map(|s| s.trim()) {
        Some(raw) if !raw.is_empty() => {
            Some(raw.split(',').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect())
        }
        _ => None,
    };

    let mut conn = db_conn(&state)?;
    let items: Vec<ItemRow> =
        acl::search_visible(&mut conn, user_id, &q, sources.clone(), limit)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Search failed: {e}")))?;

    let sources_hit: Vec<String> = sources.unwrap_or_else(|| {
        registry::registered_kinds().into_iter().map(|k| k.to_string()).collect()
    });
    audit::audit_query(&mut conn, Some(user_id), &sha256_hex(&q), &sources_hit, items.len());

    Ok(Json(serde_json::json!({
        "query": { "q_hash": sha256_hex(&q), "sources": sources_hit, "limit": limit },
        "count": items.len(),
        "items": items,
    })))
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/connectors", get(list_connections).post(create_connection))
        .route("/api/connectors/:id/sync", post(sync_connection_route))
        .route("/api/connectors/:id", delete(delete_connection))
        .route("/api/connectors/search", get(search_connectors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use hmac::{Hmac, Mac};

    const HS256_HEADER: &str = r#"{"alg":"HS256","typ":"JWT"}"#;
    const NONE_HEADER: &str = r#"{"alg":"none","typ":"JWT"}"#;
    const FOREIGN_SECRET: &str = "a-key-the-server-never-issued-0000000000000000";

    fn b64(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn sign(signing_input: &str, secret: &str) -> String {
        let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return String::new(),
        };
        mac.update(signing_input.as_bytes());
        b64(&mac.finalize().into_bytes())
    }

    fn token(header: &str, payload: &str, secret: &str) -> String {
        let signing_input = format!("{}.{}", b64(header.as_bytes()), b64(payload.as_bytes()));
        format!("{signing_input}.{}", sign(&signing_input, secret))
    }

    fn server_secret() -> String {
        botsecurity_core::verified_jwt::resolve_saas_jwt_secret()
            .expect("saas jwt secret resolves in tests")
    }

    fn header_with_token(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let value = format!("Bearer {token}");
        headers.insert(
            "authorization",
            axum::http::HeaderValue::from_str(&value).expect("token is a valid header value"),
        );
        headers
    }

    fn claims_payload(role: &str) -> String {
        format!(
            r#"{{"sub":"00000000-0000-0000-0000-000000000001","org_id":"00000000-0000-0000-0000-000000000002","roles":["{role}"],"exp":{}}}"#,
            chrono::Utc::now().timestamp() + 3600
        )
    }

    #[test]
    fn signed_admin_token_grants_admin_and_exposes_verified_claims() {
        let token = token(HS256_HEADER, &claims_payload("admin"), &server_secret());
        let headers = header_with_token(&token);
        let claims = verified_admin(&headers).expect("genuine admin token is accepted");
        assert!(claims.roles.iter().any(|role| role.eq_ignore_ascii_case("admin")));
        assert_eq!(
            claims.org_id,
            Some(Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("valid uuid"))
        );
        assert_eq!(
            verified_user_id(&claims).expect("uuid subject"),
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("valid uuid")
        );
    }

    #[test]
    fn unsigned_admin_payload_is_rejected() {
        // The pre-fix decoder accepted exactly this shape: any payload, with a
        // placeholder signature and no real signature at all.
        let token = format!(
            "{}.{}.c2ln",
            b64(NONE_HEADER.as_bytes()),
            b64(claims_payload("admin").as_bytes())
        );
        let headers = header_with_token(&token);
        let error = verified_admin(&headers).expect_err("alg:none admin token is refused");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn foreign_signed_admin_payload_is_rejected() {
        let token = token(HS256_HEADER, &claims_payload("admin"), FOREIGN_SECRET);
        let headers = header_with_token(&token);
        let error = verified_admin(&headers).expect_err("foreign-key admin token is refused");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn tampered_admin_payload_is_rejected() {
        let payload = claims_payload("admin");
        let signing_input =
            format!("{}.{}", b64(HS256_HEADER.as_bytes()), b64(payload.as_bytes()));
        let signature = sign(&signing_input, &server_secret());
        // Caller rewrites the signed payload but keeps the old signature.
        let tampered = payload.replace(r#""roles":["admin"]"#, r#""roles":["user"]"#);
        assert_ne!(tampered, payload);
        let token = format!("{}.{}.{signature}", b64(HS256_HEADER.as_bytes()), b64(tampered.as_bytes()));
        let headers = header_with_token(&token);
        let error = verified_admin(&headers).expect_err("tampered payload is refused");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn signed_non_admin_token_cannot_gain_admin() {
        let token = token(HS256_HEADER, &claims_payload("user"), &server_secret());
        let headers = header_with_token(&token);
        let error = verified_admin(&headers).expect_err("verified non-admin is refused");
        assert_eq!(error.0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn missing_token_is_rejected() {
        let error = verified_admin(&HeaderMap::new()).expect_err("missing token is refused");
        assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn hash_is_stable_hex() {
        assert_eq!(sha256_hex("abc").len(), 64);
        assert_eq!(sha256_hex("abc"), sha256_hex("abc"));
        assert_ne!(sha256_hex("abc"), sha256_hex("abd"));
    }
}
