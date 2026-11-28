use super::*;
use crate::installer_vault::token_recovery_cli::{encoded_token, parse_init};
use std::cell::RefCell;

#[derive(Default)]
struct FakeVault {
    lookup: Option<TokenState>,
    generate_fails: bool,
    calls: RefCell<Vec<String>>,
}

impl FakeVault {
    fn with(state: TokenState) -> Self {
        Self { lookup: Some(state), ..Self::default() }
    }
    fn log(&self, call: &str) {
        self.calls.borrow_mut().push(call.to_string());
    }
    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl VaultOps for FakeVault {
    fn lookup(&self, _token: &str) -> TokenState {
        self.log("lookup");
        self.lookup.unwrap_or(TokenState::Unreachable)
    }
    fn renew(&self, _token: &str) -> bool {
        self.log("renew");
        true
    }
    fn generate_root(&self, keys: &[String]) -> std::result::Result<String, String> {
        self.log(&format!("generate_root:{}", keys.len()));
        if self.generate_fails {
            Err("starting root token generation".to_string())
        } else {
            Ok("root-once".to_string())
        }
    }
    fn create_scoped(&self, root: &str) -> std::result::Result<String, String> {
        self.log(&format!("create_scoped:{root}"));
        Ok("scoped-new".to_string())
    }
    fn revoke_self(&self, root: &str) -> bool {
        self.log(&format!("revoke:{root}"));
        true
    }
    fn store(&self, token: &str) -> std::result::Result<(), String> {
        self.log(&format!("store:{token}"));
        Ok(())
    }
}

fn keys() -> Vec<String> {
    vec!["k1".to_string(), "k2".to_string(), "k3".to_string()]
}

#[test]
fn valid_token_is_renewed_and_kept() -> Result<()> {
    let vault = FakeVault::with(TokenState::Valid);
    let token = recover(&vault, Some("scoped-old".to_string()), &keys())?;
    assert_eq!(token, "scoped-old");
    assert_eq!(vault.calls(), ["lookup", "renew"]);
    Ok(())
}

#[test]
fn expired_token_is_reissued_through_generate_root_and_the_root_is_revoked() -> Result<()> {
    let vault = FakeVault::with(TokenState::Rejected);
    let token = recover(&vault, Some("scoped-old".to_string()), &keys())?;
    assert_eq!(token, "scoped-new");
    assert_eq!(
        vault.calls(),
        ["lookup", "generate_root:3", "create_scoped:root-once", "revoke:root-once", "store:scoped-new"]
    );
    Ok(())
}

#[test]
fn missing_token_is_reissued_too() -> Result<()> {
    let vault = FakeVault::default();
    assert_eq!(recover(&vault, None, &keys())?, "scoped-new");
    assert_eq!(vault.calls()[0], "generate_root:3");
    Ok(())
}

#[test]
fn failed_reissue_names_the_step_and_what_to_do_and_stores_nothing() {
    let vault = FakeVault { lookup: Some(TokenState::Rejected), generate_fails: true, ..FakeVault::default() };
    let message = match recover(&vault, Some("scoped-old".to_string()), &keys()) {
        Ok(_) => String::new(),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("starting root token generation"), "{message}");
    assert!(message.contains("Restart the app"), "{message}");
    assert!(message.contains("back up the whole stack folder"), "{message}");
    assert!(!message.contains("scoped-old") && !message.contains("k1"), "{message}");
    assert!(!vault.calls().iter().any(|call| call.starts_with("store")));
}

#[test]
fn unreachable_vault_keeps_the_token_and_no_keys_is_an_error() {
    let vault = FakeVault::with(TokenState::Unreachable);
    assert_eq!(recover(&vault, Some("scoped-old".to_string()), &keys()).ok().as_deref(), Some("scoped-old"));
    assert_eq!(vault.calls(), ["lookup"]);
    let vault = FakeVault::with(TokenState::Rejected);
    assert!(recover(&vault, Some("scoped-old".to_string()), &[]).is_err());
    assert_eq!(vault.calls(), ["lookup"]);
}

#[test]
fn lookup_failures_are_classified() {
    assert_eq!(classify_lookup(true, ""), TokenState::Valid);
    assert_eq!(classify_lookup(false, "Error looking up token: Code: 403. Errors:\n* permission denied"), TokenState::Rejected);
    assert_eq!(classify_lookup(false, "* bad token"), TokenState::Rejected);
    assert_eq!(classify_lookup(false, "Code: 503. Errors:\n* Vault is sealed"), TokenState::Unreachable);
    assert_eq!(classify_lookup(false, "dial tcp 127.0.0.1:8200: connection refused"), TokenState::Unreachable);
}

#[test]
fn decisions_cover_every_state() {
    assert_eq!(decide(TokenState::Valid, false), Action::Renew);
    assert_eq!(decide(TokenState::Rejected, true), Action::Reissue);
    assert_eq!(decide(TokenState::Missing, true), Action::Reissue);
    assert_eq!(decide(TokenState::Rejected, false), Action::NoKeys);
    assert_eq!(decide(TokenState::Unreachable, true), Action::KeepAsIs);
}

#[test]
fn generate_root_answers_are_parsed() {
    let init = serde_json::json!({"nonce": "n-1", "otp": "o-1", "complete": false});
    assert_eq!(parse_init(&init), Some(("n-1".to_string(), "o-1".to_string())));
    assert_eq!(parse_init(&serde_json::json!({"nonce": "n-1", "otp": ""})), None);
    assert_eq!(encoded_token(&serde_json::json!({"complete": false, "encoded_token": "e"})), None);
    assert_eq!(encoded_token(&serde_json::json!({"complete": true, "encoded_token": "e"})).as_deref(), Some("e"));
    assert_eq!(
        encoded_token(&serde_json::json!({"complete": true, "encoded_token": "", "encoded_root_token": "r"})).as_deref(),
        Some("r")
    );
}
