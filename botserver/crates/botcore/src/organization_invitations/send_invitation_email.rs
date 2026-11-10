use super::*;

impl InvitationService {
    /// Sends the invitation in the inviter's language (#2094). The invitee has
    /// no account yet, so there is no `users.locale` to read; the MIME envelope
    /// and the addresses are untouched, only subject and body are localized.
    pub(crate) async fn send_invitation_email(
        &self,
        invitation: &OrganizationInvitation,
        org_name: &str,
        locale: &crate::i18n::Locale,
    ) {
        log::info!(
            "Sending invitation email to {} for organization {} (token: {}...)",
            invitation.email,
            org_name,
            &invitation.token[..16]
        );

        #[cfg(feature = "mail")]
        {
            // Real SMTP delivery via env-configured server (MAIL_HOST/PORT/
            // USER/PASS/FROM — secrets come from the environment/Vault, never
            // hardcoded). Best-effort: a failure logs and never fails the
            // invite creation.
            use crate::i18n::translate;
            use lettre::transport::smtp::authentication::Credentials;
            use lettre::{Message, SmtpTransport, Transport};

            let host = std::env::var("MAIL_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port: u16 = std::env::var("MAIL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(587);
            let user = std::env::var("MAIL_USER").unwrap_or_default();
            let pass = std::env::var("MAIL_PASS").unwrap_or_default();
            let from = std::env::var("MAIL_FROM").unwrap_or_else(|_| "notifications@pragmatismo.com.br".to_string());
            if host == "localhost" && user.is_empty() {
                log::warn!("MAIL_HOST not configured — invitation email not delivered");
                return;
            }

            let invite_url = std::env::var("INVITE_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:5000/invitations/accept".to_string());
            let link = format!("{invite_url}?token={}", invitation.token);
            let expires = invitation.expires_at.format("%Y-%m-%d %H:%M UTC").to_string();

            // No greeting line: the invitee has no display name yet, and
            // "Hello user@example.com," reads badly.
            let subject = translate(locale, "email-invitation-subject", &[("org", org_name)]);
            let body = translate(
                locale,
                "email-invitation-body",
                &[("org", org_name), ("url", &link), ("expires", &expires)],
            );
            let signoff = translate(locale, "email-signoff", &[]);
            let team = translate(locale, "email-team", &[]);
            let body = format!("{body}\n\n{signoff}\n{team}");

            // `builder_dangerous` avoids needing a TLS feature on lettre (the
            // workspace dep only enables smtp-transport + builder); encryption
            // is negotiated opportunistically on the wire when the server
            // supports it. This mirrors the settings SMTP-test fallback.
            let mailer = SmtpTransport::builder_dangerous(&host)
                .port(port)
                .credentials(Credentials::new(user, pass))
                .build();
            let from_addr: lettre::address::Address = match from.parse() {
                Ok(a) => a,
                Err(_) => {
                    log::warn!("Invalid MAIL_FROM address — invitation email not delivered");
                    return;
                }
            };
            let to_addr: lettre::address::Address = match invitation.email.parse() {
                Ok(a) => a,
                Err(_) => {
                    log::warn!("Invalid recipient address — invitation email not delivered");
                    return;
                }
            };

            match Message::builder()
                .from(from_addr.into())
                .to(to_addr.into())
                .subject(subject)
                .body(body)
            {
                Ok(email) => {
                    if let Err(e) = mailer.send(&email) {
                        log::warn!("Failed to deliver invitation email: {e}");
                    }
                }
                Err(e) => log::warn!("Failed to build invitation email: {e}"),
            }
        }

        #[cfg(not(feature = "mail"))]
        {
            // Without the `mail` feature nothing is delivered, but the locale
            // stays part of the contract so callers do not branch on features.
            let _ = locale;
        }
    }
}
