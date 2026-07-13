use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::api::{get_branch_id_from_jwt, is_super_admin};
use crate::SaasService;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema_ext::bot_domains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BotDomain {
    pub id: Uuid,
    pub domain: String,
    pub bot_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDomainBody {
    pub domain: String,
    pub bot_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDomainBody {
    pub domain: Option<String>,
    pub bot_id: Option<Uuid>,
}

/// Resolve scope filter: admin sees all, user sees only their branch.
fn get_scope_filter(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<(bool, Option<Uuid>), (StatusCode, String)> {
    let admin = is_super_admin(headers, conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let user_branch = if admin {
        None
    } else {
        get_branch_id_from_jwt(headers, conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
    };
    Ok((admin, user_branch))
}

/// `GET /api/cloud/domains` — list domain mappings scoped to user's branch (or all for admin)
pub async fn list_domains(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (admin, user_branch) = get_scope_filter(&headers, &mut conn)?;

    use crate::schema_ext::bot_domains::dsl::{bot_domains, created_at, branch_id as br_id};

    let domains = if admin {
        bot_domains
            .order(created_at.desc())
            .load::<BotDomain>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query domains: {e}")))?
    } else if let Some(bid) = user_branch {
        bot_domains
            .filter(br_id.eq(bid))
            .order(created_at.desc())
            .load::<BotDomain>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query domains: {e}")))?
    } else {
        Vec::new()
    };

    Ok(Json(serde_json::json!({ "domains": domains })))
}

/// `POST /api/cloud/domains` — create a domain mapping
pub async fn create_domain(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Json(body): Json<CreateDomainBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (_admin, user_branch) = get_scope_filter(&headers, &mut conn)?;

    let domain = body.domain.trim().to_lowercase();
    if domain.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Domain is required".to_string()));
    }

    let now = Utc::now();
    let id = Uuid::new_v4();

    use crate::schema_ext::bot_domains::dsl::{bot_domains, id as did, domain as dcol, bot_id as bcol, org_id as ocol, branch_id as brcol, created_at as cat, updated_at as uat};

    diesel::insert_into(bot_domains)
        .values((
            did.eq(id),
            dcol.eq(&domain),
            bcol.eq(body.bot_id),
            ocol.eq(body.org_id),
            brcol.eq(body.branch_id.or(user_branch)),
            cat.eq(now),
            uat.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::CONFLICT, format!("Failed to create domain mapping: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "id": id,
        "domain": domain,
        "bot_id": body.bot_id,
    })))
}

/// `PUT /api/cloud/domains/{id}` — update a domain mapping
pub async fn update_domain(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Path(domain_id): Path<Uuid>,
    Json(body): Json<UpdateDomainBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (admin, user_branch) = get_scope_filter(&headers, &mut conn)?;

    let now = Utc::now();

    if let Some(ref d) = body.domain {
        let d = d.trim().to_lowercase();
        if !d.is_empty() {
            if admin {
                diesel::sql_query("UPDATE bot_domains SET domain = $1, updated_at = $2 WHERE id = $3")
                    .bind::<diesel::sql_types::Text, _>(&d)
                    .bind::<diesel::sql_types::Timestamptz, _>(now)
                    .bind::<diesel::sql_types::Uuid, _>(domain_id)
                    .execute(&mut conn)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update domain: {e}")))?;
            } else if let Some(bid) = user_branch {
                diesel::sql_query("UPDATE bot_domains SET domain = $1, updated_at = $2 WHERE id = $3 AND (branch_id = $4 OR branch_id IS NULL)")
                    .bind::<diesel::sql_types::Uuid, _>(bid)
                    .bind::<diesel::sql_types::Text, _>(&d)
                    .bind::<diesel::sql_types::Timestamptz, _>(now)
                    .bind::<diesel::sql_types::Uuid, _>(domain_id)
                    .execute(&mut conn)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update domain: {e}")))?;
            } else {
                return Err((StatusCode::FORBIDDEN, "Not authorized".to_string()));
            }
        }
    }

    if let Some(b) = body.bot_id {
        if admin {
            diesel::sql_query("UPDATE bot_domains SET bot_id = $1, updated_at = $2 WHERE id = $3")
                .bind::<diesel::sql_types::Uuid, _>(b)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .bind::<diesel::sql_types::Uuid, _>(domain_id)
                .execute(&mut conn)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update bot_id: {e}")))?;
        } else if let Some(bid) = user_branch {
            diesel::sql_query("UPDATE bot_domains SET bot_id = $1, updated_at = $2 WHERE id = $3 AND (branch_id = $4 OR branch_id IS NULL)")
                .bind::<diesel::sql_types::Uuid, _>(bid)
                .bind::<diesel::sql_types::Uuid, _>(b)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .bind::<diesel::sql_types::Uuid, _>(domain_id)
                .execute(&mut conn)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update bot_id: {e}")))?;
        } else {
            return Err((StatusCode::FORBIDDEN, "Not authorized".to_string()));
        }
    }

    Ok(Json(serde_json::json!({ "status": "updated" })))
}

/// `DELETE /api/cloud/domains/{id}` — delete a domain mapping
pub async fn delete_domain(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Path(domain_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    let (admin, user_branch) = get_scope_filter(&headers, &mut conn)?;

    let deleted = if admin {
        diesel::sql_query("DELETE FROM bot_domains WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(domain_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?
    } else if let Some(bid) = user_branch {
        diesel::sql_query("DELETE FROM bot_domains WHERE id = $1 AND (branch_id = $2 OR branch_id IS NULL)")
            .bind::<diesel::sql_types::Uuid, _>(domain_id)
            .bind::<diesel::sql_types::Uuid, _>(bid)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?
    } else {
        return Err((StatusCode::FORBIDDEN, "Not authorized".to_string()));
    };

    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Domain mapping not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// Try to match the host against a wildcard pattern like `%.example.com`.
fn wildcard_matches(pattern: &str, host: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix("\\\\*.") {
        host.ends_with(&format!(".{suffix}"))
    } else if let Some(suffix) = pattern.strip_prefix("*.") {
        host.ends_with(&format!(".{suffix}"))
    } else {
        false
    }
}

/// `GET /api/domains/resolve?host=<host>` — resolve hostname to bot name (public).
/// Tries exact match first, then wildcard match against `*.domain` patterns.
pub async fn resolve_domain(
    State(service): State<Arc<SaasService>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let host = params.get("host")
        .map(|h| h.trim().to_lowercase())
        .filter(|h| !h.is_empty())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing host parameter".to_string()))?;

    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::bot_domains::dsl::{bot_domains, domain as dcol, bot_id};

    #[derive(diesel::QueryableByName, Debug)]
    struct BotNameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    // Step 1: exact match
    let result: Option<(Uuid, Option<Uuid>, Option<Uuid>)> = bot_domains
        .filter(dcol.eq(&host))
        .select((bot_id, crate::schema_ext::bot_domains::dsl::org_id, crate::schema_ext::bot_domains::dsl::branch_id))
        .first::<(Uuid, Option<Uuid>, Option<Uuid>)>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query: {e}")))?;

    if let Some((bot_uuid, org_id, branch_id)) = result {
        let bot_name: Option<String> = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
            .get_result::<BotNameRow>(&mut conn)
            .ok()
            .map(|r| r.name);

        return match bot_name {
            Some(name) => Ok(Json(serde_json::json!({
                "found": true, "bot_id": bot_uuid, "bot_name": name,
                "org_id": org_id, "branch_id": branch_id,
                "match_type": "exact",
            }))),
            None => Ok(Json(serde_json::json!({
                "found": false, "error": "Bot not found for the mapped domain"
            }))),
        };
    }

    // Step 2: wildcard match for `*.domain` patterns — fetch all wildcard domains and test
    #[derive(diesel::QueryableByName, Debug)]
    struct WildcardRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        domain: String,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        b_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        org_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        branch_id: Option<Uuid>,
    }

    let wildcards: Vec<WildcardRow> = diesel::sql_query(
        "SELECT domain, bot_id, org_id, branch_id FROM bot_domains WHERE domain LIKE '\\\\*.%' OR domain LIKE '*.'",
    )
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Wildcard query: {e}")))?;

    for w in &wildcards {
        if wildcard_matches(&w.domain, &host) {
            let bot_name: Option<String> = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(w.b_id)
                .get_result::<BotNameRow>(&mut conn)
                .ok()
                .map(|r| r.name);

            return match bot_name {
                Some(name) => Ok(Json(serde_json::json!({
                    "found": true, "bot_id": w.b_id, "bot_name": name,
                    "org_id": w.org_id, "branch_id": w.branch_id,
                    "match_type": "wildcard",
                }))),
                None => break,
            };
        }
    }

    Ok(Json(serde_json::json!({ "found": false })))
}
