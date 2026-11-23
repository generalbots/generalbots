/// Sudoers snippet that grants the running user passwordless access
/// to only the specific commands needed by the security fix workflow.
///
/// Write this to `/etc/sudoers.d/gb-security` once (requires initial sudo).
/// After that, `botserver security fix` runs fully unattended.
///
/// Jail configuration is *never* staged in a world-writable directory: the only
/// write grant is to a root-owned helper installed by [`print_bootstrap_instructions`].
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

/// Absolute path of the root-owned helper that writes `jail.local`.
pub const JAIL_WRITER_PATH: &str = "/usr/local/sbin/gb-write-fail2ban-jail";

/// Root-owned helper installed once by the operator. It builds the config in a
/// private temporary file inside `/etc/fail2ban` and atomically renames it into
/// place. No world-writable directory is ever a source of privileged config.
pub const JAIL_WRITER_SCRIPT: &str = r#"#!/bin/sh
# General Bots — fail2ban jail writer. Managed by botserver. DO NOT EDIT.
set -eu
umask 077
tmp=$(mktemp /etc/fail2ban/.jail.local.XXXXXX)
trap 'rm -f "$tmp"' EXIT HUP INT TERM
cat > "$tmp" <<'GB_JAIL_EOF'
[DEFAULT]
bantime  = 1h
findtime = 10m
maxretry = 5
backend  = systemd

[sshd]
enabled  = true
port     = ssh
logpath  = %(sshd_log)s
maxretry = 3

[postfix]
enabled  = true
port     = smtp,465,587
logpath  = %(postfix_log)s

[dovecot]
enabled  = true
port     = pop3,pop3s,imap,imaps,submission,465,sieve
logpath  = %(dovecot_log)s
GB_JAIL_EOF
chown root:root "$tmp"
chmod 0644 "$tmp"
mv -f "$tmp" /etc/fail2ban/jail.local
trap - EXIT HUP INT TERM
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

    #[test]
    fn sudoers_never_copies_from_world_writable_path() {
        assert!(!SUDOERS_CONTENT.contains("/tmp/"), "no /tmp source in sudoers");
        assert!(!SUDOERS_CONTENT.contains("/usr/bin/cp"), "no cp grant");
        assert!(SUDOERS_CONTENT.contains(JAIL_WRITER_PATH));
    }

    #[test]
    fn no_sudoers_constant_contains_a_wildcard() {
        assert!(!SUDOERS_CONTENT.contains('*'), "security fix sudoers has a wildcard");
        #[cfg(not(windows))]
        assert!(
            !super::installer::SUDOERS_CONTENT.contains('*'),
            "protection installer sudoers has a wildcard"
        );
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
}
