use super::*;


/// Catalogue monthly prices in cents, keyed by SKU. Used to reconcile what a
/// provider actually charged against what the customer was quoted.
pub(crate) fn store_item_monthly_cents(store_item_id: &str) -> Option<i64> {
    match store_item_id {
        "vps-small" => Some(999),
        "vps-medium" => Some(1999),
        "vps-large" => Some(3999),
        "vps-xl" => Some(7999),
        "gpu-basic" => Some(3999),
        "gpu-pro" => Some(9999),
        "gpu-enterprise" => Some(29999),
        _ => None,
    }
}

/// Provisions whatever kind of resource the catalogue item represents.
///
/// Compute goes through the provider chain; a domain SKU is provisioned by the
/// registrar path (#1469); anything else is recorded without an external call so
/// the resource does not sit on "provisioning" forever.
pub(crate) async fn provision_resource(
    pool: &crate::DbPool,
    store_item_id: &str,
    resource_id: Uuid,
    workspace_id: Uuid,
    org_id: Uuid,
) -> Result<(), String> {
    if store_item_id.starts_with("domain-") {
        return crate::domain_provisioning::provision_domain(
            pool,
            store_item_id,
            resource_id,
            workspace_id,
            org_id,
        )
        .await;
    }
    if crate::compute_provisioning::sku_for(store_item_id).is_some() {
        return provision_compute_resource(pool, store_item_id, resource_id, workspace_id, org_id).await;
    }

    // No provisioning path for this item yet: mark it active rather than leaving
    // a row that reports "provisioning" indefinitely.
    crate::compute_provisioning::record_outcome(
        pool,
        resource_id,
        "active",
        serde_json::json!({ "note": "no external provisioning required" }),
    )
}

/// Provisions a compute resource off the request path.
///
/// The work happens in a spawned task, so this records the outcome on the
/// resource row and returns; the client polls `GET .../resources` for the
/// status. Every policy decision — candidate chain, credentials, cost
/// reconciliation — lives in [`crate::compute_provisioning`].
pub(crate) async fn provision_compute_resource(
    pool: &crate::DbPool,
    store_item_id: &str,
    resource_id: Uuid,
    _workspace_id: Uuid,
    org_id: Uuid,
) -> Result<(), String> {
    use crate::compute_provisioning as compute;

    let sku = compute::sku_for(store_item_id)
        .ok_or_else(|| format!("Unknown store item: {store_item_id}"))?;

    let settings = compute::load_settings(pool, org_id)?;

    // A key on any candidate is enough to try; without one there is nothing to
    // attempt, and the resource must say so instead of sitting on "provisioning".
    if !sku
        .candidates
        .iter()
        .any(|name| settings.key_for(name).is_some())
    {
        let _ = compute::record_outcome(
            pool,
            resource_id,
            compute::STATUS_NO_KEY,
            serde_json::json!({ "error": "no provider credential for this organization" }),
        );
        return Err("No provider API key configured for organization".into());
    }

    // Region is a deployment-wide default today; threading a per-request region
    // through the assign endpoint is a follow-up, not a silent assumption.
    let region = std::env::var("GB_COMPUTE_REGION").unwrap_or_else(|_| "US".into());

    let mut outcome = compute::provision_with_fallback(&sku, &region, &settings).await?;
    outcome.cost_variance_pct = store_item_monthly_cents(store_item_id)
        .map(|cents| cents as f64 / 100.0)
        .and_then(|monthly| {
            compute::reconcile_cost(outcome.result.hourly_cost, monthly)
        });
    if let Some(variance) = outcome.cost_variance_pct {
        tracing::warn!(
            "Provider {} billed {variance:.1}% above the catalogue price for {store_item_id}",
            outcome.provider_name
        );
    }

    compute::record_outcome(
        pool,
        resource_id,
        "active",
        compute::success_config(&outcome),
    )
}

/// `DELETE /api/cloud/organizations/{org_id}/workspaces/{ws_id}/resources/{res_id}`
pub(crate) async fn remove_resource(
    State(service): State<Arc<SaasService>>,
    axum::extract::Path((_org_id, _ws_id, res_id_param)): axum::extract::Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = service.pool().get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {e}")))?;

    use crate::schema_ext::workspace_resources::dsl as wr;
    diesel::delete(wr::workspace_resources.filter(wr::id.eq(res_id_param)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete: {e}")))?;

    Ok(Json(serde_json::json!({ "status": "removed" })))
}

// ─────────────────────────────────────────────────────────────────────────────
// Branches per organization
