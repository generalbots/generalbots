use super::*;
/// `GET /api/external-drives/callback` — provider redirect target.
///
/// Anonymous by necessity; the signed `state` is what proves the flow belongs
/// to this branch, and the PKCE verifier it carries is what completes the code
/// exchange. On success the browser is sent back to the Drive app with the
/// connection already stored, so the user never lands on a raw JSON page.
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<CallbackQuery>,
) -> Response {
    let base = config::base_from_headers(&headers, None, None).unwrap_or_else(String::new);
    let back = |query: &str| {
        let prefix = if base.is_empty() {
            "/drive".to_string()
        } else {
            format!("{base}/drive")
        };
        Redirect::to(&format!("{prefix}?{query}"))
    };

    if let Some(provider_error) = params.error.as_deref().filter(|e| !e.is_empty()) {
        log::warn!("external drive oauth denied by provider: {provider_error}");
        return back(&format!(
            "external=error&reason={}",
            urlencode(provider_error)
        ))
        .into_response();
    }
    let Some(token) = params.state.as_deref().filter(|s| !s.is_empty()) else {
        return back("external=error&reason=missing_state").into_response();
    };
    // Authenticate the signed state and require the PKCE verifier it must
    // carry. Missing, malformed, forged, expired or verifier-less state all
    // fail closed here instead of reaching the token endpoint.
    let Some((branch_id, provider, verifier)) =
        crypto::verify_callback_state(token, chrono::Utc::now())
    else {
        log::warn!("external drive oauth state or PKCE verifier rejected");
        return back("external=error&reason=invalid_state").into_response();
    };
    let Some(code) = params.code.as_deref().filter(|c| !c.is_empty()) else {
        return back("external=error&reason=missing_code").into_response();
    };
    let Some(client) = config::client_for(provider) else {
        return back("external=error&reason=not_configured").into_response();
    };
    let redirect_uri =
        config::redirect_uri(config::base_from_headers(&headers, None, None).as_deref());
    let http = match engine::http_client() {
        Ok(c) => c,
        Err(_) => return back("external=error&reason=client_unavailable").into_response(),
    };

    // Each provider has its own token response type, so the two arms are
    // flattened into the fields that actually get stored. #1643 — `scope`
    // carries what the provider reports as granted, for the consent record.
    let (access_token, refresh_token, expires_at, granted_scope) = match provider {
        Provider::OneDrive => {
            match crate::external::onedrive::exchange_code_with_pkce(
                &http,
                &client.tenant_id,
                &client.client_id,
                client.client_secret.as_deref(),
                &redirect_uri,
                code,
                Some(verifier.as_str()),
            )
            .await
            {
                Ok(t) => (t.access_token, t.refresh_token, t.expires_at, t.scope),
                Err(e) => {
                    log::warn!("external drive oauth exchange failed: {e}");
                    return back("external=error&reason=exchange_failed").into_response();
                }
            }
        }
        Provider::GoogleDrive => {
            match crate::external::gdrive::exchange_code_with_pkce(
                &http,
                &client.client_id,
                client.client_secret.as_deref(),
                &redirect_uri,
                code,
                Some(verifier.as_str()),
            )
            .await
            {
                Ok(t) => (t.access_token, t.refresh_token, t.expires_at, t.scope),
                Err(e) => {
                    log::warn!("external drive oauth exchange failed: {e}");
                    return back("external=error&reason=exchange_failed").into_response();
                }
            }
        }
    };

    let stored = (|| -> Result<(), String> {
        let access_enc = crypto::encrypt_token(&access_token)?;
        let refresh_enc = refresh_token
            .as_deref()
            .map(crypto::encrypt_token)
            .transpose()?;
        let mut conn = db::conn(&state.conn)?;
        db::upsert_connection(
            &mut conn,
            branch_id,
            provider,
            &access_enc,
            refresh_enc.as_deref(),
            Some(expires_at),
            None,
            granted_scope.as_deref(),
        )
    })();
    if let Err(e) = stored {
        log::error!("external drive connection could not be stored: {e}");
        return back("external=error&reason=storage_failed").into_response();
    }

    // The first pass mirrors the account, so the tab has content immediately
    // instead of an empty list the user has to trigger.
    let report = engine::sync_connection(&state.conn, branch_id, provider).await;
    if !report.ok() {
        log::warn!(
            "external drive {} connected but the first sync reported {:?}",
            provider.as_str(),
            report.error
        );
    }
    back(&format!(
        "external=connected&provider={}&files={}",
        provider.as_str(),
        report.upserted
    ))
    .into_response()
}
