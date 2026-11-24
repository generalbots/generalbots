use super::*;

pub(super) async fn exchange(
    pool: &DbPool,
    code: &str,
    verifier: &str,
    user_id: Uuid,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client: {error}"))?;
    let response = client
        .post(token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri().as_str()),
            ("client_id", client_id().as_str()),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|error| format!("token request: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("token response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "the token endpoint refused the code ({status}): {}",
            body.get("error_description")
                .or_else(|| body.get("error"))
                .and_then(Value::as_str)
                .unwrap_or("no detail")
        ));
    }
    let access = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "the token endpoint returned no access token".to_string())?;
    let refresh = body
        .get("refresh_token")
        .and_then(Value::as_str)
        .unwrap_or("");
    let id_token = body.get("id_token").and_then(Value::as_str).unwrap_or("");
    let expires_in = body
        .get("expires_in")
        .and_then(Value::as_u64)
        .unwrap_or(3600);
    let claims = decode_jwt_payload(id_token).unwrap_or(Value::Null);
    let email = claims
        .get("email")
        .and_then(Value::as_str)
        .map(str::to_string);
    let account_id = claim_string(&claims, "chatgpt_account_id");
    let plan = claim_string(&claims, "chatgpt_plan_type");

    let mut data = HashMap::new();
    data.insert("access_token".to_string(), access.to_string());
    data.insert("refresh_token".to_string(), refresh.to_string());
    data.insert("id_token".to_string(), id_token.to_string());
    data.insert(
        "expires_at".to_string(),
        (now_secs() + expires_in).to_string(),
    );
    data.insert(
        "base_url".to_string(),
        std::env::var("OPENAI_RESPONSES_BASE_URL")
            .unwrap_or_else(|_| botllm::openai_responses::DEFAULT_BASE_URL.to_string()),
    );
    if let Some(account) = account_id.clone() {
        data.insert("account_id".to_string(), account);
    }
    if let Some(email) = email.clone() {
        data.insert("email".to_string(), email);
    }
    if let Some(plan) = plan.clone() {
        data.insert("plan".to_string(), plan);
    }
    let manager = botcoresecrets::SecretsManager::get()
        .map_err(|error| format!("Vault is unavailable: {error}"))?;
    let vault_path = crate::llm::subscription::chatgpt_vault_path(user_id);
    manager
        .put_secret(&vault_path, data)
        .await
        .map_err(|error| format!("could not store the token in Vault: {error}"))?;
    ai_accounts::ensure_user(pool, user_id, email.as_deref())
        .map_err(|error| format!("could not record the account owner: {error}"))?;
    ai_accounts::mark_linked(
        pool,
        user_id,
        ai_accounts::CHATGPT_PLAN,
        email.as_deref(),
        plan.as_deref(),
        None,
        Some(&vault_path),
    )
    .map_err(|error| format!("could not record the linked account: {error}"))?;
    Ok(())
}

/// Reads the stored token, refreshing it when it is close to expiry.
pub(crate) async fn access_token(user_id: Uuid) -> Result<(String, Option<String>), String> {
    let manager = botcoresecrets::SecretsManager::get()
        .map_err(|error| format!("Vault is unavailable: {error}"))?;
    let path = crate::llm::subscription::chatgpt_vault_path(user_id);
    let secret = manager
        .get_secret(&path)
        .await
        .map_err(|error| format!("could not read the ChatGPT token: {error}"))?;
    let access = secret.get("access_token").cloned().map_or_else(Default::default, |v| v);
    let expires_at = secret
        .get("expires_at")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let account = secret.get("account_id").cloned();
    if !access.is_empty() && expires_at > now_secs() + 60 {
        return Ok((access, account));
    }
    let refresh = secret.get("refresh_token").cloned().map_or_else(Default::default, |v| v);
    if refresh.is_empty() {
        return Err("The ChatGPT plan token expired; connect the account again.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client: {error}"))?;
    let response = client
        .post(token_url())
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.as_str()),
            ("client_id", client_id().as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("refresh request: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("refresh response: {error}"))?;
    if !status.is_success() {
        return Err("The ChatGPT plan session expired; connect the account again.".to_string());
    }
    let access = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "the refresh response returned no access token".to_string())?;
    let expires_in = body
        .get("expires_in")
        .and_then(Value::as_u64)
        .unwrap_or(3600);
    let mut updated = secret;
    updated.insert("access_token".to_string(), access.to_string());
    updated.insert(
        "expires_at".to_string(),
        (now_secs() + expires_in).to_string(),
    );
    if let Some(refresh) = body.get("refresh_token").and_then(Value::as_str) {
        if !refresh.is_empty() {
            updated.insert("refresh_token".to_string(), refresh.to_string());
        }
    }
    manager
        .put_secret(&path, updated)
        .await
        .map_err(|error| format!("could not refresh the ChatGPT token: {error}"))?;
    Ok((access.to_string(), account))
}
