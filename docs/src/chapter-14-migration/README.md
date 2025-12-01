# Chapter 14: Migration Guide

Migrate from cloud services to self-hosted General Bots with complete data sovereignty.

## Why Migrate?

| Cloud Services | General Bots |
|----------------|--------------|
| Data on vendor servers | Data on YOUR servers |
| $40-60/user/month | ~$7/user/month |
| Vendor-controlled AI | Transparent, traceable logic |
| Black box processing | Extensible via BASIC |
| Subscription forever | One-time deployment |

## Core Principles

### Component Architecture

Install only what you need:

```bash
./botserver package install mail      # Email
./botserver package install drive     # Storage
./botserver package install directory # Users
./botserver package install meet      # Video
```

### Standard Protocols

- **Storage**: S3 API (MinIO)
- **Email**: SMTP/IMAP/JMAP
- **Auth**: OIDC/SAML/LDAP
- **Video**: WebRTC

### Knowledge Base Integration

```basic
USE KB "company_docs"
USE WEBSITE "https://sharepoint.company.com/docs"
' Documents now searchable via natural language
```

## Migration Timeline

| Phase | Duration | Activities |
|-------|----------|------------|
| **Assessment** | Week 1-2 | Inventory services, identify dependencies |
| **Infrastructure** | Week 2-3 | Deploy BotServer, configure auth/storage |
| **Data Migration** | Week 3-6 | Users, email, files, documents |
| **Process Migration** | Week 6-8 | Convert workflows to .gbdialog |
| **Validation** | Week 8-10 | Testing, training, documentation |
| **Cutover** | Week 10-12 | User migration, decommission old |

## Migration Paths

| Source | Guide |
|--------|-------|
| Microsoft 365 | [M365 Migration](./microsoft-365.md) |
| Google Workspace | [Google Migration](./google-workspace.md) |
| Dialogflow | [Dialogflow Migration](./dialogflow.md) |
| Botpress | [Botpress Migration](./botpress.md) |
| n8n / Zapier / Make | [Automation Migration](./zapier-make.md) |
| Notion | [Notion Migration](./notion.md) |

## Prerequisites Checklist

- [ ] Executive sponsorship
- [ ] Infrastructure provisioned
- [ ] Backup strategy defined
- [ ] Rollback plan documented
- [ ] User communication ready

## Success Metrics

- **Performance**: Response times, availability
- **Adoption**: User login frequency
- **Cost**: TCO reduction (target: 80%+)
- **Security**: Compliance achievement

## See Also

- [Common Concepts](./common-concepts.md) - Shared migration patterns
- [Comparison Matrix](./comparison-matrix.md) - Feature mapping
- [KB Migration](./kb-migration.md) - Document conversion
- [Validation](./validation.md) - Testing procedures