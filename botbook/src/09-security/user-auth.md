# User Authentication

General Bots uses a directory service component for user authentication and authorization. No passwords are stored internally in General Bots.

## Overview

Authentication in General Bots is handled entirely by the directory service, which provides:
- User identity management
- OAuth 2.0 / OpenID Connect (OIDC) authentication
- Single Sign-On (SSO) capabilities
- Multi-factor authentication (MFA)
- User and organization management
- Role-based access control (RBAC)

## Architecture

### Directory Service Integration

General Bots integrates with the directory service through:
- **DirectoryClient**: Client for API communication
- **AuthService**: Service layer for authentication operations
- **OIDC Flow**: Standard OAuth2/OIDC authentication flow
- **Service Account**: For administrative operations

### No Internal Password Storage

- **No password_hash columns**: Users table only stores directory user IDs
- **No Argon2 hashing**: All password operations handled by directory service
- **No password reset logic**: Managed through directory service's built-in flows
- **Session tokens only**: General Bots only manages session state

## Authentication Flow

### Authentication Architecture

<svg viewBox="0 0 800 600" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <marker id="arrowauth1" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666"/>
    </marker>
    <marker id="arrowauth1b" markerWidth="10" markerHeight="10" refX="0" refY="3" orient="auto">
      <polygon points="10 0, 0 3, 10 6" fill="#666"/>
    </marker>
  </defs>
  
  <!-- Browser Box -->
  <rect x="50" y="50" width="150" height="80" rx="5" fill="none" stroke="#666" stroke-width="2"/>
  <text x="125" y="85" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">Browser</text>
  
  <!-- General Bots Box -->
  <rect x="325" y="50" width="150" height="80" rx="5" fill="none" stroke="#666" stroke-width="2"/>
  <text x="400" y="85" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">General Bots</text>
  
  <!-- Directory Service Box -->
  <rect x="600" y="50" width="150" height="80" rx="5" fill="none" stroke="#666" stroke-width="2"/>
  <text x="675" y="85" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">Directory</text>
  <text x="675" y="105" text-anchor="middle" fill="#333" font-family="monospace" font-size="14">Service</text>
  
  <!-- Database Box -->
  <rect x="325" y="480" width="150" height="80" rx="5" fill="none" stroke="#666" stroke-width="2"/>
  <text x="400" y="505" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">PostgreSQL</text>
  <text x="400" y="525" text-anchor="middle" fill="#666" font-family="monospace" font-size="12">• Sessions</text>
  <text x="400" y="545" text-anchor="middle" fill="#666" font-family="monospace" font-size="12">• User Refs</text>
  
  <!-- Browser-General Bots bidirectional arrow -->
  <path d="M 200 90 L 325 90" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowauth1)"/>
  <path d="M 325 90 L 200 90" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowauth1b)"/>
  
  <!-- General Bots-Directory bidirectional arrow -->
  <path d="M 475 90 L 600 90" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowauth1)"/>
  <path d="M 600 90 L 475 90" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowauth1b)"/>
  
  <!-- Vertical flow lines -->
  <line x1="125" y1="130" x2="125" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="400" y1="130" x2="400" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="675" y1="130" x2="675" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  
  <!-- Flow steps -->
  <!-- 1. Login Request -->
  <path d="M 125 170 L 395 170" stroke="#4CAF50" stroke-width="2" fill="none" marker-end="url(#arrowauth1)"/>
  <text x="260" y="160" text-anchor="middle" fill="#4CAF50" font-family="monospace" font-size="11">1. Login Request</text>
  
  <!-- 2. Redirect to OIDC -->
  <path d="M 405 200 L 670 200" stroke="#2196F3" stroke-width="2" fill="none" marker-end="url(#arrowauth1)"/>
  <text x="537" y="190" text-anchor="middle" fill="#2196F3" font-family="monospace" font-size="11">2. Redirect to OIDC</text>
  
  <!-- 3. Show Login Page -->
  <path d="M 670 230 L 130 230" stroke="#FF9800" stroke-width="2" fill="none" marker-end="url(#arrowauth1b)"/>
  <text x="400" y="220" text-anchor="middle" fill="#FF9800" font-family="monospace" font-size="11">3. Show Login Page</text>
  
  <!-- 4. Enter Credentials -->
  <path d="M 130 260 L 670 260" stroke="#9C27B0" stroke-width="2" fill="none" marker-end="url(#arrowauth1)"/>
  <text x="400" y="250" text-anchor="middle" fill="#9C27B0" font-family="monospace" font-size="11">4. Enter Credentials</text>
  
  <!-- 5. Return Tokens -->
  <path d="M 670 290 L 405 290" stroke="#E91E63" stroke-width="2" fill="none" marker-end="url(#arrowauth1b)"/>
  <text x="537" y="280" text-anchor="middle" fill="#E91E63" font-family="monospace" font-size="11">5. Return Tokens</text>
  
  <!-- 6. Set Session Cookie -->
  <path d="M 395 320 L 130 320" stroke="#795548" stroke-width="2" fill="none" marker-end="url(#arrowauth1b)"/>
  <text x="262" y="310" text-anchor="middle" fill="#795548" font-family="monospace" font-size="11">6. Set Session Cookie</text>
  
  <!-- 7. Authenticated! -->
  <text x="125" y="360" text-anchor="middle" fill="#4CAF50" font-family="monospace" font-size="12" font-weight="bold">7. Authenticated!</text>
  
  <!-- Database sync -->
  <path d="M 675 450 L 475 520" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowauth1)" stroke-dasharray="5,5"/>
  <text x="575" y="480" text-anchor="middle" fill="#666" font-family="monospace" font-size="11">User Sync</text>
</svg>

### User Registration

<svg viewBox="0 0 700 500" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <marker id="arrowreg" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666"/>
    </marker>
  </defs>
  
  <!-- Title -->
  <text x="350" y="30" text-anchor="middle" fill="#333" font-family="monospace" font-size="16" font-weight="bold">User Registration Flow</text>
  
  <!-- Entity headers -->
  <text x="100" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">User</text>
  <text x="350" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">General Bots</text>
  <text x="600" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">Directory</text>
  
  <!-- Vertical lifelines -->
  <line x1="100" y1="80" x2="100" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="350" y1="80" x2="350" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="600" y1="80" x2="600" y2="450" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  
  <!-- Register -->
  <path d="M 100 120 L 345 120" stroke="#4CAF50" stroke-width="2" fill="none" marker-end="url(#arrowreg)"/>
  <text x="225" y="110" text-anchor="middle" fill="#4CAF50" font-family="monospace" font-size="12">Register</text>
  
  <!-- Create -->
  <path d="M 355 150 L 595 150" stroke="#2196F3" stroke-width="2" fill="none" marker-end="url(#arrowreg)"/>
  <text x="475" y="140" text-anchor="middle" fill="#2196F3" font-family="monospace" font-size="12">Create</text>
  
  <!-- Zitadel operations -->
  <rect x="605" y="170" width="180" height="100" rx="3" fill="#f0f4f8" stroke="#666" stroke-width="1"/>
  <text x="615" y="190" fill="#666" font-family="monospace" font-size="11">► Generate ID</text>
  <text x="615" y="215" fill="#666" font-family="monospace" font-size="11">► Hash Password</text>
  <text x="615" y="240" fill="#666" font-family="monospace" font-size="11">► Store User</text>
  
  <!-- User ID returned -->
  <path d="M 595 290 L 355 290" stroke="#FF9800" stroke-width="2" fill="none" marker-end="url(#arrowreg)"/>
  <text x="475" y="280" text-anchor="middle" fill="#FF9800" font-family="monospace" font-size="12">User ID</text>
  
  <!-- General Bots operations -->
  <rect x="355" y="310" width="180" height="75" rx="3" fill="#f0f4f8" stroke="#666" stroke-width="1"/>
  <text x="365" y="330" fill="#666" font-family="monospace" font-size="11">► Create Local Ref</text>
  <text x="365" y="355" fill="#666" font-family="monospace" font-size="11">► Start Session</text>
  
  <!-- Token returned -->
  <path d="M 345 405 L 105 405" stroke="#9C27B0" stroke-width="2" fill="none" marker-end="url(#arrowreg)"/>
  <text x="225" y="395" text-anchor="middle" fill="#9C27B0" font-family="monospace" font-size="12">Token</text>
</svg>

1. User registration request sent to directory service
2. Directory service creates user account
3. User ID returned to botserver
4. General Bots creates local user reference
5. Session established with General Bots

### User Login

<svg viewBox="0 0 800 700" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <marker id="arrowlogin" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666"/>
    </marker>
  </defs>
  
  <!-- Title -->
  <text x="400" y="30" text-anchor="middle" fill="#333" font-family="monospace" font-size="16" font-weight="bold">User Login Flow</text>
  
  <!-- Entity headers -->
  <text x="100" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">Browser</text>
  <text x="400" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">General Bots</text>
  <text x="700" y="70" text-anchor="middle" fill="#333" font-family="monospace" font-size="14" font-weight="bold">Directory</text>
  
  <!-- Vertical lifelines -->
  <line x1="100" y1="80" x2="100" y2="650" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="400" y1="80" x2="400" y2="650" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  <line x1="700" y1="80" x2="700" y2="650" stroke="#666" stroke-width="1" stroke-dasharray="2,2"/>
  
  <!-- 1. GET /login -->
  <path d="M 100 120 L 395 120" stroke="#4CAF50" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="250" y="110" text-anchor="middle" fill="#4CAF50" font-family="monospace" font-size="11">GET /login</text>
  
  <!-- 2. 302 Redirect -->
  <path d="M 395 160 L 105 160" stroke="#2196F3" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="250" y="150" text-anchor="middle" fill="#2196F3" font-family="monospace" font-size="11">302 Redirect</text>
  <text x="250" y="175" text-anchor="middle" fill="#2196F3" font-family="monospace" font-size="10">to Directory</text>
  
  <!-- 3. Show Login Form -->
  <path d="M 105 220 L 695 220" stroke="#FF9800" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <path d="M 695 250 L 105 250" stroke="#FF9800" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="400" y="210" text-anchor="middle" fill="#FF9800" font-family="monospace" font-size="11">Show Login Form</text>
  
  <!-- 4. Submit Credentials -->
  <path d="M 105 290 L 695 290" stroke="#9C27B0" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="400" y="280" text-anchor="middle" fill="#9C27B0" font-family="monospace" font-size="11">Submit Credentials</text>
  
  <!-- Directory validation -->
  <rect x="705" y="310" width="150" height="75" rx="3" fill="#f0f4f8" stroke="#666" stroke-width="1"/>
  <text x="715" y="330" fill="#666" font-family="monospace" font-size="11">► Validate</text>
  <text x="715" y="355" fill="#666" font-family="monospace" font-size="11">► Generate Tokens</text>
  
  <!-- 5. Redirect + Tokens -->
  <path d="M 695 405 L 105 405" stroke="#E91E63" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="400" y="395" text-anchor="middle" fill="#E91E63" font-family="monospace" font-size="11">Redirect + Tokens</text>
  
  <!-- 6. /auth/callback -->
  <path d="M 105 445 L 395 445" stroke="#795548" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="250" y="435" text-anchor="middle" fill="#795548" font-family="monospace" font-size="11">/auth/callback</text>
  
  <!-- General Bots validation -->
  <rect x="405" y="465" width="160" height="100" rx="3" fill="#f0f4f8" stroke="#666" stroke-width="1"/>
  <text x="415" y="485" fill="#666" font-family="monospace" font-size="11">► Validate Tokens</text>
  <text x="415" y="510" fill="#666" font-family="monospace" font-size="11">► Create Session</text>
  <text x="415" y="535" fill="#666" font-family="monospace" font-size="11">► Store in DB</text>
  
  <!-- 7. Set Cookie -->
  <path d="M 395 585 L 105 585" stroke="#607D8B" stroke-width="2" fill="none" marker-end="url(#arrowlogin)"/>
  <text x="250" y="575" text-anchor="middle" fill="#607D8B" font-family="monospace" font-size="11">Set Cookie</text>
  <text x="250" y="600" text-anchor="middle" fill="#607D8B" font-family="monospace" font-size="10">Redirect to App</text>
</svg>

1. User redirected to directory service login page
2. Credentials validated by directory service
3. OIDC tokens returned via callback
4. General Bots validates tokens
5. Local session created
6. Session token issued to client

### Token Validation

<svg viewBox="0 0 800 600" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <marker id="arrowval" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666"/>
    </marker>
  </defs>
  
  <!-- Title -->
  <text x="400" y="30" text-anchor="middle" fill="#333" font-family="monospace" font-size="16" font-weight="bold">Token Validation Flow</text>
  
  <!-- Request Flow Column -->
  <text x="200" y="70" text-anchor="middle" fill="#666" font-family="monospace" font-size="13">Request Flow</text>
  
  <!-- Validation Pipeline Column -->
  <text x="600" y="70" text-anchor="middle" fill="#666" font-family="monospace" font-size="13">Validation Pipeline</text>
  
  <!-- Flow arrows -->
  <path d="M 200 85 L 200 110" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  <path d="M 600 85 L 600 110" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  
  <!-- Request + Cookie -->
  <rect x="125" y="120" width="150" height="60" rx="5" fill="#f0f4f8" stroke="#666" stroke-width="2"/>
  <text x="200" y="145" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Request</text>
  <text x="200" y="165" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">+ Cookie</text>
  
  <!-- Extract Token -->
  <rect x="450" y="120" width="300" height="60" rx="5" fill="#e3f2fd" stroke="#2196F3" stroke-width="2"/>
  <text x="600" y="145" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Extract Token</text>
  <text x="600" y="165" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">from Cookie/Header</text>
  
  <!-- General Bots Validates -->
  <rect x="125" y="220" width="150" height="60" rx="5" fill="#f0f4f8" stroke="#666" stroke-width="2"/>
  <text x="200" y="245" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">General Bots</text>
  <text x="200" y="265" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Validates</text>
  
  <!-- Check Session -->
  <rect x="450" y="220" width="300" height="60" rx="5" fill="#e8f5e9" stroke="#4CAF50" stroke-width="2"/>
  <text x="600" y="245" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Check Session</text>
  <text x="600" y="265" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">in Local Cache</text>
  
  <!-- Connection between boxes -->
  <path d="M 450 250 L 275 250" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  
  <!-- Decision branches -->
  <text x="760" y="255" fill="#4CAF50" font-family="monospace" font-size="11">► Valid? Continue</text>
  <text x="760" y="275" fill="#FF9800" font-family="monospace" font-size="11">► Expired?</text>
  
  <!-- Directory Refresh -->
  <rect x="125" y="320" width="150" height="60" rx="5" fill="#f0f4f8" stroke="#666" stroke-width="2"/>
  <text x="200" y="345" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Directory</text>
  <text x="200" y="365" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Refresh</text>
  
  <!-- Refresh with Directory API -->
  <rect x="450" y="320" width="300" height="60" rx="5" fill="#fff3e0" stroke="#FF9800" stroke-width="2"/>
  <text x="600" y="345" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Refresh with</text>
  <text x="600" y="365" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Directory API</text>
  
  <!-- Connection for refresh -->
  <path d="M 450 350 L 275 350" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  
  <!-- Process Request -->
  <rect x="125" y="420" width="150" height="60" rx="5" fill="#f0f4f8" stroke="#666" stroke-width="2"/>
  <text x="200" y="445" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Process</text>
  <text x="200" y="465" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Request</text>
  
  <!-- Load User Context -->
  <rect x="450" y="420" width="300" height="60" rx="5" fill="#f3e5f5" stroke="#9C27B0" stroke-width="2"/>
  <text x="600" y="445" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Load User Context</text>
  <text x="600" y="465" text-anchor="middle" fill="#333" font-family="monospace" font-size="12">Apply Permissions</text>
  
  <!-- Flow arrows between stages -->
  <path d="M 200 180 L 200 220" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  <path d="M 200 280 L 200 320" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  <path d="M 200 380 L 200 420" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  
  <path d="M 600 180 L 600 220" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  <path d="M 600 280 L 600 320" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
  <path d="M 600 380 L 600 420" stroke="#666" stroke-width="2" fill="none" marker-end="url(#arrowval)"/>
</svg>

1. Client includes session token
2. General Bots validates local session
3. Optional: Refresh with directory service if expired
4. User context loaded from directory service
5. Request processed with user identity

## Directory Service Configuration

### Auto-Configuration

During bootstrap, General Bots automatically:
1. Installs directory service via installer.rs
2. Configures directory service with PostgreSQL
3. Creates default organization
4. Sets up service account
5. Creates initial admin user
6. Configures OIDC application

## Database Schema

### Users Table (Simplified)

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Internal General Bots ID |
| directory_id | TEXT | User ID in directory service |
| username | TEXT | Cached username |
| email | TEXT | Cached email |
| created_at | TIMESTAMPTZ | First login time |
| updated_at | TIMESTAMPTZ | Last sync with directory |

Note: No password_hash or any password-related fields exist.

### User Sessions Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Session ID |
| user_id | UUID | Reference to users table |
| session_token | TEXT | General Bots session token |
| directory_token | TEXT | Cached OIDC token |
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

## Directory Service Features

### User Management

- Create, update, delete users
- Password reset flows
- Email verification
- Profile management
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

### Directory Client Implementation

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

Creating users via Directory Client:
- Username: john_doe
- Email: john@example.com
- First name: John
- Last name: Doe
- Password: Set through Directory UI or email flow

### Getting User Info

User information is fetched from the Directory service using the directory ID.

### Managing Sessions

Sessions are managed locally by General Bots but authenticated through Directory Service:
- Session creation after Directory auth
- Local session tokens for performance
- Periodic validation with Zitadel
- Session termination on logout

## Default Users

During bootstrap, the system creates:

1. **Admin User**
   - Username: admin (configurable)
   - Email: admin@localhost
   - Password: **Randomly generated** (displayed once during setup)
   - Role: Administrator

2. **Regular User**
   - Username: user
   - Email: user@default
   - Password: **Randomly generated** (displayed once during setup)
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

### Directory Service Audit Logs

- All authentication events logged
- User actions tracked
- Administrative changes recorded
- Security events monitored

### Session Metrics

General Bots tracks:
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

General Bots' integration with the Directory Service provides enterprise-grade authentication without the complexity and risk of managing passwords internally. All authentication operations are delegated to the Directory Service, while General Bots focuses on session management and bot interactions.