/// Sudoers snippet that grants the running user passwordless access
/// to only the specific commands needed by the security fix workflow.
///
/// Write this to `/etc/sudoers.d/gb-security` once (requires initial sudo).
/// After that, `botserver security fix` runs fully unattended.
pub const SUDOERS_CONTENT: &str = r#"# General Bots — security fix sudoers
# Managed by botserver. DO NOT EDIT MANUALLY.
{user} ALL=(ALL) NOPASSWD: /usr/sbin/ufw
{user} ALL=(ALL) NOPASSWD: /usr/bin/apt-get install -y ufw
{user} ALL=(ALL) NOPASSWD: /usr/bin/apt-get install -y fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/bin/fail2ban-client *
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl enable --now fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart fail2ban
{user} ALL=(ALL) NOPASSWD: /usr/bin/cp /tmp/gb-jail.local /etc/fail2ban/jail.local
"#;

/// Print the sudoers line the operator must add once to enable unattended security fix.
pub fn print_bootstrap_instructions() {
    let user = std::env::var("USER").unwrap_or_else(|_| "rodriguez".to_string());
    let content = SUDOERS_CONTENT.replace("{user}", &user);
    println!("=== One-time setup required ===");
    println!("Run this on the host as root:");
    println!();
    println!("cat > /etc/sudoers.d/gb-security << 'EOF'");
    print!("{content}");
    println!("EOF");
    println!("chmod 440 /etc/sudoers.d/gb-security");
    println!();
    println!("Then run: botserver security fix");
}
