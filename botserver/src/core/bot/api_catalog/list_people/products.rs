use super::super::*;
use diesel::OptionalExtension;
use uuid::Uuid;

/// Lists products/services (branch-scoped, read-only).
pub(super) async fn list_products(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    category: Option<&str>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    // A suite/default bot often has no tenant branch catalog; fall back to the
    // global (nil) catalog so chat lists the same seeded products the Store/HTTP
    // reads (suite token -> nil branch) instead of answering empty (#2204).
    let nil = Uuid::nil();
    let rows: Vec<ProductRow> = match category.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(cat) => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE (branch_id = $1 OR ($1 <> $3 AND branch_id = $3)) AND category = $2 \
             ORDER BY (branch_id = $1) DESC, name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Text, _>(cat)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
        None => diesel::sql_query(
            "SELECT id AS product_id, sku, name, price, product_type, stock_quantity FROM products \
             WHERE branch_id = $1 OR ($1 <> $2 AND branch_id = $2) \
             ORDER BY (branch_id = $1) DESC, name ASC LIMIT 20",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .load(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?,
    };
    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "product_id": r.product_id,
                "sku": r.sku.unwrap_or_default(),
                "name": r.name,
                "price": r.price.to_string(),
                "product_type": r.product_type,
                "stock_quantity": r.stock_quantity,
                "deep_link": format!("app://products?product_id={}", r.product_id),
            })
        })
        .collect();
    Ok(json!({ "count": items.len(), "products": items }))
}

/// Creates a product in the catalog (chat command, branch-scoped).
pub(super) async fn create_product_item(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "params.name is required".to_string())?;
    let price = params
        .get("price")
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .ok_or_else(|| "params.price is required".to_string())?;
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let sku = params
        .get("sku")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let description = params
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let category = params
        .get("category")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let currency = params
        .get("currency")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("USD");
    let product_type = params
        .get("product_type")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("physical");
    let unit = params
        .get("unit")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("unit");
    let stock_quantity = params
        .get("stock_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    let cost = params.get("cost").and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    });

    diesel::sql_query(
        "INSERT INTO products (id, org_id, bot_id, sku, name, description, category, product_type, \
         price, cost, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, \
         images, attributes, created_at, updated_at, branch_id, category_id, is_public) \
         VALUES ($1, NULL, NULL, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9, 0, $10, $11, 10, \
         true, '[]'::jsonb, '{}'::jsonb, $12, $12, $13, NULL, false)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(sku.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Text, _>(name.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(category.map(|s| s.to_string()))
    .bind::<diesel::sql_types::Text, _>(product_type.to_string())
    .bind::<diesel::sql_types::Text, _>(price.to_string())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(cost.map(|c| c.to_string()))
    .bind::<diesel::sql_types::Text, _>(currency.to_string())
    .bind::<diesel::sql_types::Text, _>(unit.to_string())
    .bind::<diesel::sql_types::Int4, _>(stock_quantity)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(|e| format!("Insert error: {e}"))?;

    Ok(json!({
        "success": true,
        "product_id": id,
        "name": name,
        "price": price,
        "deep_link": format!("app://products?product_id={id}"),
    }))
}

/// Updates a product in the catalog (chat command, branch-scoped).
pub(super) async fn update_product_item(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let nil = Uuid::nil();
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let product_id = params
        .get("product_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s.trim()).ok());
    let lookup_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    #[derive(diesel::QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }

    let target_id: Uuid = if let Some(pid) = product_id {
        pid
    } else if let Some(ref n) = lookup_name {
        // Prefer exact/ilike match in bot branch, then global nil branch.
        let pattern = format!("%{n}%");
        let row: Option<IdRow> = diesel::sql_query(
            "SELECT id FROM products \
             WHERE (branch_id = $1 OR ($1 <> $2 AND branch_id = $2)) AND name ILIKE $3 \
             ORDER BY (branch_id = $1) DESC, (lower(name) = lower($4)) DESC, name ASC LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .bind::<diesel::sql_types::Text, _>(pattern)
        .bind::<diesel::sql_types::Text, _>(n.clone())
        .get_result(&mut conn)
        .optional()
        .map_err(|e| format!("Query error: {e}"))?;
        row.map(|r| r.id)
            .ok_or_else(|| format!("product not found: {n}"))?
    } else {
        return Err("params.product_id or params.name is required".to_string());
    };

    let now = chrono::Utc::now();
    let mut updates = 0_i64;

    if let Some(price) = params.get("price").and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    }) {
        updates += diesel::sql_query(
            "UPDATE products SET price = $1::numeric, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Text, _>(price.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    if let Some(desc) = params.get("description").and_then(|v| v.as_str()) {
        updates += diesel::sql_query(
            "UPDATE products SET description = $1, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Text, _>(desc.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    if let Some(active) = params.get("is_active").and_then(|v| v.as_bool()) {
        updates += diesel::sql_query(
            "UPDATE products SET is_active = $1, updated_at = $2 \
             WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
        )
        .bind::<diesel::sql_types::Bool, _>(active)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))? as i64;
    }
    // Rename: if name provided together with product_id, treat as new name.
    if product_id.is_some() {
        if let Some(new_name) = params
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            updates += diesel::sql_query(
                "UPDATE products SET name = $1, updated_at = $2 \
                 WHERE id = $3 AND (branch_id = $4 OR branch_id = $5)",
            )
            .bind::<diesel::sql_types::Text, _>(new_name.to_string())
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(target_id)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Uuid, _>(nil)
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))? as i64;
        }
    }

    if updates == 0 {
        // Touch updated_at to confirm row exists/reachable when only lookup name matched without field changes.
        let touched = diesel::sql_query(
            "UPDATE products SET updated_at = $1 WHERE id = $2 AND (branch_id = $3 OR branch_id = $4)",
        )
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(target_id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(nil)
        .execute(&mut conn)
        .map_err(|e| format!("Update error: {e}"))?;
        if touched == 0 {
            return Err("product not found or not in scope".to_string());
        }
    }

    Ok(json!({
        "success": true,
        "product_id": target_id,
        "deep_link": format!("app://products?product_id={target_id}"),
    }))
}
