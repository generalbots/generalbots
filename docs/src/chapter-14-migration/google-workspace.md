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

**Prerequisites:**
- IMAP enabled in Google Admin
- App-specific passwords or OAuth
- Target mail server ready

**Process:**
- Use imapsync or Got Your Back (GYB)
- Migrate labels as folders
- Preserve read/unread status

**Considerations:**
- Gmail labels don't map perfectly to folders
- Some users may have 15+ years of email
- Attachments can consume significant space

### 2. File Migration (Google Drive)

**Prerequisites:**
- Google Drive API access
- Service account or OAuth credentials
- Storage capacity planning

**Process:**
- Use rclone with Google Drive backend
- Export Google Docs to portable formats
- Maintain folder structure

**Considerations:**
- Google Docs need format conversion
- Shared drives require separate handling
- Comments and suggestions are lost

### 3. User Migration (Google Directory)

**Prerequisites:**
- Google Admin SDK access
- Target identity provider configured

**Process:**
- Export via Admin SDK or GAM tool
- Transform to target format
- Import to new system

**Considerations:**
- No password export possible
- 2FA needs reconfiguration
- Groups and OUs need mapping

## Google-Specific Challenges

### Format Conversion
Google's proprietary formats require conversion:
- Google Docs → .docx or .odt
- Google Sheets → .xlsx or .ods  
- Google Slides → .pptx or .odp
- Google Drawings → .svg or .png

### API Quotas
Google enforces strict quotas:
- Drive API: 1,000 queries per 100 seconds
- Gmail API: 250 quota units per user per second
- Admin SDK: Various limits per API

### Data Takeout
Google Takeout option:
- User-initiated bulk export
- Includes most Google services
- ZIP files can be huge (100GB+)
- Not suitable for organization-wide migration

## Tools and Utilities

### Google Admin Tools
- GAM (Google Apps Manager) - Command-line tool
- GAMADV-XTD - Enhanced GAM version
- Google Admin console for manual exports

### Got Your Back (GYB)
- Python-based Gmail backup tool
- Supports full mailbox export
- Can restore to different account

### rclone Configuration
- Supports team drives
- Handles Google Photos separately
- Can preserve modification times

## Common Issues

### Large Attachments
- Gmail allows 25MB attachments
- Some mail servers have lower limits
- May need to store separately

### Shared Resources
- Shared drives need owner reassignment
- Calendar sharing needs recreation
- Document collaboration links break

### Google Photos
- Not part of standard Google Drive
- Needs separate migration approach
- Original quality vs compressed

## Migration Strategy

### Phased Approach
1. Start with pilot group
2. Migrate inactive users first
3. Schedule department by department
4. Keep Google active during transition

### Hybrid Period
- MX records can split email delivery
- Users can access both systems
- Gradual cutover reduces risk

### Data Validation
- Compare file counts
- Verify email folders
- Check user access

## Post-Migration

### User Training
Key differences to document:
- No real-time collaboration like Google Docs
- Different UI/UX in alternatives
- Changed sharing workflows

### Feature Gaps
Features that may be lost:
- Smart Compose in Gmail
- Google Assistant integration
- Automatic photo organization
- Version history in Docs

### Maintaining Archive Access
Options for historical data:
- Keep reduced Google license for archive
- Export everything to static storage
- Convert to standard formats

## Cost Factors

### Google Workspace Pricing
- Business Starter: $6/user/month
- Business Standard: $12/user/month
- Business Plus: $18/user/month
- Need to maintain during migration

### Data Export Costs
- No direct egress fees
- But API quotas may extend timeline
- Consider bandwidth costs

## Timeline Estimates

Migration duration depends on:
- Number of users
- Data volume per user
- Available bandwidth
- Conversion requirements

Typical timelines:
- Small org (<50 users): 2-3 weeks
- Medium org (50-500 users): 1-3 months
- Large org (500+ users): 3-6 months

## Best Practices

1. **Inventory First**: Document what you have before starting
2. **Test Thoroughly**: Pilot with IT team first
3. **Communicate Often**: Keep users informed
4. **Plan Rollback**: Have contingency plans
5. **Archive Everything**: Keep backups of original data

## Next Steps

- [Common Concepts](./common-concepts.md) - General migration principles
- [Validation](./validation.md) - Testing procedures