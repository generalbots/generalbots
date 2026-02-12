//! Bot Database Management Module
//!
//! This module handles per-bot database management, including:
//! - Getting bot database names from the bots table
//! - Creating connection pools to bot-specific databases
//! - Ensuring bot databases exist and are properly initialized
//! - Syncing bot databases on server startup

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::PgConnection;
use log::{error, info};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::core::shared::utils::DbPool;

/// Cache for bot database connection pools
pub struct BotDatabaseManager {
    /// Main database pool (for accessing bots table)
    main_pool: DbPool,
    /// Cached connection pools for bot databases
    bot_pools: Arc<RwLock<HashMap<Uuid, DbPool>>>,
    /// Base connection URL (without database name)
    base_url: String,
}

#[derive(QueryableByName, Debug)]
pub struct BotDatabaseInfo {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
    pub database_name: Option<String>,
}

#[derive(QueryableByName)]
struct DbExists {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    exists: bool,
}

impl BotDatabaseManager {
    /// Create a new BotDatabaseManager
    pub fn new(main_pool: DbPool, database_url: &str) -> Self {
        let base_url = Self::extract_base_url(database_url);
        Self {
            main_pool,
            bot_pools: Arc::new(RwLock::new(HashMap::new())),
            base_url,
        }
    }

    /// Extract base URL without database name
    /// Converts "postgres://user:pass@host:port/dbname" to "postgres://user:pass@host:port"
    fn extract_base_url(database_url: &str) -> String {
        if let Some(last_slash_pos) = database_url.rfind('/') {
            // Check if there's a query string
            let db_part = &database_url[last_slash_pos..];
            if let Some(query_pos) = db_part.find('?') {
                // Keep query string, just remove db name
                format!(
                    "{}{}",
                    &database_url[..last_slash_pos],
                    &db_part[query_pos..]
                )
            } else {
                database_url[..last_slash_pos].to_string()
            }
        } else {
            database_url.to_string()
        }
    }

    /// Get the database name for a specific bot
    pub fn get_bot_database_name(
        &self,
        bot_id: Uuid,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.main_pool.get()?;

        let result: Option<BotDatabaseInfo> = sql_query(
            "SELECT id, name, database_name FROM bots WHERE id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result(&mut conn)
        .optional()?;

        Ok(result.and_then(|info| info.database_name))
    }

    /// Get or create a connection pool to a bot's specific database
    pub fn get_bot_pool(
        &self,
        bot_id: Uuid,
    ) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
        // Check cache first
        {
            let pools = self.bot_pools.read().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(pool) = pools.get(&bot_id) {
                return Ok(<DbPool as Clone>::clone(pool));
            }
        }

        // Get database name for this bot
        let db_name = self.get_bot_database_name(bot_id)?
            .ok_or_else(|| format!("No database configured for bot {}", bot_id))?;

        // Create new pool
        let pool = self.create_pool_for_database(&db_name)?;

        // Cache it
        {
            let mut pools = self.bot_pools.write().map_err(|e| format!("Lock error: {}", e))?;
            pools.insert(bot_id, pool.clone());
        }

        Ok(pool)
    }

    /// Create a connection pool for a specific database
    fn create_pool_for_database(
        &self,
        database_name: &str,
    ) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = format!("{}/{}", self.base_url, database_name);
        let manager = ConnectionManager::<PgConnection>::new(&database_url);

        Pool::builder()
            .max_size(5) // Smaller pool for per-bot databases
            .min_idle(Some(0))
            .connection_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(Some(std::time::Duration::from_secs(300)))
            .max_lifetime(Some(std::time::Duration::from_secs(1800)))
            .build(manager)
            .map_err(|e| format!("Failed to create pool for database {}: {}", database_name, e).into())
    }

    /// Create a database if it doesn't exist
    pub fn ensure_database_exists(
        &self,
        database_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let safe_db_name: String = database_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        if safe_db_name.is_empty() || safe_db_name.len() > 63 {
            return Err("Invalid database name".into());
        }

        let mut conn = self.main_pool.get()?;

        // Check if database exists
        let check_query = format!(
            "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = '{}') as exists",
            safe_db_name
        );

        let exists = sql_query(&check_query)
            .get_result::<DbExists>(&mut conn)
            .map(|r| r.exists)
            .unwrap_or(false);

        if exists {
            info!("Database {} already exists", safe_db_name);
            return Ok(false); // Already existed
        }

        // Create database
        let create_query = format!("CREATE DATABASE {}", safe_db_name);
        if let Err(e) = sql_query(&create_query).execute(&mut conn) {
            let err_str = e.to_string();
            if err_str.contains("already exists") {
                info!("Database {} already exists (concurrent creation)", safe_db_name);
                return Ok(false);
            }
            return Err(format!("Failed to create database: {}", e).into());
        }

        info!("Created database: {}", safe_db_name);
        Ok(true) // Newly created
    }

    /// Generate a database name for a bot
    pub fn generate_database_name(bot_name: &str) -> String {
        format!(
            "bot_{}",
            bot_name
                .replace(['-', ' '], "_")
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        )
    }

    /// Ensure a bot has a database and update the bots table if needed
    pub fn ensure_bot_has_database(
        &self,
        bot_id: Uuid,
        bot_name: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if bot already has a database_name
        let existing_db_name = self.get_bot_database_name(bot_id)?;

        let db_name = if let Some(name) = existing_db_name {
            name
        } else {
            // Generate and set database name
            let new_db_name = Self::generate_database_name(bot_name);
            let mut conn = self.main_pool.get()?;

            sql_query("UPDATE bots SET database_name = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Varchar, _>(&new_db_name)
                .bind::<diesel::sql_types::Uuid, _>(bot_id)
                .execute(&mut conn)?;

            info!("Set database_name for bot {} to {}", bot_id, new_db_name);
            new_db_name
        };

        // Ensure the database exists
        self.ensure_database_exists(&db_name)?;

        Ok(db_name)
    }

    /// Get all active bots and their database info
    pub fn get_all_bots(&self) -> Result<Vec<BotDatabaseInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.main_pool.get()?;

        let bots: Vec<BotDatabaseInfo> = sql_query(
            "SELECT id, name, database_name FROM bots WHERE is_active = true",
        )
        .get_results(&mut conn)?;

        Ok(bots)
    }

    /// Sync all bot databases - ensures each bot has a database
    /// Call this during server startup
    pub fn sync_all_bot_databases(&self) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let bots = self.get_all_bots()?;
        let mut result = SyncResult::default();

        for bot in bots {
            match self.ensure_bot_has_database(bot.id, &bot.name) {
                Ok(db_name) => {
                    if bot.database_name.is_none() {
                        result.databases_created += 1;
                        info!("Created database for bot {}: {}", bot.name, db_name);
                    } else {
                        result.databases_verified += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to ensure database for bot {}: {}", bot.name, e);
                    result.errors.push(format!("Bot {}: {}", bot.name, e));
                }
            }
        }

        info!(
            "Bot database sync complete: {} created, {} verified, {} errors",
            result.databases_created,
            result.databases_verified,
            result.errors.len()
        );

        Ok(result)
    }

    /// Execute a table creation SQL in a bot's database
    pub fn create_table_in_bot_database(
        &self,
        bot_id: Uuid,
        create_sql: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_bot_pool(bot_id)?;
        let mut conn = pool.get()?;

        sql_query(create_sql).execute(&mut conn)?;

        Ok(())
    }

    /// Clear cached pool for a bot (useful when database is recreated)
    pub fn clear_bot_pool_cache(&self, bot_id: Uuid) {
        if let Ok(mut pools) = self.bot_pools.write() {
            let _: Option<_> = pools.remove(&bot_id);
        }
    }

    /// Clear all cached pools
    pub fn clear_all_pool_caches(&self) {
        if let Ok(mut pools) = self.bot_pools.write() {
            std::collections::HashMap::clear(&mut pools);
        }
    }
}

/// Result of syncing bot databases
#[derive(Default, Debug)]
pub struct SyncResult {
    pub databases_created: usize,
    pub databases_verified: usize,
    pub errors: Vec<String>,
}

/// Helper function to create a bot database manager from AppState
pub fn create_bot_database_manager(pool: DbPool, database_url: &str) -> BotDatabaseManager {
    BotDatabaseManager::new(pool, database_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_url() {
        assert_eq!(
            BotDatabaseManager::extract_base_url("postgres://user:pass@localhost:5432/mydb"),
            "postgres://user:pass@localhost:5432"
        );

        assert_eq!(
            BotDatabaseManager::extract_base_url("postgres://user:pass@localhost:5432/mydb?sslmode=require"),
            "postgres://user:pass@localhost:5432?sslmode=require"
        );

        assert_eq!(
            BotDatabaseManager::extract_base_url("postgres://user:pass@localhost/mydb"),
            "postgres://user:pass@localhost"
        );
    }

    #[test]
    fn test_generate_database_name() {
        assert_eq!(
            BotDatabaseManager::generate_database_name("my-bot"),
            "bot_my_bot"
        );

        assert_eq!(
            BotDatabaseManager::generate_database_name("My Bot 2"),
            "bot_my_bot_2"
        );

        assert_eq!(
            BotDatabaseManager::generate_database_name("test@bot!"),
            "bot_testbot"
        );
    }
}
