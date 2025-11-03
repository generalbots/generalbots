# Authentication and Security

## User Authentication

GeneralBots provides robust authentication with:

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

```rust
// Example password hashing
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2.hash_password(password.as_bytes(), &salt);
```

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
```rust
auth_service.create_user(username, email, password);
```

### Verifying Users
```rust
auth_service.verify_user(username, password);
```

### Updating Passwords
```rust
auth_service.update_user_password(user_id, new_password);
```

## Bot Authentication

- Bots can be authenticated by name
- Each bot can have custom authentication scripts
- Authentication scripts are stored in `.gbdialog/auth.ast`

```bas
// Example bot auth script
IF token != "secret" THEN
    RETURN false
ENDIF
RETURN true
```

## Security Considerations

- All authentication requests are logged
- Failed attempts are rate-limited
- Session tokens have limited lifetime
- Password hashes are never logged
