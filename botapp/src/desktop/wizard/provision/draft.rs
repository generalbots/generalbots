//! Draft persistence and OAuth client handoff to the running API.

use super::super::{draft_path, stack_path, ProvisionRequest};
use log::{info, warn};

const API_BOTS: &str = "http://127.0.0.1:8080/api/bots/list";

pub(super) fn save_draft(request: &ProvisionRequest) -> std::io::Result<()> {
    let services: std::collections::HashMap<&str, serde_json::Value> = request
        .services
        .iter()
        .filter(|(_, service)| !service.client_id.trim().is_empty())
        .map(|(provider, service)| {
            (
                provider.as_str(),
                serde_json::json!({ "client_id": service.client_id, "client_secret": "" }),
            )
        })
        .collect();
    let draft = serde_json::json!({
        "model": {
            "mode": request.model.mode,
            "provider": request.model.provider,
            "endpoint": request.model.endpoint,
            "model": request.model.model,
        },
        "workspace": request.workspace,
        "display_name": request.display_name,
        "services": services,
        "skip_all_agentic_work": request.skip_all_agentic_work,
    });
    let raw = serde_json::to_string_pretty(&draft)
        .map_err(|e| std::io::Error::other(format!("serialize draft: {e}")))?;
    let stack = stack_path();
    std::fs::create_dir_all(&stack)?;
    let path = draft_path();
    std::fs::write(&path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
    Ok(())
}

/// Stores the OAuth client ids/secrets the user entered, through the running
/// API, once. Secrets never touch a config file and are never logged.
pub(super) async fn apply_integrations(request: &ProvisionRequest) {
    if request.services.is_empty() {
        return;
    }
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            warn!("wizard: could not create the integration client: {e}");
            return;
        }
    };
    let bot_id = match client.get(API_BOTS).send().await {
        Ok(response) if response.status().is_success() => {
            let bots: serde_json::Value = response.json().await.unwrap_or_default();
            bots.as_array()
                .and_then(|list| list.first())
                .and_then(|bot| bot.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| id.to_string())
        }
        Ok(response) => {
            warn!(
                "wizard: could not list bots ({}) to apply integrations",
                response.status()
            );
            None
        }
        Err(e) => {
            warn!("wizard: could not list bots to apply integrations: {e}");
            None
        }
    };
    let Some(bot_id) = bot_id else {
        return;
    };
    for (provider, service) in &request.services {
        if service.client_id.trim().is_empty() {
            continue;
        }
        let url =
            format!("http://127.0.0.1:8080/api/bots/{bot_id}/integrations/oauth/{provider}/client");
        let body = serde_json::json!({
            "client_id": service.client_id,
            "client_secret": if service.client_secret.is_empty() {
                None
            } else {
                Some(&service.client_secret)
            },
        });
        match client.post(&url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                info!("wizard: stored the OAuth client for {provider}");
            }
            Ok(response) => {
                warn!(
                    "wizard: the API rejected the OAuth client for {provider} ({})",
                    response.status()
                );
            }
            Err(e) => {
                warn!("wizard: could not store the OAuth client for {provider}: {e}");
            }
        }
    }
}
