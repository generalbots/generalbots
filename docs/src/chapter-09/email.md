# Email Integration

BotServer provides email integration capabilities through IMAP/SMTP protocols, allowing bots to read, send, and manage emails.

## Overview

Email integration in BotServer enables:
- Reading emails via IMAP
- Sending emails via SMTP
- Email account management
- Draft creation and management
- Folder organization
- Email-based automation

## Configuration

### Environment Variables

Email configuration requires these environment variables:

```bash
# IMAP Settings
EMAIL_IMAP_SERVER=imap.gmail.com
EMAIL_IMAP_PORT=993
EMAIL_USERNAME=your-email@example.com
EMAIL_PASSWORD=your-app-password

# SMTP Settings
EMAIL_SMTP_SERVER=smtp.gmail.com
EMAIL_SMTP_PORT=587
EMAIL_FROM=your-email@example.com
```

### Feature Flag

Email functionality requires the `email` feature flag during compilation:
```bash
cargo build --features email
```

## Email Module Structure

Located in `src/email/`:
- `mod.rs` - Email service implementation
- Account management
- Message handling
- IMAP/SMTP clients

## Database Schema

### User Email Accounts

Stores email account configurations:
- `user_email_accounts` table
- Encrypted password storage
- Multiple accounts per user
- IMAP/SMTP server details

### Email Drafts

Draft management:
- `email_drafts` table
- To/CC/BCC addresses
- Subject and body
- Attachment metadata
- Auto-save support

### Email Folders

Folder organization:
- `email_folders` table
- IMAP folder mapping
- Message counts
- Unread tracking
- Hierarchical structure

## BASIC Keywords for Email

### SEND_MAIL

Send emails from BASIC scripts:

```basic
SEND_MAIL "recipient@example.com", "Subject", "Email body content"

# With variables
let to = "user@example.com"
let subject = "Meeting Reminder"
let body = "Don't forget our meeting at 2 PM"
SEND_MAIL to, subject, body
```

### Email Automation

```basic
# Check for new emails
let new_emails = GET_EMAILS("INBOX", "UNSEEN")

# Process each email
FOR EACH email IN new_emails {
    let sender = email.from
    let subject = email.subject
    let body = email.body
    
    # Auto-reply logic
    if (subject CONTAINS "urgent") {
        SEND_MAIL sender, "Re: " + subject, "I'll get back to you soon."
    }
}
```

## Email Operations

### Reading Emails

The system can:
- Connect to IMAP servers
- Fetch message headers
- Download full messages
- Search by criteria
- Mark as read/unread
- Move between folders

### Sending Emails

SMTP operations include:
- Authentication
- Plain text emails
- HTML emails
- Attachments (planned)
- Reply/forward
- Bulk sending (with limits)

## Security

### Password Storage

- Passwords encrypted using AES-GCM
- Never stored in plaintext
- Decrypted only when needed
- Memory cleared after use

### Connection Security

- TLS/SSL required
- Certificate validation
- Secure authentication methods
- No plaintext transmission

## Use Cases

### Support Ticket System

```basic
# Monitor support inbox
let support_emails = GET_EMAILS("support", "UNSEEN")

FOR EACH email IN support_emails {
    # Create ticket
    let ticket_id = CREATE_TICKET(email.from, email.subject, email.body)
    
    # Send confirmation
    let response = "Ticket #" + ticket_id + " created. We'll respond within 24 hours."
    SEND_MAIL email.from, "Ticket Created: " + email.subject, response
}
```

### Newsletter Distribution

```basic
# Send newsletter to subscribers
let subscribers = GET "subscribers.csv"
let newsletter = GET "newsletter.html"

FOR EACH subscriber IN subscribers {
    SEND_MAIL subscriber.email, "Monthly Newsletter", newsletter
    WAIT 1  # Rate limiting
}
```

### Email-to-Task Conversion

```basic
# Convert emails to tasks
let task_emails = GET_EMAILS("tasks", "UNSEEN")

FOR EACH email IN task_emails {
    CREATE_TASK email.subject, email.body, email.from
    MARK_AS_READ email
}
```

## Integration with Other Features

### With Calendar

- Meeting invitations
- Event reminders
- Schedule updates

### With Tasks

- Task creation from emails
- Status updates via email
- Deadline reminders

### With Knowledge Base

- Email archival
- Searchable email history
- Context for conversations

## Limitations

### Current Limitations

- No attachment handling yet
- Basic HTML email support
- No email templates
- Limited filtering options
- No OAuth2 support (requires app passwords)

### Rate Limiting

- Provider-specific limits apply
- Implement delays between sends
- Monitor for throttling
- Use batch operations wisely

## Email Provider Setup

### Gmail Configuration

1. Enable 2-factor authentication
2. Generate app-specific password
3. Enable IMAP access
4. Use these settings:
   - IMAP: imap.gmail.com:993
   - SMTP: smtp.gmail.com:587

### Outlook/Office 365

1. Enable IMAP in settings
2. Use app password if 2FA enabled
3. Settings:
   - IMAP: outlook.office365.com:993
   - SMTP: smtp.office365.com:587

### Custom Email Servers

Configure with appropriate:
- Server addresses
- Port numbers
- Security settings (TLS/SSL)
- Authentication method

## Error Handling

### Connection Errors

```basic
# Handle email errors
try {
    SEND_MAIL recipient, subject, body
    TALK "Email sent successfully"
} catch (error) {
    TALK "Failed to send email: " + error
    # Log error for admin
}
```

### Common Issues

- Authentication failures
- Network timeouts
- Server unavailable
- Quota exceeded
- Invalid addresses

## Best Practices

1. **Use App Passwords**: Never use primary account passwords
2. **Rate Limit**: Respect provider limits
3. **Error Recovery**: Implement retry logic
4. **Validate Addresses**: Check format before sending
5. **Monitor Usage**: Track sent/received counts
6. **Secure Storage**: Encrypt sensitive data
7. **Audit Trail**: Log email operations

## Monitoring

### Metrics to Track

- Emails sent/received
- Failed operations
- Connection failures
- Processing time
- Queue size

### Health Checks

- IMAP connectivity
- SMTP availability
- Account validity
- Folder synchronization

## Future Enhancements

Planned improvements:
- Full attachment support
- Email templates
- OAuth2 authentication
- Rich HTML editor
- Email scheduling
- Advanced filtering
- Spam detection
- Email analytics

## Summary

Email integration in BotServer enables powerful email-based automation and communication. Through IMAP/SMTP protocols and BASIC script integration, bots can manage email workflows, automate responses, and integrate email with other bot features.