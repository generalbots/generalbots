# Admin vs User Views

The General Bots Suite separates functionality into two distinct interfaces: the **User View** for personal productivity and the **Admin View** for organization management. This separation ensures users only see features relevant to their role while administrators have access to system-wide controls.

## Overview

| View | Access | Purpose |
|------|--------|---------|
| **User View** | All authenticated users | Personal settings, files, tasks, calendar |
| **Admin View** | Users with `admin` role | Organization management, user provisioning, DNS |

## User View

The User View is the default interface for all authenticated users. It provides access to personal productivity tools and settings.

### Accessing User Settings

1. Click your **avatar** in the top-right corner
2. Select **Settings**

### User Settings Sections

**Profile**
- Display name and avatar
- Email address
- Language and timezone

**Security**
- Change password
- Two-factor authentication (2FA)
- Active sessions management
- Trusted devices

**Appearance**
- Theme selection (dark, light, blue, purple, green, orange)
- Accent color
- Font size preferences

**Notifications**
- Email notification preferences
- Desktop alerts
- Sound settings

**Storage**
- View storage quota usage
- Manage connected storage providers
- Clear cache

**Integrations**
- API keys for external access
- Webhook configurations
- Connected OAuth providers

**Privacy**
- Data visibility settings
- Online status preferences
- Data export and account deletion

### User API Endpoints

All user endpoints use the `/api/user/` prefix:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/user/profile` | GET, PUT | User profile data |
| `/api/user/password` | POST | Change password |
| `/api/user/security/2fa/status` | GET | 2FA status |
| `/api/user/security/2fa/enable` | POST | Enable 2FA |
| `/api/user/security/sessions` | GET | Active sessions |
| `/api/user/notifications/preferences` | GET, PUT | Notification settings |
| `/api/user/storage` | GET | Storage quota |
| `/api/user/api-keys` | GET, POST, DELETE | API key management |
| `/api/user/webhooks` | GET, POST, DELETE | Webhook management |
| `/api/user/data/export` | POST | Request data export |

## Admin View

The Admin View provides organization-wide management capabilities. Access requires the `admin` role.

### Accessing Admin Panel

1. Click your **avatar** in the top-right corner
2. Select **Admin Panel**

If you don't see "Admin Panel", you don't have administrator privileges.

### Admin Panel Sections

**Dashboard**
- Quick statistics (users, groups, bots, storage)
- System health overview
- Recent activity feed
- Quick action buttons

**Users**
- View all organization users
- Create new users
- Edit user details and roles
- Disable or delete accounts
- Reset user passwords

**Groups**
- Create and manage groups
- Assign users to groups
- Set group permissions
- Manage group invitations

**Bots**
- View deployed bots
- Bot configuration management
- Usage statistics per bot

**DNS**
- Register custom hostnames
- Manage DNS records
- SSL certificate status

**Audit Log**
- View all system events
- Filter by user, action, or date
- Export audit reports

**Organization Billing** (Admin-level)
- Organization subscription status
- Usage across all users
- Payment methods for organization
- Invoice history

### Admin API Endpoints

All admin endpoints use the `/api/admin/` prefix and require `admin` role:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/admin/dashboard` | GET | Dashboard statistics |
| `/api/admin/users` | GET, POST | List/create users |
| `/api/admin/users/:id` | GET, PUT, DELETE | Manage specific user |
| `/api/admin/groups` | GET, POST | List/create groups |
| `/api/admin/groups/:id` | GET, PUT, DELETE | Manage specific group |
| `/api/admin/bots` | GET | List organization bots |
| `/api/admin/dns` | GET, POST, DELETE | DNS management |
| `/api/admin/audit` | GET | Audit log entries |
| `/api/admin/stats/*` | GET | Various statistics |
| `/api/admin/health` | GET | System health status |
| `/api/admin/activity/recent` | GET | Recent activity feed |

## Permission Levels

The system uses role-based access control (RBAC):

| Role | User View | Admin View | Description |
|------|-----------|------------|-------------|
| `guest` | Limited | ❌ | Read-only chat access |
| `user` | ✅ | ❌ | Standard user features |
| `manager` | ✅ | Partial | Can view monitoring |
| `admin` | ✅ | ✅ | Full system access |

### Checking User Role

In BASIC scripts, check the user's role:

```basic
role = user.role

IF role = "admin" THEN
    TALK "Welcome, administrator!"
ELSE
    TALK "Welcome, " + user.name
END IF
```

## Desktop App Considerations

When running the Suite as a desktop application (via Tauri), additional features become available:

**Desktop-Only Features**
- Local file system access
- Rclone-based file synchronization
- System tray integration
- Native notifications

**Sync Feature**
The desktop app can sync local folders with cloud Drive using rclone:

1. Configure remote in Settings → Storage → Sync
2. Select local folder to sync
3. Start/stop sync from Drive sidebar

Note: Sync controls (`/files/sync/start`, `/files/sync/stop`) communicate with the local rclone process on the desktop. These features are not available in the web-only version.

## Security Best Practices

**For Users**
- Enable 2FA on your account
- Review active sessions regularly
- Use strong, unique passwords
- Revoke unused API keys

**For Administrators**
- Follow principle of least privilege
- Review audit logs regularly
- Rotate service account credentials
- Monitor for unusual activity
- Keep user list current (remove departed employees)

## Related Documentation

- [Permissions Matrix](../09-security/permissions-matrix.md) - Detailed permission definitions
- [User Authentication](../09-security/user-auth.md) - Login and session management
- [REST Endpoints](../08-rest-api-tools/README.md) - Complete API reference
- [Suite User Manual](./suite-manual.md) - End-user guide