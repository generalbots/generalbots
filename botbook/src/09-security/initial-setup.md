# Initial Setup & Admin Bootstrap

When General Bots is installed for the first time, it automatically creates an administrator account. This page explains the bootstrap process and how to access your new installation.

## Automatic Bootstrap

On first startup, General Bots checks if any admin users exist in the directory service (Zitadel). If no admin is found, it automatically:

1. Creates an `admin` user
2. Generates a secure random password
3. Creates a default organization ("General Bots")
4. Assigns admin roles to the user
5. Displays credentials in the server console

### Console Output

When bootstrap completes, you'll see output similar to this in your server console:

```
╔════════════════════════════════════════════════════════════╗
║                                                            ║
║       🤖 GENERAL BOTS - INITIAL SETUP COMPLETE            ║
║                                                            ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║  Administrator account has been created:                   ║
║                                                            ║
║  ┌──────────────────────────────────────────────────────┐  ║
║  │                                                      │  ║
║  │  Username:             admin                         │  ║
║  │  Email:                admin@localhost               │  ║
║  │  Password:             xK3$mP9@vL2nQ7&w              │  ║
║  │                                                      │  ║
║  └──────────────────────────────────────────────────────┘  ║
║                                                            ║
║  Organization: General Bots (abc12345)                     ║
║                                                            ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║  ⚠️  IMPORTANT: Save these credentials securely!           ║
║      This information will not be shown again.             ║
║                                                            ║
║  To login, navigate to:                                    ║
║      http://localhost:PORT/auth/login                      ║
║                                                            ║
╚════════════════════════════════════════════════════════════╝
```

> **Important**: Save these credentials immediately! The password is only displayed once during the initial startup.

## First Login

1. Navigate to `http://localhost:PORT/auth/login` (replace PORT with your configured port)
2. Enter the username: `admin`
3. Enter the password shown in the console
4. Click "Sign In"

## What Gets Created

| Item | Value | Description |
|------|-------|-------------|
| **Username** | `admin` | Default administrator username |
| **Email** | `admin@localhost` | Default admin email |
| **Password** | (random) | 14+ character secure password |
| **Organization** | "General Bots" | Default organization |
| **Roles** | `admin`, `org_owner`, `user_manager` | Full administrative access |

## Password Security

The auto-generated password includes:

- 4+ lowercase letters (a-z)
- 4+ uppercase letters (A-Z)
- 4+ digits (0-9)
- 2+ special characters (!@#$%&*)
- Randomly shuffled for unpredictability

## After First Login

Once logged in as admin, you should:

1. **Change your password** (recommended)
2. **Update admin email** to a real email address
3. **Create additional users** via Settings → Users
4. **Configure your organization** settings

## Creating Additional Users

As an admin, you can create users through the Settings UI:

1. Go to Settings → Users
2. Click "Add User"
3. Fill in user details:
   - Username
   - Email
   - First/Last name
   - Role (user, admin, etc.)
4. The user will be created in the directory service (Zitadel)
5. The user will automatically belong to your organization

## Organization Structure

```
Organization (e.g., "Acme Corp")
├── Users
│   ├── admin (org_owner, admin)
│   ├── john.doe (user)
│   └── jane.smith (bot_operator)
├── Bots
│   ├── sales-bot
│   ├── support-bot
│   └── hr-bot
└── Drive Storage
    ├── acme-sales-bot.gbai/
    ├── acme-support-bot.gbai/
    └── acme-hr-bot.gbai/
```

## Manual Bootstrap (Recovery)

If you need to manually create an admin (e.g., for recovery), you can use the bootstrap endpoint:

### 1. Set Bootstrap Secret

Add to your environment variables:

```bash
export GB_BOOTSTRAP_SECRET=your-secure-random-secret
```

### 2. Access Bootstrap Page

Navigate to: `http://localhost:PORT/auth/bootstrap`

### 3. Fill in the Form

- **Bootstrap Secret**: The value you set in `GB_BOOTSTRAP_SECRET`
- **Organization Name**: Your company/org name
- **Admin Details**: Username, email, password

### 4. Submit

The admin account will be created and you can login normally.

> **Note**: The manual bootstrap endpoint only works when `GB_BOOTSTRAP_SECRET` is set and no admin users exist.

## Troubleshooting

### "Admin user already exists"

This means bootstrap already completed. If you've lost the password:

1. Access Zitadel console directly (usually port 8300)
2. Use Zitadel's password reset functionality
3. Or delete the user in Zitadel and restart General Bots

### Bootstrap Not Running

Check that:

1. Zitadel (directory service) is running and healthy
2. The Zitadel configuration in your `.env` is correct
3. Check server logs for connection errors

### Cannot Connect to Directory Service

```bash
# Check if Zitadel is running
curl http://localhost:8300/healthz

# Check logs
cat botserver-stack/logs/directory/zitadel.log
```

## Security Considerations

1. **First-time setup**: Bootstrap only runs once when no admins exist
2. **Console only**: Credentials are never logged to files, only displayed in console
3. **Secure password**: Auto-generated passwords meet enterprise security requirements
4. **No default passwords**: Every installation gets a unique password

## API Reference

### Bootstrap Endpoint

```
POST /api/auth/bootstrap
Content-Type: application/json

{
  "bootstrap_secret": "your-secret",
  "organization_name": "My Company",
  "first_name": "John",
  "last_name": "Doe",
  "username": "admin",
  "email": "admin@example.com",
  "password": "<your-secure-password>"
}
```

**Response (Success)**:

```json
{
  "success": true,
  "message": "Admin user 'admin' created successfully...",
  "user_id": "abc123...",
  "organization_id": "org456..."
}
```

**Response (Error - Admin Exists)**:

```json
{
  "error": "Admin user already exists",
  "details": "Bootstrap can only be used for initial setup"
}
```

## Next Steps

After completing initial setup:

- [User Authentication](./user-auth.md) - Learn about login flows
- [Permissions Matrix](./permissions-matrix.md) - Understand role-based access
- [Security Features](./security-features.md) - Configure security options
- [API Endpoints](./api-endpoints.md) - Integrate with your applications