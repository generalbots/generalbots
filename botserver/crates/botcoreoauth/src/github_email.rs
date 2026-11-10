//! GitHub email ownership lookup (#2134).
//!
//! `/user` returns an `email` field that may be a *public profile* address —
//! it proves nothing about who controls the mailbox. `/user/emails` (scope
//! `user:email`, already requested during login) reports `verified` per
//! address, which is the only GitHub signal accepted before an email may be
//! matched to an existing local account.
//!
//! The lookup is best effort: any failure leaves the address unverified, which
//! can only *deny* an email-based match, never grant one.

use reqwest::Client;

const USER_EMAILS_URL: &str = "https://api.github.com/user/emails";

/// Fetches the account's primary address and its verification flag.
pub(crate) async fn fetch_primary_email(
    client: &Client,
    access_token: &str,
) -> Option<(String, bool)> {
    let response = client
        .get(USER_EMAILS_URL)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .bearer_auth(access_token)
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let entries: serde_json::Value = response.json().await.ok()?;
    pick_primary_email(&entries)
}

/// Selects the primary entry, falling back to the first verified one.
fn pick_primary_email(entries: &serde_json::Value) -> Option<(String, bool)> {
    let list = entries.as_array()?;
    let pick = list
        .iter()
        .find(|entry| entry["primary"].as_bool().unwrap_or(false))
        .or_else(|| {
            list.iter()
                .find(|entry| entry["verified"].as_bool().unwrap_or(false))
        })?;

    let email = pick["email"].as_str()?.trim();
    if email.is_empty() {
        return None;
    }

    Some((email.to_string(), pick["verified"].as_bool().unwrap_or(false)))
}

#[cfg(test)]
mod tests {
    use super::pick_primary_email;
    use serde_json::json;

    #[test]
    fn primary_address_is_preferred() {
        let entries = json!([
            { "email": "second@example.com", "primary": false, "verified": true },
            { "email": "main@example.com", "primary": true, "verified": true }
        ]);

        assert_eq!(
            pick_primary_email(&entries),
            Some(("main@example.com".to_string(), true))
        );
    }

    #[test]
    fn unverified_primary_stays_unverified() {
        let entries = json!([
            { "email": "claimed@example.com", "primary": true, "verified": false }
        ]);

        assert_eq!(
            pick_primary_email(&entries),
            Some(("claimed@example.com".to_string(), false))
        );
    }

    #[test]
    fn no_primary_falls_back_to_the_first_verified_address() {
        let entries = json!([
            { "email": "unverified@example.com", "primary": false, "verified": false },
            { "email": "verified@example.com", "primary": false, "verified": true }
        ]);

        assert_eq!(
            pick_primary_email(&entries),
            Some(("verified@example.com".to_string(), true))
        );
    }

    #[test]
    fn a_list_without_usable_addresses_yields_nothing() {
        assert_eq!(pick_primary_email(&json!([])), None);
        assert_eq!(
            pick_primary_email(&json!([
                { "email": "unverified@example.com", "primary": false, "verified": false }
            ])),
            None
        );
        assert_eq!(pick_primary_email(&json!({ "message": "Not Found" })), None);
    }
}
