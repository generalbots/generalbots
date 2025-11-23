# User Authentication

BotServer uses Zitadel, an open-source identity management platform, for user authentication and authorization. No passwords are stored internally in BotServer.

## Overview

Authentication in BotServer is handled entirely by Zitadel, which provides:
- User identity management
- OAuth 2.0 / OpenID Connect (OIDC) authentication
- Single Sign-On (SSO) capabilities
- Multi-factor authentication (MFA)
- User and organization management
- Role-based access control (RBAC)

## Architecture

### Zitadel Integration

BotServer integrates with Zitadel through:
- **ZitadelClient**: Client for API communication
- **AuthService**: Service layer for authentication operations
- **OIDC Flow**: Standard OAuth2/OIDC authentication flow
- **Service Account**: For administrative operations

### No Internal Password Storage

- **No password_hash columns**: Users table only stores Zitadel user IDs
- **No Argon2 hashing**: All password operations handled by Zitadel
- **No password reset logic**: Managed through Zitadel's built-in flows
- **Session tokens only**: BotServer only manages session state

## Authentication Flow

### User Registration

1. User registration request sent to Zitadel
2. Zitadel creates user account
3. User ID returned to BotServer
4. BotServer creates local user reference
5. Session established with BotServer

### User Login

1. User redirected to Zitadel login page
2. Credentials validated by Zitadel
3. OIDC tokens returned via callback
4. BotServer validates tokens
5. Local session created
6. Session token issued to client

### Token Validation

1. Client includes session token
2. BotServer validates local session
3. Optional: Refresh with Zitadel if expired
4. User context loaded from Zitadel
5. Request processed with user identity

## Zitadel Configuration

### Environment Variables

```bash
# Zitadel connection
ZITADEL_ISSUER_URL=http://localhost:8080
ZITADEL_API_URL=http://localhost:8080/management/v1
ZITADEL_CLIENT_ID=<generated-client-id>
ZITADEL_CLIENT_SECRET=<generated-client-secret>
ZITADEL_PROJECT_ID=<project-id>

# Service account for admin operations
ZITADEL_SERVICE_ACCOUNT_KEY=<service-account-key>
DIRECTORY_ADMIN_TOKEN=zitadel-admin-sa

# OAuth redirect
ZITADEL_REDIRECT_URI=http://localhost:8080/auth/callback
```

### Auto-Configuration

During bootstrap, BotServer automatically:
1. Installs Zitadel binary
2. Configures Zitadel with PostgreSQL
3. Creates default organization
4. Sets up service account
5. Creates initial admin user
6. Configures OIDC application

## Database Schema

### Users Table (Simplified)

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Internal BotServer ID |
| zitadel_id | TEXT | User ID in Zitadel |
| username | TEXT | Cached username |
| email | TEXT | Cached email |
| created_at | TIMESTAMPTZ | First login time |
| updated_at | TIMESTAMPTZ | Last sync with Zitadel |

Note: No password_hash or any password-related fields exist.

### User Sessions Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Session ID |
| user_id | UUID | Reference to users table |
| session_token | TEXT | BotServer session token |
| zitadel_token | TEXT | Cached OIDC token |
| expires_at | TIMESTAMPTZ | Session expiration |
| created_at | TIMESTAMPTZ | Session start |

## Authentication Endpoints

### Login Initiation

```
GET /auth/login
```

Redirects to Zitadel login page with OIDC parameters.

### OAuth Callback

```
GET /auth/callback?code=...&state=...
```

Handles return from Zitadel after successful authentication.

### Logout

```
POST /auth/logout
```

Terminates local session and optionally triggers Zitadel logout.

### Session Validation

```
GET /auth/validate
Headers: Authorization: Bearer {session_token}
```

Validates current session without calling Zitadel.

## Zitadel Features

### User Management

- Self-registration (configurable)
- Email verification
- Password policies (managed in Zitadel)
- Account locking
- Password recovery

### Multi-Factor Authentication

Configured in Zitadel:
- TOTP (Time-based One-Time Passwords)
- WebAuthn/FIDO2
- SMS OTP (if configured)
- Email OTP

### Single Sign-On

- One login for all applications
- Session management across services
- Centralized user directory
- External IdP integration

### Organizations

- Multi-tenant support
- Organization-specific policies
- Delegated administration
- User isolation

## Directory Service Integration

### ZitadelClient Implementation

Located in `src/directory/client.rs`:
- Manages API communication
- Handles token refresh
- Caches access tokens
- Provides user operations

### AuthService

Located in `src/directory/mod.rs`:
- High-level authentication operations
- Session management
- User profile caching
- Group/role management

## Security Benefits

### Centralized Security

- Professional identity platform
- Regular security updates
- Compliance certifications
- Audit logging

### No Password Liability

- No password storage risks
- No hashing implementation errors
- No password database leaks
- Reduced compliance burden

### Advanced Features

- Passwordless authentication
- Adaptive authentication
- Risk-based access control
- Session security policies

## User Operations

### Creating Users

```rust
// Via ZitadelClient
client.create_user(
    username: "john_doe",
    email: "john@example.com", 
    first_name: "John",
    last_name: "Doe"
)
// Password set through Zitadel UI or email flow
```

### Getting User Info

```rust
// Fetch from Zitadel
let user_info = client.get_user(zitadel_id).await?;
```

### Managing Sessions

Sessions are managed locally by BotServer but authenticated through Zitadel:
- Session creation after Zitadel auth
- Local session tokens for performance
- Periodic validation with Zitadel
- Session termination on logout

## Default Users

During bootstrap, the system creates:

1. **Admin User**
   - Username: admin (configurable)
   - Email: admin@localhost
   - Password: BotServer123! (must be changed)
   - Role: Administrator

2. **Regular User**
   - Username: user
   - Email: user@default
   - Password: User123!
   - Role: Standard user

## Groups and Roles

### Organization Management

- Organizations created in Zitadel
- Users assigned to organizations
- Roles defined per organization
- Permissions inherited from roles

### Role-Based Access

- Admin: Full system access
- User: Standard bot interaction
- Custom roles: Defined in Zitadel

## Monitoring and Audit

### Zitadel Audit Logs

- All authentication events logged
- User actions tracked
- Administrative changes recorded
- Security events monitored

### Session Metrics

BotServer tracks:
- Active sessions count
- Session creation rate
- Failed authentication attempts
- Token refresh frequency

## Troubleshooting

### Common Issues

1. **Zitadel Connection Failed**
   - Check Zitadel is running on port 8080
   - Verify ZITADEL_ISSUER_URL
   - Check network connectivity

2. **Authentication Fails**
   - Verify client credentials
   - Check redirect URI configuration
   - Review Zitadel logs

3. **Session Issues**
   - Clear browser cookies
   - Check session expiry settings
   - Verify token refresh logic

## Best Practices

1. **Use Zitadel UI**: Manage users through Zitadel interface
2. **Configure MFA**: Enable multi-factor for admin accounts
3. **Regular Updates**: Keep Zitadel updated
4. **Monitor Logs**: Review authentication logs regularly
5. **Session Timeout**: Configure appropriate session duration
6. **Secure Communication**: Use HTTPS in production

## Migration from Other Systems

When migrating from password-based systems:
1. Export user data (without passwords)
2. Import users into Zitadel
3. Force password reset for all users
4. Update application to use OIDC flow
5. Remove password-related code

## Summary

BotServer's integration with Zitadel provides enterprise-grade authentication without the complexity and risk of managing passwords internally. All authentication operations are delegated to Zitadel, while BotServer focuses on session management and bot interactions.