/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use crate::shared::models::{TriggerKind, UserSession};
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::trace;
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use uuid::Uuid;

/// Webhook registration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub endpoint: String,
    pub script_name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Register the WEBHOOK keyword
///
/// WEBHOOK "order-received"
///
/// This creates an endpoint at /api/botname/webhook/order-received
/// When called, it triggers the script containing the WEBHOOK declaration
/// Request params become available as variables in the script
pub fn webhook_keyword(state: &AppState, _user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();

    engine
        .register_custom_syntax(&["WEBHOOK", "$expr$"], false, move |context, inputs| {
            let endpoint = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("WEBHOOK registration for endpoint: {}", endpoint);

            // Note: Actual webhook registration happens during compilation/preprocessing
            // This runtime keyword is mainly for documentation and validation

            Ok(Dynamic::from(format!("webhook:{}", endpoint)))
        })
        .unwrap();
}

/// Execute webhook registration during preprocessing
/// This is called by the compiler when it finds a WEBHOOK declaration
pub fn execute_webhook_registration(
    conn: &mut diesel::PgConnection,
    endpoint: &str,
    script_name: &str,
    bot_uuid: Uuid,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    trace!(
        "Registering WEBHOOK endpoint: {}, script: {}, bot_id: {:?}",
        endpoint,
        script_name,
        bot_uuid
    );

    // Verify bot exists
    use crate::shared::models::bots::dsl::bots;
    let bot_exists: bool = diesel::select(diesel::dsl::exists(
        bots.filter(crate::shared::models::bots::dsl::id.eq(bot_uuid)),
    ))
    .get_result(conn)?;

    if !bot_exists {
        return Err(format!("Bot with id {} does not exist", bot_uuid).into());
    }

    // Clean the endpoint name (remove quotes, spaces)
    let clean_endpoint = endpoint
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();

    // Register in system_automations table with kind = Webhook
    use crate::shared::models::system_automations::dsl::*;

    let new_automation = (
        bot_id.eq(bot_uuid),
        kind.eq(TriggerKind::Webhook as i32),
        target.eq(&clean_endpoint),
        param.eq(script_name),
        is_active.eq(true),
    );

    // First try to update existing
    let update_result = diesel::update(system_automations)
        .filter(bot_id.eq(bot_uuid))
        .filter(kind.eq(TriggerKind::Webhook as i32))
        .filter(target.eq(&clean_endpoint))
        .set((param.eq(script_name), is_active.eq(true)))
        .execute(&mut *conn)?;

    // If no rows updated, insert new
    let result = if update_result == 0 {
        diesel::insert_into(system_automations)
            .values(&new_automation)
            .execute(&mut *conn)?
    } else {
        update_result
    };

    Ok(json!({
        "command": "webhook",
        "endpoint": clean_endpoint,
        "script": script_name,
        "bot_id": bot_uuid.to_string(),
        "rows_affected": result
    }))
}

/// Remove webhook registration
pub fn remove_webhook_registration(
    conn: &mut diesel::PgConnection,
    endpoint: &str,
    bot_uuid: Uuid,
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    use crate::shared::models::system_automations::dsl::*;

    let clean_endpoint = endpoint
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_lowercase()
        .replace(' ', "-");

    let result = diesel::delete(
        system_automations
            .filter(bot_id.eq(bot_uuid))
            .filter(kind.eq(TriggerKind::Webhook as i32))
            .filter(target.eq(&clean_endpoint)),
    )
    .execute(&mut *conn)?;

    Ok(result)
}

/// Get all webhooks for a bot
pub fn get_bot_webhooks(
    conn: &mut diesel::PgConnection,
    bot_uuid: Uuid,
) -> Result<Vec<(String, String, bool)>, Box<dyn Error + Send + Sync>> {
    #[derive(QueryableByName)]
    struct WebhookRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        webhook_target: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        webhook_param: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        webhook_is_active: bool,
    }

    let results: Vec<WebhookRow> = diesel::sql_query(
        "SELECT target as webhook_target, param as webhook_param, is_active as webhook_is_active FROM system_automations WHERE bot_id = $1 AND kind = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
    .bind::<diesel::sql_types::Int4, _>(TriggerKind::Webhook as i32)
    .load(conn)?;

    Ok(results
        .into_iter()
        .map(|r| {
            (
                r.webhook_target.unwrap_or_default(),
                r.webhook_param,
                r.webhook_is_active,
            )
        })
        .collect())
}

/// Find webhook script by endpoint
pub fn find_webhook_script(
    conn: &mut diesel::PgConnection,
    bot_uuid: Uuid,
    endpoint: &str,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    use crate::shared::models::system_automations::dsl::*;

    let clean_endpoint = endpoint
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_lowercase();

    let result: Option<String> = system_automations
        .filter(bot_id.eq(bot_uuid))
        .filter(kind.eq(TriggerKind::Webhook as i32))
        .filter(target.eq(&clean_endpoint))
        .filter(is_active.eq(true))
        .select(param)
        .first(conn)
        .optional()?;

    Ok(result)
}

/// Webhook request data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query_params: std::collections::HashMap<String, String>,
    pub body: Value,
    pub path: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebhookRequest {
    /// Create a new webhook request
    pub fn new(
        method: &str,
        headers: std::collections::HashMap<String, String>,
        query_params: std::collections::HashMap<String, String>,
        body: Value,
        path: &str,
    ) -> Self {
        Self {
            method: method.to_string(),
            headers,
            query_params,
            body,
            path: path.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Convert to Dynamic for use in BASIC scripts
    pub fn to_dynamic(&self) -> Dynamic {
        let mut map = rhai::Map::new();

        map.insert("method".into(), Dynamic::from(self.method.clone()));
        map.insert("path".into(), Dynamic::from(self.path.clone()));
        map.insert(
            "timestamp".into(),
            Dynamic::from(self.timestamp.to_rfc3339()),
        );

        // Convert headers
        let mut headers_map = rhai::Map::new();
        for (k, v) in &self.headers {
            headers_map.insert(k.clone().into(), Dynamic::from(v.clone()));
        }
        map.insert("headers".into(), Dynamic::from(headers_map));

        // Convert query params
        let mut params_map = rhai::Map::new();
        for (k, v) in &self.query_params {
            params_map.insert(k.clone().into(), Dynamic::from(v.clone()));
        }
        map.insert("params".into(), Dynamic::from(params_map));

        // Convert body
        map.insert("body".into(), json_to_dynamic(&self.body));

        Dynamic::from(map)
    }
}

/// Webhook response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Value,
}

impl Default for WebhookResponse {
    fn default() -> Self {
        Self {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: json!({"status": "ok"}),
        }
    }
}

impl WebhookResponse {
    /// Create a success response
    pub fn success(data: Value) -> Self {
        Self {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: data,
        }
    }

    /// Create an error response
    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            headers: std::collections::HashMap::new(),
            body: json!({"error": message}),
        }
    }

    /// Convert from Dynamic (returned by BASIC script)
    pub fn from_dynamic(value: &Dynamic) -> Self {
        if value.is_map() {
            let map = value.clone().try_cast::<rhai::Map>().unwrap_or_default();

            let status = map
                .get("status")
                .and_then(|v| v.as_int().ok())
                .unwrap_or(200) as u16;

            let body = map
                .get("body")
                .map(dynamic_to_json)
                .unwrap_or(json!({"status": "ok"}));

            let mut headers = std::collections::HashMap::new();
            if let Some(h) = map.get("headers") {
                match h.clone().try_cast::<rhai::Map>() {
                    Some(headers_map) => {
                        for (k, v) in headers_map {
                            headers.insert(k.to_string(), v.to_string());
                        }
                    }
                    None => {}
                }
            }

            Self {
                status,
                headers,
                body,
            }
        } else {
            Self::success(dynamic_to_json(value))
        }
    }
}

/// Convert JSON Value to Rhai Dynamic
fn json_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let rhai_arr: rhai::Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(rhai_arr)
        }
        Value::Object(obj) => {
            let mut map = rhai::Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

/// Convert Rhai Dynamic to JSON Value
fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if value.is_bool() {
        Value::Bool(value.as_bool().unwrap_or(false))
    } else if value.is_int() {
        Value::Number(value.as_int().unwrap_or(0).into())
    } else if value.is_float() {
        if let Ok(f) = value.as_float() {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    } else if value.is_string() {
        Value::String(value.to_string())
    } else if value.is_array() {
        let arr = value.clone().into_array().unwrap_or_default();
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if value.is_map() {
        let map = value.clone().try_cast::<rhai::Map>().unwrap_or_default();
        let obj: serde_json::Map<String, Value> = map
            .iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        Value::Object(obj)
    } else {
        Value::String(value.to_string())
    }
}
