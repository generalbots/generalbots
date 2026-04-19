# RBAC Configuration Guide

This guide covers how to configure Role-Based Access Control (RBAC) in General Bots, including role management, group setup, permission assignment, and best practices for enterprise deployments.

## Overview

General Bots RBAC provides:

- **Hierarchical Roles**: Roles inherit permissions from parent roles
- **Group-based Access**: Organize users into groups for easier management
- **Permission Inheritance**: Permissions flow down through the hierarchy
- **Resource-level Control**: Fine-grained access to bots, apps, and knowledge bases

## Default Roles

General Bots includes predefined system roles that cannot be deleted:

| Role | Hierarchy Level | Description |
|------|----------------|-------------|
| Owner | 100 | Full organization control, including deletion |
| Admin | 90 | Manage all resources except organization deletion |
| Manager | 70 | Create and manage bots, KB, and apps |
| Member | 50 | Standard access to organization resources |
| Viewer | 30 | Read-only access to bots and knowledge bases |
| Guest | 10 | Limited public access only |

### Role Hierarchy

Higher-level roles automatically inherit permissions from lower-level roles:

```
Owner (100)
  └── Admin (90)
        └── Manager (70)
              └── Member (50)
                    └── Viewer (30)
                          └── Guest (10)
```

An Admin can manage any role below them (Manager, Member, Viewer, Guest) but cannot modify Owner permissions.

## Configuring Roles

### Creating a Custom Role

Custom roles can extend the default hierarchy:

1. Navigate to **Settings** → **Access Control** → **Roles**
2. Click **Create Role**
3. Configure:
   - **Name**: Internal identifier (lowercase, no spaces)
   - **Display Name**: Human-readable name
   - **Hierarchy Level**: Position in hierarchy (1-99)
   - **Parent Roles**: Roles to inherit from
   - **Permissions**: Additional permissions

### Permission Format

Permissions use a colon-separated format:

```
resource:action
resource:action:scope
```

Examples:

| Permission | Description |
|------------|-------------|
| `bot:create` | Create new bots |
| `bot:view` | View bot details |
| `bot:edit` | Edit bot configuration |
| `bot:delete` | Delete bots |
| `bot:*` | All bot permissions |
| `kb:read` | Read knowledge base content |
| `kb:write` | Write to knowledge bases |
| `kb:admin` | Administer KB settings |
| `app:create` | Create apps (Forms, Sites) |
| `app:view` | View apps |
| `app:edit` | Edit apps |
| `org:manage` | Manage organization settings |
| `org:billing` | Access billing information |
| `org:members` | Manage organization members |
| `*` | Wildcard - all permissions |

### Role Configuration Example

```yaml
name: content_editor
display_name: Content Editor
hierarchy_level: 55
parent_roles:
  - member
permissions:
  - kb:read
  - kb:write
  - bot:view
  - app:view
  - app:edit
```

## Configuring Groups

Groups provide an additional layer of organization for users.

### Default Groups

| Group | Description |
|-------|-------------|
| everyone | All authenticated users |
| developers | Users who create bots and apps |
| content_managers | Users who manage knowledge bases |
| support | Support team with analytics access |

### Creating Groups

1. Navigate to **Settings** → **Access Control** → **Groups**
2. Click **Create Group**
3. Configure:
   - **Name**: Internal identifier
   - **Display Name**: Human-readable name
   - **Parent Group**: Optional hierarchy
   - **Permissions**: Group-specific permissions

### Group Hierarchy

Groups can have parent-child relationships:

```
everyone
├── developers
│     └── senior_developers
├── content_managers
│     └── kb_admins
└── support
      └── tier2_support
```

Child groups inherit permissions from parent groups.

## Assigning Permissions

### To Users

Assign roles directly to users:

```
User: john@company.com
Roles: manager, content_editor
Groups: developers, content_managers
```

### To Bots

Control who can access each bot:

```yaml
bot_id: my-support-bot
visibility: organization
allowed_roles:
  - member
  - viewer
allowed_groups:
  - support
denied_users: []
```

### To Apps

Control app access (Forms, Sites, Dashboards):

```yaml
app_id: customer-feedback-form
app_type: form
visibility: public
allowed_roles: []
submission_requires_auth: false
```

### To Knowledge Base Folders

See [KB Permissions Guide](./kb-permissions.md) for detailed folder configuration.

## Permission Inheritance Resolution

When checking if a user has permission, the system evaluates:

1. **Direct User Permissions**: Explicitly assigned to the user
2. **Role Permissions**: From all assigned roles (including inherited)
3. **Group Permissions**: From all group memberships (including inherited)

### Resolution Example

```
User: alice@company.com
Direct Permissions: analytics:export
Roles: manager (inherits from member, viewer)
Groups: content_managers (inherits from everyone)

Effective Permissions:
├── analytics:export (direct)
├── org:members:view (from manager)
├── bot:create (from manager)
├── bot:edit (from manager)
├── bot:view (from member, inherited by manager)
├── kb:read (from member)
├── kb:write (from content_managers)
├── kb:admin (from content_managers)
└── basic:access (from everyone)
```

## Wildcard Permissions

Use wildcards for broad access:

| Pattern | Matches |
|---------|---------|
| `*` | All permissions |
| `bot:*` | All bot permissions |
| `kb:*` | All knowledge base permissions |
| `app:*` | All app permissions |
| `org:*` | All organization permissions |

## Configuration via API

### List Roles

```http
GET /api/settings/rbac/roles
Authorization: Bearer <token>
```

### Create Role

```http
POST /api/settings/rbac/roles
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "custom_role",
  "display_name": "Custom Role",
  "description": "A custom role for specific needs",
  "hierarchy_level": 45,
  "parent_roles": ["member"],
  "permissions": ["kb:read", "kb:write"]
}
```

### Assign Role to User

```http
POST /api/settings/rbac/users/{user_id}/roles/{role_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "expires_at": "2025-12-31T23:59:59Z"
}
```

### Add User to Group

```http
POST /api/settings/rbac/users/{user_id}/groups/{group_id}
Authorization: Bearer <token>
```

### Get Effective Permissions

```http
GET /api/settings/rbac/users/{user_id}/permissions
Authorization: Bearer <token>
```

Response:

```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "permissions": [
    "bot:view",
    "bot:create",
    "kb:read",
    "kb:write"
  ],
  "sources": [
    {
      "permission": "bot:view",
      "source_type": "role",
      "source_name": "member"
    },
    {
      "permission": "kb:write",
      "source_type": "group",
      "source_name": "content_managers"
    }
  ]
}
```

## Audit Logging

All permission changes are logged for compliance:

| Event | Logged Data |
|-------|-------------|
| Role Assignment | Actor, target user, role, timestamp |
| Role Revocation | Actor, target user, role, timestamp |
| Group Addition | Actor, target user, group, timestamp |
| Group Removal | Actor, target user, group, timestamp |
| Permission Grant | Actor, target, permission, timestamp |
| Access Denied | Actor, resource, required permission |

Access audit logs at **Settings** → **Security** → **Audit Log**.

## Best Practices

### 1. Use Groups Over Direct Assignment

Instead of assigning roles to individual users, create groups:

```
✓ Create "Sales Team" group with viewer + CRM permissions
✓ Add users to the group
✗ Assign roles individually to 50 users
```

### 2. Follow Least Privilege

Start with minimal permissions and add as needed:

```
✓ New users get "viewer" role by default
✓ Promote to "member" after onboarding
✗ Give everyone "admin" for convenience
```

### 3. Use Role Expiration

For temporary access, set expiration dates:

```http
POST /api/settings/rbac/users/{user_id}/roles/{role_id}
{
  "expires_at": "2025-03-01T00:00:00Z"
}
```

### 4. Regular Permission Reviews

Schedule quarterly reviews:

1. Export current permissions
2. Review access patterns in audit logs
3. Remove unused permissions
4. Update role definitions as needed

### 5. Document Custom Roles

Maintain documentation for custom roles:

```markdown
## Custom Role: Project Lead

**Purpose**: Lead project teams with limited admin access

**Permissions**:
- All member permissions
- bot:create, bot:edit
- app:create, app:edit
- org:members:view

**Assigned To**: Project leads and tech leads
**Created**: 2025-01-15
**Last Review**: 2025-01-21
```

## Troubleshooting

### User Cannot Access Resource

1. Check user's effective permissions:
   ```http
   GET /api/settings/rbac/users/{user_id}/permissions
   ```

2. Verify resource permissions:
   ```http
   GET /api/bots/{bot_id}/access
   ```

3. Check audit log for denied access attempts

### Permission Not Working After Assignment

1. Clear user's session cache
2. User may need to log out and back in
3. Check if permission is blocked by a deny rule

### Role Changes Not Reflected

1. Permission cache has 5-minute TTL
2. Force refresh: User logs out/in
3. Check if role assignment has expiration date

## Related Topics

- [Security Matrix Reference](./security-matrix.md)
- [KB Permissions Guide](./kb-permissions.md)
- [Organization Management](./organizations.md)
- [SOC 2 Compliance](./soc2-compliance.md)