use super::*;

pub fn get_store() -> &'static PlanStore {
    STORE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}
/// Attach the database pool so mutations persist to `plan_snapshots`.
pub fn set_pool(pool: DbPool) {
    let _ = POOL.set(Some(pool));
}
/// Hydrate the in-memory store from any persisted plan snapshots. Fire-and-
/// forget: spawns a task so the route mount stays synchronous.
pub fn load_from_db() {
    let pool = match POOL.get().cloned().flatten() {
        Some(p) => p,
        None => return,
    };
    tokio::spawn(async move {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            plan_id: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            payload: serde_json::Value,
        }
        let loaded = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            diesel::sql_query("SELECT plan_id, payload FROM plan_snapshots")
                .load::<Row>(&mut conn)
                .map_err(|e| e.to_string())
        })
        .await;

        let rows = match loaded {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                log::warn!("plan load failed: {e}");
                return;
            }
            Err(e) => {
                log::warn!("plan load join failed: {e}");
                return;
            }
        };

        let mut store = get_store().write().await;
        for row in rows {
            match serde_json::from_value::<Plan>(row.payload) {
                Ok(plan) => {
                    store.entry(row.plan_id).or_insert(plan);
                }
                Err(e) => log::warn!("plan snapshot parse failed: {e}"),
            }
        }
    });
}
/// Persist every plan to `plan_snapshots` (one JSONB row per plan). Called
/// after each mutation; failures are logged, never fatal.
///
/// The JSONB payload is the source of truth; `owner_id`/`org_id`/`branch_id`
/// are projected into their own columns (issue #2136) so ownership and tenant
/// scope can be audited with SQL without unpacking the document.
pub async fn persist_all() {
    let pool = match POOL.get().cloned().flatten() {
        Some(p) => p,
        None => return,
    };
    let snapshot: Vec<(String, serde_json::Value)> = {
        let store = get_store().read().await;
        store
            .iter()
            .filter_map(|(id, plan)| serde_json::to_value(plan).ok().map(|v| (id.clone(), v)))
            .collect()
    };
    let res = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        for (plan_id, payload) in snapshot {
            diesel::sql_query(
                "INSERT INTO plan_snapshots \
                     (plan_id, payload, owner_id, org_id, branch_id, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, NOW()) \
                 ON CONFLICT (plan_id) DO UPDATE SET payload = EXCLUDED.payload, \
                     owner_id = EXCLUDED.owner_id, org_id = EXCLUDED.org_id, \
                     branch_id = EXCLUDED.branch_id, updated_at = NOW()",
            )
            .bind::<diesel::sql_types::Text, _>(&plan_id)
            .bind::<diesel::sql_types::Jsonb, _>(payload.clone())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(json_text(
                &payload, "owner_id",
            ))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(json_uuid(
                &payload, "org_id",
            ))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(json_uuid(
                &payload, "branch_id",
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    })
    .await;

    if let Err(e) = res {
        log::warn!("plan persist_all failed: {e}");
    }
}

fn json_text(payload: &serde_json::Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn json_uuid(payload: &serde_json::Value, key: &str) -> Option<Uuid> {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
}

/// Clones the plan with `id`, if the store holds it. No identity check here:
/// callers must go through [`crate::handlers`], which authorize first.
pub async fn load_plan(plan_id: &str) -> Option<Plan> {
    let store = get_store().read().await;
    store.get(plan_id).cloned()
}

/// Removes the persisted snapshot of a deleted plan, so a restart does not
/// resurrect it (`persist_all` only upserts what the store still holds).
pub async fn delete_snapshot(plan_id: &str) {
    let pool = match POOL.get().cloned().flatten() {
        Some(p) => p,
        None => return,
    };
    let plan_id = plan_id.to_string();
    let res = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        diesel::sql_query("DELETE FROM plan_snapshots WHERE plan_id = $1")
            .bind::<diesel::sql_types::Text, _>(&plan_id)
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await;
    if let Err(e) = res {
        log::warn!("plan snapshot delete failed: {e}");
    }
}
