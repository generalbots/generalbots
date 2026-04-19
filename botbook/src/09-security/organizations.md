# Organization Management Guide

This guide covers how to manage organizations in General Bots, including creating organizations, managing members, configuring settings, and switching between organizations.

## Overview

Organizations are the top-level tenant in General Bots, providing:

- **Complete Data Isolation**: Each organization has separate bots, knowledge bases, and data
- **Independent Billing**: Separate subscriptions and quotas per organization
- **Member Management**: Users can belong to multiple organizations
- **Custom Settings**: Branding, security policies, and preferences per organization

## Creating an Organization

### Via UI

1. Click your profile avatar in the top-right corner
2. Click **Create Organization**
3. Fill in the organization details:
   - **Name**: Display name for the organization
   - **Slug**: URL-friendly identifier (auto-generated from name)
   - **Description**: Optional description
4. Click **Create**

### Via API

```http
POST /api/organizations
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Acme Corporation",
  "description": "Main organization for Acme Corp"
}
```

Response:

```json
{
  "organization": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Acme Corporation",
    "slug": "acme-corporation",
    "description": "Main organization for Acme Corp",
    "plan_id": "free",
    "owner_id": "user-uuid",
    "created_at": "2025-01-21T10:00:00Z"
  },
  "roles": [...],
  "groups": [...],
  "owner_member": {...},
  "owner_role": {...}
}
```

When you create an organization, you automatically become the owner with full permissions.

## Organization Structure

Each organization includes:

```
Organization
├── Settings
│   ├── General (name, logo, website)
│   ├── Security (2FA, SSO, IP whitelist)
│   ├── Branding (colors, custom CSS)
│   └── Billing (plan, quotas)
├── Members
│   ├── Users
│   ├── Roles
│   └── Groups
├── Bots
│   └── [bot configurations]
├── Apps
│   ├── Forms
│   ├── Sites
│   └── Projects
└── Knowledge Bases
    └── [.gbkb packages]
```

## Managing Members

### Inviting Members

1. Navigate to **Settings** → **Members**
2. Click **Invite Member**
3. Enter email address and select role
4. Click **Send Invitation**

```http
POST /api/organizations/{org_id}/invitations
Authorization: Bearer <token>
Content-Type: application/json

{
  "email": "newuser@company.com",
  "role": "member"
}
```

### Member Roles

When inviting members, assign an appropriate role:

| Role | Typical Use |
|------|-------------|
| Admin | Department heads, IT managers |
| Manager | Team leads, project managers |
| Member | Regular employees |
| Viewer | Stakeholders, external reviewers |

### Accepting Invitations

Invited users receive an email with a link to accept the invitation:

1. Click the invitation link
2. Sign in or create an account
3. Review organization details
4. Click **Accept Invitation**

### Removing Members

1. Navigate to **Settings** → **Members**
2. Find the member in the list
3. Click the menu icon (⋮)
4. Select **Remove from Organization**

```http
DELETE /api/organizations/{org_id}/members/{user_id}
Authorization: Bearer <token>
```

## Organization Settings

### General Settings

| Setting | Description |
|---------|-------------|
| Name | Display name |
| Slug | URL identifier |
| Description | Organization description |
| Logo URL | Logo image URL |
| Website | Organization website |

### Security Settings

| Setting | Description | Default |
|---------|-------------|---------|
| Require 2FA | Require two-factor authentication | false |
| Allowed Email Domains | Restrict sign-ups to specific domains | [] |
| SSO Enabled | Enable Single Sign-On | false |
| SSO Provider | SSO provider configuration | null |
| IP Whitelist | Allowed IP addresses | [] |
| Audit Log Retention | Days to keep audit logs | 90 |

### Custom Branding

| Setting | Description |
|---------|-------------|
| Primary Color | Main brand color |
| Secondary Color | Accent color |
| Logo URL | Custom logo |
| Favicon URL | Browser favicon |
| Custom CSS | Additional styling |

### Configuration Example

```yaml
settings:
  allow_public_bots: false
  require_2fa: true
  allowed_email_domains:
    - company.com
    - subsidiary.com
  default_user_role: member
  max_members: 100
  sso_enabled: true
  sso_provider: okta
  audit_log_retention_days: 365
  ip_whitelist:
    - 10.0.0.0/8
    - 192.168.1.0/24
  custom_branding:
    primary_color: "#0066cc"
    secondary_color: "#004499"
    logo_url: "https://cdn.company.com/logo.svg"
```

## Switching Organizations

Users who belong to multiple organizations can switch between them.

### Via UI

1. Click your profile avatar or the organization name
2. A dropdown shows all your organizations
3. Click the organization to switch to

### Via API

```http
POST /api/user/switch-organization
Authorization: Bearer <token>
Content-Type: application/json

{
  "org_id": "target-organization-uuid"
}
```

The response includes a new session token scoped to the selected organization.

## Organization Switcher Component

The organization switcher displays:

- Current organization name and logo
- User's role in current organization
- List of other organizations
- Quick actions (create new, manage current)

```html
<div class="org-selector">
  <div class="selected-org">
    <div class="org-avatar">AC</div>
    <div class="org-info">
      <span class="org-name">Acme Corporation</span>
      <span class="org-role">Admin</span>
    </div>
  </div>
  <div class="org-dropdown">
    <div class="org-dropdown-item">
      <div class="org-avatar">XY</div>
      <span>XYZ Partners</span>
    </div>
    <div class="org-dropdown-actions">
      <button>+ Create Organization</button>
    </div>
  </div>
</div>
```

## Quotas and Limits

Each organization has quotas based on their plan:

| Quota | Free | Pro | Enterprise |
|-------|------|-----|------------|
| Members | 5 | 50 | Unlimited |
| Bots | 2 | 20 | Unlimited |
| Storage | 1 GB | 50 GB | 1 TB |
| API Calls/month | 10,000 | 500,000 | Unlimited |
| Messages/month | 1,000 | 100,000 | Unlimited |

### Checking Usage

```http
GET /api/organizations/{org_id}/usage
Authorization: Bearer <token>
```

Response:

```json
{
  "organization_id": "org-uuid",
  "period": "2025-01",
  "quotas": {
    "members": { "used": 12, "limit": 50, "percent": 24 },
    "storage_mb": { "used": 5120, "limit": 51200, "percent": 10 },
    "api_calls": { "used": 45000, "limit": 500000, "percent": 9 },
    "messages": { "used": 8500, "limit": 100000, "percent": 8.5 }
  }
}
```

## Deleting an Organization

Only the organization owner can delete an organization.

1. Navigate to **Settings** → **General**
2. Scroll to **Danger Zone**
3. Click **Delete Organization**
4. Type the organization name to confirm
5. Click **Delete Permanently**

```http
DELETE /api/organizations/{org_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "confirmation": "organization-name"
}
```

Deletion is permanent and removes:

- All organization data
- All bots and configurations
- All knowledge bases
- All apps (Forms, Sites, Projects)
- All member associations
- All billing data

## Multi-Organization Patterns

### Separate Environments

Use organizations for dev/staging/production:

```
Acme Corp - Development
Acme Corp - Staging
Acme Corp - Production
```

### Department Isolation

Separate organizations per department:

```
Acme Corp - Sales
Acme Corp - Support
Acme Corp - Engineering
```

### Client Projects

Agencies can create organizations per client:

```
Client A - Project X
Client B - Project Y
Internal - Agency Tools
```

## API Reference

### List User's Organizations

```http
GET /api/user/organizations
Authorization: Bearer <token>
```

### Get Organization Details

```http
GET /api/organizations/{org_id}
Authorization: Bearer <token>
```

### Update Organization

```http
PATCH /api/organizations/{org_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "New Name",
  "description": "Updated description"
}
```

### List Organization Members

```http
GET /api/organizations/{org_id}/members
Authorization: Bearer <token>
```

### Update Organization Settings

```http
PUT /api/organizations/{org_id}/settings
Authorization: Bearer <token>
Content-Type: application/json

{
  "require_2fa": true,
  "allowed_email_domains": ["company.com"]
}
```

## Best Practices

### 1. Clear Naming Conventions

Use consistent naming for multiple organizations:

```
[Company] - [Environment/Purpose]
Acme Corp - Production
Acme Corp - Development
```

### 2. Appropriate Role Assignment

Don't over-assign admin roles:

- One or two owners maximum
- Admins for department heads
- Managers for team leads
- Members for everyone else

### 3. Regular Member Audits

Review members quarterly:

- Remove departed employees
- Adjust roles as responsibilities change
- Check for inactive accounts

### 4. Security Configuration

For enterprise organizations:

- Enable 2FA requirement
- Configure SSO if available
- Set IP whitelist for office networks
- Increase audit log retention

### 5. Quota Monitoring

Set up alerts for quota usage:

- 80% warning for planning
- 90% critical for immediate action
- Monitor trends over time

## Related Topics

- [RBAC Configuration](./rbac-configuration.md)
- [Subscription & Billing](../12-ecosystem-reference/billing.md)
- [Security Matrix](./security-matrix.md)
- [SOC 2 Compliance](./soc2-compliance.md)