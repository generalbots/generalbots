




















use super::stalwart_client::{
    AccountUpdate, AutoResponderConfig, EmailRule, RuleAction, RuleCondition, StalwartClient,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;





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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDistributionList {
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub email_alias: String,
    pub description: Option<String>,
    pub members: Vec<String>,
}


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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMailboxMemberDto {
    pub id: Uuid,
    pub mailbox_id: Uuid,
    pub user_id: Uuid,
    pub permission_level: String,
    pub added_at: DateTime<Utc>,
}










pub struct StalwartSyncService {
    stalwart: Arc<StalwartClient>,
}

impl StalwartSyncService {

    pub fn new(stalwart_client: Arc<StalwartClient>) -> Self {
        Self {
            stalwart: stalwart_client,
        }
    }


    pub fn stalwart(&self) -> &StalwartClient {
        &self.stalwart
    }







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


    pub async fn delete_distribution_list_from_stalwart(&self, stalwart_id: &str) -> Result<()> {
        self.stalwart
            .delete_account(stalwart_id)
            .await
            .context("Failed to delete distribution list from Stalwart")?;

        info!("Deleted distribution list {} from Stalwart", stalwart_id);
        Ok(())
    }







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


    pub async fn disable_auto_responder_in_stalwart(&self, account_id: &str) -> Result<()> {
        self.stalwart
            .disable_auto_responder(account_id)
            .await
            .context("Failed to disable auto-responder in Stalwart")?;

        info!("Disabled auto-responder for account {}", account_id);
        Ok(())
    }







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


    pub async fn delete_shared_mailbox_from_stalwart(&self, stalwart_id: &str) -> Result<()> {
        self.stalwart
            .delete_account(stalwart_id)
            .await
            .context("Failed to delete shared mailbox from Stalwart")?;

        info!("Deleted shared mailbox {} from Stalwart", stalwart_id);
        Ok(())
    }







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


    pub async fn health_check(&self) -> Result<bool> {
        self.stalwart.health_check().await
    }
}


