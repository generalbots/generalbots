use super::*;
use botsecurity_core::validation::{resolve_checked, validate_url_ssrf_strict};
use url::Url;

/// Exact hosts allowed to receive a custom-channel Discord webhook (#2641).
/// A suffix or substring match would let `discord.com.evil.example` through.
const WEBHOOK_HOSTS: [&str; 2] = ["discord.com", "discordapp.com"];

/// Longest preview of an untrusted provider body that may reach the log.
const BODY_LOG_LIMIT: usize = 256;

/// Single generic refusal used for every rejected webhook target. The detail
/// (host, reason) goes to the server log only, never into a `ChannelError`
/// that the UI renders back to the caller.
fn webhook_rejected() -> ChannelError {
    ChannelError::ApiError {
        code: None,
        message: "Discord webhook URL is not permitted".to_string(),
    }
}

/// Synchronous parse + host pin. `validate_url_ssrf_strict` rejects non-http(s)
/// schemes, userinfo, local/metadata hosts, obfuscated IPv4, non-global IP
/// literals and control-plane ports; the exact-host check then closes the
/// remaining SSRF surface. No DNS is performed, so this is also safe to call
/// while validating stored credentials.
pub(super) fn parse_webhook_url(raw: &str) -> Result<Url, ChannelError> {
    let parsed = validate_url_ssrf_strict(raw).map_err(|error| {
        log::warn!("Refusing Discord webhook URL: {}", error.detail());
        webhook_rejected()
    })?;

    if parsed.scheme() != "https" {
        log::warn!("Refusing Discord webhook URL: scheme is not https");
        return Err(webhook_rejected());
    }

    let host = parsed.host_str().unwrap_or("").to_ascii_lowercase();
    if !WEBHOOK_HOSTS.contains(&host.as_str()) {
        log::warn!("Refusing Discord webhook URL: host is not a Discord host");
        return Err(webhook_rejected());
    }

    Ok(parsed)
}

/// Resolves the pinned host before the POST so a poisoned DNS answer cannot
/// redirect the request to a private address (mixed public/private answers are
/// refused too). reqwest's rustls backend then verifies the TLS peer for the
/// same host, completing the peer check.
pub(super) async fn check_webhook_target(raw: &str) -> Result<Url, ChannelError> {
    let parsed = parse_webhook_url(raw)?;
    resolve_checked(&parsed).await.map_err(|error| {
        log::warn!(
            "Refusing Discord webhook after DNS check: {}",
            error.detail()
        );
        webhook_rejected()
    })?;
    Ok(parsed)
}

/// Provider bodies are untrusted (and, for a webhook, may echo caller data).
/// Keep them out of user-facing errors; log only a bounded, printable preview.
pub(super) fn sanitized_body_preview(body: &str) -> String {
    let mut preview = String::with_capacity(body.len().min(BODY_LOG_LIMIT));
    let mut printed = 0usize;
    for ch in body.chars() {
        if ch.is_control() {
            continue;
        }
        if printed == BODY_LOG_LIMIT {
            preview.push_str("...[truncated]");
            return preview;
        }
        preview.push(ch);
        printed += 1;
    }
    preview
}

pub(super) fn provider_api_error(
    status: reqwest::StatusCode,
    body: &str,
    context: &str,
) -> ChannelError {
    log::warn!(
        "Discord {context} failed with status {status}: {}",
        sanitized_body_preview(body)
    );
    ChannelError::ApiError {
        code: Some(status.to_string()),
        message: format!("Discord API request failed ({status})"),
    }
}

fn provider_decode_error(error: reqwest::Error, context: &str) -> ChannelError {
    log::warn!("Discord {context} response could not be decoded: {error}");
    ChannelError::ApiError {
        code: None,
        message: format!("Discord {context} response could not be decoded"),
    }
}

fn provider_auth_error(status: reqwest::StatusCode, body: &str) -> ChannelError {
    log::warn!(
        "Discord credential check failed with status {status}: {}",
        sanitized_body_preview(body)
    );
    ChannelError::AuthenticationFailed("Discord rejected the bot credentials".to_string())
}

impl DiscordProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://discord.com/api/v10".to_string(),
        }
    }

    pub async fn send_message(
        &self,
        bot_token: &str,
        channel_id: &str,
        content: &MessageContent,
    ) -> Result<DiscordMessage, ChannelError> {
        let response = self
            .client
            .post(format!("{}/channels/{}/messages", self.base_url, channel_id))
            .header("Authorization", format!("Bot {}", bot_token))
            .header("Content-Type", "application/json")
            .json(content)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(ChannelError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = crate::response_error_text(response).await;
            return Err(provider_api_error(status, &error_text, "message send"));
        }

        response
            .json::<DiscordMessage>()
            .await
            .map_err(|e| provider_decode_error(e, "message send"))
    }

    pub(crate) async fn send_webhook_message(
        &self,
        webhook_url: &str,
        content: &WebhookContent,
    ) -> Result<DiscordMessage, ChannelError> {
        // #2641 — re-check at send time: credentials may have been stored before
        // this guard existed, and DNS may have changed since configuration.
        let parsed = check_webhook_target(webhook_url).await?;
        let url = format!("{}?wait=true", parsed.as_str());

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(content)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(ChannelError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = crate::response_error_text(response).await;
            return Err(provider_api_error(status, &error_text, "webhook delivery"));
        }

        response
            .json::<DiscordMessage>()
            .await
            .map_err(|e| provider_decode_error(e, "webhook delivery"))
    }

    pub(crate) async fn get_current_user(&self, bot_token: &str) -> Result<DiscordUser, ChannelError> {
        let response = self
            .client
            .get(format!("{}/users/@me", self.base_url))
            .header("Authorization", format!("Bot {}", bot_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = crate::response_error_text(response).await;
            return Err(provider_auth_error(status, &error_text));
        }

        response
            .json::<DiscordUser>()
            .await
            .map_err(|e| provider_decode_error(e, "credential check"))
    }

    pub(super) fn build_message_url(guild_id: &str, channel_id: &str, message_id: &str) -> String {
        format!(
            "https://discord.com/channels/{}/{}/{}",
            guild_id, channel_id, message_id
        )
    }

    pub(super) fn create_embeds(content: &PostContent) -> Vec<DiscordEmbed> {
        let mut embeds = Vec::new();

        if content.link.is_some() || !content.image_urls.is_empty() {
            let mut embed = DiscordEmbed {
                title: None,
                description: None,
                url: content.link.clone(),
                color: Some(0x5865F2),
                image: None,
                thumbnail: None,
                fields: vec![],
                footer: None,
                timestamp: None,
            };

            if let Some(first_image) = content.image_urls.first() {
                embed.image = Some(EmbedImage {
                    url: first_image.clone(),
                });
            }

            embeds.push(embed);

            for image_url in content.image_urls.iter().skip(1).take(3) {
                embeds.push(DiscordEmbed {
                    title: None,
                    description: None,
                    url: None,
                    color: None,
                    image: Some(EmbedImage {
                        url: image_url.clone(),
                    }),
                    thumbnail: None,
                    fields: vec![],
                    footer: None,
                    timestamp: None,
                });
            }
        }

        embeds
    }
}
impl Default for DiscordProvider {
    fn default() -> Self {
        Self::new()
    }
}
