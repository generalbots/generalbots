# Knowledge Base Permissions Guide

This guide explains how to configure folder-level permissions in General Bots Knowledge Bases (.gbkb), enabling fine-grained access control that integrates with Qdrant vector search.

## Overview

Knowledge Base permissions allow you to:

- Control access to specific folders within a KB
- Filter search results based on user permissions
- Integrate with RBAC roles and groups
- Support public, authenticated, and restricted content

## Permission File Format

Each .gbkb can include a `kb.permissions.yaml` file at its root:

```yaml
version: 1
default_access: authenticated

folders:
  public:
    access: all
    index_visibility: all
    
  sales:
    access: role_based
    roles: [sales_team, management]
    index_visibility: role_based
    
  hr:
    access: group_based
    groups: [hr_department]
    index_visibility: group_based
    
  executive:
    access: user_based
    users: [ceo@company.com, cfo@company.com]
    index_visibility: user_based
    
  internal:
    access: authenticated
    index_visibility: authenticated

inheritance: true
```

## Configuration Options

### Version

```yaml
version: 1
```

Schema version for forward compatibility. Currently only version 1 is supported.

### Default Access

```yaml
default_access: authenticated
```

Access level for folders without explicit configuration:

| Value | Description |
|-------|-------------|
| `all` | Anyone can access, including anonymous users |
| `authenticated` | Only logged-in users (default) |
| `role_based` | Requires specific roles |
| `group_based` | Requires group membership |
| `user_based` | Specific users only |
| `none` | No access allowed |

### Folder Permissions

Each folder entry supports these options:

```yaml
folders:
  folder_name:
    access: <access_level>
    roles: [role1, role2]           # For role_based access
    groups: [group1, group2]        # For group_based access
    users: [email1, email2, uuid1]  # For user_based access
    index_visibility: <level>       # Search result visibility
    inherit_parent: true|false      # Override inheritance
```

### Access Levels

| Level | Description | Requirements |
|-------|-------------|--------------|
| `all` | Public access | None |
| `authenticated` | Logged-in users | Valid session |
| `role_based` | Role membership | User has any listed role |
| `group_based` | Group membership | User is in any listed group |
| `user_based` | Named users | User ID or email matches |
| `none` | Blocked | No one can access |

### Index Visibility

Controls whether content appears in search results:

```yaml
folders:
  confidential:
    access: role_based
    roles: [management]
    index_visibility: role_based  # Only management sees in search
    
  semi_public:
    access: role_based
    roles: [employees]
    index_visibility: all  # Everyone sees titles, only employees can open
```

Use `index_visibility` to:
- Show content exists without revealing details
- Hide sensitive content from search entirely
- Create "teaser" content that requires authentication

### Inheritance

```yaml
inheritance: true
```

When enabled, subfolders inherit parent permissions unless explicitly configured:

```
documents/
├── public/           # access: all
│   └── guides/       # inherits: all
├── internal/         # access: authenticated
│   ├── policies/     # inherits: authenticated
│   └── hr/           # explicit: group_based (overrides)
└── restricted/       # access: user_based
    └── legal/        # inherits: user_based
```

Disable inheritance per folder:

```yaml
folders:
  parent:
    access: role_based
    roles: [managers]
    
  parent/child:
    access: authenticated
    inherit_parent: false  # Does NOT inherit role_based
```

## Qdrant Integration

When documents are indexed, permission metadata is stored with each vector:

```json
{
  "id": "doc-123",
  "vector": [...],
  "payload": {
    "content": "Document text...",
    "folder": "sales/reports",
    "access_level": "role_based",
    "allowed_roles": ["sales_team", "management"],
    "allowed_groups": [],
    "allowed_users": [],
    "is_public": false,
    "requires_auth": true
  }
}
```

### Search Filtering

When a user searches, the system automatically adds permission filters:

For anonymous users:
```json
{
  "must": [
    { "key": "is_public", "match": { "value": true } }
  ]
}
```

For authenticated users:
```json
{
  "should": [
    { "key": "is_public", "match": { "value": true } },
    { "key": "access_level", "match": { "value": "authenticated" } },
    { "key": "allowed_roles", "match": { "any": ["sales_team"] } },
    { "key": "allowed_groups", "match": { "any": ["sales_department"] } },
    { "key": "allowed_users", "match": { "any": ["user-uuid", "user@email.com"] } }
  ],
  "min_should": { "min_count": 1 }
}
```

## Complete Example

### Directory Structure

```
my-kb.gbkb/
├── kb.permissions.yaml
├── public/
│   ├── faq.md
│   └── getting-started.md
├── products/
│   ├── catalog.md
│   └── pricing.md
├── internal/
│   ├── processes/
│   │   └── onboarding.md
│   └── policies/
│       └── code-of-conduct.md
├── hr/
│   ├── benefits.md
│   └── salary-bands.md
└── executive/
    ├── board-minutes.md
    └── financials.md
```

### Permission Configuration

```yaml
version: 1
default_access: authenticated
inheritance: true

folders:
  public:
    access: all
    index_visibility: all
    
  products:
    access: all
    index_visibility: all
    
  products/pricing:
    access: role_based
    roles: [sales_team, account_managers]
    index_visibility: authenticated
    inherit_parent: false
    
  internal:
    access: authenticated
    index_visibility: authenticated
    
  internal/policies:
    access: authenticated
    index_visibility: all
    
  hr:
    access: group_based
    groups: [hr_department, management]
    index_visibility: group_based
    
  executive:
    access: user_based
    users:
      - ceo@company.com
      - cfo@company.com
      - coo@company.com
    index_visibility: none
```

### Access Matrix

| Folder | Anonymous | Authenticated | Sales | HR | Executive |
|--------|-----------|---------------|-------|-----|-----------|
| public | ✓ | ✓ | ✓ | ✓ | ✓ |
| products | ✓ | ✓ | ✓ | ✓ | ✓ |
| products/pricing | ✗ | ✗ | ✓ | ✗ | ✓ |
| internal | ✗ | ✓ | ✓ | ✓ | ✓ |
| hr | ✗ | ✗ | ✗ | ✓ | ✓ |
| executive | ✗ | ✗ | ✗ | ✗ | ✓ |

## API Usage

### Check Folder Access

```http
GET /api/kb/{kb_id}/folders/{path}/access
Authorization: Bearer <token>
```

Response:

```json
{
  "allowed": true,
  "reason": "Role matched: sales_team",
  "matched_rule": "roles: [sales_team, management]",
  "index_visible": true
}
```

### Get Folder Permissions

```http
GET /api/kb/{kb_id}/folders/{path}/permissions
Authorization: Bearer <token>
```

Response:

```json
{
  "folder": "products/pricing",
  "access": "role_based",
  "roles": ["sales_team", "account_managers"],
  "groups": [],
  "users": [],
  "index_visibility": "authenticated",
  "inherit_parent": false,
  "effective_access": "role_based"
}
```

### Update Folder Permissions

```http
PUT /api/kb/{kb_id}/folders/{path}/permissions
Authorization: Bearer <token>
Content-Type: application/json

{
  "access": "group_based",
  "groups": ["premium_customers"],
  "index_visibility": "all"
}
```

## Best Practices

### 1. Start Restrictive

Default to authenticated access and open up as needed:

```yaml
default_access: authenticated

folders:
  public:
    access: all  # Explicitly mark public content
```

### 2. Use Groups Over Users

Prefer group-based access for easier management:

```yaml
# ✓ Good - easy to maintain
folders:
  hr:
    access: group_based
    groups: [hr_team]

# ✗ Avoid - hard to maintain
folders:
  hr:
    access: user_based
    users: [alice@co.com, bob@co.com, carol@co.com]
```

### 3. Document Sensitive Folders

Add comments explaining access decisions:

```yaml
folders:
  # Financial data - SOC 2 requires strict access
  financials:
    access: user_based
    users: [cfo@company.com, controller@company.com]
    index_visibility: none
```

### 4. Regular Permission Audits

Export and review permissions quarterly:

```http
GET /api/kb/{kb_id}/permissions/export
```

### 5. Test with Different Users

Verify access works correctly:

1. Test anonymous access
2. Test basic authenticated user
3. Test each role/group combination
4. Verify search results match expectations

## Troubleshooting

### Document Not Appearing in Search

1. Check `index_visibility` setting
2. Verify user has required role/group
3. Re-index the document after permission changes

### Access Denied Despite Correct Role

1. Check if folder has `inherit_parent: false`
2. Verify role name matches exactly (case-sensitive)
3. Check for deny rules at parent level

### Inheritance Not Working

1. Confirm `inheritance: true` at root level
2. Check for `inherit_parent: false` on subfolder
3. Verify parent folder has explicit permissions

## Related Topics

- [RBAC Configuration](./rbac-configuration.md)
- [Organization Management](./organizations.md)
- [Security Matrix](./security-matrix.md)