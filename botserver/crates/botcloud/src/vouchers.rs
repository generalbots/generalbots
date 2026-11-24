use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use diesel::prelude::*;

use crate::api::{base64_url_decode, get_branch_id_from_jwt, is_super_admin};
use crate::SaasService;

mod policy;
pub(crate) use policy::{
    generate_voucher_code, voucher_code_with, redemption_rejection, redemption_scope,
    redemption_matches_branch, insert_fields_are_branch_scoped,
    TRIAL_DAYS_RANGE, TRIAL_SUBSCRIPTION_COLUMNS,
};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema_ext::cloud_vouchers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Voucher {
    pub id: Uuid,
    pub code: String,
    pub plan: String,
    pub trial_days: i32,
    pub max_uses: i32,
    pub uses_count: i32,
    pub created_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema_ext::cloud_voucher_redemptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VoucherRedemption {
    pub id: Uuid,
    pub voucher_id: Uuid,
    pub contact_id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub trial_days: i32,
    pub redeemed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoucherRequest {
    pub trial_days: i32,
    pub plan: Option<String>,
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RedeemVoucherRequest {
    pub code: String,
}

/// `POST /api/cloud/vouchers`
pub async fn create_voucher(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Json(body): Json<CreateVoucherRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if !is_super_admin(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
    {
        return Err((StatusCode::FORBIDDEN, "Only the SaaS owner can manage vouchers".to_string()));
    }

    let creator_id = if let Some(auth_val) = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let parts: Vec<&str> = auth_val.split('.').collect();
        if parts.len() == 3 {
            if let Ok(decoded) = base64_url_decode(parts[1]) {
                if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded) {
                    if let Some(user_email) = payload.get("email").and_then(|v| v.as_str()) {
                        use crate::schema_ext::crm_contacts::dsl::{crm_contacts, email, id};
                        crm_contacts.filter(email.eq(user_email)).select(id).first::<Uuid>(&mut conn).ok()
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    } else { None };

    let trial_days = body.trial_days;
    if !TRIAL_DAYS_RANGE.contains(&trial_days) {
        return Err((StatusCode::BAD_REQUEST, "Trial days must be between 1 and 180".to_string()));
    }

    let plan = body.plan.unwrap_or_else(|| "shared".to_string());
    let max_uses = body.max_uses.unwrap_or(1);
    let code = generate_voucher_code(trial_days);
    let id = Uuid::new_v4();
    let now = Utc::now();

    use crate::schema_ext::cloud_vouchers::dsl::{cloud_vouchers, id as vid, code as vcode, plan as vplan, trial_days as vtd, max_uses as vmu, uses_count as vuc, created_by as vcb, expires_at as vex, created_at as vcat};
    
    diesel::insert_into(cloud_vouchers)
        .values((
            vid.eq(id),
            vcode.eq(&code),
            vplan.eq(&plan),
            vtd.eq(trial_days),
            vmu.eq(max_uses),
            vuc.eq(0),
            vcb.eq(creator_id),
            vex.eq(body.expires_at),
            vcat.eq(now)
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert voucher failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "id": id,
        "code": code,
        "plan": plan,
        "trial_days": trial_days,
        "max_uses": max_uses,
        "expires_at": body.expires_at.map(|e| e.to_rfc3339()),
    })))
}

/// `GET /api/cloud/vouchers`
pub async fn list_vouchers(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    if !is_super_admin(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
    {
        return Err((StatusCode::FORBIDDEN, "Only the SaaS owner can manage vouchers".to_string()));
    }

    use crate::schema_ext::cloud_vouchers::dsl::{cloud_vouchers, created_at};
    
    let list = cloud_vouchers
        .order(created_at.desc())
        .limit(100)
        .load::<Voucher>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query vouchers failed: {e}")))?;

    Ok(Json(serde_json::json!({ "vouchers": list })))
}

/// `POST /api/cloud/vouchers/redeem`
pub async fn redeem_voucher(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Json(body): Json<RedeemVoucherRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // 1. Authenticate user from JWT and get their branch/org/contact context
    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No branch associated with authenticated user".to_string()))?;

    // Look up the actual org_id (organization) from branch_id
    #[derive(diesel::QueryableByName)]
    struct OrgIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let actual_org_id: Uuid = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_branch_id)
        .get_result::<OrgIdRow>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Lookup org from branch: {e}")))?
        .org_id;

    // #2868 — resolve the exact (org, branch) scope once. Every row written
    // below shares it; a nil branch can never become a tenant scope.
    let (scope_org_id, scope_branch_id) = redemption_scope(actual_org_id, user_branch_id)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid redemption scope".to_string()))?;

    // Get contact and organization details from crm_contacts
    let (ctct_id, _, ctct_name, ctct_email) = if let Some(auth_val) = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let parts: Vec<&str> = auth_val.split('.').collect();
        if parts.len() == 3 {
            if let Ok(decoded_bytes) = base64_url_decode(parts[1]) {
                if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded_bytes) {
                    if let Some(user_email) = payload.get("email").and_then(|v| v.as_str()) {
                        use crate::schema_ext::crm_contacts::dsl::{crm_contacts as cc_tbl, email as ce, id as cid, branch_id as c_branch_id, first_name as cfn, last_name as cln};
                        let c: (Uuid, Uuid, Option<String>, Option<String>, Option<String>) = cc_tbl
                            .filter(ce.eq(user_email))
                            .select((cid, c_branch_id, cfn, cln, ce))
                            .first(&mut conn)
                            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Lookup user contact: {e}")))?;

                        let name = format!("{} {}", c.2.as_deref().unwrap_or(""), c.3.as_deref().unwrap_or("")).trim().to_string();
                        (c.0, c.1, if name.is_empty() { "SaaS User".to_string() } else { name }, c.4.unwrap_or_else(String::new))
                    } else { return Err((StatusCode::UNAUTHORIZED, "Invalid claims".to_string())); }
                } else { return Err((StatusCode::UNAUTHORIZED, "Invalid JWT".to_string())); }
            } else { return Err((StatusCode::UNAUTHORIZED, "Invalid payload encoding".to_string())); }
        } else { return Err((StatusCode::UNAUTHORIZED, "Malformed token".to_string())); }
    } else {
        return Err((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()));
    };

    // 2. Lookup voucher code
    use crate::schema_ext::cloud_vouchers::dsl::{cloud_vouchers as cv_tbl, code as vcode, id as vid, uses_count as vuc};
    let voucher: Voucher = cv_tbl
        .filter(vcode.eq(&body.code))
        .first::<Voucher>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Voucher query: {e}")))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Voucher code not found".to_string()))?;

    // 3. Validation checks — one pure policy function, so the endpoint and the
    // tests can never disagree about precedence (#2069 CLD-8).
    use crate::schema_ext::cloud_voucher_redemptions::dsl::{cloud_voucher_redemptions as cvr_tbl, voucher_id as cvid, contact_id as cctid};
    let already_redeemed = cvr_tbl
        .filter(cvid.eq(voucher.id))
        .filter(cctid.eq(ctct_id))
        .first::<VoucherRedemption>(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Check redemption: {e}")))?;

    // #2868 — a voucher already redeemed by this contact under a *different*
    // branch must not grant a second trial here (cross-tenant replay).
    if let Some(existing) = &already_redeemed {
        if !redemption_matches_branch(Some(existing.branch_id), scope_branch_id) {
            return Err((
                StatusCode::BAD_REQUEST,
                "Voucher already redeemed in another branch".to_string(),
            ));
        }
    }

    if let Some(reason) = redemption_rejection(
        voucher.uses_count,
        voucher.max_uses,
        voucher.expires_at,
        already_redeemed.is_some(),
        Utc::now(),
    ) {
        return Err((StatusCode::BAD_REQUEST, reason.to_string()));
    }

    // 4. Create trial subscription
    // Generate a default bot ID for the user's branch (first default bot of the branch)
    #[derive(diesel::QueryableByName)]
    struct BotIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let bot_id = diesel::sql_query("SELECT id FROM bots WHERE branch_id = $1 AND is_default_for_branch = true LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(user_branch_id)
        .get_result::<BotIdRow>(&mut conn)
        .map(|r| r.id)
        .unwrap_or_else(|_| Uuid::new_v4());

    let sub_id = Uuid::new_v4();
    let now = Utc::now();
    let trial_end_date = now.date_naive() + Duration::days(voucher.trial_days as i64);

    // 4. Create the branch-scoped trial subscription. `org_id` is the real
    // organization and `branch_id` is the verified caller branch (NOT NULL),
    // so branch-scoped subscription handlers can see the trial (#2868).
    if !insert_fields_are_branch_scoped(TRIAL_SUBSCRIPTION_COLUMNS) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Trial INSERT is missing branch scope".to_string(),
        ));
    }
    let insert_sql = format!(
        "INSERT INTO billing_recurring ({}) VALUES ($1, $2, $3, $4, $5, $6, 'trialing', $7, $8, $9, $10, $11, $12, $13, NULL, 0, $14, $14)",
        TRIAL_SUBSCRIPTION_COLUMNS.join(", ")
    );

    // All three writes commit atomically: a failed redemption insert must not
    // leave a trial row or an incremented use counter behind (#2868).
    conn.build_transaction()
        .read_write()
        .run::<_, diesel::result::Error, _>(|tx| {
            diesel::sql_query(insert_sql.as_str())
                .bind::<diesel::sql_types::Uuid, _>(sub_id)
                .bind::<diesel::sql_types::Uuid, _>(scope_org_id)
                .bind::<diesel::sql_types::Uuid, _>(scope_branch_id)
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .bind::<diesel::sql_types::Text, _>(&ctct_name)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(ctct_email))
                .bind::<diesel::sql_types::Text, _>("monthly")
                .bind::<diesel::sql_types::Int4, _>(1)
                .bind::<diesel::sql_types::Numeric, _>(botbilling::api_models::bd(0.0))
                .bind::<diesel::sql_types::Text, _>("USD")
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(format!(
                    "Voucher {} - {} Days Trial",
                    voucher.code, voucher.trial_days
                )))
                .bind::<diesel::sql_types::Date, _>(trial_end_date)
                .bind::<diesel::sql_types::Date, _>(now.date_naive())
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .execute(tx)?;

            // 5. Register redemption record — same verified scope.
            diesel::insert_into(cvr_tbl)
                .values((
                    crate::schema_ext::cloud_voucher_redemptions::dsl::voucher_id.eq(voucher.id),
                    crate::schema_ext::cloud_voucher_redemptions::dsl::contact_id.eq(ctct_id),
                    crate::schema_ext::cloud_voucher_redemptions::dsl::org_id.eq(scope_org_id),
                    crate::schema_ext::cloud_voucher_redemptions::dsl::branch_id
                        .eq(scope_branch_id),
                    crate::schema_ext::cloud_voucher_redemptions::dsl::subscription_id
                        .eq(Some(sub_id)),
                    crate::schema_ext::cloud_voucher_redemptions::dsl::trial_days.eq(voucher.trial_days),
                ))
                .execute(tx)?;

            // 6. Conditional increment under the write transaction (#2867).
            // A stale `voucher.uses_count + 1` let two concurrent redemptions
            // both pass the max_uses check and both write N+1. The predicate
            // is re-evaluated atomically by the database; zero rows means a
            // concurrent redeemer consumed the last slot, so roll back the
            // trial and redemption rows written above.
            let updated = diesel::update(
                cv_tbl
                    .filter(vid.eq(voucher.id))
                    .filter(vuc.lt(voucher.max_uses)),
            )
            .set(vuc.eq(vuc + 1))
            .execute(tx)?;
            if updated != 1 {
                return Err(diesel::result::Error::RollbackTransaction);
            }
            Ok(())
        })
        .map_err(|e| match e {
            diesel::result::Error::RollbackTransaction => (
                StatusCode::CONFLICT,
                "Voucher fully redeemed by a concurrent request".to_string(),
            ),
            other => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redemption failed: {other}"),
            ),
        })?;

    Ok(Json(serde_json::json!({
        "status": "redeemed",
        "voucher": voucher.code,
        "subscription": {
            "id": sub_id,
            "plan": voucher.plan,
            "status": "trialing",
            "trial_days": voucher.trial_days,
            "trial_end": trial_end_date.to_string(),
        }
    })))
}

/// `GET /api/cloud/vouchers/my`
pub async fn get_my_redemptions(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    // Authenticate user from JWT
    let user_branch_id = get_branch_id_from_jwt(&headers, &mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No branch associated with authenticated user".to_string()))?;

    use crate::schema_ext::cloud_voucher_redemptions::dsl::{cloud_voucher_redemptions, branch_id, redeemed_at};
    
    let redemptions = cloud_voucher_redemptions
        .filter(branch_id.eq(user_branch_id))
        .order(redeemed_at.desc())
        .load::<VoucherRedemption>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query redemptions failed: {e}")))?;

    Ok(Json(serde_json::json!({ "redemptions": redemptions })))
}

#[cfg(test)]
#[path = "vouchers_tests.rs"]
mod vouchers_tests;
