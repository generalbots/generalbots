use anyhow::Result;
use log::{error, info, warn};
use rand::Rng;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::client::ZitadelClient;

const ADMIN_USERNAME: &str = "admin";
const DEFAULT_ORG_NAME: &str = "General Bots";

pub struct BootstrapResult {
    pub user_id: String,
    pub organization_id: Option<String>,
    pub username: String,
    pub email: String,
    pub initial_password: String,
    pub setup_url: String,
}

pub async fn check_and_bootstrap_admin(client: &ZitadelClient) -> Result<Option<BootstrapResult>> {
    info!("Checking if bootstrap is needed...");

    match client.list_users(10, 0).await {
        Ok(users) => {
            if !users.is_empty() {
                let has_admin = users.iter().any(|u| {
                    let username = u
                        .get("userName")
                        .or_else(|| u.get("username"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let has_admin_role = u
                        .get("roles")
                        .and_then(|r| r.as_array())
                        .map(|roles| {
                            roles.iter().any(|r| {
                                r.as_str()
                                    .map(|s| s.to_lowercase().contains("admin"))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);

                    username == ADMIN_USERNAME || has_admin_role
                });

                if has_admin {
                    info!("Admin user already exists, skipping bootstrap");
                    return Ok(None);
                }
            }
        }
        Err(e) => {
            warn!(
                "Could not check existing users (may be first run): {}",
                e
            );
        }
    }

    info!("No admin user found, bootstrapping initial admin account...");

    let result = create_bootstrap_admin(client).await?;

    print_bootstrap_credentials(&result);

    Ok(Some(result))
}

fn generate_secure_password() -> String {
    let mut rng = rand::rng();

    let lowercase: Vec<char> = (b'a'..=b'z').map(|c| c as char).collect();
    let uppercase: Vec<char> = (b'A'..=b'Z').map(|c| c as char).collect();
    let digits: Vec<char> = (b'0'..=b'9').map(|c| c as char).collect();
    let special: Vec<char> = "!@#$%&*".chars().collect();

    let mut password = Vec::with_capacity(16);

    password.push(lowercase[rng.random_range(0..lowercase.len())]);
    password.push(uppercase[rng.random_range(0..uppercase.len())]);
    password.push(digits[rng.random_range(0..digits.len())]);
    password.push(special[rng.random_range(0..special.len())]);

    let all_chars: Vec<char> = lowercase
        .iter()
        .chain(uppercase.iter())
        .chain(digits.iter())
        .chain(special.iter())
        .copied()
        .collect();

    for _ in 0..12 {
        password.push(all_chars[rng.random_range(0..all_chars.len())]);
    }

    for i in (1..password.len()).rev() {
        let j = rng.random_range(0..=i);
        password.swap(i, j);
    }

    password.into_iter().collect()
}

async fn create_bootstrap_admin(client: &ZitadelClient) -> Result<BootstrapResult> {
    let email = format!("{}@localhost", ADMIN_USERNAME);

    let user_id = client
        .create_user(&email, "System", "Administrator", Some(ADMIN_USERNAME))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create admin user: {}", e))?;

    info!("Created admin user with ID: {}", user_id);

    let initial_password = generate_secure_password();

    if let Err(e) = client.set_user_password(&user_id, &initial_password, true).await {
        warn!("Failed to set initial password via API: {}. User may need to use password reset flow.", e);
    } else {
        info!("Initial password set for admin user");
    }

    let org_id = match create_default_organization(client).await {
        Ok(id) => {
            info!("Created default organization with ID: {}", id);

            let admin_roles = vec![
                "admin".to_string(),
                "org_owner".to_string(),
                "user_manager".to_string(),
            ];
            if let Err(e) = client.add_org_member(&id, &user_id, admin_roles).await {
                warn!("Failed to add admin to organization: {}", e);
            }

            Some(id)
        }
        Err(e) => {
            warn!("Failed to create default organization: {}", e);
            None
        }
    };

    let base_url = client.api_url();
    let setup_url = format!("{}/ui/login", base_url);

    let result = BootstrapResult {
        user_id: user_id.clone(),
        organization_id: org_id,
        username: ADMIN_USERNAME.to_string(),
        email: email.clone(),
        initial_password: initial_password.clone(),
        setup_url: setup_url.clone(),
    };

    save_setup_credentials(&result);

    Ok(result)
}

async fn create_default_organization(client: &ZitadelClient) -> Result<String> {
    let url = format!("{}/v2/organizations", client.api_url());

    let body = serde_json::json!({
        "name": DEFAULT_ORG_NAME
    });

    let response = client
        .http_post(url)
        .await
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create organization: {}", e))?;

    if response.status().is_success() {
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

        let org_id = data
            .get("organizationId")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No organization ID in response"))?
            .to_string();

        Ok(org_id)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(anyhow::anyhow!(
            "Failed to create organization: {}",
            error_text
        ))
    }
}

fn save_setup_credentials(result: &BootstrapResult) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let creds_path = format!("{}/.gb-setup-credentials", home);

    let content = format!(
        r#"# General Bots Initial Setup Credentials
# Created: {}
# DELETE THIS FILE AFTER FIRST LOGIN

╔════════════════════════════════════════════════════════════╗
║              ADMIN LOGIN CREDENTIALS (OTP)                 ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║  Username: {:<46}║
║  Password: {:<46}║
║  Email:    {:<46}║
║                                                            ║
║  Login URL: {:<45}║
║                                                            ║
╚════════════════════════════════════════════════════════════╝

IMPORTANT:
- This is a one-time password (OTP)
- You will be required to change it on first login
- Delete this file after you have logged in successfully

Alternative access via Zitadel console:
1. Go to: {}/ui/console
2. Login with admin PAT from: ./botserver-stack/conf/directory/admin-pat.txt
3. Find user '{}' and manage settings
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        result.username,
        result.initial_password,
        result.email,
        result.setup_url,
        result.setup_url.split("/ui/").next().unwrap_or("http://localhost:8300"),
        result.username
    );

    match fs::write(&creds_path, &content) {
        Ok(_) => {
            #[cfg(unix)]
            {
                if let Err(e) = fs::set_permissions(&creds_path, fs::Permissions::from_mode(0o600)) {
                    warn!("Failed to set file permissions: {}", e);
                }
            }
            info!("Setup credentials saved to: {}", creds_path);
        }
        Err(e) => {
            error!("Failed to save setup credentials: {}", e);
        }
    }
}

fn print_bootstrap_credentials(result: &BootstrapResult) {
    let separator = "═".repeat(60);

    println!();
    println!("╔{}╗", separator);
    println!("║{:^60}║", "");
    println!("║{:^60}║", "🤖 GENERAL BOTS - INITIAL SETUP");
    println!("║{:^60}║", "");
    println!("╠{}╣", separator);
    println!("║{:^60}║", "");
    println!("║  {:56}║", "Administrator account created!");
    println!("║{:^60}║", "");
    println!("╠{}╣", separator);
    println!("║{:^60}║", "");
    println!("║{:^60}║", "🔐 ONE-TIME PASSWORD (OTP) FOR LOGIN:");
    println!("║{:^60}║", "");
    println!("║  {:<58}║", format!("Username: {}", result.username));
    println!("║  {:<58}║", format!("Password: {}", result.initial_password));
    println!("║  {:<58}║", format!("Email:    {}", result.email));
    println!("║{:^60}║", "");

    if let Some(ref org_id) = result.organization_id {
        println!(
            "║  {:<58}║",
            format!("Organization: {} ({})", DEFAULT_ORG_NAME, &org_id[..8.min(org_id.len())])
        );
        println!("║{:^60}║", "");
    }

    println!("╠{}╣", separator);
    println!("║{:^60}║", "");
    println!("║  {:56}║", "🌐 LOGIN URL:");
    println!("║{:^60}║", "");

    let url_display = if result.setup_url.len() > 54 {
        format!("{}...", &result.setup_url[..51])
    } else {
        result.setup_url.clone()
    };
    println!("║  {:56}║", url_display);
    println!("║{:^60}║", "");
    println!("╠{}╣", separator);
    println!("║{:^60}║", "");
    println!("║  ⚠️  {:<53}║", "IMPORTANT - SAVE THESE CREDENTIALS!");
    println!("║{:^60}║", "");
    println!("║  {:<56}║", "• This password will NOT be shown again");
    println!("║  {:<56}║", "• You must change it on first login");
    println!("║  {:<56}║", "• Credentials also saved to: ~/.gb-setup-credentials");
    println!("║{:^60}║", "");
    println!("╚{}╝", separator);
    println!();

    info!(
        "Bootstrap complete: admin user '{}' created with OTP password",
        result.username
    );
}

pub fn print_existing_admin_notice() {
    println!();
    println!("ℹ️  Admin user already exists. Skipping bootstrap.");
    println!("   If you forgot your password, use Zitadel console to reset it.");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_password() {
        let password = generate_secure_password();

        assert!(password.len() >= 14);

        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%&*".contains(c));

        assert!(has_lower, "Password should contain lowercase");
        assert!(has_upper, "Password should contain uppercase");
        assert!(has_digit, "Password should contain digits");
        assert!(has_special, "Password should contain special chars");
    }

    #[test]
    fn test_password_uniqueness() {
        let passwords: Vec<String> = (0..10).map(|_| generate_secure_password()).collect();

        for i in 0..passwords.len() {
            for j in (i + 1)..passwords.len() {
                assert_ne!(
                    passwords[i], passwords[j],
                    "Generated passwords should be unique"
                );
            }
        }
    }
}
