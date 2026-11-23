//! `PLUGIN_CALL` / `PLUGIN_LIST` (#2833).
//!
//! A call is only attempted for a manifest that is installed in the global
//! registry, signed, host-allowlisted, scoped to the calling bot, contained in
//! the work directory and granted `plugin:call`. Credentials are resolved from
//! Vault by `botcorepkg`, never read from the manifest.

use botbasic_types::{BasicRuntime, UserSession};
use botcorepkg::plugin;
use botsecurity::validation::validate_url_ssrf_strict;
use rhai::Engine;
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT_SECONDS: u64 = 15;

fn error_text(message: &str) -> String {
    format!("PluginError: {message}")
}

pub fn register_plugin_keywords(
    _state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let call_user = user.clone();
    engine.register_fn("plugin_call", move |name: &str, fn_name: &str| -> String {
        let registry = match plugin::global_registry() {
            Some(registry) => registry,
            None => return error_text("plugin registry not initialized"),
        };
        call_plugin_fn(registry, &call_user, name, fn_name)
    });

    engine.register_fn("plugin_list", || -> Vec<String> {
        let registry = match plugin::global_registry() {
            Some(registry) => registry,
            None => return Vec::new(),
        };
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(_) => return Vec::new(),
        };
        runtime.block_on(async {
            registry
                .list()
                .await
                .into_iter()
                .map(|manifest| manifest.name)
                .collect()
        })
    });
}

fn call_plugin_fn(
    registry: &plugin::SharedPluginRegistry,
    user: &UserSession,
    name: &str,
    fn_name: &str,
) -> String {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => return error_text("failed to create runtime"),
    };
    let bot_id = user.bot_id.to_string();

    runtime.block_on(async {
        // Every gate lives in `authorize`: signature, bot scope, work-dir
        // containment, permission and host allowlist.
        let manifest = match registry.authorize(name, &bot_id).await {
            Ok(manifest) => manifest,
            Err(e) => {
                log::warn!("[plugin_call] denied plugin '{name}': {e}");
                return error_text("plugin is not authorized");
            }
        };
        let request = match manifest.build_request(fn_name).await {
            Ok(request) => request,
            Err(e) => {
                log::warn!("[plugin_call] cannot build request for '{name}': {e}");
                return error_text("plugin request could not be built");
            }
        };

        // #2131 / #2833: the URL comes from a signed, allowlisted manifest, but
        // re-check it before the fetch. Redirects are disabled below so a
        // redirect cannot turn an allowed endpoint into an internal one.
        if let Err(e) = validate_url_ssrf_strict(&request.url) {
            log::warn!("[plugin_call] blocked endpoint for plugin '{name}': {e}");
            return error_text("plugin endpoint rejected by the outbound request policy");
        }

        let client = match reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .build()
        {
            Ok(client) => client,
            Err(_) => return error_text("failed to build HTTP client"),
        };

        let mut builder = match request.method.to_uppercase().as_str() {
            "GET" => client.get(&request.url),
            "POST" => client.post(&request.url),
            "PUT" => client.put(&request.url),
            "PATCH" => client.patch(&request.url),
            "DELETE" => client.delete(&request.url),
            other => return error_text(&format!("unsupported method {other}")),
        };

        for (key, value) in &request.headers {
            match (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                (Ok(name), Ok(value)) => builder = builder.header(name, value),
                _ => return error_text("invalid plugin header"),
            }
        }
        if let Some(body) = &request.body {
            builder = builder.json(body);
        }

        match builder.send().await {
            Ok(response) => match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    log::error!("[plugin_call] response read failed for '{name}': {e}");
                    error_text("failed to read response")
                }
            },
            Err(e) => {
                log::error!("[plugin_call] request failed for '{name}': {e}");
                error_text("request failed")
            }
        }
    })
}
