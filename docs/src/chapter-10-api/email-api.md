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

## See Also

- [SEND MAIL Keyword](../chapter-06-gbdialog/keyword-send-mail.md) - BASIC email
- [CREATE DRAFT Keyword](../chapter-06-gbdialog/keyword-create-draft.md) - Draft creation
- [External Services](../appendix-external-services/README.md) - Service configuration