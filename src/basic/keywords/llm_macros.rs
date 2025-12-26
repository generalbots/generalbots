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

use crate::core::config::ConfigManager;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Array, Dynamic, Engine, Map};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub fn register_llm_macros(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_calculate_keyword(state.clone(), user.clone(), engine);
    register_validate_keyword(state.clone(), user.clone(), engine);
    register_translate_keyword(state.clone(), user.clone(), engine);
    register_summarize_keyword(state, user, engine);
}

async fn call_llm(
    state: &AppState,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config_manager = ConfigManager::new(state.conn.clone());
    let model = config_manager
        .get_config(&Uuid::nil(), "llm-model", None)
        .unwrap_or_default();
    let key = config_manager
        .get_config(&Uuid::nil(), "llm-key", None)
        .unwrap_or_default();

    let handler = crate::llm::llm_models::get_handler(&model);
    let raw_response = state
        .llm_provider
        .generate(prompt, &serde_json::Value::Null, &model, &key)
        .await?;
    let processed = handler.process_content(&raw_response);
    Ok(processed)
}

fn run_llm_with_timeout(
    state: Arc<AppState>,
    prompt: String,
    timeout_secs: u64,
) -> Result<String, Box<rhai::EvalAltResult>> {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build();

        let send_err = if let Ok(rt) = rt {
            let result = rt.block_on(async move { call_llm(&state, &prompt).await });
            tx.send(result).err()
        } else {
            tx.send(Err("Failed to build tokio runtime".into())).err()
        };

        if send_err.is_some() {
            error!("Failed to send LLM result from thread");
        }
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
            format!("LLM call failed: {}", e).into(),
            rhai::Position::NONE,
        ))),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(Box::new(
            rhai::EvalAltResult::ErrorRuntime("LLM call timed out".into(), rhai::Position::NONE),
        )),
        Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
            format!("LLM thread failed: {}", e).into(),
            rhai::Position::NONE,
        ))),
    }
}

pub fn register_calculate_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["CALCULATE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let formula = context.eval_expression_tree(&inputs[0])?.to_string();
                let variables = context.eval_expression_tree(&inputs[1])?;

                trace!(
                    "CALCULATE: formula='{}', variables={:?}",
                    formula,
                    variables
                );

                let state_for_task = Arc::clone(&state_clone);
                let prompt = build_calculate_prompt(&formula, &variables);

                let result = run_llm_with_timeout(state_for_task, prompt, 60)?;
                parse_calculate_result(&result)
            },
        )
        .unwrap();
}

fn build_calculate_prompt(formula: &str, variables: &Dynamic) -> String {
    let vars_str = if variables.is_map() {
        let map = variables.clone().cast::<Map>();
        let pairs: Vec<String> = map.iter().map(|(k, v)| format!("{} = {}", k, v)).collect();
        pairs.join(", ")
    } else if variables.is_unit() {
        "none".to_string()
    } else {
        variables.to_string()
    };

    format!(
        r"You are a precise calculator. Evaluate the following expression.

Formula: {}
Variables: {}

Instructions:
1. Substitute the variables into the formula
2. Perform the calculation
3. Return ONLY the final result (number, boolean, or text)
4. No explanations, just the result

Result:",
        formula, vars_str
    )
}

fn parse_calculate_result(result: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let trimmed = result.trim();

    if let Ok(i) = trimmed.parse::<i64>() {
        return Ok(Dynamic::from(i));
    }

    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Dynamic::from(f));
    }

    match trimmed.to_lowercase().as_str() {
        "true" | "yes" => return Ok(Dynamic::from(true)),
        "false" | "no" => return Ok(Dynamic::from(false)),
        _ => {}
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Ok(json_to_dynamic(&json));
    }

    Ok(Dynamic::from(trimmed.to_string()))
}

pub fn register_validate_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["VALIDATE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?;
                let rules = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("VALIDATE: data={:?}, rules='{}'", data, rules);

                let state_for_task = Arc::clone(&state_clone);
                let prompt = build_validate_prompt(&data, &rules);

                let result = run_llm_with_timeout(state_for_task, prompt, 60)?;
                parse_validate_result(&result)
            },
        )
        .unwrap();
}

fn build_validate_prompt(data: &Dynamic, rules: &str) -> String {
    let data_str = if data.is_map() {
        let map = data.clone().cast::<Map>();
        serde_json::to_string_pretty(&map_to_json(&map)).unwrap_or_else(|_| data.to_string())
    } else {
        data.to_string()
    };

    format!(
        r#"You are a data validator. Validate the following data against the rules.

Data:
{}

Rules:
{}

Return a JSON object:
{{
  "is_valid": true/false,
  "errors": ["list of error messages"],
  "warnings": ["list of warning messages"]
}}

Return ONLY the JSON:"#,
        data_str, rules
    )
}

fn parse_validate_result(result: &str) -> Result<Dynamic, Box<rhai::EvalAltResult>> {
    let trimmed = result.trim();

    let json_str = if trimmed.starts_with("```") {
        trimmed
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        trimmed.to_string()
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let mut map = Map::new();

        let is_valid = json["is_valid"].as_bool().unwrap_or(false);
        map.insert("is_valid".into(), Dynamic::from(is_valid));

        let errors: Array = json["errors"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| Dynamic::from(v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();
        map.insert("errors".into(), Dynamic::from(errors));

        let warnings: Array = json["warnings"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| Dynamic::from(v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();
        map.insert("warnings".into(), Dynamic::from(warnings));

        return Ok(Dynamic::from(map));
    }

    let is_valid =
        trimmed.to_lowercase().contains("valid") && !trimmed.to_lowercase().contains("invalid");

    let mut map = Map::new();
    map.insert("is_valid".into(), Dynamic::from(is_valid));
    map.insert("errors".into(), Dynamic::from(Array::new()));
    map.insert("warnings".into(), Dynamic::from(Array::new()));

    Ok(Dynamic::from(map))
}

pub fn register_translate_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["TRANSLATE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let text = context.eval_expression_tree(&inputs[0])?.to_string();
                let language = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!(
                    "TRANSLATE: text length={}, target='{}'",
                    text.len(),
                    language
                );

                let state_for_task = Arc::clone(&state_clone);
                let prompt = build_translate_prompt(&text, &language);

                run_llm_with_timeout(state_for_task, prompt, 120).map(Dynamic::from)
            },
        )
        .unwrap();
}

fn build_translate_prompt(text: &str, language: &str) -> String {
    format!(
        r"Translate the following text to {}.

Text:
{}

Return ONLY the translated text, no explanations:

Translation:",
        language, text
    )
}

pub fn register_summarize_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(["SUMMARIZE", "$expr$"], false, move |context, inputs| {
            let text = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("SUMMARIZE: text length={}", text.len());

            let state_for_task = Arc::clone(&state_clone);
            let prompt = build_summarize_prompt(&text);

            run_llm_with_timeout(state_for_task, prompt, 120).map(Dynamic::from)
        })
        .unwrap();
}

fn build_summarize_prompt(text: &str) -> String {
    format!(
        r"Summarize the following text concisely.

Text:
{}

Return ONLY the summary:

Summary:",
        text
    )
}

fn json_to_dynamic(value: &serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let rhai_arr: Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(rhai_arr)
        }
        serde_json::Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

fn map_to_json(map: &Map) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in map.iter() {
        obj.insert(k.to_string(), dynamic_to_json(v));
    }
    serde_json::Value::Object(obj)
}

fn dynamic_to_json(data: &Dynamic) -> serde_json::Value {
    if data.is_unit() {
        serde_json::Value::Null
    } else if data.is_bool() {
        serde_json::Value::Bool(data.as_bool().unwrap_or(false))
    } else if data.is_int() {
        serde_json::Value::Number(serde_json::Number::from(data.as_int().unwrap_or(0)))
    } else if data.is_float() {
        if let Some(n) = serde_json::Number::from_f64(data.as_float().unwrap_or(0.0)) {
            serde_json::Value::Number(n)
        } else {
            serde_json::Value::Null
        }
    } else if data.is_string() {
        serde_json::Value::String(data.to_string())
    } else if data.is_array() {
        let arr = data.clone().cast::<Array>();
        serde_json::Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if data.is_map() {
        let map = data.clone().cast::<Map>();
        map_to_json(&map)
    } else {
        serde_json::Value::String(data.to_string())
    }
}
