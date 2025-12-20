//! Stalwart Sync Service
//!
//! This module provides synchronization between General Bots database tables and
//! Stalwart Mail Server. It handles bi-directional sync for:
//!
//! - Distribution Lists (sync with Stalwart principals)
//! - Auto-Responders (sync with Stalwart Sieve scripts)
//! - Email Rules/Filters (sync with Stalwart Sieve scripts)
//! - Shared Mailboxes (sync with Stalwart group principals)
//!
//! # Version: 6.1.0
//!
//! # Architecture
//!
//! The sync service follows a "write-through" pattern:
//! 1. Create/Update in Stalwart first (source of truth for email delivery)
//! 2. Store reference ID in our database (for UI and caching)
//! 3. On delete, remove from both systems
//!
//! This ensures email functionality remains intact even if our DB has issues.

use super::stalwart_client::{
    AccountUpdate, AutoResponderConfig, EmailRule, RuleAction, RuleCondition, StalwartClient,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

// Data Transfer Objects (matching 6.1.0_enterprise_suite migration)
// These are simplified DTOs for the sync layer - not direct ORM mappings

/// Distribution list DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionListDto {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub email_alias: Option<String>,
    pub description: Option<String>,
    pub members: Vec<String>,
    pub is_public: bool,
    pub stalwart_principal_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New distribution list for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDistributionList {
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub email_alias: String,
    pub description: Option<String>,
    pub members: Vec<String>,
}

/// Auto-responder DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResponderDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub responder_type: String,
    pub subject: String,
    pub body_html: String,
    pub body_plain: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub send_to_internal_only: bool,
    pub exclude_addresses: Option<String>,
    pub is_active: bool,
    pub stalwart_sieve_id: Option<String>,
}

/// New auto-responder for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAutoResponder {
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub subject: String,
    pub body_html: String,
    pub body_plain: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub only_contacts: bool,
}

/// Email rule DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRuleDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub is_active: bool,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub stop_processing: bool,
    pub stalwart_sieve_id: Option<String>,
}

/// New email rule for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEmailRule {
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub stop_processing: bool,
}

/// Shared mailbox DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMailboxDto {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub email_address: String,
    pub display_name: String,
    pub description: Option<String>,
    pub stalwart_account_id: Option<String>,
    pub is_active: bool,
}

/// Shared mailbox member DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMailboxMemberDto {
    pub id: Uuid,
    pub mailbox_id: Uuid,
    pub user_id: Uuid,
    pub permission_level: String,
    pub added_at: DateTime<Utc>,
}

// Sync Service

/// Service for synchronizing data between General Bots and Stalwart
///
/// This service handles the synchronization logic between our database
/// and Stalwart Mail Server. It requires a database pool and Stalwart client.
///
/// Note: Database operations should be implemented by the caller using this
/// service - this keeps the sync logic separate from ORM details.
pub struct StalwartSyncService {
    stalwart: Arc<StalwartClient>,
}

impl StalwartSyncService {
    /// Create a new sync service
    pub fn new(stalwart_client: Arc<StalwartClient>) -> Self {
        Self {
            stalwart: stalwart_client,
        }
    }

    /// Get reference to the Stalwart client
    pub fn stalwart(&self) -> &StalwartClient {
        &self.stalwart
    }

    // ========================================================================
    // Distribution Lists
    // ========================================================================

    /// Create a distribution list in Stalwart
    /// Returns the Stalwart principal ID to store in database
    pub async fn create_distribution_list_in_stalwart(
        &self,
        list: &NewDistributionList,
    ) -> Result<String> {
        info!(
            "Creating distribution list '{}' with email '{}' in Stalwart",
            list.name, list.email_alias
        );

        let stalwart_id = self
            .stalwart
            .create_distribution_list(&list.name, &list.email_alias, list.members.clone())
            .await
            .context("Failed to create distribution list in Stalwart")?;

        info!(
            "Created distribution list in Stalwart with ID: {}",
            stalwart_id
        );

        Ok(stalwart_id.to_string())
    }

    /// Update a distribution list in Stalwart
    pub async fn update_distribution_list_in_stalwart(
        &self,
        stalwart_id: &str,
        name: Option<&str>,
        members: Option<&[String]>,
    ) -> Result<()> {
        let mut updates = Vec::new();

        if let Some(n) = name {
            updates.push(AccountUpdate::set("description", n.to_string()));
        }

        if let Some(m) = members {
            // Clear and re-add members
            updates.push(AccountUpdate::clear("members"));
            for member in m {
                updates.push(AccountUpdate::add_item("members", member.clone()));
            }
        }

        if !updates.is_empty() {
            self.stalwart
                .update_account(stalwart_id, updates)
                .await
                .context("Failed to update distribution list in Stalwart")?;
        }

        info!("Updated distribution list {} in Stalwart", stalwart_id);
        Ok(())
    }

    /// Delete a distribution list from Stalwart
    pub async fn delete_distribution_list_from_stalwart(&self, stalwart_id: &str) -> Result<()> {
        self.stalwart
            .delete_account(stalwart_id)
            .await
            .context("Failed to delete distribution list from Stalwart")?;

        info!("Deleted distribution list {} from Stalwart", stalwart_id);
        Ok(())
    }

    // ========================================================================
    // Auto-Responders
    // ========================================================================

    /// Create/update an auto-responder in Stalwart via Sieve script
    /// Returns the Sieve script ID to store in database
    pub async fn set_auto_responder_in_stalwart(
        &self,
        account_id: &str,
        responder: &NewAutoResponder,
    ) -> Result<String> {
        info!(
            "Setting auto-responder for user {} in account {}",
            responder.user_id, account_id
        );

        let config = AutoResponderConfig {
            enabled: true,
            subject: responder.subject.clone(),
            body_plain: responder.body_plain.clone().unwrap_or_default(),
            body_html: Some(responder.body_html.clone()),
            start_date: responder.start_date.map(|dt| dt.date_naive()),
            end_date: responder.end_date.map(|dt| dt.date_naive()),
            only_contacts: responder.only_contacts,
            vacation_days: 1,
        };

        let sieve_id = self
            .stalwart
            .set_auto_responder(account_id, &config)
            .await
            .context("Failed to set auto-responder in Stalwart")?;

        info!("Created auto-responder Sieve script: {}", sieve_id);
        Ok(sieve_id)
    }

    /// Disable an auto-responder in Stalwart
    pub async fn disable_auto_responder_in_stalwart(&self, account_id: &str) -> Result<()> {
        self.stalwart
            .disable_auto_responder(account_id)
            .await
            .context("Failed to disable auto-responder in Stalwart")?;

        info!("Disabled auto-responder for account {}", account_id);
        Ok(())
    }

    // ========================================================================
    // Email Rules/Filters
    // ========================================================================

    /// Create/update an email rule in Stalwart via Sieve script
    /// Returns the Sieve script ID to store in database
    pub async fn set_email_rule_in_stalwart(
        &self,
        account_id: &str,
        rule: &NewEmailRule,
        rule_id: Uuid,
    ) -> Result<String> {
        info!(
            "Setting email rule '{}' for user {} in account {}",
            rule.name, rule.user_id, account_id
        );

        let stalwart_rule = EmailRule {
            id: rule_id.to_string(),
            name: rule.name.clone(),
            priority: rule.priority,
            enabled: true,
            conditions: rule.conditions.clone(),
            actions: rule.actions.clone(),
            stop_processing: rule.stop_processing,
        };

        let sieve_id = self
            .stalwart
            .set_filter_rule(account_id, &stalwart_rule)
            .await
            .context("Failed to set email rule in Stalwart")?;

        info!("Created email rule Sieve script: {}", sieve_id);
        Ok(sieve_id)
    }

    /// Delete an email rule from Stalwart
    pub async fn delete_email_rule_from_stalwart(
        &self,
        account_id: &str,
        rule_id: &str,
    ) -> Result<()> {
        self.stalwart
            .delete_filter_rule(account_id, rule_id)
            .await
            .context("Failed to delete email rule from Stalwart")?;

        info!("Deleted email rule {} from Stalwart", rule_id);
        Ok(())
    }

    // ========================================================================
    // Shared Mailboxes
    // ========================================================================

    /// Create a shared mailbox in Stalwart
    /// Returns the Stalwart account ID to store in database
    pub async fn create_shared_mailbox_in_stalwart(
        &self,
        name: &str,
        email: &str,
        initial_members: Vec<String>,
    ) -> Result<String> {
        info!("Creating shared mailbox '{}' with email '{}'", name, email);

        let stalwart_id = self
            .stalwart
            .create_shared_mailbox(name, email, initial_members)
            .await
            .context("Failed to create shared mailbox in Stalwart")?;

        info!(
            "Created shared mailbox in Stalwart with ID: {}",
            stalwart_id
        );

        Ok(stalwart_id.to_string())
    }

    /// Add a member to a shared mailbox in Stalwart
    pub async fn add_shared_mailbox_member_in_stalwart(
        &self,
        stalwart_id: &str,
        member_email: &str,
    ) -> Result<()> {
        self.stalwart
            .add_members(stalwart_id, vec![member_email.to_string()])
            .await
            .context("Failed to add member to shared mailbox in Stalwart")?;

        info!(
            "Added member {} to shared mailbox {} in Stalwart",
            member_email, stalwart_id
        );
        Ok(())
    }

    /// Remove a member from a shared mailbox in Stalwart
    pub async fn remove_shared_mailbox_member_in_stalwart(
        &self,
        stalwart_id: &str,
        member_email: &str,
    ) -> Result<()> {
        self.stalwart
            .remove_members(stalwart_id, vec![member_email.to_string()])
            .await
            .context("Failed to remove member from shared mailbox in Stalwart")?;

        info!(
            "Removed member {} from shared mailbox {} in Stalwart",
            member_email, stalwart_id
        );
        Ok(())
    }

    /// Delete a shared mailbox from Stalwart
    pub async fn delete_shared_mailbox_from_stalwart(&self, stalwart_id: &str) -> Result<()> {
        self.stalwart
            .delete_account(stalwart_id)
            .await
            .context("Failed to delete shared mailbox from Stalwart")?;

        info!("Deleted shared mailbox {} from Stalwart", stalwart_id);
        Ok(())
    }

    // ========================================================================
    // Bulk Sync Operations
    // ========================================================================

    /// Sync a distribution list to Stalwart (for recovery/migration)
    /// Returns the Stalwart principal ID
    pub async fn sync_distribution_list_to_stalwart(
        &self,
        name: &str,
        email_alias: &str,
        members: Vec<String>,
    ) -> Result<String> {
        match self
            .stalwart
            .create_distribution_list(name, email_alias, members.clone())
            .await
        {
            Ok(stalwart_id) => {
                info!(
                    "Synced distribution list '{}' to Stalwart with ID: {}",
                    name, stalwart_id
                );
                Ok(stalwart_id.to_string())
            }
            Err(e) => {
                warn!(
                    "Failed to sync distribution list '{}' to Stalwart: {}",
                    name, e
                );
                Err(e)
            }
        }
    }

    /// Verify Stalwart connectivity
    pub async fn health_check(&self) -> Result<bool> {
        self.stalwart.health_check().await
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_distribution_list() {
        let list = NewDistributionList {
            bot_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            name: "Test List".to_string(),
            email_alias: "test@example.com".to_string(),
            description: Some("A test list".to_string()),
            members: vec![
                "user1@example.com".to_string(),
                "user2@example.com".to_string(),
            ],
        };

        assert_eq!(list.name, "Test List");
        assert_eq!(list.members.len(), 2);
    }

    #[test]
    fn test_new_auto_responder() {
        let responder = NewAutoResponder {
            bot_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            subject: "Out of Office".to_string(),
            body_html: "<p>I am away</p>".to_string(),
            body_plain: Some("I am away".to_string()),
            start_date: Some(Utc::now()),
            end_date: None,
            only_contacts: false,
        };

        assert_eq!(responder.subject, "Out of Office");
    }

    #[test]
    fn test_new_email_rule() {
        let rule = NewEmailRule {
            bot_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Move newsletters".to_string(),
            priority: 10,
            conditions: vec![RuleCondition {
                field: "from".to_string(),
                operator: "contains".to_string(),
                value: "newsletter".to_string(),
                header_name: None,
                case_sensitive: false,
            }],
            actions: vec![RuleAction {
                action_type: "move".to_string(),
                value: "Newsletters".to_string(),
            }],
            stop_processing: true,
        };

        assert_eq!(rule.name, "Move newsletters");
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.actions.len(), 1);
    }

    #[test]
    fn test_distribution_list_dto() {
        let dto = DistributionListDto {
            id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            name: "Sales Team".to_string(),
            email_alias: Some("sales@example.com".to_string()),
            description: Some("Sales distribution list".to_string()),
            members: vec!["alice@example.com".to_string()],
            is_public: false,
            stalwart_principal_id: Some("123".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(dto.name, "Sales Team");
        assert!(dto.stalwart_principal_id.is_some());
    }
}
