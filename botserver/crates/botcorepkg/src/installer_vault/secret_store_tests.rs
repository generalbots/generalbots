//! Regression tests for #2498: no plaintext Vault root token or unseal keys
//! may ever be written into the generated stack artifacts.
//!
//! A single test owns `GB_SECRET_DIR` so parallel tests cannot race on it.

use super::secret_store as secrets;
use std::error::Error;

#[test]
fn generated_stack_has_no_plaintext_vault_secrets() -> Result<(), Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("gb-secret-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::env::set_var("GB_SECRET_DIR", &dir);
    let dir = secrets::directory()?;

    // A stack from an older build: root token in plaintext in `.env`.
    let env_file = dir.join(".env");
    secrets::write_private(
        &env_file,
        "OTHER=value\nVAULT_ADDR=http://127.0.0.1:8200\nVAULT_TOKEN=hvs.ROOTTOKEN\nVAULT_CACERT=/ca.crt\n",
    )?;

    secrets::write_env_file(&env_file, "https://127.0.0.1:8200", None)?;
    let generated = std::fs::read_to_string(&env_file)?;

    assert!(
        !generated.contains("VAULT_TOKEN="),
        "root token leaked: {generated}"
    );
    assert!(
        !generated.contains("hvs.ROOTTOKEN"),
        "token value leaked: {generated}"
    );
    assert!(
        !generated.contains("root_token"),
        "root_token leaked: {generated}"
    );
    assert!(
        generated.contains("VAULT_TOKEN_FILE="),
        "missing token path: {generated}"
    );
    assert!(
        generated.contains("OTHER=value"),
        "unrelated settings lost: {generated}"
    );

    // The scoped token itself is stored outside the stack, with no world access.
    let token_path = secrets::write(secrets::VAULT_TOKEN_SECRET, "hvs.SCOPED")?;
    assert!(
        token_path.starts_with(&dir),
        "secret written into the stack"
    );
    let stored = std::fs::read_to_string(&token_path)?;
    assert_eq!(stored, "hvs.SCOPED");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&token_path)?.permissions().mode() & 0o777;
        assert_eq!(mode & 0o007, 0, "secret is world-readable: {mode:o}");
        assert_eq!(mode & 0o020, 0, "secret is group-writable: {mode:o}");
    }

    // init.json must never carry the root token or the unseal keys.
    let init = serde_json::json!({
        "root_token": "hvs.ROOTTOKEN",
        "unseal_keys_b64": ["key1", "key2"],
        "recovery_keys_b64": ["rec1"],
        "initialized": true
    });
    let safe_text = serde_json::to_string(&secrets::sanitized_init_json(&init))?;
    assert!(!safe_text.contains("root_token"));
    assert!(!safe_text.contains("unseal_keys_b64"));
    assert!(!safe_text.contains("recovery_keys_b64"));
    assert!(!safe_text.contains("hvs.ROOTTOKEN"));
    assert!(safe_text.contains("initialized"));

    // Walk the generated tree the same way a build gate would: any file that
    // contains `VAULT_TOKEN=` or `root_token` is a failure.
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) => return Err(error.into()),
        };
        assert!(
            !content.contains("VAULT_TOKEN="),
            "{} contains VAULT_TOKEN=",
            path.display()
        );
        assert!(
            !content.contains("root_token"),
            "{} contains root_token",
            path.display()
        );
    }

    std::env::remove_var("GB_SECRET_DIR");
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
