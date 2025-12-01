# Chapter 12: Authentication & Security

User authentication and permission management for BotServer.

## Overview

BotServer provides enterprise-grade security with flexible authentication options and granular permissions.

## Authentication Methods

| Method | Use Case |
|--------|----------|
| **Session Token** | Web/API access |
| **OAuth2/OIDC** | SSO integration |
| **API Key** | Service accounts |
| **Bot Auth** | Bot-to-bot communication |

## Quick Start

```basic
' Check if user is authenticated
IF user.authenticated THEN
  TALK "Welcome, " + user.name
ELSE
  TALK "Please log in first"
END IF
```

## Security Features

- **Password Hashing**: Argon2 with secure defaults
- **Session Management**: Cryptographic tokens, configurable expiry
- **Rate Limiting**: Prevent brute force attacks
- **Audit Logging**: Track all authentication events
- **Encryption**: AES-GCM for data at rest

## Permission Levels

| Level | Access |
|-------|--------|
| `admin` | Full system access |
| `manager` | Bot management |
| `user` | Standard access |
| `guest` | Read-only |

## Configuration

```csv
name,value
auth-session-ttl,3600
auth-max-attempts,5
auth-lockout-duration,900
```

## Chapter Contents

- [User Authentication](./user-auth.md) - Login flows
- [Password Security](./password-security.md) - Password policies
- [API Endpoints](./api-endpoints.md) - Auth API reference
- [Bot Authentication](./bot-auth.md) - Service accounts
- [Security Features](./security-features.md) - Protection mechanisms
- [Security Policy](./security-policy.md) - Best practices
- [Compliance Requirements](./compliance-requirements.md) - GDPR, LGPD, HIPAA
- [Permissions Matrix](./permissions-matrix.md) - Access control
- [User vs System Context](./user-system-context.md) - Execution contexts

## See Also

- [REST API](../chapter-10-api/README.md) - API authentication
- [Configuration](../chapter-08-config/README.md) - Auth settings