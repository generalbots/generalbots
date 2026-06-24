use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::trace;
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

/// Inventory and POS BASIC keywords for issue #620.
///
/// Provides: CREATE PRODUCT, ADD VARIATION, SET STOCK, TRANSFER STOCK,
/// GET STOCK, SET PRICE, OPEN POS SESSION, ADD TO CART, CHECKOUT.
pub fn register_inventory_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_create_product(state.clone(), user.clone(), engine);
    register_add_variation(state.clone(), user.clone(), engine);
    register_set_stock(state.clone(), user.clone(), engine);
    register_transfer_stock(state.clone(), user.clone(), engine);
    register_get_stock(state.clone(), user.clone(), engine);
    register_set_price(state.clone(), user.clone(), engine);
    register_open_pos_session(state.clone(), user.clone(), engine);
    register_add_to_cart(state.clone(), user.clone(), engine);
    register_checkout(state, user, engine);
}

fn register_create_product(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["CREATE", "PRODUCT", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let sku = context.eval_expression_tree(&inputs[1])?.to_string();
                let price_expr = context.eval_expression_tree(&inputs[2])?.to_string();
                let price: f64 = price_expr.parse().unwrap_or(0.0);
                trace!("CREATE PRODUCT: {name} sku={sku} price={price}");
                let result = json!({
                    "kind": "product",
                    "action": "create",
                    "name": name,
                    "sku": sku,
                    "price": price,
                    "is_active": true,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid CREATE PRODUCT syntax");
}

fn register_add_variation(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            [
                "ADD", "VARIATION", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let product_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let sku = context.eval_expression_tree(&inputs[1])?.to_string();
                let name = context.eval_expression_tree(&inputs[2])?.to_string();
                let price_expr = context.eval_expression_tree(&inputs[3])?.to_string();
                let price: f64 = price_expr.parse().unwrap_or(0.0);
                trace!("ADD VARIATION: {name} sku={sku} to {product_id}");
                let result = json!({
                    "kind": "product_variation",
                    "action": "create",
                    "product_id": product_id,
                    "sku": sku,
                    "name": name,
                    "price": price,
                    "is_active": true,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid ADD VARIATION syntax");
}

fn register_set_stock(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["SET", "STOCK", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let product_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let branch_id = context.eval_expression_tree(&inputs[1])?.to_string();
                let qty_expr = context.eval_expression_tree(&inputs[2])?.to_string();
                let qty: f64 = qty_expr.parse().unwrap_or(0.0);
                trace!("SET STOCK: product={product_id} branch={branch_id} qty={qty}");
                let result = json!({
                    "kind": "product_stock",
                    "action": "upsert",
                    "product_id": product_id,
                    "branch_id": branch_id,
                    "quantity": qty,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid SET STOCK syntax");
}

fn register_transfer_stock(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            [
                "TRANSFER", "STOCK", "$expr$", ",", "$expr$", ",", "$expr$", ",", "$expr$",
            ],
            false,
            move |context, inputs| {
                let product_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let from_branch = context.eval_expression_tree(&inputs[1])?.to_string();
                let to_branch = context.eval_expression_tree(&inputs[2])?.to_string();
                let qty_expr = context.eval_expression_tree(&inputs[3])?.to_string();
                let qty: f64 = qty_expr.parse().unwrap_or(0.0);
                trace!("TRANSFER STOCK: {product_id} {from_branch}->{to_branch} qty={qty}");
                let result = json!({
                    "kind": "inventory_movement",
                    "movement_type": "transfer",
                    "product_id": product_id,
                    "from_branch_id": from_branch,
                    "to_branch_id": to_branch,
                    "quantity": qty,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid TRANSFER STOCK syntax");
}

fn register_get_stock(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["GET", "STOCK", "$expr$"],
            false,
            move |context, inputs| {
                let product_id = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("GET STOCK: {product_id}");
                let result = json!({
                    "kind": "stock_query",
                    "product_id": product_id,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid GET STOCK syntax");
}

fn register_set_price(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["SET", "PRICE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let product_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let price_list = context.eval_expression_tree(&inputs[1])?.to_string();
                let price_expr = context.eval_expression_tree(&inputs[2])?.to_string();
                let price: f64 = price_expr.parse().unwrap_or(0.0);
                trace!("SET PRICE: {product_id} list={price_list} price={price}");
                let result = json!({
                    "kind": "product_price",
                    "action": "upsert",
                    "product_id": product_id,
                    "price_list_name": price_list,
                    "price": price,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid SET PRICE syntax");
}

fn register_open_pos_session(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["OPEN", "POS", "SESSION", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let branch_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let opening_expr = context.eval_expression_tree(&inputs[1])?.to_string();
                let opening: f64 = opening_expr.parse().unwrap_or(0.0);
                trace!("OPEN POS SESSION: branch={branch_id} opening={opening}");
                let result = json!({
                    "kind": "pos_session",
                    "action": "open",
                    "branch_id": branch_id,
                    "opening_amount": opening,
                    "status": "open",
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid OPEN POS SESSION syntax");
}

fn register_add_to_cart(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["ADD", "TO", "CART", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let product_id = context.eval_expression_tree(&inputs[1])?.to_string();
                let qty_expr = context.eval_expression_tree(&inputs[2])?.to_string();
                let qty: f64 = qty_expr.parse().unwrap_or(1.0);
                trace!("ADD TO CART: session={session_id} product={product_id} qty={qty}");
                let result = json!({
                    "kind": "cart_item",
                    "session_id": session_id,
                    "product_id": product_id,
                    "quantity": qty,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid ADD TO CART syntax");
}

fn register_checkout(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["CHECKOUT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let session_id = context.eval_expression_tree(&inputs[0])?.to_string();
                let payment_method = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("CHECKOUT: session={session_id} payment={payment_method}");
                let result = json!({
                    "kind": "pos_sale",
                    "action": "create",
                    "session_id": session_id,
                    "payment_method": payment_method,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid CHECKOUT syntax");
}

fn serde_json_to_dynamic(v: &Value) -> Dynamic {
    Dynamic::from(v.to_string())
}
