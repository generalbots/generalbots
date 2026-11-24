//! Pure policy for enabling internal service-to-service mTLS.
//!
//! Containerized installs provision an internal CA and a `botserver` client
//! identity under `<stack>/conf/system/certificates`. Only when the install is
//! containerized *and* that credential material exists do internal HTTP
//! clients switch to mTLS. Desktop-local / single-host localhost installs keep
//! their loopback-binding + file-permission threat model (mTLS stays off).

use std::path::Path;

/// Decide whether internal service clients should use mTLS.
///
/// Pure and side-effect free apart from checking `cert_dir` on disk: it never
/// reads Vault or the network, so it is cheap and deterministic to test.
/// Returns `false` for any non-containerized install even when certificates
/// happen to be present (e.g. a local dev stack reusing a provisioned tree).
///
/// The three required files are the internal CA, the botserver client cert and
/// its private key — the exact triad the Vault installer generates and
/// `botcore::package_manager::installer::fetch_vault_credentials` uses.
pub fn should_enable_mtls(containerized: bool, cert_dir: &Path) -> bool {
    if !containerized {
        return false;
    }

    cert_dir.join("ca/ca.crt").is_file()
        && cert_dir.join("botserver/client.crt").is_file()
        && cert_dir.join("botserver/client.key").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    fn empty_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gb-mtls-policy-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp cert dir");
        dir
    }

    fn provisioned_dir() -> PathBuf {
        let dir = empty_dir();
        std::fs::create_dir_all(dir.join("ca")).expect("create ca dir");
        std::fs::create_dir_all(dir.join("botserver")).expect("create botserver dir");
        std::fs::write(dir.join("ca/ca.crt"), b"ca").expect("write ca");
        std::fs::write(dir.join("botserver/client.crt"), b"cert").expect("write client cert");
        std::fs::write(dir.join("botserver/client.key"), b"key").expect("write client key");
        dir
    }

    #[test]
    fn non_containerized_never_enables_mtls() {
        let dir = provisioned_dir();
        assert!(!should_enable_mtls(false, &dir));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn containerized_without_certs_does_not_enable_mtls() {
        let dir = empty_dir();
        assert!(!should_enable_mtls(true, &dir));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn containerized_with_missing_client_key_does_not_enable_mtls() {
        let dir = provisioned_dir();
        let _ = std::fs::remove_file(dir.join("botserver/client.key"));
        assert!(!should_enable_mtls(true, &dir));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn containerized_with_missing_ca_does_not_enable_mtls() {
        let dir = provisioned_dir();
        let _ = std::fs::remove_file(dir.join("ca/ca.crt"));
        assert!(!should_enable_mtls(true, &dir));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn containerized_with_full_cert_triad_enables_mtls() {
        let dir = provisioned_dir();
        assert!(should_enable_mtls(true, &dir));
        let _ = std::fs::remove_dir_all(dir);
    }
}
