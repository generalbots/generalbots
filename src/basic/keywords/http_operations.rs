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

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use reqwest::{header::HeaderMap, header::HeaderName, header::HeaderValue, Client, Method};
use rhai::{Dynamic, Engine, Map};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

thread_local! {
    // Thread-local storage for HTTP headers
    static HTTP_HEADERS: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

/// Register all HTTP operation keywords
pub fn register_http_operations(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_post_keyword(state.clone(), user.clone(), engine);
    register_put_keyword(state.clone(), user.clone(), engine);
    register_patch_keyword(state.clone(), user.clone(), engine);
    register_delete_http_keyword(state.clone(), user.clone(), engine);
    register_set_header_keyword(state.clone(), user.clone(), engine);
    register_graphql_keyword(state.clone(), user.clone(), engine);
    register_soap_keyword(state.clone(), user.clone(), engine);
    register_clear_headers_keyword(state.clone(), user.clone(), engine);
}

/// POST "url", data
/// Sends an HTTP POST request with JSON body
pub fn register_post_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["POST", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("POST request to: {}", url);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = url.clone();
                let data_clone = dynamic_to_json(&data);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_http_request(Method::POST, &url_clone, Some(data_clone), None)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send POST result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("POST failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "POST request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("POST thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// PUT "url", data
/// Sends an HTTP PUT request with JSON body
pub fn register_put_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["PUT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("PUT request to: {}", url);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = url.clone();
                let data_clone = dynamic_to_json(&data);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_http_request(Method::PUT, &url_clone, Some(data_clone), None)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send PUT result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("PUT failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "PUT request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("PUT thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// PATCH "url", data
/// Sends an HTTP PATCH request with JSON body
pub fn register_patch_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["PATCH", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("PATCH request to: {}", url);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = url.clone();
                let data_clone = dynamic_to_json(&data);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_http_request(Method::PATCH, &url_clone, Some(data_clone), None)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send PATCH result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("PATCH failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "PATCH request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("PATCH thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// DELETE "url"
/// Sends an HTTP DELETE request
pub fn register_delete_http_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    // DELETE HTTP (space-separated - preferred)
    let _state_clone2 = Arc::clone(&state);
    engine
        .register_custom_syntax(
            &["DELETE", "HTTP", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();

                trace!("DELETE HTTP request to: {}", url);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = url.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_http_request(Method::DELETE, &url_clone, None, None).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send DELETE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // DELETE HTTP (spaces - preferred syntax)
    engine
        .register_custom_syntax(
            &["DELETE", "HTTP", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();

                trace!("DELETE HTTP request to: {}", url);

                let (tx, rx) = std::sync::mpsc::channel();
                let url_clone = url.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_http_request(Method::DELETE, &url_clone, None, None).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send DELETE HTTP result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE HTTP failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE HTTP request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE HTTP thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// SET HEADER "name", "value"
/// Sets an HTTP header for subsequent requests
pub fn register_set_header_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    // Use a shared state for headers that persists across calls
    let headers: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let headers_clone = Arc::clone(&headers);
    let headers_clone2 = Arc::clone(&headers);

    // SET HEADER (space-separated - preferred)
    engine
        .register_custom_syntax(
            &["SET", "HEADER", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let value = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("SET HEADER: {} = {}", name, value);

                // Store in thread-local storage
                HTTP_HEADERS.with(|h| {
                    h.borrow_mut().insert(name.clone(), value.clone());
                });

                // Also store in shared state
                if let Ok(mut h) = headers_clone.lock() {
                    h.insert(name, value);
                }

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();

    // SET_HEADER (underscore - backwards compatibility)
    engine
        .register_custom_syntax(
            &["SET_HEADER", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let name = context.eval_expression_tree(&inputs[0])?.to_string();
                let value = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("SET_HEADER: {} = {}", name, value);

                // Store in thread-local storage
                HTTP_HEADERS.with(|h| {
                    h.borrow_mut().insert(name.clone(), value.clone());
                });

                // Also store in shared state
                if let Ok(mut h) = headers_clone2.lock() {
                    h.insert(name, value);
                }

                Ok(Dynamic::UNIT)
            },
        )
        .unwrap();
}

/// CLEAR HEADERS
/// Clears all previously set HTTP headers
pub fn register_clear_headers_keyword(
    _state: Arc<AppState>,
    _user: UserSession,
    engine: &mut Engine,
) {
    // CLEAR HEADERS (space-separated - preferred)
    engine
        .register_custom_syntax(&["CLEAR", "HEADERS"], false, move |_context, _inputs| {
            trace!("CLEAR HEADERS");

            HTTP_HEADERS.with(|h| {
                h.borrow_mut().clear();
            });

            Ok(Dynamic::UNIT)
        })
        .unwrap();

    // CLEAR_HEADERS (underscore - backwards compatibility)
    engine
        .register_custom_syntax(&["CLEAR_HEADERS"], false, move |_context, _inputs| {
            trace!("CLEAR_HEADERS");

            HTTP_HEADERS.with(|h| {
                h.borrow_mut().clear();
            });

            Ok(Dynamic::UNIT)
        })
        .unwrap();
}

/// GRAPHQL "endpoint", "query", variables
/// Executes a GraphQL query
pub fn register_graphql_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["GRAPHQL", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let endpoint = context.eval_expression_tree(&inputs[0])?.to_string();
                let query = context.eval_expression_tree(&inputs[1])?.to_string();
                let variables = context.eval_expression_tree(&inputs[2])?;

                trace!("GRAPHQL request to: {}", endpoint);

                let (tx, rx) = std::sync::mpsc::channel();
                let endpoint_clone = endpoint.clone();
                let query_clone = query.clone();
                let variables_json = dynamic_to_json(&variables);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_graphql(&endpoint_clone, &query_clone, variables_json).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send GRAPHQL result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GRAPHQL failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "GRAPHQL request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GRAPHQL thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// SOAP "wsdl", "operation", params
/// Executes a SOAP API call
pub fn register_soap_keyword(state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    let _state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            &["SOAP", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let wsdl = context.eval_expression_tree(&inputs[0])?.to_string();
                let operation = context.eval_expression_tree(&inputs[1])?.to_string();
                let params = context.eval_expression_tree(&inputs[2])?;

                trace!("SOAP request to: {}, operation: {}", wsdl, operation);

                let (tx, rx) = std::sync::mpsc::channel();
                let wsdl_clone = wsdl.clone();
                let operation_clone = operation.clone();
                let params_json = dynamic_to_json(&params);

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_soap(&wsdl_clone, &operation_clone, params_json).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send SOAP result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                    Ok(Ok(response)) => Ok(json_to_dynamic(&response)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SOAP failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "SOAP request timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("SOAP thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// Execute an HTTP request with the specified method
async fn execute_http_request(
    method: Method,
    url: &str,
    body: Option<Value>,
    custom_headers: Option<HashMap<String, String>>,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            error!("Failed to build HTTP client: {}", e);
            e
        })?;

    // Build headers
    let mut headers = HeaderMap::new();

    // Add stored headers from thread-local storage
    HTTP_HEADERS.with(|h| {
        for (name, value) in h.borrow().iter() {
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                headers.insert(header_name, header_value);
            }
        }
    });

    // Add custom headers if provided
    if let Some(custom) = custom_headers {
        for (name, value) in custom {
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }

    // Set default content type for requests with body
    if body.is_some() && !headers.contains_key("content-type") {
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
    }

    let mut request = client.request(method.clone(), url).headers(headers);

    if let Some(body_data) = body {
        request = request.json(&body_data);
    }

    let response = request.send().await.map_err(|e| {
        error!("HTTP {} request failed for URL {}: {}", method, url, e);
        e
    })?;

    let status = response.status();
    let response_headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body_text = response.text().await.unwrap_or_default();

    // Try to parse as JSON, fall back to text
    let body_value: Value = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text));

    trace!(
        "HTTP {} request to {} completed with status: {}",
        method,
        url,
        status
    );

    Ok(json!({
        "status": status.as_u16(),
        "statusText": status.canonical_reason().unwrap_or(""),
        "headers": response_headers,
        "data": body_value
    }))
}

/// Execute a GraphQL query
async fn execute_graphql(
    endpoint: &str,
    query: &str,
    variables: Value,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let graphql_body = json!({
        "query": query,
        "variables": variables
    });

    execute_http_request(Method::POST, endpoint, Some(graphql_body), None).await
}

/// Execute a SOAP request
async fn execute_soap(
    endpoint: &str,
    operation: &str,
    params: Value,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    // Build SOAP envelope
    let soap_body = build_soap_envelope(operation, &params);

    let mut headers = HashMap::new();
    headers.insert(
        "Content-Type".to_string(),
        "text/xml; charset=utf-8".to_string(),
    );
    headers.insert("SOAPAction".to_string(), format!("\"{}\"", operation));

    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            error!("Failed to build HTTP client: {}", e);
            e
        })?;

    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(name.as_str()),
            HeaderValue::try_from(value.as_str()),
        ) {
            header_map.insert(header_name, header_value);
        }
    }

    let response = client
        .post(endpoint)
        .headers(header_map)
        .body(soap_body)
        .send()
        .await
        .map_err(|e| {
            error!("SOAP request failed for endpoint {}: {}", endpoint, e);
            e
        })?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    // Parse SOAP response (basic XML to JSON conversion)
    let parsed_response = parse_soap_response(&body_text);

    trace!(
        "SOAP request to {} completed with status: {}",
        endpoint,
        status
    );

    Ok(json!({
        "status": status.as_u16(),
        "data": parsed_response
    }))
}

/// Build a SOAP envelope from operation and parameters
fn build_soap_envelope(operation: &str, params: &Value) -> String {
    let mut params_xml = String::new();

    if let Value::Object(obj) = params {
        for (key, value) in obj {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            params_xml.push_str(&format!("<{}>{}</{}>", key, value_str, key));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <{operation} xmlns="http://tempuri.org/">
            {params_xml}
        </{operation}>
    </soap:Body>
</soap:Envelope>"#,
        operation = operation,
        params_xml = params_xml
    )
}

/// Parse SOAP response XML to JSON (basic implementation)
fn parse_soap_response(xml: &str) -> Value {
    // Basic XML parsing - extracts content between Body tags
    if let Some(body_start) = xml.find("<soap:Body>") {
        if let Some(body_end) = xml.find("</soap:Body>") {
            let body_content = &xml[body_start + 11..body_end];
            return json!({
                "raw": body_content.trim(),
                "xml": xml
            });
        }
    }

    // Also check for SOAP 1.2 format
    if let Some(body_start) = xml.find("<soap12:Body>") {
        if let Some(body_end) = xml.find("</soap12:Body>") {
            let body_content = &xml[body_start + 13..body_end];
            return json!({
                "raw": body_content.trim(),
                "xml": xml
            });
        }
    }

    json!({
        "raw": xml,
        "xml": xml
    })
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
        if let Some(f) = value.as_float().ok() {
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
        let map = value.clone().try_cast::<Map>().unwrap_or_default();
        let obj: serde_json::Map<String, Value> = map
            .iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        Value::Object(obj)
    } else {
        Value::String(value.to_string())
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
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_to_json_string() {
        let dynamic = Dynamic::from("hello");
        let json = dynamic_to_json(&dynamic);
        assert_eq!(json, Value::String("hello".to_string()));
    }

    #[test]
    fn test_dynamic_to_json_number() {
        let dynamic = Dynamic::from(42_i64);
        let json = dynamic_to_json(&dynamic);
        assert_eq!(json, Value::Number(42.into()));
    }

    #[test]
    fn test_build_soap_envelope() {
        let params = json!({"name": "John", "age": 30});
        let envelope = build_soap_envelope("GetUser", &params);
        assert!(envelope.contains("<GetUser"));
        assert!(envelope.contains("<name>John</name>"));
        assert!(envelope.contains("<age>30</age>"));
    }

    #[test]
    fn test_parse_soap_response() {
        let xml = r#"<?xml version="1.0"?><soap:Envelope><soap:Body><Result>Success</Result></soap:Body></soap:Envelope>"#;
        let result = parse_soap_response(xml);
        assert!(result.get("raw").is_some());
    }
}
