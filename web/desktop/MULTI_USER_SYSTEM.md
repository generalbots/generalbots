# Multi-User System Documentation

## Overview

This document describes the multi-user authentication system that enables users to manage their email accounts, drive storage, and chat sessions with proper authentication.

## Architecture

### User Authentication Model

- **Anonymous Access**: Chat can work without authentication
- **Authenticated Access**: Email, Drive, and Tasks require user login
- **User Accounts**: Stored in `users` table with credentials
- **Session Management**: JWT tokens stored in `user_login_tokens` table

### Database Schema

#### New Tables (Migration 6.0.6)

1. **user_email_accounts**
   - Stores user email account credentials (IMAP/SMTP)
   - Supports multiple accounts per user
   - Passwords encrypted (base64 for now, should use AES-256 in production)
   - Primary account flagging

2. **email_drafts**
   - Stores email drafts per user/account
   - Supports to/cc/bcc, subject, body, attachments

3. **email_folders**
   - Caches IMAP folder structure and counts
   - Tracks unread/total counts per folder

4. **user_preferences**
   - Stores user preferences (theme, notifications, etc.)
   - JSON-based flexible storage

5. **user_login_tokens**
   - Session token management
   - Tracks device, IP, expiration
   - Supports token revocation

## API Endpoints

### Email Account Management

```
GET  /api/email/accounts              - List user's email accounts
POST /api/email/accounts/add          - Add new email account
DELETE /api/email/accounts/{id}       - Delete email account
```

### Email Operations

```
POST /api/email/list                  - List emails from account
POST /api/email/send                  - Send email
POST /api/email/draft                 - Save draft
GET  /api/email/folders/{account_id}  - List IMAP folders
```

### Request/Response Examples

#### Add Email Account
```json
POST /api/email/accounts/add
{
  "email": "user@gmail.com",
  "display_name": "John Doe",
  "imap_server": "imap.gmail.com",
  "imap_port": 993,
  "smtp_server": "smtp.gmail.com",
  "smtp_port": 587,
  "username": "user@gmail.com",
  "password": "app_password_here",
  "is_primary": true
}
```

#### List Emails
```json
POST /api/email/list
{
  "account_id": "uuid-here",
  "folder": "INBOX",
  "limit": 50,
  "offset": 0
}
```

#### Send Email
```json
POST /api/email/send
{
  "account_id": "uuid-here",
  "to": "recipient@example.com",
  "cc": "cc@example.com",
  "bcc": "bcc@example.com",
  "subject": "Test Email",
  "body": "Email body content",
  "is_html": false
}
```

## Frontend Components

### Account Management (`account.html`)

- Profile management
- Email account configuration
- Drive settings
- Security (password change, active sessions)

Features:
- Add/edit/delete email accounts
- Test IMAP/SMTP connections
- Set primary account
- Provider presets (Gmail, Outlook, Yahoo)

### Mail Client (`mail/mail.html`, `mail/mail.js`)

- Multi-account support
- Folder navigation (Inbox, Sent, Drafts, etc.)
- Compose, reply, forward emails
- Real-time email loading from IMAP
- Read/unread tracking
- Email deletion

### Drive (`drive/drive.html`, `drive/drive.js`)

- Already supports multi-user through bucket isolation
- Connected to MinIO/S3 backend
- File browser with upload/download
- Folder creation and navigation

## Usage Flow

### 1. User Registration/Login (TODO)

```javascript
// Register new user
POST /api/auth/register
{
  "username": "john",
  "email": "john@example.com",
  "password": "secure_password"
}

// Login
POST /api/auth/login
{
  "username": "john",
  "password": "secure_password"
}
// Returns: { token: "jwt_token", user_id: "uuid" }
```

### 2. Add Email Account

1. Navigate to Account Settings
2. Click "Email Accounts" tab
3. Click "Add Account"
4. Fill in IMAP/SMTP details
5. Test connection (optional)
6. Save

### 3. Use Mail Client

1. Navigate to Mail section
2. Select account (if multiple)
3. View emails from selected account
4. Compose/send emails using selected account

### 4. Drive Access

1. Navigate to Drive section
2. Files are automatically scoped to user
3. Upload/download/manage files

## Security Considerations

### Current Implementation

- Passwords stored with base64 encoding (TEMPORARY)
- Session tokens in database
- HTTPS recommended for production

### Production Requirements

1. **Encryption**
   - Replace base64 with AES-256-GCM for password encryption
   - Use encryption key from environment variable
   - Rotate keys periodically

2. **Authentication**
   - Implement JWT token-based authentication
   - Add middleware to verify tokens on protected routes
   - Implement refresh tokens

3. **Rate Limiting**
   - Add rate limiting on login attempts
   - Rate limit email sending
   - Rate limit API calls per user

4. **CSRF Protection**
   - Implement CSRF tokens for state-changing operations
   - Use SameSite cookies

5. **Input Validation**
   - Validate all email addresses
   - Sanitize email content (prevent XSS)
   - Validate IMAP/SMTP server addresses

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/botserver

# Email (global fallback)
EMAIL_IMAP_SERVER=imap.example.com
EMAIL_IMAP_PORT=993
EMAIL_SMTP_SERVER=smtp.example.com
EMAIL_SMTP_PORT=587
EMAIL_USERNAME=default@example.com
EMAIL_PASSWORD=password

# Drive
DRIVE_SERVER=minio:9000
DRIVE_ACCESSKEY=minioadmin
DRIVE_SECRET=minioadmin

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

## Email Provider Configuration

### Gmail
- IMAP: `imap.gmail.com:993`
- SMTP: `smtp.gmail.com:587`
- Note: Enable "Less secure app access" or use App Password

### Outlook/Office 365
- IMAP: `outlook.office365.com:993`
- SMTP: `smtp.office365.com:587`
- Note: Modern auth supported

### Yahoo Mail
- IMAP: `imap.mail.yahoo.com:993`
- SMTP: `smtp.mail.yahoo.com:587`
- Note: Requires app-specific password

### Custom IMAP/SMTP
- Supports any standard IMAP/SMTP server
- SSL/TLS on standard ports (993/587)

## Testing

### Manual Testing

1. Add email account through UI
2. Test connection
3. List emails (should see recent emails)
4. Send test email
5. Check sent folder
6. Save draft
7. Delete email

### API Testing with cURL

```bash
# List accounts
curl http://localhost:8080/api/email/accounts

# Add account
curl -X POST http://localhost:8080/api/email/accounts/add \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@gmail.com",
    "imap_server": "imap.gmail.com",
    "imap_port": 993,
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 587,
    "username": "test@gmail.com",
    "password": "app_password",
    "is_primary": true
  }'

# List emails
curl -X POST http://localhost:8080/api/email/list \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": "account-uuid-here",
    "folder": "INBOX",
    "limit": 10
  }'
```

## Migration

### Running Migrations

```bash
# Run new migration
diesel migration run

# Rollback if needed
diesel migration revert
```

### Migration Status

- ✅ 6.0.0 - Initial schema (users, bots, sessions)
- ✅ 6.0.1 - Bot memories
- ✅ 6.0.2 - KB tools
- ✅ 6.0.3 - KB session tables
- ✅ 6.0.4 - Config management
- ✅ 6.0.5 - Automation updates
- ✅ 6.0.6 - User accounts (email, preferences, tokens) **NEW**

## TODO - Future Enhancements

### Authentication System
- [ ] Implement JWT token generation
- [ ] Add login/logout endpoints
- [ ] Add registration endpoint with email verification
- [ ] Add password reset flow
- [ ] Implement OAuth2 (Google, Microsoft, etc.)

### Email Features
- [ ] Attachment support (upload/download)
- [ ] HTML email composition
- [ ] Email search
- [ ] Filters and labels
- [ ] Email threading/conversations
- [ ] Push notifications for new emails

### Security
- [ ] Replace base64 with proper encryption (AES-256)
- [ ] Add 2FA support
- [ ] Implement rate limiting
- [ ] Add audit logging
- [ ] Session timeout handling

### Drive Features
- [ ] Per-user storage quotas
- [ ] File sharing with permissions
- [ ] File versioning
- [ ] Trash/restore functionality
- [ ] Search across files

### UI/UX
- [ ] Better error messages
- [ ] Loading states
- [ ] Progress indicators for uploads
- [ ] Drag and drop file upload
- [ ] Email preview without opening
- [ ] Keyboard shortcuts

## Troubleshooting

### Common Issues

1. **Cannot connect to IMAP server**
   - Check firewall rules
   - Verify IMAP server address and port
   - Ensure SSL/TLS is supported
   - Check if "less secure apps" is enabled (Gmail)

2. **Email sending fails**
   - Verify SMTP credentials
   - Check SMTP port (587 for STARTTLS, 465 for SSL)
   - Some providers require app-specific passwords

3. **Password encryption errors**
   - Ensure base64 encoding/decoding is working
   - Plan migration to proper encryption

4. **No emails loading**
   - Check if account is active
   - Verify IMAP folder name (case-sensitive)
   - Check database for account record

## Contributing

When adding features to the multi-user system:

1. Update database schema with migrations
2. Add corresponding Diesel table definitions
3. Implement backend API endpoints
4. Update frontend components
5. Add to this documentation
6. Test with multiple users
7. Consider security implications

## License

Same as BotServer - AGPL-3.0