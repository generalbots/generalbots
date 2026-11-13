use super::*;

pub async fn create_product(
    State(state): State<Arc<ProductsState>>,
    headers: HeaderMap,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, ApiError> {
    let locale = locale_from_headers(&headers);
    // Resolve the tenant branch before the pool: an anonymous or unverifiable
    // caller is rejected with `401` and never reaches the insert. There is no
    // default-branch fallback — a write may only land in the caller's own
    // catalogue (issue #2128).
    let branch_id = verified_branch(&headers, &state.pool).ok_or_else(|| unauthorized(&locale))?;
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let product = Product {
        id,
        org_id: None,
        bot_id: None,
        sku: req.sku,
        name: req.name,
        description: req.description,
        category: req.category,
        product_type: req.product_type.unwrap_or_else(|| "physical".to_string()),
        price: bd(req.price),
        cost: req.cost.map(bd),
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        tax_rate: bd(req.tax_rate.unwrap_or(0.0)),
        unit: req.unit.unwrap_or_else(|| "unit".to_string()),
        stock_quantity: req.stock_quantity.unwrap_or(0),
        low_stock_threshold: req.low_stock_threshold.unwrap_or(10),
        is_active: true,
        images: serde_json::json!(req.images.unwrap_or_default()),
        attributes: serde_json::json!({}),
        created_at: now,
        updated_at: now,
        branch_id,
        category_id: None,
        is_public: false,
    };

    diesel::insert_into(products::table)
        .values(&product)
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(product))
}
pub async fn list_products(
    State(state): State<Arc<ProductsState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let has_token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("Bearer ") && v.len() > 7)
        .unwrap_or(false);

    // Anonymous storefront callers (no token) may only see the public global
    // catalog (branch nil, `is_public` rows); authenticated callers get their
    // own tenant branch. A token that resolves to no branch reads the same
    // global catalog: a suite session (opaque token, no SaaS `branch_id`
    // claim, no CRM contact row) used to get `401` here while
    // `/api/products/stats` answered `200` for the very same token, so the
    // Products app grid rendered "No products yet" against a seeded catalogue
    // (#2204). Reads stay scoped — never another tenant's branch — and writes
    // still fail closed in `create_product` (issue #2128).
    let branch_id = if has_token {
        verified_branch(&headers, &state.pool).unwrap_or_else(Uuid::nil)
    } else {
        Uuid::nil()
    };
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = products::table
        .filter(products::branch_id.eq(branch_id))
        .into_boxed();

    if !has_token {
        q = q.filter(products::is_public.eq(true))
            .filter(products::is_active.eq(true));
    }

    if let Some(is_active) = query.is_active {
        q = q.filter(products::is_active.eq(is_active));
    }

    if let Some(category) = query.category {
        q = q.filter(products::category.eq(category));
    }

    if let Some(true) = query.low_stock {
        q = q.filter(products::stock_quantity.le(products::low_stock_threshold));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            products::name
                .ilike(pattern.clone())
                .or(products::sku.ilike(pattern.clone()))
                .or(products::description.ilike(pattern)),
        );
    }

    let prods: Vec<Product> = q
        .order(products::name.asc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Anonymous storefront callers get display fields only (issue #2128):
    // cost, stock and the internal scope ids never leave the server. The
    // authenticated products app keeps the full row.
    if !has_token {
        return Ok(Json(prods.iter().map(public_product).collect()));
    }

    Ok(Json(prods.iter().map(|product| serde_json::json!(product)).collect()))
}

/// Display-only projection of a catalogue row served to anonymous callers.
pub(crate) fn public_product(product: &Product) -> serde_json::Value {
    serde_json::json!({
        "sku": product.sku,
        "name": product.name,
        "description": product.description,
        "category": product.category,
        "product_type": product.product_type,
        "price": product.price,
        "currency": product.currency,
        "unit": product.unit,
        "images": product.images,
        "attributes": product.attributes,
        "is_active": product.is_active,
    })
}
pub async fn get_product(
    State(state): State<Arc<ProductsState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductWithVariants>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Object-level authorization (issue #734): a product may only be read by
    // a caller whose workspace branch owns it.
    let branch_id = verified_branch(&headers, &state.pool).ok_or_else(|| unauthorized(&locale))?;

    let product: Product = products::table
        .filter(products::id.eq(id))
        .filter(products::branch_id.eq(branch_id))
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-product-not-found"))?;

    let variants: Vec<ProductVariant> = product_variants::table
        .filter(product_variants::product_id.eq(id))
        .order(product_variants::name.asc())
        .load(&mut conn)
        .unwrap_or_default();

    Ok(Json(ProductWithVariants { product, variants }))
}
pub async fn update_product(
    State(state): State<Arc<ProductsState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<Product>, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Object-level authorization: only the owning branch may modify a product.
    let branch_id = verified_branch(&headers, &state.pool).ok_or_else(|| unauthorized(&locale))?;
    let now = Utc::now();

    diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
        .set(products::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if let Some(name) = req.name {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(description) = req.description {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::description.eq(description))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(price) = req.price {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::price.eq(bd(price)))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(stock_quantity) = req.stock_quantity {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::stock_quantity.eq(stock_quantity))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(is_active) = req.is_active {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::is_active.eq(is_active))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(category) = req.category {
        diesel::update(products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))))
            .set(products::category.eq(category))
            .execute(&mut conn)
            .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    let product: Product = products::table
        .filter(products::id.eq(id))
        .filter(products::branch_id.eq(branch_id))
        .first(&mut conn)
        .map_err(|_| api_error(&locale, StatusCode::NOT_FOUND, "error-product-not-found"))?;

    Ok(Json(product))
}
pub async fn delete_product(
    State(state): State<Arc<ProductsState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let locale = locale_from_headers(&headers);
    let mut conn = state
        .pool
        .get()
        .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Object-level authorization: only the owning branch may delete a product.
    let branch_id = verified_branch(&headers, &state.pool).ok_or_else(|| unauthorized(&locale))?;

    let deleted = diesel::delete(
        products::table.filter(products::id.eq(id).and(products::branch_id.eq(branch_id))),
    )
    .execute(&mut conn)
    .map_err(|e| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if deleted == 0 {
        return Err(api_error(&locale, StatusCode::NOT_FOUND, "error-product-not-found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
