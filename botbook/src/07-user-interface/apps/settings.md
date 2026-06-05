# Settings - User Preferences

> **Your profile, security, and preferences**

<img src="../../assets/suite/settings-screen.svg" alt="Settings Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Settings is the user preferences application in General Bots Suite. Manage your profile, security credentials, appearance, notifications, and third-party integrations. Settings gives you full control over how the platform looks, feels, and behaves across all your sessions.

---

## Features

### Profile

| Setting | Description |
|---------|-------------|
| Avatar | Upload or change your profile picture |
| Display Name | Your visible name across the suite |
| Email Address | Primary email for notifications |
| Bio | Short description of your role |
| Language | Preferred language for the interface |

### Security

| Setting | Description |
|---------|-------------|
| Change Password | Update your account password |
| Two-Factor Authentication | Enable 2FA via authenticator app |
| Active Sessions | View and revoke active sessions |
| Login History | Recent login attempts and locations |
| API Keys | Generate and manage access tokens |

### Appearance

| Setting | Description |
|---------|-------------|
| Theme | Light, dark, or system default |
| Accent Color | Primary color for the interface |
| Font Size | Small, medium, or large |
| Compact Mode | Reduce spacing for dense views |
| Sidebar Position | Left or right placement |

### Notifications

| Channel | Options |
|---------|---------|
| Email | Frequency, digest, critical-only |
| Push | Browser push notifications |
| In-App | Real-time badges and banners |
| Sound | Notification sounds on/off |
| Quiet Hours | Do-not-disturb schedule |

### Integrations

| Integration | Description |
|-------------|-------------|
| API Keys | Create and revoke programmatic access |
| Webhooks | Configure event-driven callbacks |
| OAuth Apps | Manage connected applications |
| SSO Settings | Single sign-on configuration |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `G` then `P` | Go to Profile |
| `G` then `S` | Go to Security |
| `G` then `A` | Go to Appearance |
| `G` then `N` | Go to Notifications |
| `G` then `I` | Go to Integrations |
| `Ctrl+S` | Save changes |
| `Escape` | Cancel editing |

---

## Settings via Chat

### Changing Your Theme

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Change my theme to dark</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎨 Theme updated to <strong>Dark</strong>.</p>
      <p>The interface will now use a dark color scheme across all pages.</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
</div>

### Enabling Two-Factor Authentication

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Enable 2FA</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔐 Two-factor authentication setup initiated.</p>
      <p>Scan this QR code with your authenticator app, then enter the 6-digit code to verify:</p>
      <p><img src="../../assets/suite/2fa-qr-placeholder.svg" alt="QR Code" style="max-width: 200px;"></p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
</div>

### Updating Your Profile

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Update my display name to Marketing Lead</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Display name updated to <strong>Marketing Lead</strong>.</p>
      <p>This change is reflected across all suite applications.</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>
</div>

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/users/me` | GET | Get current user profile |
| `/api/users/me` | PATCH | Update profile fields |
| `/api/users/me/security` | GET | Security settings and session list |
| `/api/users/me/security/password` | PUT | Change password |
| `/api/users/me/security/2fa` | POST | Enable 2FA |
| `/api/users/me/security/2fa` | DELETE | Disable 2FA |
| `/api/users/me/sessions` | GET | List active sessions |
| `/api/users/me/sessions/:id` | DELETE | Revoke a session |
| `/api/users/me/notifications` | GET | Notification preferences |
| `/api/users/me/notifications` | PUT | Update notification preferences |
| `/api/users/me/api-keys` | GET | List API keys |
| `/api/users/me/api-keys` | POST | Generate new API key |
| `/api/users/me/api-keys/:id` | DELETE | Revoke API key |

### Profile Update Request

```json
{
    "display_name": "Marketing Lead",
    "avatar_url": "https://cdn.example.com/avatars/user123.png",
    "language": "en",
    "bio": "Leading marketing initiatives"
}
```

### Profile Response

```json
{
    "id": "usr-abc123",
    "display_name": "Marketing Lead",
    "email": "user@company.com",
    "avatar_url": "https://cdn.example.com/avatars/user123.png",
    "language": "en",
    "theme": "dark",
    "created_at": "2024-01-15T08:00:00Z"
}
```

---

## Configuration

Settings are stored per-user in the database. Default values can be overridden in `config.csv`:

```csv
key,value
default-theme,dark
default-language,en
allow-2fa,true
max-sessions,5
```

---

## Troubleshooting

### Theme Not Applying

1. Clear browser cache
2. Check if system theme override is active
3. Refresh the page
4. Verify no conflicting browser extensions

### 2FA Locked Out

1. Use your backup codes from initial setup
2. Contact an administrator to reset 2FA
3. Verify authenticator app time is synced

### Notifications Not Arriving

1. Check notification preferences are enabled
2. Verify browser push notification permissions
3. Check email spam folder for digest emails
4. Ensure quiet hours are not blocking delivery

---

## See Also

- [Suite Manual](../suite-manual.md) - Complete user guide
- [Admin Panel](./admin.md) - System-wide settings
- [Security Guide](../../08-rest-api-tools/security-guide.md) - Best practices
