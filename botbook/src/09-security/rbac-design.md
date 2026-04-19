# RBAC & Security Design

This document describes the Role-Based Access Control (RBAC) system and security architecture for General Bots.

## Overview

General Bots implements a comprehensive RBAC system that controls access at multiple levels:

1. **Organization Level** - Who can access the organization
2. **Bot Level** - Who can use specific bots
3. **App Level** - Who can use apps within a bot (Forms, Sites, Projects)
4. **Knowledge Base Level** - Who can access specific KB folders and documents
5. **Resource Level** - Granular permissions on individual resources

## Security Matrix

### Organization Permissions

| Permission | Global Admin | Billing Admin | User Admin | Bot Admin | KB Manager | Editor | Viewer |
|------------|:------------:|:-------------:|:----------:|:---------:|:----------:|:------:|:------:|
| Create Organization | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Delete Organization | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Manage Settings | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| View Billing | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Manage Billing | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Invite Members | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Remove Members | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Manage Roles | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |

### Bot Permissions

| Permission | Global Admin | Bot Admin | KB Manager | Editor | Viewer | Guest |
|------------|:------------:|:---------:|:----------:|:------:|:------:|:-----:|
| Create Bot | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Delete Bot | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Configure Bot | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| View Bot Settings | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Use Bot Chat | ✅ | ✅ | ✅ | ✅ | ✅ | ✅* |
| View Analytics | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Export Analytics | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |

*Guest access limited by plan quotas

### Knowledge Base Permissions

| Permission | Global Admin | Bot Admin | KB Manager | Editor | Viewer |
|------------|:------------:|:---------:|:----------:|:------:|:------:|
| Create KB | ✅ | ✅ | ✅ | ❌ | ❌ |
| Delete KB | ✅ | ✅ | ✅ | ❌ | ❌ |
| Create Folders | ✅ | ✅ | ✅ | ✅ | ❌ |
| Upload Files | ✅ | ✅ | ✅ | ✅ | ❌ |
| Edit Files | ✅ | ✅ | ✅ | ✅ | ❌ |
| Delete Files | ✅ | ✅ | ✅ | ❌ | ❌ |
| View Files | ✅ | ✅ | ✅ | ✅ | ✅ |
| Manage Permissions | ✅ | ✅ | ✅ | ❌ | ❌ |
| Re-index Content | ✅ | ✅ | ✅ | ❌ | ❌ |

### App Permissions (Forms, Sites, Projects)

| Permission | Global Admin | Bot Admin | App Developer | Editor | Viewer |
|------------|:------------:|:---------:|:-------------:|:------:|:------:|
| Create App | ✅ | ✅ | ✅ | ❌ | ❌ |
| Delete App | ✅ | ✅ | ✅ | ❌ | ❌ |
| Edit App | ✅ | ✅ | ✅ | ✅ | ❌ |
| Publish App | ✅ | ✅ | ✅ | ❌ | ❌ |
| Use App | ✅ | ✅ | ✅ | ✅ | ✅ |
| View Submissions | ✅ | ✅ | ✅ | ✅ | ❌ |
| Export Data | ✅ | ✅ | ✅ | ❌ | ❌ |

## Organization Multi-Tenancy

### User-Organization Relationship

Users can belong to multiple organizations, similar to Azure Active Directory tenants:

```
┌─────────────────────────────────────────────────────────────┐
│                        User: john@example.com                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Org A     │  │   Org B     │  │   Org C     │         │
│  │  (Admin)    │  │  (Editor)   │  │  (Viewer)   │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### Organization Switcher

The avatar menu includes an organization switcher:

1. Current organization indicator
2. List of user's organizations
3. Option to create new organization
4. Organization settings access (for admins)

## Knowledge Base Folder Security

### Permission File Format

Each `.gbkb` package can contain a `kb.permissions.yaml` file:

```yaml
version: 1
default_access: authenticated

folders:
  public:
    access: all
    description: "Publicly accessible content"
    index_visibility: all
    
  internal:
    access: role_based
    roles:
      - employee
      - contractor
    index_visibility: role_based
    
  hr-policies:
    access: group_based
    groups:
      - hr_department
      - management
    index_visibility: group_based
    
  executive:
    access: user_based
    users:
      - ceo@company.com
      - cfo@company.com
      - coo@company.com
    index_visibility: user_based

inheritance: true
```

### Folder Permission Levels

| Access Level | Description | Who Can Access |
|--------------|-------------|----------------|
| `all` | Public access | Anyone, including anonymous |
| `authenticated` | Logged in users | Any authenticated user |
| `role_based` | By role | Users with specified roles |
| `group_based` | By group | Users in specified groups |
| `user_based` | By user | Specifically named users |

### Permission Inheritance

When `inheritance: true`, subfolders inherit parent permissions unless explicitly overridden:

```
hr-policies/           (groups: hr_department, management)
├── onboarding/        (inherits parent)
├── compensation/      (groups: hr_department only - more restrictive)
└── public-handbook/   (access: authenticated - less restrictive)
```

## Qdrant Vector Integration

### Permission Metadata in Vectors

When indexing KB content, permission metadata is stored with each vector:

```json
{
  "id": "doc-uuid",
  "vector": [...],
  "payload": {
    "content": "Document text...",
    "source": "hr-policies/compensation/salary-bands.md",
    "folder": "hr-policies/compensation",
    "access_level": "group_based",
    "allowed_groups": ["hr_department"],
    "allowed_roles": [],
    "allowed_users": [],
    "indexed_at": "2025-01-21T10:00:00Z"
  }
}
```

### Search Query Filtering

When a user searches, their permissions are used to filter results:

```rust
// Pseudo-code for permission filtering
fn build_search_filter(user: &User) -> Filter {
    Filter::should([
        // Public content
        Filter::must([
            FieldCondition::match_value("access_level", "all")
        ]),
        // Authenticated content
        Filter::must([
            FieldCondition::match_value("access_level", "authenticated")
        ]),
        // Role-based content
        Filter::must([
            FieldCondition::match_value("access_level", "role_based"),
            FieldCondition::match_any("allowed_roles", user.roles)
        ]),
        // Group-based content  
        Filter::must([
            FieldCondition::match_value("access_level", "group_based"),
            FieldCondition::match_any("allowed_groups", user.groups)
        ]),
        // User-based content
        Filter::must([
            FieldCondition::match_value("access_level", "user_based"),
            FieldCondition::match_value("allowed_users", user.email)
        ])
    ])
}
```

## Bot-Level Access Control

### Bot Access Configuration

Each bot has access control settings:

```yaml
bot_access:
  id: "bot-uuid"
  name: "HR Assistant"
  access_type: "groups"  # organization | groups | users | public
  
  # For groups access type
  allowed_groups:
    - hr_department
    - management
    
  # For users access type  
  allowed_users:
    - specific.user@company.com
    
  # Public bots
  public: false
  anonymous_allowed: false
```

### App-Level Access (1 Bot : N Apps)

A bot can have multiple apps, each with its own permissions:

```yaml
apps:
  - id: "app-uuid-1"
    name: "Leave Request Form"
    type: "form"
    bot_id: "bot-uuid"
    access:
      type: "inherit"  # Inherits from bot
      
  - id: "app-uuid-2"  
    name: "Salary Calculator"
    type: "form"
    bot_id: "bot-uuid"
    access:
      type: "custom"
      allowed_groups:
        - hr_department  # More restrictive than bot
        
  - id: "app-uuid-3"
    name: "Company Directory"
    type: "site"
    bot_id: "bot-uuid"
    access:
      type: "custom"
      allowed_groups:
        - all_users  # Less restrictive - all employees
```

## Default Roles and Groups

### Default Roles (Microsoft 365-like)

| Role | Description |
|------|-------------|
| `global_admin` | Full access to everything |
| `billing_admin` | Manage subscriptions and payments |
| `user_admin` | Manage users and groups |
| `bot_admin` | Create and manage bots |
| `kb_manager` | Manage knowledge bases |
| `app_developer` | Create and manage apps |
| `support_agent` | Handle support conversations |
| `viewer` | Read-only access |

### Default Groups

| Group | Description |
|-------|-------------|
| `all_users` | All organization members |
| `guests` | External guests with limited access |
| `external_users` | External collaborators |

### Default Permission Assignments

```yaml
role_permissions:
  global_admin:
    - "*"  # All permissions
    
  billing_admin:
    - "billing.view"
    - "billing.manage"
    - "subscription.view"
    - "subscription.manage"
    
  user_admin:
    - "users.view"
    - "users.create"
    - "users.update"
    - "users.delete"
    - "groups.view"
    - "groups.create"
    - "groups.update"
    - "groups.delete"
    - "roles.assign"
    
  bot_admin:
    - "bots.view"
    - "bots.create"
    - "bots.update"
    - "bots.delete"
    - "kb.view"
    - "kb.create"
    - "kb.update"
    - "kb.delete"
    
  kb_manager:
    - "kb.view"
    - "kb.create"
    - "kb.update"
    - "kb.index"
    - "kb.permissions"
    
  app_developer:
    - "apps.view"
    - "apps.create"
    - "apps.update"
    - "apps.delete"
    - "apps.publish"
    
  support_agent:
    - "bots.use"
    - "conversations.view"
    - "conversations.respond"
    
  viewer:
    - "bots.use"
    - "kb.view"
    - "apps.use"
    - "analytics.view"
```

## Audit Logging

All permission-related actions are logged:

```json
{
  "timestamp": "2025-01-21T10:30:00Z",
  "event_type": "permission_change",
  "actor": "admin@company.com",
  "action": "grant_role",
  "target_user": "employee@company.com",
  "role": "kb_manager",
  "organization_id": "org-uuid",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0..."
}
```

## Implementation Checklist

- [ ] Organization multi-tenancy tables
- [ ] Organization switcher UI component
- [ ] Role and permission tables
- [ ] Default roles/groups seeding
- [ ] KB permission file parser
- [ ] Qdrant metadata enrichment during indexing
- [ ] Qdrant search filter by permissions
- [ ] Bot access control
- [ ] App access control
- [ ] Audit logging
- [ ] Permission sync job for KB updates

## Related Documentation

- [Subscription & Billing](../12-ecosystem-reference/billing.md)
- [SOC 2 Compliance](./soc2-compliance.md)
- [API Authentication](../09-security/README.md)