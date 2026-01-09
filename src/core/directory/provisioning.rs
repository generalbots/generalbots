use anyhow::Result;
use aws_sdk_s3::Client as S3Client;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct UserProvisioningService {
    db_pool: DbPool,
    s3_client: Option<Arc<S3Client>>,
    base_url: String,
}

impl std::fmt::Debug for UserProvisioningService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserProvisioningService")
            .field("base_url", &self.base_url)
            .field("has_s3_client", &self.s3_client.is_some())
            .finish_non_exhaustive()
    }
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
    pub fn new(db_pool: DbPool, s3_client: Option<Arc<S3Client>>, base_url: String) -> Self {
        Self {
            db_pool,
            s3_client,
            base_url,
        }
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn build_profile_url(&self, username: &str) -> String {
        format!("{}/users/{}/profile", self.base_url, username)
    }

    pub async fn provision_user(&self, account: &UserAccount) -> Result<()> {
        log::info!(
            "Provisioning user: {} via directory at {}",
            account.username,
            self.base_url
        );

        let user_id = self.create_database_user(account)?;

        for bot_access in &account.bots {
            self.create_s3_home(account, bot_access).await?;
        }

        if let Err(e) = self.setup_email_account(account) {
            log::warn!("Email account creation failed: {}", e);
        }

        self.setup_oauth_config(&user_id, account)?;

        let profile_url = self.build_profile_url(&account.username);
        log::info!(
            "User {} provisioned successfully. Profile: {}",
            account.username,
            profile_url
        );
        Ok(())
    }

    fn create_database_user(&self, account: &UserAccount) -> Result<String> {
        use crate::shared::models::schema::users;
        use argon2::{
            password_hash::{rand_core::OsRng, SaltString},
            Argon2, PasswordHasher,
        };
        use diesel::prelude::*;

        let user_id = Uuid::new_v4();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(Uuid::new_v4().to_string().as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
            .to_string();

        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        diesel::insert_into(users::table)
            .values((
                users::id.eq(user_id),
                users::username.eq(&account.username),
                users::email.eq(&account.email),
                users::password_hash.eq(&password_hash),
                users::is_admin.eq(account.is_admin),
                users::created_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(user_id.to_string())
    }

    async fn create_s3_home(&self, account: &UserAccount, bot_access: &BotAccess) -> Result<()> {
        let Some(s3_client) = &self.s3_client else {
            log::warn!("S3 client not configured, skipping S3 home creation");
            return Ok(());
        };

        let bucket_name = format!("{}.gbdrive", bot_access.bot_name);
        let home_path = format!("home/{}/", account.username);

        if s3_client
            .head_bucket()
            .bucket(&bucket_name)
            .send()
            .await
            .is_err()
        {
            s3_client
                .create_bucket()
                .bucket(&bucket_name)
                .send()
                .await?;
        }

        s3_client
            .put_object()
            .bucket(&bucket_name)
            .key(&home_path)
            .body(aws_sdk_s3::primitives::ByteStream::from(vec![]))
            .send()
            .await?;

        for folder in &["documents", "projects", "shared"] {
            let folder_key = format!("{}{}/", home_path, folder);
            s3_client
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

    fn setup_email_account(&self, account: &UserAccount) -> Result<()> {
        use crate::shared::models::schema::user_email_accounts;
        use diesel::prelude::*;

        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;

        let user_uuid = Uuid::new_v4();
        diesel::insert_into(user_email_accounts::table)
            .values((
                user_email_accounts::user_id.eq(user_uuid),
                user_email_accounts::email.eq(&account.email),
                user_email_accounts::imap_server.eq("localhost"),
                user_email_accounts::imap_port.eq(993),
                user_email_accounts::smtp_server.eq("localhost"),
                user_email_accounts::smtp_port.eq(465),
                user_email_accounts::username.eq(&account.username),
                user_email_accounts::password_encrypted.eq("oauth"),
                user_email_accounts::is_active.eq(true),
            ))
            .execute(&mut conn)?;

        log::info!("Setup email configuration for: {}", account.email);
        Ok(())
    }

    fn setup_oauth_config(&self, _user_id: &str, account: &UserAccount) -> Result<()> {
        use crate::shared::models::schema::bot_configuration;
        use diesel::prelude::*;

        let services = vec![
            ("oauth-drive-enabled", "true"),
            ("oauth-email-enabled", "true"),
            ("oauth-git-enabled", "true"),
            ("oauth-provider", "zitadel"),
        ];

        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        for (key, value) in services {
            diesel::insert_into(bot_configuration::table)
                .values((
                    bot_configuration::bot_id.eq(Uuid::nil()),
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
                .execute(&mut conn)?;
        }

        log::info!("Setup OAuth configuration for user: {}", account.username);
        Ok(())
    }

    pub async fn deprovision_user(&self, username: &str) -> Result<()> {
        log::info!("Deprovisioning user: {}", username);

        self.remove_s3_data(username).await?;
        self.remove_email_config(username)?;
        self.remove_user_from_db(username)?;

        log::info!("User {} deprovisioned successfully", username);
        Ok(())
    }

    fn remove_user_from_db(&self, username: &str) -> Result<()> {
        use crate::shared::models::schema::users;
        use diesel::prelude::*;

        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        diesel::delete(users::table.filter(users::username.eq(username))).execute(&mut conn)?;

        Ok(())
    }

    async fn remove_s3_data(&self, username: &str) -> Result<()> {
        if let Some(s3_client) = &self.s3_client {
            let buckets_result = s3_client.list_buckets().send().await?;

            if let Some(buckets) = buckets_result.buckets {
                for bucket in buckets {
                    if let Some(name) = bucket.name {
                        if name.ends_with(".gbdrive") {
                            let prefix = format!("home/{}/", username);

                            let objects = s3_client
                                .list_objects_v2()
                                .bucket(&name)
                                .prefix(&prefix)
                                .send()
                                .await?;

                            if let Some(contents) = objects.contents {
                                for object in contents {
                                    if let Some(key) = object.key {
                                        s3_client
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
        }

        Ok(())
    }

    fn remove_email_config(&self, username: &str) -> Result<()> {
        use crate::shared::models::schema::user_email_accounts;
        use diesel::prelude::*;

        let mut conn = self
            .db_pool
            .get()
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;
        diesel::delete(
            user_email_accounts::table.filter(user_email_accounts::username.eq(username)),
        )
        .execute(&mut conn)?;

        Ok(())
    }
}
