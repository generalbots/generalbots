/// Sudoers snippet that grants the running user passwordless access
/// to only the specific commands needed by the security fix workflow.
///
/// Write this to `/etc/sudoers.d/gb-security` once (requires initial sudo).
/// After that, `botserver security fix` runs fully unattended.
///
/// Jail configuration is *never* staged at a predictable path: botserver writes
/// it to a fresh, private `0700` directory with a `0600` file (O_EXCL) and hands
/// that exact path to a root-owned helper installed by
/// [`print_bootstrap_instructions`]. The helper re-verifies the source before
/// the privileged copy. There is no `cp` grant and no wildcard (issue #2608).
pub const SUDOERS_CONTENT: &str = r#"# General Bots — security fix sudoers
# Managed by botserver. DO NOT EDIT MANUALLY.
{user} ALL=(ALL) NOPASSWD: /usr/sbin/ufw
{user} ALL=(ALL) NOPASSWD: /usr/bin/apt-get install -y ufw
{user} ALL=(ALL) NOPASSWD: /usr/bin/apt-get install -y fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/bin/fail2ban-client status
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable --now fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/local/sbin/gb-write-fail2ban-jail
"#;

/// The exact sudoers rule lines granting `fail2ban-client` access. Every entry
/// is a complete, fixed command — no wildcards, no trailing arguments. The
/// inline test below keeps this list in sync with [`SUDOERS_CONTENT`].
pub const FAIL2BAN_CLIENT_GRANTS: &[&str] =
    &["{user} ALL=(ALL) NOPASSWD: /usr/bin/fail2ban-client status"];

/// Exact argv vectors permitted for `sudo fail2ban-client`, mirroring
/// [`FAIL2BAN_CLIENT_GRANTS`]. `status` is the only invocation the protection
/// module actually makes (see `fail2ban::status`), so nothing broader is
/// granted. Add an entry here only together with its exact sudoers rule.
pub const FAIL2BAN_CLIENT_ALLOWED_ARGS: &[&[&str]] = &[&["status"]];

/// Pure allowlist decision for the fail2ban sudoers rule. Arbitrary arguments,
/// extra flags and wildcard tokens are always rejected; only the exact argv
/// vectors in [`FAIL2BAN_CLIENT_ALLOWED_ARGS`] are accepted.
pub fn is_allowed_fail2ban_client_args(args: &[&str]) -> bool {
    FAIL2BAN_CLIENT_ALLOWED_ARGS
        .iter()
        .any(|allowed| *allowed == args)
}

/// Absolute path of the root-owned helper that writes `jail.local`.
pub const JAIL_WRITER_PATH: &str = "/usr/local/sbin/gb-write-fail2ban-jail";

/// Root-owned helper installed once by the operator. It receives the exact
/// path of a file staged by `SecureStaging`, re-verifies that it is a `0600`
/// regular file inside a private `0700` staging directory owned by the
/// invoking user, then installs it atomically from inside `/etc/fail2ban`.
/// No predictable path is ever a source of privileged config (issue #2608).
pub const JAIL_WRITER_SCRIPT: &str = r#"#!/bin/sh
# General Bots — fail2ban jail writer. Managed by botserver. DO NOT EDIT.
set -eu
umask 077
src=$1
case "$src" in
  /*) ;;
  *) echo "jail source must be an absolute path" >&2; exit 1 ;;
esac
stage_dir=${src%/*}
stage_name=${stage_dir##*/}
payload_name=${src##*/}
case "$stage_name" in
  gb-secure-stage-*) ;;
  *) echo "jail source is not inside a private staging directory" >&2; exit 1 ;;
esac
case "$payload_name" in
  payload-*) ;;
  *) echo "jail source is not a staged payload" >&2; exit 1 ;;
esac
[ -d "$stage_dir" ] || { echo "staging directory missing" >&2; exit 1; }
[ "$(stat -c %a "$stage_dir")" = 700 ] || { echo "staging directory must be 0700" >&2; exit 1; }
[ -f "$src" ] && [ ! -L "$src" ] || { echo "staged payload must be a regular file" >&2; exit 1; }
[ "$(stat -c %a "$src")" = 600 ] || { echo "staged payload must be 0600" >&2; exit 1; }
if [ -n "${SUDO_USER:-}" ]; then
  [ "$(stat -c %u "$stage_dir")" = "$(id -u "$SUDO_USER")" ] || {
    echo "staging directory not owned by invoking user" >&2; exit 1;
  }
fi
tmp=$(mktemp /etc/fail2ban/.jail.local.XXXXXX)
trap 'rm -f "$tmp"' EXIT HUP INT TERM
cat "$src" > "$tmp"
chown root:root "$tmp"
chmod 0644 "$tmp"
mv -f "$tmp" /etc/fail2ban/jail.local
trap - EXIT HUP INT TERM
# The staging directory is removed by botserver's RAII Drop guard on every
# exit path; the helper never writes there, so leave it untouched.
"#;

/// Strict POSIX-account validation before a name is substituted into a
/// root-owned sudoers file. Rejects newlines, separators and leading digits.
pub fn validate_sudo_user(user: &str) -> Result<(), String> {
    if user.is_empty() || user.len() > 32 {
        return Err("user name must be 1-32 characters".to_string());
    }
    let mut chars = user.chars();
    let first = chars.next().ok_or_else(|| "empty user name".to_string())?;
    if !first.is_ascii_lowercase() && first != '_' {
        return Err("user name must start with a lowercase letter or underscore".to_string());
    }
    for character in chars {
        if !character.is_ascii_lowercase()
            && !character.is_ascii_digit()
            && !matches!(character, '_' | '-' | '$')
        {
            return Err("user name contains an invalid character".to_string());
        }
    }
    Ok(())
}

/// Print the one-time setup commands the operator must run as root to enable
/// unattended security fix. An invalid or missing `USER` fails closed: no
/// sudoers snippet is emitted at all.
pub fn print_bootstrap_instructions() {
    let user = match std::env::var("USER") {
        Ok(user) => user,
        Err(_) => {
            eprintln!("USER is not set; refusing to generate a sudoers file");
            return;
        }
    };
    if let Err(error) = validate_sudo_user(&user) {
        eprintln!("Invalid USER for sudoers generation: {error}");
        return;
    }
    let content = SUDOERS_CONTENT.replace("{user}", &user);
    println!("=== One-time setup required ===");
    println!("Run this on the host as root:");
    println!();
    println!("install -d -m 0755 /usr/local/sbin");
    println!("cat > {JAIL_WRITER_PATH} << 'GB_HELPER_EOF'");
    print!("{JAIL_WRITER_SCRIPT}");
    println!("GB_HELPER_EOF");
    println!("chown root:root {JAIL_WRITER_PATH}");
    println!("chmod 0755 {JAIL_WRITER_PATH}");
    println!();
    println!("cat > /etc/sudoers.d/gb-security << 'EOF'");
    print!("{content}");
    println!("EOF");
    println!("chmod 440 /etc/sudoers.d/gb-security");
    println!();
    println!("Then run: botserver security fix");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protection::installer;

    /// Every `{...}` placeholder in a sudoers template must be the validated
    /// `user` placeholder; any other placeholder would be substituted without
    /// the [`validate_sudo_user`] guard (#2609).
    fn assert_placeholders_are_user(content: &str) {
        let mut rest = content;
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            let end = after.find('}').unwrap_or(0);
            assert_eq!(
                &after[..end],
                "user",
                "unknown or unterminated sudoers placeholder"
            );
            rest = &after[end + 1..];
        }
    }

    #[test]
    fn sudoers_never_copies_from_world_writable_path() {
        assert!(
            !SUDOERS_CONTENT.contains("/tmp/"),
            "no /tmp source in sudoers"
        );
        assert!(!SUDOERS_CONTENT.contains("/usr/bin/cp"), "no cp grant");
        assert!(SUDOERS_CONTENT.contains(JAIL_WRITER_PATH));
    }

    #[test]
    fn no_sudoers_constant_contains_a_wildcard() {
        assert!(
            !SUDOERS_CONTENT.contains('*'),
            "security fix sudoers has a wildcard"
        );
        #[cfg(not(windows))]
        assert!(
            !installer::SUDOERS_CONTENT.contains('*'),
            "protection installer sudoers has a wildcard"
        );
    }

    #[test]
    fn fail2ban_client_grant_is_an_exact_allowlist() {
        // Positive assertion: the narrow grant must actually be present. A
        // test that only checks for the absence of `*` would pass even if the
        // grant were deleted or widened to a different fixed command (#2609).
        let granted: Vec<&str> = SUDOERS_CONTENT
            .lines()
            .filter(|line| line.contains("fail2ban-client"))
            .collect();
        assert_eq!(
            granted,
            FAIL2BAN_CLIENT_GRANTS.to_vec(),
            "sudoers must grant only the enumerated fail2ban-client commands"
        );

        // Pure decision: only the exact allowlisted argv is accepted.
        assert!(is_allowed_fail2ban_client_args(&["status"]));
        for rejected in [
            &[][..],
            &["*"][..],
            &["status", "--help"][..],
            &["unban", "1.2.3.4"][..],
            &["status;rm -rf /"][..],
            &["", "status"][..],
        ] {
            assert!(
                !is_allowed_fail2ban_client_args(rejected),
                "arbitrary fail2ban-client args must be rejected"
            );
        }
    }

    #[test]
    fn sudoers_placeholders_are_all_validated_user() {
        assert_placeholders_are_user(SUDOERS_CONTENT);
        #[cfg(not(windows))]
        assert_placeholders_are_user(installer::SUDOERS_CONTENT);
    }

    #[test]
    fn username_validation_rejects_injection() {
        assert!(validate_sudo_user("rodriguez").is_ok());
        assert!(validate_sudo_user("_svc-user$").is_ok());
        assert!(validate_sudo_user("root\nALL=(ALL) NOPASSWD: ALL").is_err());
        assert!(validate_sudo_user("1user").is_err());
        assert!(validate_sudo_user("user:group").is_err());
        assert!(validate_sudo_user("").is_err());
    }

    #[test]
    fn jail_writer_is_root_owned_and_atomic() {
        assert!(JAIL_WRITER_SCRIPT.contains("mktemp /etc/fail2ban/"));
        assert!(JAIL_WRITER_SCRIPT.contains("chown root:root"));
        assert!(JAIL_WRITER_SCRIPT.contains("mv -f \"$tmp\" /etc/fail2ban/jail.local"));
        assert!(!JAIL_WRITER_SCRIPT.contains("/tmp/"));
    }

    #[test]
    fn jail_writer_verifies_staged_source_before_privileged_copy() {
        // The helper must re-check the staging dir/file modes and reject any
        // source that is not a random payload inside a private staging dir.
        assert!(JAIL_WRITER_SCRIPT.contains("gb-secure-stage-"));
        assert!(JAIL_WRITER_SCRIPT.contains("payload-*"));
        assert!(JAIL_WRITER_SCRIPT.contains("stat -c %a \"$stage_dir\""));
        assert!(JAIL_WRITER_SCRIPT.contains("stat -c %a \"$src\""));
        assert!(JAIL_WRITER_SCRIPT.contains("[ ! -L \"$src\" ]"));
        assert!(JAIL_WRITER_SCRIPT.contains("stat -c %u \"$stage_dir\""));
        // The script no longer embeds a fixed /tmp jail config heredoc.
        assert!(!JAIL_WRITER_SCRIPT.contains("GB_JAIL_EOF"));
    }
}
