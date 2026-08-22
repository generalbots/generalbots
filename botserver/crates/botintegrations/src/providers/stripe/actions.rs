//! Stripe action implementations for the provider adapter (#950 slice 2).
//!
//! Every handler validates its parameters completely before any network
//! activity, requests a single bounded first page, and projects the response
//! down to a small, redaction-safe subset of fields.

use serde_json::{json, Value};

use super::client::{self, StripeCreds};
use crate::providers::rest_client::{
    bounded_limit, invalid, optional_text, outcome, push_query_pair,
};
use crate::providers::ActionOutcome;

const MAX_EMAIL_LEN: usize = 254;
const MAX_NAME_LEN: usize = 256;
const LIST_LIMIT_MAX: usize = 50;

/// Validates a Stripe object identifier such as `pi_...` before it is used
/// in a URL path segment.
fn stripe_id(raw: &str, prefix: &str) -> Result<String, String> {
    let rest = raw
        .strip_prefix(prefix)
        .ok_or_else(|| invalid(format!("identifier must start with {prefix}")))?;
    if rest.is_empty()
        || raw.len() > 255
        || !rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(invalid(
            "identifier contains invalid characters".to_string(),
        ));
    }
    Ok(raw.to_string())
}

/// Validates an ISO currency code accepted by the payment intents API.
fn validate_currency(code: &str) -> Result<(), String> {
    let ok = code.len() == 3 && code.chars().all(|c| c.is_ascii_lowercase());
    if ok {
        Ok(())
    } else {
        Err(invalid(
            "currency must be a three-letter lowercase ISO code".to_string(),
        ))
    }
}

/// Extracts a positive integer amount from the create/refund payload.
fn positive_amount(object: &serde_json::Map<String, Value>) -> Result<Option<i64>, String> {
    match object.get("amount") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => match value.as_i64() {
            Some(amount) if amount > 0 => Ok(Some(amount)),
            _ => Err(invalid(
                "amount must be a positive integer in the smallest currency unit".to_string(),
            )),
        },
    }
}

/// Builds the customer creation fields from the catalog `data` object; at
/// least one of name or email must be provided.
fn customer_create_fields(data: &Value) -> Result<Vec<(String, String)>, String> {
    let object = data.as_object().ok_or_else(|| {
        invalid("data must be a JSON object containing name and/or email".to_string())
    })?;
    let name = match object.get("name") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) if !text.trim().is_empty() => {
            let trimmed = text.trim();
            if trimmed.len() > MAX_NAME_LEN {
                return Err(invalid(format!(
                    "data.name must be at most {MAX_NAME_LEN} characters"
                )));
            }
            Some(trimmed.to_string())
        }
        Some(_) => return Err(invalid("data.name must be a string".to_string())),
    };
    let email = match object.get("email") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            let valid =
                trimmed.contains('@') && !trimmed.is_empty() && trimmed.len() <= MAX_EMAIL_LEN;
            if !valid {
                return Err(invalid("data.email must be a valid address".to_string()));
            }
            Some(trimmed.to_string())
        }
        Some(_) => return Err(invalid("data.email must be a string".to_string())),
    };
    if name.is_none() && email.is_none() {
        return Err(invalid(
            "data must set at least one of name or email".to_string(),
        ));
    }
    let mut fields = Vec::new();
    if let Some(name) = name {
        fields.push(("name".to_string(), name));
    }
    if let Some(email) = email {
        fields.push(("email".to_string(), email));
    }
    Ok(fields)
}

/// Builds the payment intent creation fields from the catalog `data`
/// object; amount and currency are both required by the API.
fn payment_intent_create_fields(data: &Value) -> Result<Vec<(String, String)>, String> {
    let object = data.as_object().ok_or_else(|| {
        invalid("data must be a JSON object containing amount and currency".to_string())
    })?;
    let amount = positive_amount(object)?.ok_or_else(|| {
        invalid("data.amount is required and must be a positive integer".to_string())
    })?;
    let currency = object
        .get("currency")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid("data.currency is required".to_string()))?;
    validate_currency(currency)?;
    Ok(vec![
        ("amount".to_string(), amount.to_string()),
        ("currency".to_string(), currency.to_string()),
    ])
}

fn project_balance_line(item: &Value) -> Value {
    json!({
        "amount": item.get("amount").cloned().unwrap_or_default(),
        "currency": item.get("currency").cloned().unwrap_or_default(),
    })
}

fn project_customer(item: &Value) -> Value {
    json!({
        "id": item.get("id").cloned().unwrap_or_default(),
        "name": item.get("name").cloned().unwrap_or_default(),
        "email": item.get("email").cloned().unwrap_or_default(),
        "created": item.get("created").cloned().unwrap_or_default(),
    })
}

fn project_payment_intent(item: &Value) -> Value {
    json!({
        "id": item.get("id").cloned().unwrap_or_default(),
        "amount": item.get("amount").cloned().unwrap_or_default(),
        "currency": item.get("currency").cloned().unwrap_or_default(),
        "status": item.get("status").cloned().unwrap_or_default(),
        "created": item.get("created").cloned().unwrap_or_default(),
    })
}

fn project_price(item: &Value) -> Value {
    json!({
        "id": item.get("id").cloned().unwrap_or_default(),
        "unit_amount": item.get("unit_amount").cloned().unwrap_or_default(),
        "currency": item.get("currency").cloned().unwrap_or_default(),
        "active": item.get("active").cloned().unwrap_or(Value::Bool(true)),
        "product": item.get("product").cloned().unwrap_or_default(),
    })
}

fn project_subscription(item: &Value) -> Value {
    json!({
        "id": item.get("id").cloned().unwrap_or_default(),
        "status": item.get("status").cloned().unwrap_or_default(),
        "customer": item.get("customer").cloned().unwrap_or_default(),
        "current_period_end": item.get("current_period_end").cloned().unwrap_or_default(),
    })
}

fn projected_list(document: &Value, key: &str, project: fn(&Value) -> Value) -> Vec<Value> {
    document
        .get(key)
        .and_then(Value::as_array)
        .map(|items| items.iter().map(project).collect())
        .unwrap_or_default()
}

async fn balance_retrieve(creds: &StripeCreds) -> Result<ActionOutcome, String> {
    let response = client::get(creds, "/balance").await?;
    response.require_success("balance.retrieve")?;
    let document = response.json("balance.retrieve")?;
    Ok(outcome(
        "Read current Stripe account balance".to_string(),
        json!({
            "available": document
                .get("available")
                .and_then(Value::as_array)
                .map(|items| items.iter().map(project_balance_line).collect::<Vec<_>>())
                .unwrap_or_default(),
            "pending": document
                .get("pending")
                .and_then(Value::as_array)
                .map(|items| items.iter().map(project_balance_line).collect::<Vec<_>>())
                .unwrap_or_default(),
        }),
    ))
}

async fn customers_list(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let limit = bounded_limit(params, "limit", LIST_LIMIT_MAX, LIST_LIMIT_MAX)?;
    let response = client::get(creds, &format!("/customers?limit={limit}")).await?;
    response.require_success("customers.list")?;
    let document = response.json("customers.list")?;
    let has_more = document
        .get("has_more")
        .cloned()
        .unwrap_or(Value::Bool(false));
    let customers = projected_list(&document, "data", project_customer);
    let count = customers.len();
    Ok(outcome(
        format!("Listed {count} customers"),
        json!({ "customer_count": count, "has_more": has_more, "customers": customers }),
    ))
}

async fn customers_search(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let email = required_email(params, "query")?;
    let limit = bounded_limit(params, "limit", 20, LIST_LIMIT_MAX)?;
    let mut path_query = format!("/customers?limit={limit}");
    push_query_pair(&mut path_query, "email", &email);
    let response = client::get(creds, &path_query).await?;
    response.require_success("customers.search")?;
    let document = response.json("customers.search")?;
    let customers = projected_list(&document, "data", project_customer);
    let count = customers.len();
    Ok(outcome(
        format!("Found {count} customers matching '{email}'"),
        json!({ "customer_count": count, "customers": customers }),
    ))
}

/// Validates an email-shaped query parameter used for customer search.
fn required_email(params: &Value, key: &str) -> Result<String, String> {
    let value = crate::providers::rest_client::required_text(params, key, MAX_EMAIL_LEN)?;
    if !value.contains('@') {
        return Err(invalid(format!("{key} must be a valid address")));
    }
    Ok(value)
}

async fn customers_create(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let data = params
        .get("data")
        .ok_or_else(|| invalid("data is required".to_string()))?;
    let fields = customer_create_fields(data)?;
    let response = client::post_form(creds, "/customers", fields).await?;
    response.require_success("customers.create")?;
    let document = response.json("customers.create")?;
    let id = document.get("id").cloned().unwrap_or_default();
    Ok(outcome(
        format!("Created customer {id}"),
        json!({ "customer": project_customer(&document) }),
    ))
}

async fn payment_intents_list(
    creds: &StripeCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let limit = bounded_limit(params, "limit", LIST_LIMIT_MAX, LIST_LIMIT_MAX)?;
    let response = client::get(creds, &format!("/payment_intents?limit={limit}")).await?;
    response.require_success("payment_intents.list")?;
    let document = response.json("payment_intents.list")?;
    let payments = projected_list(&document, "data", project_payment_intent);
    let count = payments.len();
    Ok(outcome(
        format!("Listed {count} payments"),
        json!({ "payment_count": count, "payments": payments }),
    ))
}

async fn payment_intents_get(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let raw_id = crate::providers::rest_client::required_text(params, "resource_id", 255)?;
    let id = stripe_id(&raw_id, "pi_")?;
    let response = client::get(creds, &format!("/payment_intents/{id}")).await?;
    response.require_success("payment_intents.get")?;
    let document = response.json("payment_intents.get")?;
    let status = document.get("status").cloned().unwrap_or_default();
    Ok(outcome(
        format!("Read payment {id} ({status})"),
        json!({ "payment": project_payment_intent(&document) }),
    ))
}

async fn payment_intents_create(
    creds: &StripeCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let data = params
        .get("data")
        .ok_or_else(|| invalid("data is required".to_string()))?;
    let fields = payment_intent_create_fields(data)?;
    let response = client::post_form(creds, "/payment_intents", fields).await?;
    response.require_success("payment_intents.create")?;
    let document = response.json("payment_intents.create")?;
    let id = document.get("id").cloned().unwrap_or_default();
    let status = document.get("status").cloned().unwrap_or_default();
    Ok(outcome(
        format!("Created payment intent {id} ({status})"),
        json!({ "payment": project_payment_intent(&document) }),
    ))
}

async fn prices_list(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let limit = bounded_limit(params, "limit", LIST_LIMIT_MAX, LIST_LIMIT_MAX)?;
    let response = client::get(creds, &format!("/prices?limit={limit}&active=true")).await?;
    response.require_success("prices.list")?;
    let document = response.json("prices.list")?;
    let prices = projected_list(&document, "data", project_price);
    let count = prices.len();
    Ok(outcome(
        format!("Listed {count} active prices"),
        json!({ "price_count": count, "prices": prices }),
    ))
}

async fn refunds_create(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let raw_intent = crate::providers::rest_client::required_text(params, "payment_intent", 255)?;
    let intent = stripe_id(&raw_intent, "pi_")?;
    let reason = optional_text(params, "reason", 64)?;
    if let Some(reason) = &reason {
        if reason != "duplicate" && reason != "fraudulent" && reason != "requested_by_customer" {
            return Err(invalid(
                "reason must be duplicate, fraudulent or requested_by_customer".to_string(),
            ));
        }
    }
    // The catalog models the partial refund amount as an integer parameter,
    // so reuse the shared bound check against a synthetic object view.
    let amount = match params.get("amount") {
        None | Some(Value::Null) => None,
        Some(value) => {
            let mut synthetic = serde_json::Map::new();
            synthetic.insert("amount".to_string(), value.clone());
            positive_amount(&synthetic)?
        }
    };

    let mut fields = vec![("payment_intent".to_string(), intent.clone())];
    if let Some(amount) = amount {
        fields.push(("amount".to_string(), amount.to_string()));
    }
    if let Some(reason) = reason {
        fields.push(("reason".to_string(), reason));
    }
    let response = client::post_form(creds, "/refunds", fields).await?;
    response.require_success("refunds.create")?;
    let document = response.json("refunds.create")?;
    let refund_id = document.get("id").cloned().unwrap_or_default();
    let refund_amount = document.get("amount").cloned().unwrap_or_default();
    let status = document.get("status").cloned().unwrap_or_default();
    Ok(outcome(
        format!("Refunded {refund_amount} from {intent} (refund {refund_id}, {status})"),
        json!({
            "refund": {
                "id": refund_id,
                "amount": refund_amount,
                "status": status,
                "payment_intent": intent,
            }
        }),
    ))
}

async fn subscriptions_list(creds: &StripeCreds, params: &Value) -> Result<ActionOutcome, String> {
    let limit = bounded_limit(params, "limit", LIST_LIMIT_MAX, LIST_LIMIT_MAX)?;
    let mut path_query = format!("/subscriptions?limit={limit}");
    push_query_pair(&mut path_query, "status", "all");
    let response = client::get(creds, &path_query).await?;
    response.require_success("subscriptions.list")?;
    let document = response.json("subscriptions.list")?;
    let subscriptions = projected_list(&document, "data", project_subscription);
    let count = subscriptions.len();
    Ok(outcome(
        format!("Listed {count} subscriptions"),
        json!({ "subscription_count": count, "subscriptions": subscriptions }),
    ))
}

/// Entry point used by [`super::StripeAdapter`]; unknown keys are rejected
/// here before any parameter validation or network activity happens.
pub(crate) async fn invoke(
    action: &str,
    creds: &StripeCreds,
    params: &Value,
) -> Result<ActionOutcome, String> {
    match action {
        "balance.retrieve" => balance_retrieve(creds).await,
        "customers.list" => customers_list(creds, params).await,
        "customers.search" => customers_search(creds, params).await,
        "customers.create" => customers_create(creds, params).await,
        "payment_intents.list" => payment_intents_list(creds, params).await,
        "payment_intents.get" => payment_intents_get(creds, params).await,
        "payment_intents.create" => payment_intents_create(creds, params).await,
        "prices.list" => prices_list(creds, params).await,
        "refunds.create" => refunds_create(creds, params).await,
        "subscriptions.list" => subscriptions_list(creds, params).await,
        _ => Err(crate::providers::ERR_UNKNOWN_ACTION.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stripe_ids_require_prefix_and_safe_charset() {
        assert_eq!(
            stripe_id("pi_1234", "pi_").ok(),
            Some("pi_1234".to_string())
        );
        assert!(stripe_id("cus_1234", "pi_").is_err());
        assert!(stripe_id("pi_", "pi_").is_err());
        assert!(stripe_id("pi_/../etc", "pi_").is_err());
        assert!(stripe_id("", "pi_").is_err());
    }

    #[test]
    fn currency_codes_are_three_lowercase_letters() {
        assert!(validate_currency("usd").is_ok());
        assert!(validate_currency("brl").is_ok());
        assert!(validate_currency("USD").is_err());
        assert!(validate_currency("us").is_err());
        assert!(validate_currency("euro").is_err());
    }

    #[test]
    fn customer_creation_requires_name_or_email() {
        assert!(customer_create_fields(&json!({})).is_err());
        assert!(customer_create_fields(&json!({ "email": "not-an-address" })).is_err());
        let fields = customer_create_fields(&json!({ "name": "Acme", "email": "a@b.co" }))
            .unwrap_or_default();
        assert!(fields.contains(&("name".to_string(), "Acme".to_string())));
        assert!(fields.contains(&("email".to_string(), "a@b.co".to_string())));
        assert!(customer_create_fields(&json!({ "name": "Only Name" })).is_ok());
    }

    #[test]
    fn payment_intents_need_positive_amount_and_currency() {
        assert!(payment_intent_create_fields(&json!({ "amount": 100 })).is_err());
        assert!(payment_intent_create_fields(&json!({ "amount": 100, "currency": "US" })).is_err());
        assert!(payment_intent_create_fields(&json!({ "amount": -5, "currency": "usd" })).is_err());
        let fields = payment_intent_create_fields(&json!({ "amount": 1500, "currency": "usd" }))
            .unwrap_or_default();
        assert_eq!(
            fields,
            vec![
                ("amount".to_string(), "1500".to_string()),
                ("currency".to_string(), "usd".to_string())
            ]
        );
    }

    #[test]
    fn unknown_actions_reject_before_any_network_activity() {
        let names = super::super::STRIPE_IMPLEMENTED_ACTIONS;
        assert!(!names.contains(&"payouts.create"));
        assert!(names.contains(&"balance.retrieve"));
    }

    #[test]
    fn email_queries_must_contain_an_at_sign() {
        assert!(required_email(&json!({ "query": "a@b.co" }), "query").is_ok());
        assert!(required_email(&json!({ "query": "plain" }), "query").is_err());
        assert!(required_email(&json!({}), "query").is_err());
    }
}
