# Google Workspace Migration Guide

Migrating from Google Workspace to self-hosted infrastructure.

## Overview

Google Workspace (formerly G Suite) provides integrated cloud services that need to be migrated to various self-hosted alternatives.

## Service Mapping

| Google Workspace Service | Self-Hosted Alternative | Migration Tool |
|-------------------------|------------------------|----------------|
| Gmail | Mail server (Stalwart, etc.) | imapsync, Got Your Back |
| Google Drive | MinIO or Nextcloud | rclone |
| Google Docs/Sheets/Slides | LibreOffice, OnlyOffice | Export to Office formats |
| Google Calendar | CalDAV server | ical export/import |
| Google Meet | Jitsi, LiveKit | No migration (ephemeral) |
| Google Chat | General Bots | API export |

## Migration Steps

### 1. Email Migration (Gmail)

Before beginning email migration, ensure IMAP is enabled in Google Admin, you have app-specific passwords or OAuth configured, and your target mail server is ready to receive data.

The migration process uses imapsync or Got Your Back (GYB) to transfer mailboxes. Migrate labels as folders since Gmail's labeling system differs from traditional folder structures. Preserve read and unread status to maintain inbox organization.

Consider that Gmail labels don't map perfectly to folders, which may require some reorganization. Some users may have 15 or more years of email history, making this a time-intensive process. Attachments can consume significant storage space on the target system.

### 2. File Migration (Google Drive)

Prerequisites include Google Drive API access, service account or OAuth credentials, and proper storage capacity planning on your target system.

Use rclone with the Google Drive backend for the migration process. Export Google Docs to portable formats since they exist as pointers rather than actual files. Maintain the folder structure during transfer to preserve organizational context.

Keep in mind that Google Docs need format conversion to work offline. Shared drives require separate handling from personal drives. Comments and suggestions on documents are typically lost in the conversion process.

### 3. User Migration (Google Directory)

You'll need Google Admin SDK access and your target identity provider configured before starting.

Export users via the Admin SDK or GAM tool. Transform the exported data to your target format such as LDIF or JSON. Import the transformed data to your new identity management system.

Note that passwords cannot be exported from Google, so all users will need to set new passwords. Two-factor authentication settings need reconfiguration on the new system. Groups and organizational units need mapping to equivalent structures.

## Google-Specific Challenges

### Format Conversion

Google's proprietary formats require conversion to standard formats. Google Docs should be converted to .docx or .odt files. Google Sheets become .xlsx or .ods files. Google Slides convert to .pptx or .odp format. Google Drawings export as .svg or .png images.

### API Quotas

Google enforces strict quotas on API usage. The Drive API allows 1,000 queries per 100 seconds. The Gmail API permits 250 quota units per user per second. The Admin SDK has various limits depending on which specific API you're accessing. Plan your migration to work within these constraints.

### Data Takeout

Google Takeout provides a user-initiated bulk export option that includes most Google services. However, the resulting ZIP files can be enormous, sometimes exceeding 100GB. This approach is not suitable for organization-wide migration but can help individual users verify their data transferred correctly.

## Tools and Utilities

### Google Admin Tools

GAM (Google Apps Manager) provides a command-line interface for managing Google Workspace. GAMADV-XTD is an enhanced version with additional capabilities. The Google Admin console offers manual export options for smaller migrations.

### Got Your Back (GYB)

GYB is a Python-based Gmail backup tool that supports full mailbox export and can restore to different accounts, making it useful for migration scenarios.

### rclone Configuration

rclone supports team drives, handles Google Photos separately from Drive, and can preserve modification times during transfer.

## Common Issues

### Large Attachments

Gmail allows attachments up to 25MB, but some mail servers have lower limits. You may need to store large attachments separately or adjust your target server's configuration.

### Shared Resources

Shared drives need owner reassignment before migration. Calendar sharing must be recreated on the new system. Document collaboration links will break and need updating.

### Google Photos

Google Photos is not part of standard Google Drive storage and needs a separate migration approach. Consider whether you want original quality or compressed versions.

## Migration Strategy

### Phased Approach

Start with a pilot group to identify issues before the broader migration. Migrate inactive users first to reduce impact if problems occur. Schedule department by department to manage support load. Keep Google active during the transition period for rollback capability.

### Hybrid Period

MX records can split email delivery between old and new systems during transition. Users can access both systems simultaneously. Gradual cutover reduces risk compared to a single migration event.

### Data Validation

After migration, compare file counts between source and destination. Verify email folders transferred correctly. Check that user access permissions work as expected.

## Post-Migration

### User Training

Document key differences for users. Explain that real-time collaboration like Google Docs may work differently. Walk through the changed UI and UX in alternative applications. Demonstrate new sharing workflows.

### Feature Gaps

Some features may be lost in migration. Smart Compose in Gmail won't transfer to other mail clients. Google Assistant integration is Google-specific. Automatic photo organization depends on Google's ML systems. Version history in Docs may not fully transfer.

### Maintaining Archive Access

For historical data access, you might keep a reduced Google license for archive purposes, export everything to static storage for reference, or convert all documents to standard formats for long-term preservation.

## Cost Factors

### Google Workspace Pricing

Business Starter costs $6 per user per month. Business Standard costs $12 per user per month. Business Plus costs $18 per user per month. You'll need to maintain these subscriptions during the migration period.

### Data Export Costs

There are no direct egress fees from Google, but API quotas may extend your timeline. Consider bandwidth costs on your receiving infrastructure.

## Timeline Estimates

Migration duration depends on several factors including number of users, data volume per user, available bandwidth, and conversion requirements.

Typical timelines range from 2-3 weeks for small organizations under 50 users, 1-3 months for medium organizations between 50-500 users, and 3-6 months for large organizations with over 500 users.

## Best Practices

Inventory your existing environment first by documenting what you have before starting. Test thoroughly by piloting with your IT team before broader rollout. Communicate often to keep users informed throughout the process. Plan for rollback by having contingency plans if issues arise. Archive everything by keeping backups of original data in case you need to reference it later.

## Next Steps

Review [Common Concepts](./common-concepts.md) for general migration principles. See [Validation](./validation.md) for testing procedures to verify your migration succeeded.