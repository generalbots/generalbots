use crate::bot::get_default_bot;
use crate::core::shared::schema::products;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use crate::shared::utils;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    row_data: String,
}

pub fn products_keyword(state: &AppState, _user: UserSession, engine: &mut Engine) {
    let connection = state.conn.clone();

    engine.register_fn("PRODUCTS", {
        let conn = connection.clone();
        move || -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("PRODUCTS db error: {e}");
                    return Dynamic::from(rhai::Array::new());
                }
            };

            match get_all_products(&mut binding) {
                Ok(products) => utils::json_value_to_dynamic(&products),
                Err(e) => {
                    error!("PRODUCTS error: {e}");
                    Dynamic::from(rhai::Array::new())
                }
            }
        }
    });

    engine.register_fn("products", {
        let conn = connection.clone();
        move || -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("products db error: {e}");
                    return Dynamic::from(rhai::Array::new());
                }
            };

            match get_all_products(&mut binding) {
                Ok(products) => utils::json_value_to_dynamic(&products),
                Err(e) => {
                    error!("products error: {e}");
                    Dynamic::from(rhai::Array::new())
                }
            }
        }
    });

    engine.register_fn("PRODUCT", {
        let conn = connection.clone();
        move |id: i64| -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("PRODUCT db error: {e}");
                    return Dynamic::UNIT;
                }
            };

            match get_product_by_id(&mut binding, id) {
                Ok(Some(product)) => utils::json_value_to_dynamic(&product),
                Ok(None) => Dynamic::UNIT,
                Err(e) => {
                    error!("PRODUCT error: {e}");
                    Dynamic::UNIT
                }
            }
        }
    });

    engine.register_fn("product", {
        let conn = connection.clone();
        move |id: i64| -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("product db error: {e}");
                    return Dynamic::UNIT;
                }
            };

            match get_product_by_id(&mut binding, id) {
                Ok(Some(product)) => utils::json_value_to_dynamic(&product),
                Ok(None) => Dynamic::UNIT,
                Err(e) => {
                    error!("product error: {e}");
                    Dynamic::UNIT
                }
            }
        }
    });

    engine
        .register_custom_syntax(
            ["SEARCH", "PRODUCTS", "$expr$", ",", "$expr$"],
            false,
            {
                let conn = connection.clone();
                move |context, inputs| {
                    let query = context.eval_expression_tree(&inputs[0])?;
                    let limit = context.eval_expression_tree(&inputs[1])?;

                    let mut binding = conn.get().map_err(|e| format!("DB error: {e}"))?;
                    let query_str = query.to_string();
                    let limit_val = limit.as_int().unwrap_or(10) as i32;

                    let result = search_products(&mut binding, &query_str, limit_val)
                        .map_err(|e| format!("Search error: {e}"))?;

                    Ok(utils::json_value_to_dynamic(&result))
                }
            },
        )
        .expect("valid syntax");

    engine
        .register_custom_syntax(["SEARCH", "PRODUCTS", "$expr$"], false, {
            let conn = connection.clone();
            move |context, inputs| {
                let query = context.eval_expression_tree(&inputs[0])?;

                let mut binding = conn.get().map_err(|e| format!("DB error: {e}"))?;
                let query_str = query.to_string();

                let result = search_products(&mut binding, &query_str, 10)
                    .map_err(|e| format!("Search error: {e}"))?;

                Ok(utils::json_value_to_dynamic(&result))
            }
        })
        .expect("valid syntax");

    engine.register_fn("SEARCH_PRODUCTS", {
        let conn = connection.clone();
        move |query: String, limit: i64| -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("SEARCH_PRODUCTS db error: {e}");
                    return Dynamic::from(rhai::Array::new());
                }
            };

            match search_products(&mut binding, &query, limit as i32) {
                Ok(products) => utils::json_value_to_dynamic(&products),
                Err(e) => {
                    error!("SEARCH_PRODUCTS error: {e}");
                    Dynamic::from(rhai::Array::new())
                }
            }
        }
    });

    engine.register_fn("search_products", {
        let conn = connection.clone();
        move |query: String, limit: i64| -> Dynamic {
            let mut binding = match conn.get() {
                Ok(c) => c,
                Err(e) => {
                    error!("search_products db error: {e}");
                    return Dynamic::from(rhai::Array::new());
                }
            };

            match search_products(&mut binding, &query, limit as i32) {
                Ok(products) => utils::json_value_to_dynamic(&products),
                Err(e) => {
                    error!("search_products error: {e}");
                    Dynamic::from(rhai::Array::new())
                }
            }
        }
    });
}

fn get_all_products(conn: &mut diesel::PgConnection) -> Result<Value, String> {
    trace!("get_all_products");

    let (bot_id, _) = get_default_bot(conn);

    let query = r#"
        SELECT row_to_json(p)::text as row_data
        FROM products p
        WHERE p.bot_id = $1 AND p.is_active = true
        ORDER BY p.name
        LIMIT 100
    "#;

    let results: Vec<JsonRow> = diesel::sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .load(conn)
        .map_err(|e| e.to_string())?;

    let products: Vec<Value> = results
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.row_data).ok())
        .collect();

    Ok(json!(products))
}

fn get_product_by_id(conn: &mut diesel::PgConnection, id: i64) -> Result<Option<Value>, String> {
    trace!("get_product_by_id: {id}");

    let query = r#"
        SELECT row_to_json(p)::text as row_data
        FROM products p
        WHERE p.id = $1
        LIMIT 1
    "#;

    let uuid = uuid::Uuid::from_u64_pair(0, id as u64);

    let results: Vec<JsonRow> = diesel::sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(uuid)
        .load(conn)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .next()
        .and_then(|row| serde_json::from_str(&row.row_data).ok()))
}

fn search_products(conn: &mut diesel::PgConnection, query: &str, limit: i32) -> Result<Value, String> {
    trace!("search_products: query={query}, limit={limit}");

    let (bot_id, _) = get_default_bot(conn);
    let safe_query = query.replace('\'', "''");

    let sql = r#"
        SELECT row_to_json(p)::text as row_data
        FROM products p
        WHERE p.bot_id = $1
          AND p.is_active = true
          AND (
            p.name ILIKE '%' || $2 || '%'
            OR p.description ILIKE '%' || $2 || '%'
            OR p.sku ILIKE '%' || $2 || '%'
            OR p.brand ILIKE '%' || $2 || '%'
            OR p.category ILIKE '%' || $2 || '%'
          )
        ORDER BY
          CASE WHEN p.name ILIKE $2 || '%' THEN 0 ELSE 1 END,
          p.name
        LIMIT $3
    "#;

    let results: Vec<JsonRow> = diesel::sql_query(sql)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<Text, _>(&safe_query)
        .bind::<Integer, _>(limit)
        .load(conn)
        .map_err(|e| e.to_string())?;

    let products: Vec<Value> = results
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.row_data).ok())
        .collect();

    Ok(json!(products))
}
