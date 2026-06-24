use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use botcorepkg::plugin;
use rhai::Engine;
use std::sync::Arc;

pub fn register_plugin_keywords(
    _state: Arc<dyn BasicRuntime>,
    _user: UserSession,
    engine: &mut Engine,
) {
    engine.register_fn("plugin_call", |name: &str, fn_name: &str| -> String {
        let registry = match plugin::global_registry() {
            Some(r) => r,
            None => return "PluginError: plugin registry not initialized".to_string(),
        };
        call_plugin_fn(registry, name, fn_name)
    });

    engine.register_fn("plugin_list", || -> Vec<String> {
        let registry = match plugin::global_registry() {
            Some(r) => r,
            None => return vec![],
        };
        let rt = tokio::runtime::Runtime::new().ok();
        match rt {
            Some(runtime) => runtime.block_on(async {
                let manifests = registry.list().await;
                manifests.into_iter().map(|m| m.name).collect()
            }),
            None => vec![],
        }
    });
}

fn call_plugin_fn(registry: &plugin::SharedPluginRegistry, name: &str, fn_name: &str) -> String {
    let rt = tokio::runtime::Runtime::new().ok();
    let runtime = match rt {
        Some(r) => r,
        None => return "PluginError: failed to create runtime".to_string(),
    };

    runtime.block_on(async {
        let manifest = match registry.get(name).await {
            Some(m) => m,
            None => return format!("PluginError: plugin '{}' not found", name),
        };

        let request = match manifest.build_request(fn_name) {
            Ok(r) => r,
            Err(e) => return format!("PluginError: {}", e),
        };

        let client = reqwest::Client::new();
        let req_builder = match request.method.to_uppercase().as_str() {
            "GET" => client.get(&request.url),
            "POST" => client.post(&request.url),
            "PUT" => client.put(&request.url),
            "PATCH" => client.patch(&request.url),
            "DELETE" => client.delete(&request.url),
            _ => return format!("PluginError: unsupported method {}", request.method),
        };

        let req_builder = req_builder.headers({
            let mut h = reqwest::header::HeaderMap::new();
            for (key, val) in &request.headers {
                if let (Ok(k), Ok(v)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(val),
                ) {
                    h.insert(k, v);
                }
            }
            h
        });

        let req_builder = if let Some(body) = &request.body {
            req_builder.json(body)
        } else {
            req_builder
        };

        match req_builder.send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(e) => format!("PluginError: failed to read response: {}", e),
            },
            Err(e) => format!("PluginError: request failed: {}", e),
        }
    })
}
