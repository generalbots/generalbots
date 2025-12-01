# Microsoft 365 Migration Guide

Migrating from Microsoft 365 to self-hosted infrastructure.

## Overview

Microsoft 365 (formerly Office 365) includes multiple services that need to be migrated to different self-hosted components.

## Service Mapping

| Microsoft 365 Service | Self-Hosted Alternative | Migration Tool |
|----------------------|------------------------|----------------|
| Exchange Online | Mail server (Stalwart, etc.) | imapsync |
| OneDrive | MinIO or Nextcloud | rclone |
| SharePoint | MinIO + Wiki/CMS | rclone + export tools |
| Teams | Mattermost, General Bots, etc. | Export/Import APIs |
| Azure AD | Keycloak, Authentik, Zitadel | LDAP export |

## Migration Steps

### 1. Email Migration (Exchange Online)

Before beginning the email migration, ensure IMAP access is enabled in Exchange Online, your target mail server is configured, and you have user credentials or app passwords available.

The migration process uses imapsync for mailbox migration. Migrate in batches to avoid throttling from Microsoft's servers, and preserve folder structure and flags during transfer.

Keep in mind that Exchange uses proprietary features such as categories that may not transfer cleanly. Calendar and contacts require separate migration using CalDAV and CardDAV protocols. Shared mailboxes require special handling and may need to be migrated individually.

### 2. File Migration (OneDrive/SharePoint)

Prerequisites include having the OneDrive sync client or API access configured, sufficient storage on the target system, and adequate network bandwidth for the transfer.

Use rclone with the OneDrive backend for the migration process. Maintain folder structure during transfer and preserve timestamps where possible.

Be aware that SharePoint metadata won't transfer automatically and may need manual recreation. Version history is typically lost during migration. Permissions need to be recreated on the target system.

### 3. User Migration (Azure AD)

Prepare for user migration by setting up Azure AD Connect or API access, and ensure your target identity provider is ready to receive users.

Export users via PowerShell or Graph API, transform the data to the target format such as LDIF or JSON, then import to your new identity provider.

Important considerations include that passwords cannot be exported from Azure AD, so users will need to reset their passwords. MFA settings require reconfiguration on the new system. Group memberships need mapping to equivalent structures in the target system.

## Common Challenges

### API Throttling

Microsoft throttles API calls to protect their infrastructure. Plan for a slow, steady migration rather than attempting bulk transfers. Use batch operations where possible and consider running migrations during off-peak hours.

### Data Volume

Large organizations may have accumulated terabytes of OneDrive and SharePoint data, years of email history, and thousands of users. Factor this into your timeline and resource planning.

### Feature Parity

Some M365 features have no direct equivalent in self-hosted solutions. Power Automate workflows will need to be recreated using different automation tools. SharePoint lists and forms require alternative solutions. Teams channel history may be difficult to preserve in its original format.

## Tools and Utilities

### PowerShell for Export

The Azure AD PowerShell module handles user export operations. Exchange Online PowerShell provides mailbox information. SharePoint Online PowerShell helps with site inventory and metadata export.

### Graph API

The Graph API provides programmatic access to most M365 services and is useful for custom migration scripts. Using it requires app registration and appropriate permissions in your Azure tenant.

### Third-Party Tools

Commercial options include BitTitan MigrationWiz and Sharegate, which provide guided migration experiences. Various open-source scripts are available on GitHub for more customized approaches.

## Post-Migration

### DNS Changes

Update MX records to point to your new email server. Update autodiscover records for email client configuration. Consider keeping a hybrid setup temporarily to catch any missed emails during the transition.

### User Communication

Provide new login credentials to all users. Document any changed procedures and differences from the M365 experience. Offer training sessions on the new tools to ensure smooth adoption.

### Validation

Verify email delivery works correctly in both directions. Test file access to ensure permissions transferred properly. Confirm authentication works for all migrated users.

## Cost Considerations

### Subscription Overlap

You may need to maintain M365 subscriptions during the migration period. Consider read-only licenses for archive access if you need to retain access to historical data.

### Data Transfer Costs

Factor in egress charges from Microsoft when transferring large amounts of data. Account for bandwidth costs if transferring over the internet rather than dedicated connections.

## Timeline Estimates

Small organizations with fewer than 50 users typically complete migration in 1-2 weeks. Medium organizations with 50-500 users usually require 1-2 months. Large organizations with more than 500 users should plan for 2-6 months.

Factors affecting timeline include data volume, network speed, complexity of the existing setup, and user training needs.

## Next Steps

Review the [Common Concepts](./common-concepts.md) guide for general migration principles. See [Validation](./validation.md) for detailed testing procedures to verify your migration succeeded.