# Migration Resources

General Bots provides comprehensive tools and resources for organizations transitioning from cloud-based productivity platforms to self-hosted infrastructure.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Migration Toolkit

### Data Import Utilities

General Bots includes import tools for common enterprise data formats:

**Email Migration**
- IMAP sync for mailbox migration
- PST file import support
- Calendar (ICS) import
- Contact (VCF/CardDAV) import

**File Migration**
- Bulk file upload via S3 API
- Folder structure preservation
- Metadata retention
- Version history import where available

**User Migration**
- SCIM provisioning support
- LDAP directory sync
- CSV user import
- Bulk credential generation

### BASIC Migration Scripts

Template scripts for common migration tasks:

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

General Bots REST APIs follow familiar patterns:

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

Support SSO during migration with identity federation:

- OIDC provider integration
- SAML support via Zitadel
- Hybrid authentication during transition
- Gradual user migration

Configure in `config.csv`:

```csv
key,value
oidc-provider-url,https://identity.example.com
oidc-client-id,general-bots-client
oidc-client-secret,your-secret
```

## Industry Templates

Pre-built configurations for common industries:

### Healthcare

- HIPAA-compliant configuration
- Patient communication templates
- Appointment scheduling workflows
- Secure document handling

### Financial Services

- SOC 2 aligned settings
- Secure data handling
- Audit logging enabled
- Compliance reporting

### Education

- Student enrollment flows
- Course management
- Parent communication channels
- Assignment tracking

### Professional Services

- Client onboarding templates
- Project management workflows
- Time tracking integration
- Invoice generation

## Deployment Guides

### Infrastructure Sizing

| Organization Size | CPU | RAM | Storage | Users |
|------------------|-----|-----|---------|-------|
| Small | 2 cores | 4 GB | 100 GB | 1-50 |
| Medium | 4 cores | 8 GB | 500 GB | 50-500 |
| Large | 8 cores | 16 GB | 2 TB | 500-5000 |
| Enterprise | 16+ cores | 32+ GB | 10+ TB | 5000+ |

### High Availability

For production deployments:

- PostgreSQL replication
- Load-balanced botserver instances
- Distributed SeaweedFS storage
- Redis/Valkey clustering

### Backup Strategy

Automated backup configuration:

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

- Initial setup and configuration
- User management
- Security settings
- Monitoring and maintenance

### Developer Training

- BASIC scripting fundamentals
- API integration patterns
- Custom keyword development
- Package creation

### End User Training

- Chat interface usage
- File management
- Calendar and tasks
- Mobile access

## ROI Calculator

Estimate savings with self-hosted deployment:

| Factor | Cloud (100 users) | General Bots |
|--------|------------------|--------------|
| Annual licensing | $15,000-60,000 | $0 |
| AI assistant add-on | $36,000 | $0 |
| Infrastructure | Included | $2,400-6,000 |
| LLM API costs | Included | $600-6,000 |
| **Total Annual** | **$51,000-96,000** | **$3,000-12,000** |

Typical savings: 75-95% reduction in annual costs.

## Support Resources

### Documentation

- Complete keyword reference
- API documentation
- Configuration guides
- Troubleshooting guides

### Community

- GitHub discussions
- Issue tracking
- Feature requests
- Community contributions

### Professional Services

For enterprise deployments:

- Migration planning
- Custom development
- Training programs
- Support contracts

Contact: partners@pragmatismo.com.br

## Migration Checklist

### Pre-Migration

- [ ] Inventory current services and usage
- [ ] Identify data to migrate
- [ ] Plan user communication
- [ ] Set up test environment
- [ ] Configure identity federation

### Migration

- [ ] Deploy General Bots infrastructure
- [ ] Import users and groups
- [ ] Migrate files and documents
- [ ] Transfer email (if applicable)
- [ ] Set up integrations

### Post-Migration

- [ ] Verify data integrity
- [ ] Test all workflows
- [ ] Train users
- [ ] Update DNS/routing
- [ ] Decommission old services
- [ ] Monitor and optimize

## Case Study Template

Document your migration for internal reference:

**Organization Profile**
- Size and industry
- Previous platform
- Key requirements

**Migration Scope**
- Services migrated
- Data volume
- Timeline

**Results**
- Cost savings achieved
- Performance improvements
- User feedback

**Lessons Learned**
- Challenges encountered
- Solutions implemented
- Recommendations

## See Also

- [Migration Overview](./overview.md) - Getting started with migration
- [Validation and Testing](./validation.md) - Verify migration success
- [Enterprise Platform Migration](../chapter-11-features/m365-comparison.md) - Feature mapping
- [Quick Start](../chapter-01/quick-start.md) - Initial deployment