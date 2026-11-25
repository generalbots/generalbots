//! The Vault token the server runs with on the Windows desktop stack.
//!
//! Instead of the root token, the server uses an orphan periodic token whose
//! policy only reaches `secret/gbo/*`. That token is kept sealed in
//! `conf/vault/server-token`; the root token stays only inside the sealed
//! `conf/vault/init.json`, for recovery. Token values are never logged.

use botcorepkg::protected_file::{read_secret, write_secret};
use log::{info, warn};
use std::path::{Path, PathBuf};

pub const TOKEN_FILE: &str = "conf/vault/server-token";
const INIT_FILE: &str = "conf/vault/init.json";
const POLICY_FILE: &str = "conf/vault/server-policy.hcl";
pub const POLICY_NAME: &str = "gbo-server";
/// Renewed at every start; a token idle longer than this is replaced.
pub const PERIOD: &str = "768h";

/// What the server reads and writes in Vault: the KV v2 `secret` mount under
/// `gbo/`, plus the mount lookup the Vault CLI does before a `kv` call.
pub const POLICY: &str = r#"path "secret/data/gbo/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
path "secret/metadata/gbo/*" {
  capabilities = ["read", "list", "delete"]
}
path "secret/metadata/gbo" {
  capabilities = ["list"]
}
path "sys/internal/ui/mounts/secret/*" {
  capabilities = ["read"]
}
"#;

/// `text` without its `VAULT_TOKEN=` lines, or `None` when it has none.
pub fn without_token_lines(text: &str) -> Option<String> {
    let is_token = |line: &str| line.trim_start().starts_with("VAULT_TOKEN=");
    if !text.lines().any(is_token) {
        return None;
    }
    let mut kept: Vec<&str> = text.lines().filter(|line| !is_token(line)).collect();
    while kept.last().is_some_and(|line| line.trim().is_empty()) {
        kept.pop();
    }
    let mut out = kept.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Some(out)
}

/// The token in a `vault token create -format=json` answer.
pub fn client_token(json: &serde_json::Value) -> Option<String> {
    json.pointer("/auth/client_token")
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

/// The root token in the text of `init.json`.
pub fn root_token(init_text: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(init_text).ok()?;
    json.get("root_token")
        .and_then(serde_json::Value::as_str)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

/// True on the desktop stack once a scoped token exists: `.env` writers leave
/// the root token out then.
pub fn scoped_token_exists(stack: &Path) -> bool {
    cfg!(windows) && stack.join(TOKEN_FILE).is_file()
}

/// `env_content` as a `.env` writer should store it (see [`scoped_token_exists`]).
pub fn env_content_for(stack: &Path, env_content: String) -> String {
    if scoped_token_exists(stack) {
        without_token_lines(&env_content).unwrap_or(env_content)
    } else {
        env_content
    }
}

fn root_from_init(stack: &Path) -> Option<String> {
    let path = stack.join(INIT_FILE);
    path.is_file().then(|| read_secret(&path).ok()).flatten().as_deref().and_then(root_token)
}

/// First thing at boot: seal plain secret files and put the server's token in
/// the process environment before any reader of `VAULT_TOKEN` runs.
pub fn load_early(stack: &Path) {
    botcorepkg::desktop_hardening::harden_stack_configs(stack);
    let path = stack.join(TOKEN_FILE);
    if !path.is_file() {
        return;
    }
    match read_secret(&path).map(|t| t.trim().to_string()) {
        Ok(token) if !token.is_empty() => std::env::set_var("VAULT_TOKEN", token),
        _ => match root_from_init(stack) {
            Some(root) => {
                warn!("the server's Vault token could not be read; using the recovery token for this start");
                std::env::set_var("VAULT_TOKEN", root);
            }
            None => warn!("the server's Vault token could not be read and no recovery token is available"),
        },
    }
}

struct Vault {
    bin: PathBuf,
    addr: String,
}

impl Vault {
    fn run(&self, token: &str, args: &[&str]) -> Option<std::process::Output> {
        botlib::security::command_guard::SafeCommand::new(&self.bin.to_string_lossy())
            .and_then(|c| c.env("VAULT_ADDR", &self.addr))
            .and_then(|c| c.env("VAULT_TOKEN", token))
            .and_then(|c| c.args(args))
            .and_then(|c| c.execute())
            .ok()
    }

    fn ok(&self, token: &str, args: &[&str]) -> bool {
        self.run(token, args).is_some_and(|o| o.status.success())
    }

    /// A token works when it can read the database settings.
    fn works(&self, token: &str) -> bool {
        self.ok(token, &["kv", "get", "-format=json", "secret/gbo/tables"])
    }

    fn create_scoped(&self, root: &str, stack: &Path) -> Option<String> {
        let policy = stack.join(POLICY_FILE);
        std::fs::write(&policy, POLICY).ok()?;
        if !self.ok(root, &["policy", "write", POLICY_NAME, &policy.to_string_lossy()]) {
            return None;
        }
        let policy_arg = format!("-policy={POLICY_NAME}");
        let period_arg = format!("-period={PERIOD}");
        let output = self.run(
            root,
            &["token", "create", "-orphan", &policy_arg, &period_arg, "-display-name=botserver", "-format=json"],
        )?;
        if !output.status.success() {
            return None;
        }
        serde_json::from_slice(&output.stdout).ok().as_ref().and_then(client_token)
    }
}

/// Removes `VAULT_TOKEN` from the `.env` files the server reads.
fn strip_env_files(stack: &Path) {
    for env in [PathBuf::from(".env"), stack.join(".env")] {
        let Ok(text) = std::fs::read_to_string(&env) else { continue };
        if let Some(stripped) = without_token_lines(&text) {
            match std::fs::write(&env, stripped) {
                Ok(()) => info!("{} no longer holds a Vault token", env.display()),
                Err(e) => warn!("could not rewrite {}: {e}", env.display()),
            }
        }
    }
}

/// After Vault is unsealed: make sure the server runs with a working scoped
/// token, creating it with the recovery token when needed. Falls back to the
/// recovery token (with a warning) when no scoped token can be made.
pub fn ensure_scoped_token(stack: &Path) {
    if !cfg!(windows) || !stack.join(INIT_FILE).is_file() {
        return;
    }
    let exe = if cfg!(windows) { "vault.exe" } else { "vault" };
    let vault = Vault {
        bin: stack.join("bin/vault").join(exe),
        addr: std::env::var("VAULT_ADDR")
            .ok()
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| botcoresecrets::local_vault_addr().to_string()),
    };
    if !vault.bin.is_file() {
        return;
    }
    let token_path = stack.join(TOKEN_FILE);
    if let Ok(token) = read_secret(&token_path).map(|t| t.trim().to_string()) {
        if !token.is_empty() {
            vault.ok(&token, &["token", "renew", "-format=json"]);
            if vault.works(&token) {
                std::env::set_var("VAULT_TOKEN", &token);
                strip_env_files(stack);
                return;
            }
            warn!("the server's Vault token no longer works; replacing it");
        }
    }
    let Some(root) = root_from_init(stack) else {
        warn!("no recovery token in the Vault init data; the server keeps its current Vault token");
        return;
    };
    if !vault.works(&root) {
        warn!("Vault is not answering with the recovery token; the server keeps its current Vault token");
        return;
    }
    match vault.create_scoped(&root, stack).filter(|token| vault.works(token)) {
        Some(token) => match write_secret(&token_path, &token) {
            Ok(()) => {
                std::env::set_var("VAULT_TOKEN", &token);
                strip_env_files(stack);
                info!("the server now uses its own limited Vault token");
            }
            Err(e) => {
                warn!("could not store the server's Vault token ({e}); using the recovery token");
                std::env::set_var("VAULT_TOKEN", &root);
            }
        },
        None => {
            warn!("could not create a limited Vault token; using the recovery token");
            std::env::set_var("VAULT_TOKEN", &root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_lines_are_removed_and_the_rest_is_kept() {
        let env = "# Vault Configuration (auto-generated)\nVAULT_ADDR=http://127.0.0.1:8200\nVAULT_TOKEN=x\nVAULT_CACERT=c\n";
        let stripped = without_token_lines(env).unwrap_or_default();
        assert!(!stripped.contains("VAULT_TOKEN"));
        assert!(stripped.contains("VAULT_ADDR=http://127.0.0.1:8200\nVAULT_CACERT=c\n"));
        assert_eq!(without_token_lines(&stripped), None);
    }

    #[test]
    fn empty_env_stays_empty() {
        assert_eq!(without_token_lines("VAULT_TOKEN=x\n").as_deref(), Some(""));
        assert_eq!(without_token_lines(""), None);
    }

    #[test]
    fn create_answer_yields_the_client_token() {
        let json = serde_json::json!({"auth": {"client_token": "hvs.abc", "policies": ["gbo-server"]}});
        assert_eq!(client_token(&json).as_deref(), Some("hvs.abc"));
        assert_eq!(client_token(&serde_json::json!({"auth": {}})), None);
    }

    #[test]
    fn root_token_is_read_from_init_data_and_markers_have_none() {
        assert_eq!(root_token("{\"root_token\":\"r\",\"unseal_keys_b64\":[]}").as_deref(), Some("r"));
        assert_eq!(root_token("{}"), None);
        assert_eq!(root_token("not json"), None);
    }

    #[test]
    fn policy_stays_inside_the_server_secrets() {
        assert!(POLICY.contains("path \"secret/data/gbo/*\""));
        assert!(!POLICY.contains("sys/policy"));
        assert!(!POLICY.contains("auth/token/create"));
        assert!(!POLICY.contains("\"*\""));
    }

    #[test]
    fn env_writers_keep_the_token_until_a_scoped_one_exists() {
        let dir = std::env::temp_dir().join(format!("gb-server-token-{}", std::process::id()));
        let env = "VAULT_ADDR=a\nVAULT_TOKEN=r\n".to_string();
        assert_eq!(env_content_for(&dir, env.clone()), env);
        std::fs::create_dir_all(dir.join("conf/vault")).unwrap();
        std::fs::write(dir.join(TOKEN_FILE), "sealed").unwrap();
        let expected = if cfg!(windows) { "VAULT_ADDR=a\n".to_string() } else { env.clone() };
        assert_eq!(env_content_for(&dir, env), expected);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
