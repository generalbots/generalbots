use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::trace;
use rhai::{Dynamic, Engine, Map};
use serde_json::{json, Value};
use std::sync::Arc;

/// Banking and reconciliation BASIC keywords for issue #618.
///
/// Provides: IMPORT BANK STATEMENT, IMPORT PLATFORM ORDERS, RECONCILE,
/// GET UNMATCHED DELIVERIES, ADD RECONCILE RULE, DELIVERY MARGINS,
/// REPORT REVENUE BY PLATFORM.
pub fn register_banking_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_import_bank_statement(state.clone(), user.clone(), engine);
    register_import_platform_orders(state.clone(), user.clone(), engine);
    register_reconcile(state.clone(), user.clone(), engine);
    register_get_unmatched_deliveries(state.clone(), user.clone(), engine);
    register_add_reconcile_rule(state.clone(), user.clone(), engine);
    register_delivery_margins(state.clone(), user.clone(), engine);
    register_revenue_by_platform(state, user, engine);
}

fn parse_csv_rows(csv: &str) -> Vec<Map> {
    let mut rows: Vec<Map> = Vec::new();
    let mut lines = csv.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => return rows,
    };
    let headers: Vec<&str> = header_line.split(',').map(|s| s.trim()).collect();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let mut map = Map::new();
        for (i, h) in headers.iter().enumerate() {
            let v = values.get(i).copied().unwrap_or("").to_string();
            map.insert((*h).to_string().into(), Dynamic::from(v));
        }
        rows.push(map);
    }
    rows
}

fn parse_amount(s: &str) -> f64 {
    s.replace(['R', '$', ' '], "")
        .replace('.', "")
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(0.0)
}

fn parse_br_date(s: &str) -> Option<String> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        Some(format!("{}-{:0>2}-{:0>2}", parts[2], parts[1], parts[0]))
    } else {
        None
    }
}

fn register_import_bank_statement(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["IMPORT", "BANK", "STATEMENT", "$expr$"],
            false,
            move |context, inputs| {
                let csv = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("IMPORT BANK STATEMENT ({} bytes)", csv.len());

                let rows = parse_csv_rows(&csv);
                let mut txns: Vec<Value> = Vec::new();
                for row in &rows {
                    let date = row
                        .get("date")
                        .and_then(|v| v.clone().try_into_string().ok())
                        .and_then(|s| parse_br_date(&s))
                        .or_else(|| {
                            row.get("data")
                                .and_then(|v| v.clone().try_into_string().ok())
                                .and_then(|s| parse_br_date(&s))
                        })
                        .unwrap_or_else(|| "1970-01-01".to_string());
                    let description = row
                        .get("description")
                        .or_else(|| row.get("descricao"))
                        .or_else(|| row.get("historico"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .unwrap_or_default();
                    let amount = row
                        .get("amount")
                        .or_else(|| row.get("valor"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .map(|s| parse_amount(&s))
                        .unwrap_or(0.0);
                    let bank = row
                        .get("bank")
                        .and_then(|v| v.clone().try_into_string().ok());
                    let account = row
                        .get("account")
                        .and_then(|v| v.clone().try_into_string().ok());

                    txns.push(json!({
                        "transaction_date": date,
                        "description": description,
                        "amount": amount,
                        "bank": bank,
                        "account": account,
                        "reconciled": false,
                    }));
                }

                let result = json!({
                    "kind": "bank_transactions",
                    "count": txns.len(),
                    "rows": txns,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid IMPORT BANK STATEMENT syntax");
}

fn register_import_platform_orders(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["IMPORT", "PLATFORM", "ORDERS", "$expr$"],
            false,
            move |context, inputs| {
                let csv = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("IMPORT PLATFORM ORDERS ({} bytes)", csv.len());

                let rows = parse_csv_rows(&csv);
                let mut txns: Vec<Value> = Vec::new();
                for row in &rows {
                    let platform = row
                        .get("platform")
                        .and_then(|v| v.clone().try_into_string().ok())
                        .unwrap_or_else(|| "unknown".to_string());
                    let platform_order_id = row
                        .get("order_id")
                        .or_else(|| row.get("id"))
                        .or_else(|| row.get("pedido_id"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .unwrap_or_default();
                    let order_date = row
                        .get("date")
                        .or_else(|| row.get("data"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .and_then(|s| parse_br_date(&s))
                        .unwrap_or_else(|| "1970-01-01".to_string());
                    let customer_name = row
                        .get("customer")
                        .or_else(|| row.get("cliente"))
                        .and_then(|v| v.clone().try_into_string().ok());
                    let subtotal = row
                        .get("subtotal")
                        .or_else(|| row.get("valor"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .map(|s| parse_amount(&s))
                        .unwrap_or(0.0);
                    let delivery_fee = row
                        .get("delivery_fee")
                        .or_else(|| row.get("entrega"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .map(|s| parse_amount(&s))
                        .unwrap_or(0.0);
                    let platform_commission = row
                        .get("commission")
                        .or_else(|| row.get("comissao"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .map(|s| parse_amount(&s))
                        .unwrap_or(0.0);
                    let net_value = row
                        .get("net")
                        .or_else(|| row.get("liquido"))
                        .and_then(|v| v.clone().try_into_string().ok())
                        .map(|s| parse_amount(&s))
                        .unwrap_or(subtotal - platform_commission);

                    txns.push(json!({
                        "platform": platform,
                        "platform_order_id": platform_order_id,
                        "order_date": order_date,
                        "customer_name": customer_name,
                        "subtotal": subtotal,
                        "delivery_fee": delivery_fee,
                        "platform_commission": platform_commission,
                        "net_value": net_value,
                        "reconciled": false,
                        "status": "delivered",
                    }));
                }

                let result = json!({
                    "kind": "delivery_transactions",
                    "count": txns.len(),
                    "rows": txns,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid IMPORT PLATFORM ORDERS syntax");
}

fn register_reconcile(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["RECONCILE", "$expr$", "$expr$"],
            false,
            move |context, inputs| {
                let deliveries_json = context.eval_expression_tree(&inputs[0])?.to_string();
                let bank_json = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("RECONCILE: matching deliveries to bank txns");

                let deliveries: Vec<Value> = serde_json::from_str(&deliveries_json)
                    .unwrap_or_else(|_| Vec::new());
                let bank: Vec<Value> =
                    serde_json::from_str(&bank_json).unwrap_or_else(|_| Vec::new());

                let mut matched: Vec<Value> = Vec::new();
                let mut used: Vec<bool> = vec![false; bank.len()];
                for d in &deliveries {
                    let d_amount = d.get("net_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let d_date = d
                        .get("order_date")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let d_order_id = d
                        .get("platform_order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    for (i, b) in bank.iter().enumerate() {
                        if used[i] {
                            continue;
                        }
                        let b_amount = b.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let b_desc = b
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let amount_diff = (b_amount - d_amount).abs();
                        let id_match = !d_order_id.is_empty() && b_desc.contains(d_order_id);
                        if amount_diff < 0.01 || id_match {
                            used[i] = true;
                            matched.push(json!({
                                "delivery": d,
                                "bank": b,
                                "match_type": if id_match { "order_id" } else { "amount" },
                                "delivery_date": d_date,
                            }));
                            break;
                        }
                    }
                }

                let unmatched: Vec<Value> = bank
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !used[*i])
                    .map(|(_, v)| v.clone())
                    .collect();

                let total: f64 = matched
                    .iter()
                    .filter_map(|m| m.get("delivery")?.get("net_value")?.as_f64())
                    .sum();

                let result = json!({
                    "matched_count": matched.len(),
                    "unmatched_count": unmatched.len(),
                    "total_amount_matched": total,
                    "matched": matched,
                    "unmatched": unmatched,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid RECONCILE syntax");
}

fn register_get_unmatched_deliveries(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["GET", "UNMATCHED", "DELIVERIES", "$expr$"],
            false,
            move |context, inputs| {
                let reconciled_ids_json = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("GET UNMATCHED DELIVERIES");
                let reconciled: Vec<String> =
                    serde_json::from_str(&reconciled_ids_json).unwrap_or_default();
                let result = json!({
                    "kind": "unmatched_query",
                    "filter": { "reconciled": false, "exclude_ids": reconciled },
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid GET UNMATCHED DELIVERIES syntax");
}

fn register_add_reconcile_rule(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["ADD", "RECONCILE", "RULE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let match_field = context.eval_expression_tree(&inputs[1])?.to_string();
                let match_value = context.eval_expression_tree(&inputs[2])?.to_string();
                trace!("ADD RECONCILE RULE: {name} {match_field}={match_value}");
                let result = json!({
                    "kind": "reconciliation_rule",
                    "name": name,
                    "match_field": match_field,
                    "match_operator": "contains",
                    "match_value": match_value,
                    "is_active": true,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid ADD RECONCILE RULE syntax");
}

fn register_delivery_margins(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(["DELIVERY", "MARGINS", "$expr$"], false, move |context, inputs| {
            let deliveries_json = context.eval_expression_tree(&inputs[0])?.to_string();
            trace!("DELIVERY MARGINS");
            let deliveries: Vec<Value> =
                serde_json::from_str(&deliveries_json).unwrap_or_default();

            let mut total_subtotal = 0.0;
            let mut total_commission = 0.0;
            let mut total_delivery_fee = 0.0;
            let mut total_net = 0.0;
            let mut count = 0;
            for d in &deliveries {
                total_subtotal += d.get("subtotal").and_then(|v| v.as_f64()).unwrap_or(0.0);
                total_commission += d
                    .get("platform_commission")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                total_delivery_fee += d.get("delivery_fee").and_then(|v| v.as_f64()).unwrap_or(0.0);
                total_net += d.get("net_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                count += 1;
            }

            let margin_pct = if total_subtotal > 0.0 {
                (total_net / total_subtotal) * 100.0
            } else {
                0.0
            };

            let result = json!({
                "count": count,
                "total_subtotal": total_subtotal,
                "total_commission": total_commission,
                "total_delivery_fee": total_delivery_fee,
                "total_net": total_net,
                "margin_percent": margin_pct,
            });
            Ok(serde_json_to_dynamic(&result))
        })
        .expect("valid DELIVERY MARGINS syntax");
}

fn register_revenue_by_platform(
    state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    let _state_clone = state;

    engine
        .register_custom_syntax(
            ["REPORT", "REVENUE", "BY", "PLATFORM", "$expr$"],
            false,
            move |context, inputs| {
                let deliveries_json = context.eval_expression_tree(&inputs[0])?.to_string();
                trace!("REPORT REVENUE BY PLATFORM");
                let deliveries: Vec<Value> =
                    serde_json::from_str(&deliveries_json).unwrap_or_default();
                let mut groups: std::collections::HashMap<String, (f64, f64, i64)> =
                    std::collections::HashMap::new();
                for d in &deliveries {
                    let p = d
                        .get("platform")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let net = d.get("net_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let subtotal = d.get("subtotal").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let entry = groups.entry(p).or_insert((0.0, 0.0, 0));
                    entry.0 += net;
                    entry.1 += subtotal;
                    entry.2 += 1;
                }
                let breakdown: Vec<Value> = groups
                    .into_iter()
                    .map(|(platform, (net, subtotal, count))| {
                        json!({
                            "platform": platform,
                            "net_revenue": net,
                            "subtotal": subtotal,
                            "count": count,
                        })
                    })
                    .collect();
                let result = json!({
                    "kind": "revenue_by_platform",
                    "platforms": breakdown,
                });
                Ok(serde_json_to_dynamic(&result))
            },
        )
        .expect("valid REPORT REVENUE BY PLATFORM syntax");
}

fn serde_json_to_dynamic(v: &Value) -> Dynamic {
    let s = v.to_string();
    Dynamic::from(s)
}
