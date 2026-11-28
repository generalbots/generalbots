//! The Vault CLI behind [`super::token_recovery`]. Tokens, the OTP and the
//! unseal keys travel only as process environment or arguments of the local
//! CLI; command output that may carry them is parsed, never logged.

use super::token_recovery::{classify_lookup, TokenState, VaultOps};
use super::vault_policy::{vault_command, with_tls};
use std::path::Path;
use std::process::Output;

pub(super) struct VaultCli<'a> {
    pub addr: &'a str,
    pub ca_cert: &'a Path,
    pub bin: &'a Path,
}

impl VaultCli<'_> {
    /// Run the CLI. `secret_args` are appended unvalidated by the shell-pattern
    /// check (base64 keys contain `/` and `+`); they come from our own store.
    fn run(&self, token: Option<&str>, args: &[&str], secret_args: &[&str]) -> Option<Output> {
        let result = vault_command(self.bin)
            .and_then(|command| Ok(command.env("VAULT_ADDR", self.addr)?))
            .and_then(|command| match token {
                Some(token) => Ok(command.env("VAULT_TOKEN", token)?),
                None => Ok(command),
            })
            .and_then(|command| with_tls(command, self.ca_cert))
            .and_then(|command| Ok(command.args(args)?))
            .and_then(|command| {
                secret_args
                    .iter()
                    .try_fold(command, |command, arg| Ok::<_, anyhow::Error>(command.trusted_arg(arg)?))
            })
            .and_then(|command| Ok(command.execute()?));
        match result {
            Ok(output) => Some(output),
            Err(e) => {
                log::warn!("Vault CLI could not run ({}): {e}", args.join(" "));
                None
            }
        }
    }

    fn json(&self, args: &[&str], secret_args: &[&str]) -> Option<serde_json::Value> {
        let output = self.run(None, args, secret_args)?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok()
    }

    fn cancel_generation(&self) {
        let cancelled = self
            .run(None, &["operator", "generate-root", "-cancel"], &[])
            .is_some_and(|output| output.status.success());
        if !cancelled {
            log::debug!("No root token generation to cancel");
        }
    }

    /// Start a generation; returns the nonce and the OTP.
    fn start_generation(&self) -> Result<(String, String), String> {
        let init = self
            .json(&["operator", "generate-root", "-init", "-format=json"], &[])
            .ok_or_else(|| "starting root token generation".to_string())?;
        if let Some(started) = parse_init(&init) {
            return Ok(started);
        }
        // Older servers do not hand out the OTP themselves: make one.
        self.cancel_generation();
        let otp = self
            .run(None, &["operator", "generate-root", "-generate-otp"], &[])
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|otp| !otp.is_empty())
            .ok_or_else(|| "creating a one-time password".to_string())?;
        let otp_arg = format!("-otp={otp}");
        let init = self
            .json(&["operator", "generate-root", "-init", "-format=json"], &[&otp_arg])
            .ok_or_else(|| "starting root token generation".to_string())?;
        let nonce = nonce_of(&init).ok_or_else(|| "reading the generation nonce".to_string())?;
        Ok((nonce, otp))
    }

    fn provide_keys(&self, nonce: &str, keys: &[String]) -> Result<String, String> {
        let nonce_arg = format!("-nonce={nonce}");
        for key in keys {
            let progress = self
                .json(&["operator", "generate-root", "-format=json", &nonce_arg], &[key])
                .ok_or_else(|| "providing an unseal key share".to_string())?;
            if let Some(encoded) = encoded_token(&progress) {
                return Ok(encoded);
            }
        }
        Err("not enough unseal key shares are stored".to_string())
    }

    fn decode(&self, encoded: &str, otp: &str) -> Result<String, String> {
        let decode_arg = format!("-decode={encoded}");
        let otp_arg = format!("-otp={otp}");
        self.run(None, &["operator", "generate-root"], &[&decode_arg, &otp_arg])
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .filter(|token| !token.is_empty())
            .ok_or_else(|| "decoding the one-time root token".to_string())
    }
}

impl VaultOps for VaultCli<'_> {
    fn lookup(&self, token: &str) -> TokenState {
        match self.run(Some(token), &["token", "lookup", "-format=json"], &[]) {
            Some(output) => {
                classify_lookup(output.status.success(), &String::from_utf8_lossy(&output.stderr))
            }
            None => TokenState::Unreachable,
        }
    }

    fn renew(&self, token: &str) -> bool {
        self.run(Some(token), &["token", "renew", "-format=json"], &[])
            .is_some_and(|output| output.status.success())
    }

    fn generate_root(&self, keys: &[String]) -> Result<String, String> {
        // A half-finished attempt from an earlier start would block this one.
        self.cancel_generation();
        let (nonce, otp) = self.start_generation()?;
        let root = self
            .provide_keys(&nonce, keys)
            .and_then(|encoded| self.decode(&encoded, &otp));
        if root.is_err() {
            self.cancel_generation();
        }
        root
    }

    fn create_scoped(&self, root: &str) -> Result<String, String> {
        super::vault_policy::create_scoped_token(self.addr, root, self.ca_cert, self.bin)
            .map_err(|e| format!("creating the scoped token: {e}"))
    }

    fn revoke_self(&self, root: &str) -> bool {
        self.run(Some(root), &["token", "revoke", "-self"], &[])
            .is_some_and(|output| output.status.success())
    }

    fn store(&self, token: &str) -> Result<(), String> {
        use super::secret_store as secrets;
        secrets::write(secrets::VAULT_TOKEN_SECRET, token)
            .map(|_| ())
            .map_err(|e| format!("storing the new token: {e}"))
    }
}

fn text_field(json: &serde_json::Value, key: &str) -> Option<String> {
    json.get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn nonce_of(json: &serde_json::Value) -> Option<String> {
    text_field(json, "nonce")
}

/// Nonce and OTP from `generate-root -init -format=json`.
pub(super) fn parse_init(json: &serde_json::Value) -> Option<(String, String)> {
    Some((nonce_of(json)?, text_field(json, "otp")?))
}

/// The encoded root token once the generation is complete.
pub(super) fn encoded_token(json: &serde_json::Value) -> Option<String> {
    if json.get("complete").and_then(serde_json::Value::as_bool) != Some(true) {
        return None;
    }
    text_field(json, "encoded_token").or_else(|| text_field(json, "encoded_root_token"))
}
