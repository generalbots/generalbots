# OAuth API

> **Social login via OAuth 2.0 providers (Google, GitHub, Microsoft, etc.).**

---

## Base URL

```
/auth/oauth
```

## Authentication

The providers listing endpoint is public. The `/:provider` and `/:provider/callback` endpoints handle the OAuth flow and do not require prior authentication — authentication is established upon successful callback.

---

## Overview

The OAuth API enables users to authenticate using external identity providers. The flow works as follows:

1. Frontend calls `GET /auth/oauth/providers` to discover enabled providers
2. User clicks a provider button, which navigates to `GET /auth/oauth/:provider`
3. BotServer redirects the user to the provider's authorization page
4. After approval, the provider redirects back to `GET /auth/oauth/:provider/callback`
5. BotServer exchanges the code for a token, creates/retrieves the user, and establishes a session

---

## Endpoints

### List Enabled Providers

**`GET /auth/oauth/providers`**

List all configured and enabled OAuth providers. Returns only providers that have valid credentials configured in the active bot's `config.csv`.

**Response:**
```json
{
  "providers": [
    {
      "id": "google",
      "name": "Google",
      "icon": "google",
      "login_url": "/auth/oauth/google"
    },
    {
      "id": "github",
      "name": "GitHub",
      "icon": "github",
      "login_url": "/auth/oauth/github"
    }
  ]
}
```

---

### Start OAuth Flow

**`GET /auth/oauth/:provider`**

Initiate the OAuth authentication flow. Redirects the user to the provider's authorization URL.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| provider | string | Yes | OAuth provider name (e.g., `google`, `github`, `microsoft`) |
| redirect | string | No | URL to redirect to after successful login (default: `/`) |

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| redirect | string | No | Post-login redirect URL |

**Response:**
- **302 Redirect** — To the provider's authorization page
- **400 Bad Request** — If the provider name is invalid
- **503 Service Unavailable** — If the provider is not configured

**Error Page (400):**
```html
<h1>Invalid OAuth Provider</h1>
<p>Provider 'invalid' is not supported.</p>
<a href="/auth/login">Back to Login</a>
```

---

### OAuth Callback

**`GET /auth/oauth/:provider/callback`**

Handle the callback from the OAuth provider. This endpoint is called by the provider after the user authorizes or denies access.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| provider | string | Yes | OAuth provider name |

**Query Parameters (from provider):**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| code | string | Conditional | Authorization code (present on success) |
| state | string | Yes | Encoded state parameter for CSRF protection |
| error | string | Conditional | Error code (present on failure) |
| error_description | string | No | Human-readable error description |

**Success Response:**
- **303 See Other** — Redirects to the post-login URL
- Sets `session` cookie with the session token

**Error Responses:**
- **400 Bad Request** — Missing code, invalid state, or state expired
- **401 Unauthorized** — Provider returned an error
- **500 Internal Server Error** — Failed to exchange code or create user

---

## Examples

### Discover Available Providers

```bash
curl -X GET http://localhost:8080/auth/oauth/providers
```

### Start Google Login

```bash
curl -X GET "http://localhost:8080/auth/oauth/google?redirect=/dashboard" \
  -v
# Follow redirect to Google authorization page
```

### Direct Browser Login

```html
<a href="/auth/oauth/google?redirect=/dashboard">
  Sign in with Google
</a>
```

---

## OAuth Providers

### Supported Providers

| Provider | Config Key | Description |
|----------|------------|-------------|
| Google | `google` | Google OAuth 2.0 |
| GitHub | `github` | GitHub OAuth |
| Microsoft | `microsoft` | Microsoft/Azure AD |

### Configuration in config.csv

Each provider requires the following keys in the bot's `config.csv`:

| Key | Description |
|-----|-------------|
| `oauth-{provider}-client-id` | Client ID from the provider |
| `oauth-{provider}-client-secret` | Client secret from the provider |
| `oauth-{provider}-enabled` | Set to `true` to enable |

**Example:**
```
oauth-google-client-id,123456789.apps.googleusercontent.com
oauth-google-client-secret,GOCSPX-xxxxxxxxxxx
oauth-google-enabled,true
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success (providers list) |
| 302 | Redirect (start OAuth flow) |
| 303 | See Other (callback success) |
| 400 | Bad Request (invalid provider or missing params) |
| 401 | Unauthorized (provider denied access) |
| 500 | Internal Server Error |
| 503 | Service Unavailable (provider not configured) |

---

## Security Notes

- The `state` parameter is encoded with a timestamp and HMAC signature to prevent CSRF attacks
- State tokens expire after 10 minutes
- The provider name in the URL must match the provider in the state parameter
- Session cookies are `HttpOnly` and `SameSite=Lax`

---

## See Also

- [Security API](./security-api.md) — Authentication and access control
- [User Security](./user-security.md) — User management and permissions
- [Groups API](./groups-api.md) — Group management
