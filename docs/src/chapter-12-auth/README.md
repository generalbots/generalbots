# Authentication and Security

## User Authentication

General Bots provides robust authentication with:

- **Argon2 password hashing** for secure credential storage
- **Session management** tied to user identity
- **Anonymous user support** for guest access

### Authentication Flow

1. Client requests `/api/auth` endpoint with credentials
2. System verifies credentials against stored hash
3. New session is created or existing session is returned
4. Session token is provided for subsequent requests

## Password Security

- All passwords are hashed using Argon2 (winner of Password Hashing Competition)
- Random salt generation for each password
- Secure password update mechanism
- Password management delegated to Directory Service

## API Endpoints

### `GET /api/auth`
Authenticates user and returns session

**Parameters:**
- `bot_name`: Name of bot to authenticate against
- `token`: Authentication token (optional)

**Response:**
```json
{
  "user_id": "uuid",
  "session_id": "uuid", 
  "status": "authenticated"
}
```

## User Management

### Creating Users
Users are created through the Directory Service with randomly generated initial passwords.

### Verifying Users
User verification is handled through the Directory Service OAuth2/OIDC flow.

### Updating Passwords
Password updates are managed through the Directory Service's built-in password reset workflows.

## Bot Authentication

- Bots can be authenticated by name
- Each bot can have custom authentication scripts
- Authentication scripts are stored in `.gbdialog/auth.ast`

```bas
// Example bot auth script
IF token != generated_token THEN
    RETURN false
ENDIF
RETURN true
```

## Security Considerations

- All authentication requests are logged
- Failed attempts are rate-limited
- Session tokens have limited lifetime
- Password hashes are never logged

## See Also

- [Services Overview](./services.md) - System services architecture
- [Compliance Requirements](./compliance-requirements.md) - Security and compliance
- [Chapter 1: Installation](../chapter-01/installation.md) - Initial setup
- [Chapter 2: Packages](../chapter-02/README.md) - Bot package system
- [Chapter 3: Knowledge Base](../chapter-03/README.md) - KB infrastructure
- [Chapter 7: Configuration](../chapter-07/README.md) - System configuration
- [Chapter 9: Storage](../chapter-09/storage.md) - Storage architecture
- [Chapter 10: Development](../chapter-10/README.md) - Development environment
- [Chapter 12: Web API](../chapter-12/README.md) - API endpoints
