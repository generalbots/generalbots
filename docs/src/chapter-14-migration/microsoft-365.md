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

**Prerequisites:**
- IMAP access enabled in Exchange Online
- Target mail server configured
- User credentials or app passwords

**Process:**
- Use imapsync for mailbox migration
- Migrate in batches to avoid throttling
- Preserve folder structure and flags

**Considerations:**
- Exchange uses proprietary features (categories, etc.) that may not transfer
- Calendar and contacts need separate migration (CalDAV/CardDAV)
- Shared mailboxes require special handling

### 2. File Migration (OneDrive/SharePoint)

**Prerequisites:**
- OneDrive sync client or API access
- Sufficient storage on target
- Network bandwidth for transfer

**Process:**
- Use rclone with OneDrive backend
- Maintain folder structure
- Preserve timestamps where possible

**Considerations:**
- SharePoint metadata won't transfer automatically
- Version history is typically lost
- Permissions need to be recreated

### 3. User Migration (Azure AD)

**Prerequisites:**
- Azure AD Connect or API access
- Target identity provider ready

**Process:**
- Export users via PowerShell or Graph API
- Transform to target format (LDIF, JSON)
- Import to new identity provider

**Considerations:**
- Passwords cannot be exported
- MFA settings need reconfiguration
- Group memberships need mapping

## Common Challenges

### API Throttling
Microsoft throttles API calls:
- Plan for slow, steady migration
- Use batch operations where possible
- Consider running migrations off-peak

### Data Volume
Large organizations may have:
- Terabytes of OneDrive/SharePoint data
- Years of email history
- Thousands of users

### Feature Parity
Some M365 features have no direct equivalent:
- Power Automate workflows
- SharePoint lists and forms
- Teams channel history

## Tools and Utilities

### PowerShell for Export
- Azure AD PowerShell module for user export
- Exchange Online PowerShell for mailbox info
- SharePoint Online PowerShell for site inventory

### Graph API
- Programmatic access to most M365 services
- Useful for custom migration scripts
- Requires app registration and permissions

### Third-Party Tools
- BitTitan MigrationWiz (commercial)
- Sharegate (commercial)
- Various open-source scripts on GitHub

## Post-Migration

### DNS Changes
- Update MX records for email
- Update autodiscover records
- Consider keeping hybrid setup temporarily

### User Communication
- Provide new login credentials
- Document changed procedures
- Offer training on new tools

### Validation
- Verify email delivery
- Test file access
- Confirm authentication works

## Cost Considerations

### Subscription Overlap
- May need to maintain M365 during migration
- Consider read-only licenses for archive access

### Data Transfer Costs
- Egress charges from Microsoft
- Bandwidth costs for large transfers

## Timeline Estimates

- Small org (<50 users): 1-2 weeks
- Medium org (50-500 users): 1-2 months  
- Large org (500+ users): 2-6 months

Factors affecting timeline:
- Data volume
- Network speed
- Complexity of setup
- User training needs

## Next Steps

- [Common Concepts](./common-concepts.md) - General migration principles
- [Validation](./validation.md) - Testing procedures