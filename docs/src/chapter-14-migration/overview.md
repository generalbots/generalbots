# Migration Overview

Understanding the fundamental shift from cloud services to self-hosted components is crucial for successful enterprise migration to General Bots.

## The Architecture Paradigm Shift

<object data="../assets/migration-architecture.svg" type="image/svg+xml" style="max-height: 600px; width: 100%; display: block; background: transparent;">
  <img src="../assets/migration-architecture.svg" alt="Enterprise Migration Architecture" style="max-height: 600px; width: 100%; object-fit: contain; background: transparent;">
</object>



## Core Migration Principles

### 1. Component-Based Architecture

Unlike monolithic cloud services, General Bots uses discrete, installable components:

```bash
# Install individual components as needed
./botserver package install mail      # Email service
./botserver package install drive     # File storage
./botserver package install directory # User management
./botserver package install meet      # Video conferencing
```

### 2. Data Sovereignty

Your data stays under your control:
- **On-premises**: Physical servers in your data center
- **Private cloud**: Your own cloud infrastructure
- **Hybrid**: Mix of local and controlled cloud resources

### 3. Standard Protocols

All components use industry-standard protocols:
- **Storage**: S3 API (MinIO)
- **Email**: SMTP/IMAP/JMAP (Stalwart)
- **Auth**: OIDC/SAML/LDAP (Zitadel)
- **Video**: WebRTC (LiveKit)

### 4. Knowledge Base Integration

Transform static documents into searchable knowledge:

```basic
' Convert SharePoint documents to searchable KB
USE KB "company_docs"
USE WEBSITE "https://sharepoint.company.com/docs"

' Now accessible via natural language
question = HEAR "What would you like to know?"
FIND question
answer = LLM "Based on the search results, provide a helpful answer"
TALK answer
```

## Migration Phases

### Phase 1: Assessment (Week 1-2)
- Inventory current services and usage
- Identify dependencies and integrations
- Size infrastructure requirements
- Create migration timeline

### Phase 2: Infrastructure Setup (Week 2-3)
- Deploy General Bots instance
- Install required components
- Configure authentication (Zitadel)
- Setup storage (MinIO)

### Phase 3: Data Migration (Week 3-6)
- User accounts and permissions
- Email and calendars
- Files and documents
- Knowledge base content

### Phase 4: Process Migration (Week 6-8)
- Convert workflows to .gbdialog scripts
- Setup automation rules
- Configure integrations
- Train AI models on your data

### Phase 5: Validation & Training (Week 8-10)
- Test all migrated services
- User acceptance testing
- Training sessions
- Documentation update

### Phase 6: Cutover (Week 10-12)
- Gradual user migration
- Monitor and support
- Decommission old services
- Post-migration optimization

## Cost Comparison

### Enterprise Cloud Services
```
Microsoft 365 E3: $36/user/month
Google Workspace: $12/user/month
+ API costs
+ Storage overages
+ Add-on features
= $40-60/user/month typical
```

### General Bots Self-Hosted
```
Infrastructure: ~$500/month (100 users)
Maintenance: ~$200/month
= $7/user/month
Savings: 85%+
```

## Key Differentiators

### Mega-Prompts vs Components

| Copilot/Gemini (Mega-Prompts) | General Bots (Components) |
|-------------------------------|---------------------------|
| Black box AI responses | Transparent, traceable logic |
| Cloud processing required | Local or hybrid processing |
| Fixed capabilities | Extensible via .gbdialog |
| Subscription model | One-time deployment |
| Vendor-controlled updates | You control updates |

### Data Control

| Cloud Services | General Bots |
|---------------|--------------|
| Data on vendor servers | Data on your servers |
| Vendor terms apply | Your policies only |
| Potential AI training use | No external AI training |
| Compliance uncertainty | Full compliance control |

## Success Metrics

Monitor these KPIs during and after migration:

- **Performance**: Response times, system availability
- **Adoption**: User login frequency, feature usage
- **Cost**: Total cost of ownership reduction
- **Security**: Incident reduction, audit compliance
- **Productivity**: Task completion times

## Prerequisites Checklist

Before starting migration:

- [ ] Executive sponsorship secured
- [ ] Migration team assembled
- [ ] Infrastructure provisioned
- [ ] Backup strategy defined
- [ ] Rollback plan documented
- [ ] User communication plan ready
- [ ] Training materials prepared

## Next Steps

1. Review [Common Migration Concepts](./common-concepts.md) for shared tools and patterns
2. Choose your migration path:
   - [Microsoft 365 Migration](./microsoft-365.md)
   - [Google Workspace Migration](./google-workspace.md)
3. Explore [Automation Scripts](./automation.md) for streamlined migration
4. Plan [Knowledge Base Migration](./kb-migration.md) for document conversion

The journey from cloud dependency to self-hosted freedom starts with understanding these core concepts. General Bots provides not just an alternative, but a fundamentally better approach to enterprise computing.