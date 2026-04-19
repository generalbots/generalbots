# Chapter 9: Security

This chapter covers advanced security topics for General Bots.

## Overview

General Bots implements a comprehensive security model designed for enterprise deployments:

- **Multi-tenant Architecture**: Support for multiple organizations with complete data isolation
- **Role-Based Access Control (RBAC)**: Fine-grained permissions at every level
- **Knowledge Base Security**: Folder-level permissions with Qdrant vector search integration
- **SOC 2 Type II Compliance**: Enterprise-grade security controls and audit logging

## Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Organization Layer                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                    Bot Layer                         │    │
│  │  ┌─────────────────────────────────────────────┐    │    │
│  │  │              App Layer                       │    │    │
│  │  │  ┌─────────────────────────────────────┐    │    │    │
│  │  │  │        Resource Layer                │    │    │    │
│  │  │  │  (KB folders, files, data)          │    │    │    │
│  │  │  └─────────────────────────────────────┘    │    │    │
│  │  └─────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Organizations

Organizations are the top-level tenant in General Bots. Each organization has:

- Its own subscription and billing
- Isolated user base and permissions
- Separate bots and knowledge bases
- Independent quota management

Users can belong to multiple organizations and switch between them.

### Roles and Permissions

General Bots uses a role-based model with predefined roles:

| Role | Description |
|------|-------------|
| Global Admin | Full access to all resources |
| Billing Admin | Manage subscriptions and payments |
| User Admin | Manage users, groups, and role assignments |
| Bot Admin | Create and configure bots |
| KB Manager | Manage knowledge bases and permissions |
| App Developer | Create and publish apps (Forms, Sites, Projects) |
| Editor | Edit content and use apps |
| Viewer | Read-only access |

### Knowledge Base Security

KB folders can have individual permission settings:

- **Public**: Anyone can access
- **Authenticated**: Logged-in users only
- **Role-based**: Users with specific roles
- **Group-based**: Users in specific groups
- **User-based**: Named individual users

These permissions are enforced during vector search, ensuring users only see content they're authorized to access.

## In This Chapter

- [RBAC & Security Design](./rbac-design.md) - Complete RBAC architecture and security matrix
- [Organization Multi-Tenancy](./organizations.md) - Multi-organization support and switching
- [Knowledge Base Security](./kb-security.md) - Folder-level permissions and Qdrant integration
- [SOC 2 Compliance](./soc2-compliance.md) - Enterprise compliance controls
- [Security Matrix Reference](./security-matrix.md) - Complete permission reference tables

## Quick Links

- [Authentication & Permissions](../09-security/README.md) - Basic auth setup
- [API Security](../08-rest-api-tools/authentication.md) - API authentication
- [Subscription & Billing](../12-ecosystem-reference/billing.md) - Plan-based access control

## Best Practices

1. **Principle of Least Privilege**: Assign the minimum permissions necessary
2. **Use Groups**: Manage permissions through groups rather than individual users
3. **Regular Audits**: Review permissions and access logs periodically
4. **Secure KB by Default**: Set restrictive default permissions on sensitive folders
5. **Enable Audit Logging**: Track all permission changes and access attempts