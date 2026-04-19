# Email API

The Email API provides endpoints for email operations including sending, receiving, and managing email accounts through the Stalwart mail server integration.

## Overview

Email functionality in General Bots is available through:

1. **REST API** - Documented in this chapter
2. **BASIC Keywords** - `SEND MAIL` for scripts
3. **Email Module** - Background processing and IMAP/SMTP integration

## Endpoints

### Send Email

**POST** `/api/email/send`

Send an email message.

**Request:**
```json
{
  "to": ["recipient@example.com"],
  "cc": ["cc@example.com"],
  "bcc": [],
  "subject": "Meeting Tomorrow",
  "body": "Hi, just a reminder about our meeting.",
  "body_type": "text",
  "attachments": []
}
```

**Response:**
```json
{
  "message_id": "msg-abc123",
  "status": "sent",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Body Types:**
- `text` - Plain text
- `html` - HTML formatted

### List Emails

**GET** `/api/email/inbox`

Retrieve inbox messages.

**Query Parameters:**
- `folder` - Folder name (default: INBOX)
- `limit` - Number of messages (default: 50)
- `offset` - Pagination offset
- `unread` - Filter unread only (boolean)
- `since` - Messages since date (ISO 8601)

**Response:**
```json
{
  "messages": [
    {
      "id": "email-001",
      "from": "sender@example.com",
      "subject": "Hello",
      "preview": "Just wanted to say hi...",
      "date": "2024-01-15T09:00:00Z",
      "read": false,
      "has_attachments": false
    }
  ],
  "total": 142,
  "unread_count": 5
}
```

### Get Email

**GET** `/api/email/:id`

Get specific email details.

**Response:**
```json
{
  "id": "email-001",
  "from": {
    "name": "John Doe",
    "email": "john@example.com"
  },
  "to": [
    {
      "name": "You",
      "email": "you@example.com"
    }
  ],
  "cc": [],
  "subject": "Meeting Notes",
  "body": "Here are the notes from today's meeting...",
  "body_html": "<p>Here are the notes from today's meeting...</p>",
  "date": "2024-01-15T09:00:00Z",
  "read": true,
  "attachments": [
    {
      "id": "att-001",
      "filename": "notes.pdf",
      "size": 102400,
      "content_type": "application/pdf"
    }
  ]
}
```

### Delete Email

**DELETE** `/api/email/:id`

Delete an email message.

**Response:**
```json
{
  "status": "deleted",
  "message_id": "email-001"
}
```

### Get Attachment

**GET** `/api/email/:id/attachments/:attachment_id`

Download an email attachment.

**Response:** Binary file with appropriate Content-Type header.

### Mark as Read

**PUT** `/api/email/:id/read`

Mark email as read.

**Request:**
```json
{
  "read": true
}
```

### Move Email

**PUT** `/api/email/:id/move`

Move email to a different folder.

**Request:**
```json
{
  "folder": "Archive"
}
```

### List Folders

**GET** `/api/email/folders`

List available email folders.

**Response:**
```json
{
  "folders": [
    {
      "name": "INBOX",
      "path": "INBOX",
      "unread_count": 5,
      "total_count": 142
    },
    {
      "name": "Sent",
      "path": "Sent",
      "unread_count": 0,
      "total_count": 89
    },
    {
      "name": "Drafts",
      "path": "Drafts",
      "unread_count": 0,
      "total_count": 3
    }
  ]
}
```

### Create Draft

**POST** `/api/email/drafts`

Create an email draft.

**Request:**
```json
{
  "to": ["recipient@example.com"],
  "subject": "Draft subject",
  "body": "Draft content..."
}
```

**Response:**
```json
{
  "draft_id": "draft-001",
  "status": "saved"
}
```

### Send Draft

**POST** `/api/email/drafts/:id/send`

Send a previously saved draft.

**Response:**
```json
{
  "message_id": "msg-abc123",
  "status": "sent"
}
```

## Email Accounts

### List Accounts

**GET** `/api/email/accounts`

List configured email accounts.

**Response:**
```json
{
  "accounts": [
    {
      "id": "account-001",
      "email": "user@example.com",
      "provider": "stalwart",
      "status": "connected"
    }
  ]
}
```

### Add Account

**POST** `/api/email/accounts`

Add a new email account.

**Request:**
```json
{
  "email": "user@example.com",
  "imap_server": "imap.example.com",
  "imap_port": 993,
  "smtp_server": "smtp.example.com",
  "smtp_port": 587,
  "username": "user@example.com",
  "password": "app-specific-password"
}
```

**Response:**
```json
{
  "account_id": "account-002",
  "status": "connected",
  "message": "Account added successfully"
}
```

## BASIC Integration

Use email in your BASIC scripts:

```basic
' Simple email
SEND MAIL "recipient@example.com", "Subject", "Body"

' With variables
TALK "Who should I email?"
recipient = HEAR

TALK "What's the subject?"
subject = HEAR

TALK "What's the message?"
body = HEAR

SEND MAIL recipient, subject, body
TALK "Email sent!"
```

## Configuration

Configure email in `config.csv`:

```csv
key,value
smtp-server,smtp.gmail.com
smtp-port,587
imap-server,imap.gmail.com
imap-port,993
email-username,your-email@gmail.com
email-password,your-app-password
email-from,Your Name <your-email@gmail.com>
```

**Gmail Configuration:**
- Use App Passwords (not your main password)
- Enable IMAP in Gmail settings
- Allow less secure apps or use OAuth

## Stalwart Mail Server

When using the built-in Stalwart mail server:

**Automatic Configuration:**
- Server runs on standard ports (25, 993, 587)
- Accounts created through Zitadel integration
- TLS certificates auto-managed

**Manual Configuration:**
```csv
key,value
stalwart-enabled,true
stalwart-domain,mail.yourdomain.com
stalwart-admin-password,secure-password
```

## Error Handling

| Status Code | Error | Description |
|-------------|-------|-------------|
| 400 | `invalid_recipient` | Invalid email address |
| 401 | `unauthorized` | Authentication required |
| 403 | `forbidden` | No access to mailbox |
| 404 | `not_found` | Email not found |
| 422 | `send_failed` | SMTP delivery failed |
| 503 | `service_unavailable` | Mail server offline |

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| Send | 100/hour per user |
| Inbox | 300/hour per user |
| Attachments | 50/hour per user |

## Email Read Tracking

General Bots supports email read tracking via an invisible 1x1 pixel embedded in HTML emails. When enabled, you can track when recipients open your emails.

### Configuration

Enable tracking in `config.csv`:

```csv
name,value
email-read-pixel,true
server-url,https://yourdomain.com
```

### How It Works

1. When sending an HTML email, a tracking pixel is automatically injected
2. When the recipient opens the email, their email client loads the pixel
3. The server records the open event with timestamp and metadata
4. You can query the tracking status via API or view in the Suite UI

### Tracking Endpoints

#### Serve Tracking Pixel

**GET** `/api/email/tracking/pixel/:tracking_id`

This endpoint is called automatically by email clients when loading the tracking pixel. It returns a 1x1 transparent GIF and records the read event.

**Response:** Binary GIF image (1x1 pixel)

**Headers Set:**
- `Content-Type: image/gif`
- `Cache-Control: no-store, no-cache, must-revalidate, max-age=0`

#### Get Tracking Status

**GET** `/api/email/tracking/status/:tracking_id`

Get the read status for a specific sent email.

**Response:**
```json
{
  "success": true,
  "data": {
    "tracking_id": "550e8400-e29b-41d4-a716-446655440000",
    "to_email": "recipient@example.com",
    "subject": "Meeting Tomorrow",
    "sent_at": "2024-01-15T10:30:00Z",
    "is_read": true,
    "read_at": "2024-01-15T14:22:00Z",
    "read_count": 3
  }
}
```

#### List Tracked Emails

**GET** `/api/email/tracking/list`

List all sent emails with their tracking status.

**Query Parameters:**
- `account_id` - Filter by email account (optional)
- `limit` - Number of results (default: 50)
- `offset` - Pagination offset (default: 0)
- `filter` - Filter by status: `all`, `read`, `unread` (default: all)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "tracking_id": "550e8400-e29b-41d4-a716-446655440000",
      "to_email": "recipient@example.com",
      "subject": "Meeting Tomorrow",
      "sent_at": "2024-01-15T10:30:00Z",
      "is_read": true,
      "read_at": "2024-01-15T14:22:00Z",
      "read_count": 3
    },
    {
      "tracking_id": "661e8400-e29b-41d4-a716-446655440001",
      "to_email": "another@example.com",
      "subject": "Project Update",
      "sent_at": "2024-01-15T11:00:00Z",
      "is_read": false,
      "read_at": null,
      "read_count": 0
    }
  ]
}
```

#### Get Tracking Statistics

**GET** `/api/email/tracking/stats`

Get aggregate statistics for email tracking.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_sent": 150,
    "total_read": 98,
    "read_rate": 65.33,
    "avg_time_to_read_hours": 4.5
  }
}
```

### Tracking Data Stored

For each tracked email, the following data is recorded:

| Field | Description |
|-------|-------------|
| `tracking_id` | Unique ID embedded in the pixel URL |
| `to_email` | Recipient email address |
| `subject` | Email subject line |
| `sent_at` | Timestamp when email was sent |
| `is_read` | Whether email has been opened |
| `read_at` | Timestamp of first open |
| `read_count` | Number of times opened |
| `first_read_ip` | IP address of first open |
| `last_read_ip` | IP address of most recent open |
| `user_agent` | Browser/client user agent string |

### Privacy Considerations

- Email tracking should be used responsibly
- Consider disclosing tracking in your email footer
- Some email clients block tracking pixels by default
- Users may have images disabled, preventing tracking
- GDPR/LGPD may require consent for tracking

### Suite UI Integration

The Suite email interface shows tracking status:

- **📊 Tracking** folder shows all tracked emails
- Green checkmarks (✓✓) indicate opened emails
- Gray checkmarks indicate sent but unread
- Hover over emails to see open timestamp
- Statistics panel shows open rates

## Security Notes

1. **Never hardcode credentials** - Use config.csv
2. **Use App Passwords** - Not main account passwords
3. **Enable TLS** - Always use encrypted connections
4. **Audit sending** - Log all outbound emails

## Database Schema

```sql
-- user_email_accounts
CREATE TABLE user_email_accounts (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    email TEXT NOT NULL,
    imap_server TEXT,
    smtp_server TEXT,
    encrypted_password TEXT,
    created_at TIMESTAMPTZ
);

-- email_drafts
CREATE TABLE email_drafts (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    recipients JSONB,
    subject TEXT,
    body TEXT,
    attachments JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

## Database Schema

```sql
-- user_email_accounts
CREATE TABLE user_email_accounts (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    email TEXT NOT NULL,
    imap_server TEXT,
    smtp_server TEXT,
    encrypted_password TEXT,
    created_at TIMESTAMPTZ
);

-- email_drafts
CREATE TABLE email_drafts (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    recipients JSONB,
    subject TEXT,
    body TEXT,
    attachments JSONB,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);

-- sent_email_tracking (for read receipts)
CREATE TABLE sent_email_tracking (
    id UUID PRIMARY KEY,
    tracking_id UUID NOT NULL UNIQUE,
    bot_id UUID NOT NULL,
    account_id UUID NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    cc TEXT,
    bcc TEXT,
    subject TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    read_count INTEGER NOT NULL DEFAULT 0,
    first_read_ip VARCHAR(45),
    last_read_ip VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

## See Also

- [SEND MAIL Keyword](../04-basic-scripting/keyword-send-mail.md) - BASIC email
- [CREATE DRAFT Keyword](../04-basic-scripting/keyword-create-draft.md) - Draft creation
- [External Services](../appendix-external-services/README.md) - Service configuration
- [Configuration Parameters](../10-configuration-deployment/parameters.md) - email-read-pixel setting