use anyhow::Result;
use aws_sdk_s3::Client as S3Client;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// User provisioning service that creates accounts across all integrated services
pub struct UserProvisioningService {
    db_conn: Arc<PgConnection>,
    s3_client: Arc<S3Client>,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization: String,
    pub is_admin: bool,
    pub bots: Vec<BotAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAccess {
    pub bot_id: String,
    pub bot_name: String,
    pub role: UserRole,
    pub home_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    ReadOnly,
}

impl UserProvisioningService {
    pub fn new(db_conn: Arc<PgConnection>, s3_client: Arc<S3Client>, base_url: String) -> Self {
        Self {
            db_conn,
            s3_client,
            base_url,
        }
    }

    /// Create a new user across all services
    pub async fn provision_user(&self, account: &UserAccount) -> Result<()> {
        log::info!("Provisioning user: {}", account.username);

        // 1. Create user in database using existing user management
        let user_id = self.create_database_user(account).await?;

        // 2. Create home directories in S3 for each bot using existing drive API
        for bot_access in &account.bots {
            self.create_s3_home(account, bot_access).await?;
        }

        // 3. Create email account using existing email API
        if let Err(e) = self.setup_email_account(account).await {
            log::warn!("Email account creation failed: {}", e);
        }

        // 4. Setup OAuth linking in configuration
        self.setup_oauth_config(&user_id, account).await?;

        log::info!("User {} provisioned successfully", account.username);
        Ok(())
    }

    async fn create_database_user(&self, account: &UserAccount) -> Result<String> {
        use crate::shared::models::schema::users;
        use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
        use diesel::prelude::*;
        use uuid::Uuid;

        let user_id = Uuid::new_v4().to_string();
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(Uuid::new_v4().to_string().as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
            .to_string();

        diesel::insert_into(users::table)
            .values((
                users::id.eq(&user_id),
                users::username.eq(&account.username),
                users::email.eq(&account.email),
                users::password_hash.eq(&password_hash),
                users::is_admin.eq(account.is_admin),
                users::created_at.eq(chrono::Utc::now()),
            ))
            .execute(&*self.db_conn)?;

        Ok(user_id)
    }

    async fn create_s3_home(&self, account: &UserAccount, bot_access: &BotAccess) -> Result<()> {
        let bucket_name = format!("{}.gbdrive", bot_access.bot_name);
        let home_path = format!("home/{}/", account.username);

        // Ensure bucket exists
        match self
            .s3_client
            .head_bucket()
            .bucket(&bucket_name)
            .send()
            .await
        {
            Err(_) => {
                self.s3_client
                    .create_bucket()
                    .bucket(&bucket_name)
                    .send()
                    .await?;
            }
            Ok(_) => {}
        }

        // Create user home directory marker
        self.s3_client
            .put_object()
            .bucket(&bucket_name)
            .key(&home_path)
            .body(aws_sdk_s3::primitives::ByteStream::from(vec![]))
            .send()
            .await?;

        // Create default folders
        for folder in &["documents", "projects", "shared"] {
            let folder_key = format!("{}{}/", home_path, folder);
            self.s3_client
                .put_object()
                .bucket(&bucket_name)
                .key(&folder_key)
                .body(aws_sdk_s3::primitives::ByteStream::from(vec![]))
                .send()
                .await?;
        }

        log::info!(
            "Created S3 home for {} in {}",
            account.username,
            bucket_name
        );
        Ok(())
    }

    async fn setup_email_account(&self, account: &UserAccount) -> Result<()> {
        use crate::shared::models::schema::user_email_accounts;
        use diesel::prelude::*;

        // Store email configuration in database
        diesel::insert_into(user_email_accounts::table)
            .values((
                user_email_accounts::user_id.eq(&account.username),
                user_email_accounts::email.eq(&account.email),
                user_email_accounts::imap_server.eq("localhost"),
                user_email_accounts::imap_port.eq(993),
                user_email_accounts::smtp_server.eq("localhost"),
                user_email_accounts::smtp_port.eq(465),
                user_email_accounts::username.eq(&account.username),
                user_email_accounts::password_encrypted.eq("oauth"),
                user_email_accounts::is_active.eq(true),
            ))
            .execute(&*self.db_conn)?;

        log::info!("Setup email configuration for: {}", account.email);
        Ok(())
    }

    async fn setup_oauth_config(&self, _user_id: &str, account: &UserAccount) -> Result<()> {
        use crate::shared::models::schema::bot_configuration;
        use diesel::prelude::*;

        // Store OAuth configuration for services
        let services = vec![
            ("oauth-drive-enabled", "true"),
            ("oauth-email-enabled", "true"),
            ("oauth-git-enabled", "true"),
            ("oauth-provider", "zitadel"),
        ];

        for (key, value) in services {
            diesel::insert_into(bot_configuration::table)
                .values((
                    bot_configuration::bot_id.eq(uuid::Uuid::nil()),
                    bot_configuration::config_key.eq(key),
                    bot_configuration::config_value.eq(value),
                    bot_configuration::is_encrypted.eq(false),
                    bot_configuration::config_type.eq("string"),
                    bot_configuration::created_at.eq(chrono::Utc::now()),
                    bot_configuration::updated_at.eq(chrono::Utc::now()),
                ))
                .on_conflict((bot_configuration::bot_id, bot_configuration::config_key))
                .do_update()
                .set(bot_configuration::config_value.eq(value))
                .execute(&*self.db_conn)?;
        }

        log::info!("Setup OAuth configuration for user: {}", account.username);
        Ok(())
    }

    /// Remove user from all services
    pub async fn deprovision_user(&self, username: &str) -> Result<()> {
        log::info!("Deprovisioning user: {}", username);

        // Remove user data from all services
        self.remove_s3_data(username).await?;
        self.remove_email_config(username).await?;
        self.remove_user_from_db(username).await?;

        log::info!("User {} deprovisioned successfully", username);
        Ok(())
    }

    async fn remove_user_from_db(&self, username: &str) -> Result<()> {
        use crate::shared::models::schema::users;
        use diesel::prelude::*;

        diesel::delete(users::table.filter(users::username.eq(username)))
            .execute(&*self.db_conn)?;

        Ok(())
    }

    async fn remove_s3_data(&self, username: &str) -> Result<()> {
        // List all buckets and remove user home directories
        let buckets_result = self.s3_client.list_buckets().send().await?;

        if let Some(buckets) = buckets_result.buckets {
            for bucket in buckets {
                if let Some(name) = bucket.name {
                    if name.ends_with(".gbdrive") {
                        let prefix = format!("home/{}/", username);

                        // List and delete all objects with this prefix
                        let objects = self
                            .s3_client
                            .list_objects_v2()
                            .bucket(&name)
                            .prefix(&prefix)
                            .send()
                            .await?;

                        if let Some(contents) = objects.contents {
                            for object in contents {
                                if let Some(key) = object.key {
                                    self.s3_client
                                        .delete_object()
                                        .bucket(&name)
                                        .key(&key)
                                        .send()
                                        .await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn remove_email_config(&self, username: &str) -> Result<()> {
        use crate::shared::models::schema::user_email_accounts;
        use diesel::prelude::*;

        diesel::delete(
            user_email_accounts::table.filter(user_email_accounts::username.eq(username)),
        )
        .execute(&*self.db_conn)?;

        Ok(())
    }
}
