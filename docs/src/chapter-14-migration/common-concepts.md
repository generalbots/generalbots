# Common Migration Concepts

Core concepts for migrating from cloud services to self-hosted infrastructure.

## The Fundamental Shift

### From Cloud Services
- Data on vendor servers
- Monthly subscription costs
- Limited control over updates
- Vendor-specific APIs

### To Self-Hosted
- Data on your infrastructure
- One-time setup costs
- Full control over versions
- Standard protocols

## Component Mapping

| Cloud Service | Self-Hosted Alternative | Protocol |
|---------------|------------------------|----------|
| Cloud Storage | MinIO | S3 API |
| Email Service | Various mail servers | SMTP/IMAP |
| Identity Provider | Various auth servers | OIDC/SAML |

## Migration Stages

1. **Assessment** - What do you have?
2. **Planning** - How will you move it?
3. **Testing** - Does it work?
4. **Execution** - Do the migration
5. **Validation** - Verify everything works

## Common Challenges

### Data Volume
- Large datasets take time
- Bandwidth limitations
- Storage requirements

### Authentication
- Passwords can't be exported
- Need password reset strategy
- Federation options

### Dependencies
- Integrated services
- API changes
- Custom workflows

## Tools Categories

### File Migration
- Cloud storage sync tools
- API-based transfers
- Bulk download utilities

### Email Migration
- IMAP synchronization tools
- Export/import utilities
- Archive formats

### User Migration
- Directory export tools
- CSV/LDIF formats
- API-based extraction

## Risk Mitigation

- Always backup first
- Test with small datasets
- Keep source data intact
- Document everything
- Have rollback plan

## Next Steps

- [Microsoft 365 Migration](./microsoft-365.md) - M365 specific guidance
- [Google Workspace Migration](./google-workspace.md) - Google specific guidance
- [Knowledge Base Migration](./kb-migration.md) - Document migration