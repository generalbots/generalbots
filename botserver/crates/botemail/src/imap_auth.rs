//! Shared IMAP session handling.
//!
//! Two authentication mechanisms are supported, matching the two account
//! authentication modes:
//!
//! * `LOGIN` with a password, for self-hosted mail servers.
//! * `AUTHENTICATE XOAUTH2`, required by Microsoft 365, Outlook.com and Gmail,
//!   which no longer accept a static password over IMAP.
//!
//! The mailbox sync worker and the connectivity check performed when an account
//! is added both open their session here, so a host that is reachable in one
//! path is reachable in the other.

use log::warn;
use uuid::Uuid;

use crate::oauth::{resolve_credentials, MailMechanism};

/// Authentication material for an IMAP session.
#[derive(Debug, Clone)]
pub enum ImapAuth {
    /// `LOGIN` with a password.
    Password {
        username: String,
        password: String,
    },
    /// `AUTHENTICATE XOAUTH2` with a bearer token.
    OAuth2 {
        username: String,
        access_token: String,
    },
}

/// Session type produced by [`open_session`].
pub type ImapSession = imap::Session<imap::Connection>;

/// Opens and authenticates an IMAPS session.
pub fn open_session(host: &str, port: u16, auth: &ImapAuth) -> Result<ImapSession, String> {
    let client = imap::ClientBuilder::new(host, port)
        .connect()
        .map_err(|e| format!("IMAP connect failed: {e:?}"))?;

    match auth {
        ImapAuth::Password { username, password } => client
            .login(username, password)
            .map_err(|(e, _)| format!("IMAP login failed: {e:?}")),
        ImapAuth::OAuth2 {
            username,
            access_token,
        } => {
            let authenticator = Xoauth2Authenticator {
                username: username.clone(),
                access_token: access_token.clone(),
            };
            client
                .authenticate("XOAUTH2", &authenticator)
                .map_err(|(e, _)| format!("IMAP XOAUTH2 failed: {e:?}"))
        }
    }
}

/// Resolves the stored credentials for an account and adapts them to the IMAP
/// client. Password and OAuth2 accounts are handled in one place so the sync
/// worker and the send path always authenticate the same way.
pub fn resolve_imap_auth(
    conn: &mut diesel::PgConnection,
    account_id: Uuid,
) -> Result<ImapAuth, String> {
    let credentials = resolve_credentials(conn, account_id)?;
    Ok(match credentials.mechanism {
        MailMechanism::Password => ImapAuth::Password {
            username: credentials.username,
            password: credentials.secret,
        },
        MailMechanism::Xoauth2 => ImapAuth::OAuth2 {
            username: credentials.username,
            access_token: credentials.secret,
        },
    })
}

/// Verifies that a mailbox can be reached and opened.
///
/// Used when an account is added, so an unreachable host is rejected at
/// creation time instead of failing on every background sync pass while the
/// user sees an empty inbox.
pub fn verify_connectivity(host: &str, port: u16, auth: &ImapAuth) -> Result<(), String> {
    let mut session = open_session(host, port, auth)?;
    session
        .select("INBOX")
        .map_err(|e| format!("IMAP INBOX select failed: {e:?}"))?;
    if let Err(e) = session.logout() {
        warn!("IMAP logout failed after a successful connectivity check: {e:?}");
    }
    Ok(())
}

/// SASL authenticator for the XOAUTH2 mechanism.
struct Xoauth2Authenticator {
    username: String,
    access_token: String,
}

impl imap::Authenticator for Xoauth2Authenticator {
    type Response = String;

    fn process(&self, challenge: &[u8]) -> Self::Response {
        // The server sends an empty challenge on success and a base64-decoded
        // JSON error document on failure. Reporting it turns an opaque
        // authentication failure into something actionable.
        if !challenge.is_empty() {
            warn!(
                "XOAUTH2 challenge: {}",
                String::from_utf8_lossy(challenge)
            );
        }
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.username, self.access_token
        )
    }
}
