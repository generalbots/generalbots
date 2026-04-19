# Role-Based Access Control (RBAC) Overview

General Bots implements a comprehensive **Role-Based Access Control (RBAC)** system designed as a secure, enterprise-grade alternative to Microsoft 365 / Google Workspace permission models. This system provides fine-grained access control across all suite applications.

## Why RBAC?

RBAC is the industry standard for enterprise access control, used by:
- Microsoft Azure Active Directory
- Google Workspace Admin
- AWS IAM
- Kubernetes
- All major enterprise platforms

### Benefits Over Direct Permissions

| Approach | Pros | Cons |
|----------|------|------|
| **Direct User Permissions** | Simple for small teams | Unmanageable at scale, audit nightmare |
| **RBAC (Roles)** | Scalable, auditable, principle of least privilege | Initial setup complexity |
| **RBAC + Groups** | Best of both worlds, mirrors org structure | Requires planning |

## Core Concepts

### 1. Users
Individual accounts that authenticate to the system. Users can be:
- **Internal employees** - Full organization members
- **External guests** - Partners, contractors, clients
- **Service accounts** - For API integrations

### 2. Roles
Named collections of permissions. Roles define *what actions* can be performed.

```
┌─────────────────────────────────────────────────────────┐
│                     ROLE: Standard User                  │
├─────────────────────────────────────────────────────────┤
│ Permissions:                                             │
│   ✓ mail.read, mail.send                                │
│   ✓ calendar.read, calendar.write                       │
│   ✓ drive.read, drive.write, drive.share                │
│   ✓ docs.read, docs.write, docs.collaborate             │
│   ✓ meet.join, meet.create                              │
│   ✓ chat.read, chat.write                               │
│   ✓ tasks.read, tasks.write                             │
│   ✗ users.manage (NOT included)                         │
│   ✗ settings.organization (NOT included)                │
└─────────────────────────────────────────────────────────┘
```

### 3. Groups
Collections of users, typically mirroring organizational structure:
- Departments (IT, HR, Finance, Sales)
- Teams (Project Alpha, Support Team)
- Access levels (Managers, External Contractors)

### 4. Permissions
Granular capabilities following the pattern: `resource.action`

```
mail.read          → Can read emails
mail.send          → Can send emails  
mail.admin         → Full mail administration

drive.read         → Can view files
drive.write        → Can upload/edit files
drive.share        → Can share with others
drive.share_external → Can share outside organization
drive.admin        → Full drive administration
```

## Permission Inheritance

```
                    ┌──────────────┐
                    │  Permission  │
                    │  mail.send   │
                    └──────┬───────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │    Role     │          │    Role     │
       │ Standard    │          │   Guest     │
       │   User      │          │   User      │
       └──────┬──────┘          └─────────────┘
              │                        
    ┌─────────┴─────────┐              
    │                   │              
    ▼                   ▼              
┌─────────┐      ┌─────────────┐      
│  Group  │      │   Direct    │      
│  Sales  │      │ Assignment  │      
│  Team   │      │             │      
└────┬────┘      └──────┬──────┘      
     │                  │              
     ▼                  ▼              
┌─────────┐      ┌─────────────┐      
│  User   │      │    User     │      
│  Alice  │      │    Bob      │      
└─────────┘      └─────────────┘      
```

Users inherit permissions from:
1. **Direct role assignments** - Roles assigned directly to the user
2. **Group memberships** - Roles assigned to groups the user belongs to

## Comparison with Office 365 / Google Workspace

### Office 365 Equivalent Roles

| General Bots Role | Office 365 Equivalent |
|-------------------|----------------------|
| `global_admin` | Global Administrator |
| `billing_admin` | Billing Administrator |
| `user_admin` | User Administrator |
| `exchange_admin` | Exchange Administrator |
| `sharepoint_admin` | SharePoint Administrator |
| `teams_admin` | Teams Administrator |
| `security_admin` | Security Administrator |
| `compliance_admin` | Compliance Administrator |
| `helpdesk_admin` | Helpdesk Administrator |
| `reports_reader` | Reports Reader |

### Google Workspace Equivalent Roles

| General Bots Role | Google Workspace Equivalent |
|-------------------|----------------------------|
| `global_admin` | Super Admin |
| `user_admin` | User Management Admin |
| `groups_admin` | Groups Admin |
| `sharepoint_admin` | Drive & Docs Admin |
| `exchange_admin` | Gmail Admin |
| `teams_admin` | Meet & Chat Admin |

## Built-in System Roles

### Administrative Roles

| Role | Description | Typical Use |
|------|-------------|-------------|
| **Global Administrator** | Full system control | IT Director, CTO |
| **Billing Administrator** | Subscription & payments | Finance team |
| **Compliance Administrator** | Audit, DLP, retention | Legal, Compliance |
| **Security Administrator** | Threats, access policies | Security team |
| **User Administrator** | User & group management | HR, IT Helpdesk |
| **Groups Administrator** | Group management only | Team leads |
| **Helpdesk Administrator** | Password resets, support | IT Support |

### Service-Specific Admin Roles

| Role | Manages |
|------|---------|
| **Mail Administrator** | Mailboxes, mail flow, distribution lists |
| **Drive Administrator** | File storage, sharing policies, quotas |
| **Meet & Chat Administrator** | Video meetings, chat settings |
| **Knowledge Administrator** | Knowledge base, document libraries |

### End-User Roles

| Role | Description | Best For |
|------|-------------|----------|
| **Power User** | Full productivity + automation | Developers, analysts |
| **Standard User** | Normal productivity access | Regular employees |
| **Guest User** | Limited external access | Partners, contractors |
| **Viewer** | Read-only access | Auditors, observers |

## Permission Categories

Permissions are organized into logical categories:

### Administration (`admin`)
- `org.*` - Organization settings
- `users.*` - User management
- `groups.*` - Group management
- `roles.*` - Role management
- `dns.*` - DNS and domains

### Compliance (`compliance`)
- `audit.*` - Audit logs
- `compliance.*` - Compliance policies
- `dlp.*` - Data loss prevention
- `retention.*` - Data retention
- `ediscovery.*` - Legal discovery

### Security (`security`)
- `security.*` - Security settings
- `threats.*` - Threat management
- `secrets.*` - API keys and secrets

### Productivity Apps

| Category | Permissions |
|----------|-------------|
| `mail` | read, send, delete, organize, delegate, admin |
| `calendar` | read, write, share, delegate, rooms |
| `drive` | read, write, delete, share, sync, admin |
| `docs` | read, write, comment, share, templates |
| `sheet` | read, write, share, macros, connections |
| `slides` | read, write, share, present |
| `meet` | join, create, host, record, webinar |
| `chat` | read, write, channels, external |
| `tasks` | read, write, assign, projects, automation |

### AI & Bots (`ai`)
- `bots.*` - Bot configuration
- `ai.*` - AI assistant features
- `kb.*` - Knowledge base
- `conversations.*` - Bot conversations
- `attendant.*` - Human handoff

### Automation (`automation`)
- `autotask.*` - Automated tasks
- `workflows.*` - Workflow definitions
- `intents.*` - AI intent management

## Best Practices

### 1. Use Groups for Department Access
```
Group: Sales Team
  └── Role: Standard User
  └── Role: CRM Access (custom)
  
Group: IT Department  
  └── Role: Standard User
  └── Role: Helpdesk Administrator
```

### 2. Principle of Least Privilege
Start with the minimum permissions and add as needed:
- New employees → `Standard User`
- After training → Add specific permissions
- Temporary access → Set expiration dates

### 3. Use Time-Limited Assignments
```sql
-- Role expires in 30 days
expires_at: 2025-08-14T00:00:00Z
```

### 4. Regular Access Reviews
- Quarterly review of admin roles
- Monthly review of external access
- Automated alerts for unused permissions

### 5. Audit Everything
All permission changes are logged:
- Who made the change
- What was changed
- When it happened
- Why (if documented)

## Migration from Other Platforms

### From Microsoft 365

1. Export Azure AD groups and roles
2. Map to equivalent General Bots roles
3. Import users and create groups
4. Assign group-role mappings
5. Verify with test accounts

### From Google Workspace

1. Export Google Admin directory
2. Map organizational units to groups
3. Map admin roles to equivalent roles
4. Import and test

See [Migration Guide](../12-ecosystem-reference/overview.md) for detailed instructions.

## API Reference

### List All Roles
```http
GET /api/rbac/roles
```

### Assign Role to User
```http
POST /api/rbac/users/{user_id}/roles/{role_id}
Content-Type: application/json

{
  "expires_at": "2025-12-31T23:59:59Z"  // Optional
}
```

### Add User to Group
```http
POST /api/rbac/users/{user_id}/groups/{group_id}
```

### Get User's Effective Permissions
```http
GET /api/rbac/users/{user_id}/permissions
```

Response:
```json
{
  "user_id": "uuid",
  "direct_roles": [...],
  "group_roles": [...],
  "groups": [...],
  "effective_permissions": [
    "mail.read",
    "mail.send",
    "drive.read",
    ...
  ]
}
```

## Next Steps

- [Permissions Matrix](./permissions-matrix.md) - Complete permission reference
- [User Authentication](./user-auth.md) - Login and identity
- [Security Checklist](./security-checklist.md) - Deployment hardening
- [API Endpoints](./api-endpoints.md) - Full API documentation