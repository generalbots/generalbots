# Migration Resources

General Bots provides comprehensive tools and resources for organizations transitioning from cloud-based productivity platforms to self-hosted infrastructure.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Migration Toolkit

### Data Import Utilities

General Bots includes import tools for common enterprise data formats.

For email migration, the toolkit supports IMAP sync for mailbox migration, PST file import, calendar import via ICS format, and contact import through VCF and CardDAV.

File migration capabilities include bulk file upload via the S3 API, folder structure preservation, metadata retention, and version history import where the source system provides it.

User migration supports SCIM provisioning, LDAP directory sync, CSV user import, and bulk credential generation.

### BASIC Migration Scripts

Template scripts simplify common migration tasks. The file migration script connects to external storage and transfers files:

```basic
' migrate-files.bas
PARAM source_api AS string
PARAM auth_token AS string

DESCRIPTION "Migrate files from external storage"

SET HEADER "Authorization", "Bearer " + auth_token
files = GET source_api + "/files"

FOR EACH file IN files
    content = DOWNLOAD file.download_url
    WRITE "/" + file.path, content
    TALK "Migrated: " + file.name
NEXT file

TALK "Migration complete: " + LEN(files) + " files"
```

The user migration script imports users from a CSV export:

```basic
' migrate-users.bas
PARAM csv_path AS string

DESCRIPTION "Import users from CSV export"

users = READ csv_path
FOR EACH row IN users
    CREATE USER row.email WITH NAME row.name
NEXT row
```

## API Compatibility

### REST API Mapping

General Bots REST APIs follow familiar patterns that map to common operations:

| Common Operation | General Bots Endpoint |
|-----------------|----------------------|
| List files | `GET /api/files/list` |
| Upload file | `POST /api/files/write` |
| Download file | `GET /api/files/{path}` |
| List users | `GET /api/users` |
| Create user | `POST /api/users` |
| Send email | `POST /api/email/send` |
| List calendar events | `GET /api/calendar/events` |
| Create task | `POST /api/tasks` |

### Identity Federation

Support SSO during migration with identity federation. This enables OIDC provider integration, SAML support via Zitadel, hybrid authentication during transition periods, and gradual user migration without disrupting access.

Configure federation in `config.csv`:

```csv
key,value
oidc-provider-url,https://identity.example.com
oidc-client-id,general-bots-client
oidc-client-secret,your-secret
```

## Industry Templates

Pre-built configurations address common industry requirements.

Healthcare templates provide HIPAA-compliant configuration, patient communication templates, appointment scheduling workflows, and secure document handling.

Financial services templates include SOC 2 aligned settings, secure data handling, audit logging enabled by default, and compliance reporting.

Education templates offer student enrollment flows, course management, parent communication channels, and assignment tracking.

Professional services templates cover client onboarding, project management workflows, time tracking integration, and invoice generation.

## Deployment Guides

### Infrastructure Sizing

| Organization Size | CPU | RAM | Storage | Users |
|------------------|-----|-----|---------|-------|
| Small | 2 cores | 4 GB | 100 GB | 1-50 |
| Medium | 4 cores | 8 GB | 500 GB | 50-500 |
| Large | 8 cores | 16 GB | 2 TB | 500-5000 |
| Enterprise | 16+ cores | 32+ GB | 10+ TB | 5000+ |

### High Availability

For production deployments requiring high availability, configure PostgreSQL replication for database resilience, load-balanced botserver instances for horizontal scaling, distributed SeaweedFS storage for file redundancy, and Redis/Valkey clustering for cache availability.

### Backup Strategy

Configure automated backups to protect your data:

```basic
SET SCHEDULE "every day at 2am"

' Database backup
result = POST "https://backup.internal/postgres", #{database: "botserver"}

' File storage backup
result = POST "https://backup.internal/seaweedfs", #{bucket: "all"}

' Notify on completion
SEND MAIL TO "ops@company.com" SUBJECT "Backup Complete" BODY result
```

## Training Resources

### Administrator Training

Administrator training covers initial setup and configuration, user management, security settings, and monitoring and maintenance procedures.

### Developer Training

Developer training includes BASIC scripting fundamentals, API integration patterns, custom keyword development, and package creation.

### End User Training

End user training addresses chat interface usage, file management, calendar and tasks, and mobile access.

## ROI Calculator

Estimate savings with self-hosted deployment:

| Factor | Cloud (100 users) | General Bots |
|--------|------------------|--------------|
| Annual licensing | $15,000-60,000 | $0 |
| AI assistant add-on | $36,000 | $0 |
| Infrastructure | Included | $2,400-6,000 |
| LLM API costs | Included | $600-6,000 |
| **Total Annual** | **$51,000-96,000** | **$3,000-12,000** |

Typical savings range from 75-95% reduction in annual costs.

## Support Resources

### Documentation

Documentation resources include the complete keyword reference, API documentation, configuration guides, and troubleshooting guides.

### Community

Community support is available through GitHub discussions, issue tracking, feature requests, and community contributions.

### Professional Services

For enterprise deployments requiring additional support, professional services include migration planning, custom development, training programs, and support contracts.

Contact: partners@pragmatismo.com.br

## Migration Checklist

### Pre-Migration

Before beginning migration, inventory current services and usage, identify data to migrate, plan user communication, set up a test environment, and configure identity federation if needed.

### Migration

During migration, deploy General Bots infrastructure, import users and groups, migrate files and documents, transfer email if applicable, and set up integrations.

### Post-Migration

After migration, verify data integrity, test all workflows, train users, update DNS and routing, decommission old services, and monitor and optimize the new environment.

## Case Study Template

Document your migration for internal reference using this structure.

The organization profile section captures size and industry, previous platform, and key requirements.

The migration scope section documents services migrated, data volume, and timeline.

The results section records cost savings achieved, performance improvements, and user feedback.

The lessons learned section captures challenges encountered, solutions implemented, and recommendations for future migrations.

## See Also

Review the [Migration Overview](./overview.md) for getting started with migration concepts. See [Validation and Testing](./validation.md) to verify migration success. The [Enterprise Platform Migration](../chapter-11-features/m365-comparison.md) guide provides detailed feature mapping. Start with the [Quick Start](../chapter-01/quick-start.md) guide for initial deployment.