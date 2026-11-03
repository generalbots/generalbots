//! Email a Drive share link (#1639 follow-up).
//!
//! Two senders are supported, in this order:
//! 1. a connected mail account from `user_email_accounts` (Gmail, Outlook or
//!    any SMTP account), including OAuth2 accounts through
//!    [`botemail::oauth::resolve_credentials`];
//! 2. the deployment's native SMTP credentials (`secret/gbo/email`), which is
//!    what a Stalwart install uses.
//!
//! The link itself is minted by the same helper as the "Share link" action, so
//! expiry and revocation behave identically.

use super::*;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;

const MAX_RECIPIENTS: usize = 10;

#[derive(Debug, Deserialize)]
pub struct ShareEmailBody {
    pub bucket: Option<String>,
    pub path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
    /// Absolute site origin, used to build the clickable URL (the API itself
    /// only knows the relative `/api/files/public/<token>` path).
    pub origin: Option<String>,
    pub recipients: Vec<String>,
    pub message: Option<String>,
    pub expires_at: Option<String>,
    /// Optional connected account id. When absent the primary active account
    /// is used, then the deployment SMTP credentials.
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShareEmailAccountsQuery {
    pub user_id: Option<String>,
}

#[derive(Clone)]
struct SenderConfig {
    from: String,
    host: String,
    port: u16,
    username: String,
    secret: String,
    oauth2: bool,
    label: String,
}

fn valid_recipients(input: &[String]) -> Result<Vec<String>, (StatusCode, Json<serde_json::Value>)> {
    let mut out: Vec<String> = Vec::new();
    for raw in input {
        for part in raw.split([',', ';', ' ']) {
            let email = part.trim();
            if email.is_empty() {
                continue;
            }
            if !email.contains('@')
                || email.len() > 254
                || email.contains('<')
                || email.contains('>')
                || email.contains('\n')
                || email.contains('\r')
            {
                return Err(err(StatusCode::BAD_REQUEST, "Invalid recipient email"));
            }
            if !out.iter().any(|existing| existing.eq_ignore_ascii_case(email)) {
                out.push(email.to_string());
            }
        }
    }
    if out.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "At least one recipient is required"));
    }
    if out.len() > MAX_RECIPIENTS {
        return Err(err(StatusCode::BAD_REQUEST, "Too many recipients"));
    }
    Ok(out)
}

fn absolute_url(origin: Option<&str>, path: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let base = origin
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| std::env::var("GB_PUBLIC_URL").ok())
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "A public origin is required to email a link"))?;
    Ok(format!("{}{}", base.trim_end_matches('/'), path))
}

/// Resolves the account the user asked for, else their primary active account.
async fn connected_sender(
    state: &Arc<AppState>,
    user_id: uuid::Uuid,
    account_id: Option<&str>,
) -> Option<SenderConfig> {
    #[derive(diesel::QueryableByName)]
    struct AccountRow {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = Nullable<Text>)]
        display_name: Option<String>,
        #[diesel(sql_type = Text)]
        smtp_server: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        smtp_port: i32,
    }

    let mut conn = state.conn.get().ok()?;
    let requested = account_id.unwrap_or("").to_string();
    let account = diesel::sql_query(
        "SELECT id::text, email, display_name, smtp_server, smtp_port \
         FROM user_email_accounts \
         WHERE user_id = $1 AND is_active = true \
           AND ($2 = '' OR id::text = $2) \
         ORDER BY is_primary DESC, updated_at DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<Text, _>(&requested)
    .get_result::<AccountRow>(&mut conn)
    .ok()?;

    let account_uuid = uuid::Uuid::parse_str(&account.id).ok()?;
    let state_for_credentials = state.clone();
    let resolved = tokio::task::spawn_blocking(move || {
        let mut conn = state_for_credentials.conn.get().ok()?;
        botemail::oauth::resolve_credentials(&mut conn, account_uuid).ok()
    })
    .await
    .ok()??;

    let from = match account.display_name.as_deref().filter(|name| !name.is_empty()) {
        Some(name) => format!("{name} <{}>", account.email),
        None => account.email.clone(),
    };
    let oauth2 = resolved.is_oauth2();
    Some(SenderConfig {
        from,
        host: account.smtp_server,
        port: u16::try_from(account.smtp_port).unwrap_or(587),
        username: resolved.username,
        secret: resolved.secret,
        oauth2,
        label: account.email,
    })
}

/// Native deployment mailbox (`secret/gbo/email`), i.e. Stalwart.
async fn native_sender() -> Option<SenderConfig> {
    let config = tokio::task::spawn_blocking(|| {
        botcoresecrets::SecretsManager::get()
            .ok()
            .map(|manager| manager.get_email_config())
    })
    .await
    .ok()??;
    let (host, port, username, password, from) = config;
    if host.is_empty() || host == "none" || from.is_empty() || from == "none" {
        return None;
    }
    let label = from.clone();
    Some(SenderConfig {
        from,
        host,
        port,
        username,
        secret: password,
        oauth2: false,
        label,
    })
}

fn build_mail(
    sender: &SenderConfig,
    to: &str,
    subject: &str,
    html: String,
) -> Result<Message, String> {
    Message::builder()
        .from(sender.from.parse().map_err(|e| format!("invalid from: {e}"))?)
        .to(to.parse().map_err(|e| format!("invalid recipient: {e}"))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
        .map_err(|e| format!("message build failed: {e}"))
}

fn transport(sender: &SenderConfig) -> Result<SmtpTransport, String> {
    let local = sender.host == "localhost"
        || sender.host.starts_with("127.")
        || sender.host == "::1"
        || sender.host.ends_with(".local");
    let builder = if local {
        SmtpTransport::builder_dangerous(&sender.host)
    } else {
        SmtpTransport::relay(&sender.host).map_err(|e| format!("SMTP relay failed: {e}"))?
    };
    let mut builder = builder
        .port(sender.port)
        .credentials(Credentials::new(sender.username.clone(), sender.secret.clone()));
    if sender.oauth2 {
        builder = builder.authentication(vec![Mechanism::Xoauth2]);
    }
    Ok(builder.build())
}

fn send_blocking(sender: &SenderConfig, to: &str, subject: &str, html: String) -> Result<(), String> {
    let mail = build_mail(sender, to, subject, html)?;
    transport(sender)?
        .send(&mail)
        .map(|_| ())
        .map_err(|e| format!("SMTP send failed: {e}"))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `GET /api/files/share/email/accounts` — the sender picker. Reports the
/// user's connected accounts and whether the deployment mailbox is usable.
pub async fn share_email_accounts(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ShareEmailAccountsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    let account_user = uuid::Uuid::parse_str(&uid).unwrap_or(user.user_id);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = Nullable<Text>)]
        display_name: Option<String>,
        #[diesel(sql_type = Text)]
        auth_mode: String,
    }
    let accounts: Vec<serde_json::Value> = diesel::sql_query(
        "SELECT id::text, email, COALESCE(display_name, '') AS display_name, auth_mode \
         FROM user_email_accounts WHERE user_id = $1 AND is_active = true \
         ORDER BY is_primary DESC, updated_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(account_user)
    .load::<Row>(&mut conn)
    .unwrap_or_default()
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "id": row.id,
            "email": row.email,
            "display_name": row.display_name.unwrap_or_default(),
            "auth_mode": row.auth_mode,
        })
    })
    .collect();

    let native = native_sender().await;
    Ok(Json(serde_json::json!({
        "accounts": accounts,
        "native": {
            "configured": native.is_some(),
            "from": native.as_ref().map(|sender| sender.label.clone()).unwrap_or_default(),
        }
    })))
}

/// `POST /api/files/share/email` — mints (or reuses) a public link and emails
/// it through the chosen account or the deployment mailbox.
pub async fn send_share_email(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ShareEmailBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let recipients = valid_recipients(&req.recipients)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket =
        resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None, &user)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let expires_at = match req.expires_at.as_deref() {
        None | Some("") => None,
        Some(value) => match chrono::DateTime::parse_from_rfc3339(value) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => return Err(err(StatusCode::BAD_REQUEST, "expires_at must be RFC3339")),
        },
    };

    let sender = match connected_sender(&state, user.user_id, req.account_id.as_deref()).await {
        Some(sender) => sender,
        None => native_sender().await.ok_or_else(|| {
            err(
                StatusCode::BAD_REQUEST,
                "No email sender configured: connect Gmail/Outlook in Integrations or set secret/gbo/email",
            )
        })?,
    };

    let minted = mint_public_link(
        &state,
        &uid,
        &bucket,
        &scope,
        &prefix,
        &req.path,
        expires_at,
    )
    .await?;
    let url = absolute_url(req.origin.as_deref(), &minted.url)?;

    let file_name = req
        .path
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("file");
    let note = req.message.as_deref().unwrap_or("").trim();
    let expiry_line = match minted.expires_at {
        Some(when) => format!(
            "This link expires on {}. You can revoke it at any time.",
            when.format("%Y-%m-%d %H:%M UTC")
        ),
        None => "This link does not expire. You can revoke it at any time.".to_string(),
    };
    let html = format!(
        "<p>{note}</p><p><a href=\"{url}\">Download {file}</a></p><p style=\"color:#64748b;font-size:12px\">{expiry_line}</p>",
        note = html_escape(note),
        url = html_escape(&url),
        file = html_escape(file_name),
    );
    let subject = format!("{} shared \"{}\" with you", sender.label, file_name);

    let mut sent = 0usize;
    let mut failures = 0usize;
    for recipient in &recipients {
        let sender_for_send = sender.clone();
        let to = recipient.clone();
        let subject = subject.clone();
        let html = html.clone();
        match tokio::task::spawn_blocking(move || {
            send_blocking(&sender_for_send, &to, &subject, html)
        })
        .await
        {
            Ok(Ok(())) => sent += 1,
            Ok(Err(error)) => {
                failures += 1;
                log::warn!("share email to {recipient} failed: {error}");
            }
            Err(error) => {
                failures += 1;
                log::warn!("share email task failed: {error}");
            }
        }
    }

    if sent == 0 {
        return Err(err(
            StatusCode::BAD_GATEWAY,
            "The link was created but the email could not be sent",
        ));
    }

    botlog::safe_info!(
        "share email: {sent} sent, {failures} failed for {} via {}",
        req.path,
        sender.label
    );
    Ok(Json(serde_json::json!({
        "sent": sent,
        "failed": failures,
        "recipients": recipients,
        "url": url,
        "expires_at": format_link_time(minted.expires_at),
        "sender": sender.label,
    })))
}

#[cfg(test)]
mod tests {
    use super::{absolute_url, valid_recipients};

    #[test]
    fn recipients_are_split_deduped_and_validated() {
        let parsed = valid_recipients(&["a@example.com, b@example.com".into(), "A@example.com".into()])
            .expect("valid list");
        assert_eq!(parsed, vec!["a@example.com", "b@example.com"]);
        assert!(valid_recipients(&["not-an-email".into()]).is_err());
        assert!(valid_recipients(&[]).is_err());
    }

    #[test]
    fn origin_must_be_absolute() {
        assert_eq!(
            absolute_url(Some("https://chat.example.com"), "/api/files/public/t").unwrap(),
            "https://chat.example.com/api/files/public/t"
        );
        assert!(absolute_url(Some("chat.example.com"), "/x").is_err());
        assert!(absolute_url(None, "/x").is_err() || std::env::var("GB_PUBLIC_URL").is_ok());
    }
}
