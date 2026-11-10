//! SMTP plumbing for the Drive share email (#2094).
//!
//! Split out of `share_email.rs` so the handler file stays inside the size
//! budget. Nothing here is user-facing text: the subject and the HTML body are
//! built by the caller from the catalogue.

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{Message, SmtpTransport, Transport};

/// Everything needed to send through one mailbox.
#[derive(Clone)]
pub(super) struct SenderConfig {
    pub(super) from: String,
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) username: String,
    pub(super) secret: String,
    pub(super) oauth2: bool,
    pub(super) label: String,
}

pub(super) fn build_mail(
    sender: &SenderConfig,
    to: &str,
    subject: &str,
    html: String,
) -> Result<Message, String> {
    Message::builder()
        .from(
            sender
                .from
                .parse()
                .map_err(|e| format!("invalid from: {e}"))?,
        )
        .to(to.parse().map_err(|e| format!("invalid recipient: {e}"))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
        .map_err(|e| format!("message build failed: {e}"))
}

pub(super) fn transport(sender: &SenderConfig) -> Result<SmtpTransport, String> {
    let local = sender.host == "localhost"
        || sender.host.starts_with("127.")
        || sender.host == "::1"
        || sender.host.ends_with(".local");
    let builder = if local {
        SmtpTransport::builder_dangerous(&sender.host)
    } else {
        SmtpTransport::relay(&sender.host).map_err(|e| format!("SMTP relay failed: {e}"))?
    };
    let mut builder = builder.port(sender.port).credentials(Credentials::new(
        sender.username.clone(),
        sender.secret.clone(),
    ));
    if sender.oauth2 {
        builder = builder.authentication(vec![Mechanism::Xoauth2]);
    }
    Ok(builder.build())
}

pub(super) fn send_blocking(
    sender: &SenderConfig,
    to: &str,
    subject: &str,
    html: String,
) -> Result<(), String> {
    let mail = build_mail(sender, to, subject, html)?;
    transport(sender)?
        .send(&mail)
        .map(|_| ())
        .map_err(|e| format!("SMTP send failed: {e}"))
}

pub(super) fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
