# Email Integration

botserver provides email integration capabilities through IMAP/SMTP protocols, allowing bots to read, send, and manage emails.

## Overview

Email integration in botserver enables reading emails via IMAP, sending emails via SMTP, email account management, draft creation and management, folder organization, and email-based automation workflows.

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

The email module is located in `src/email/` and contains `mod.rs` with the email service implementation, account management functionality, message handling logic, and IMAP/SMTP client implementations.

## Database Schema

### User Email Accounts

The `user_email_accounts` table stores email account configurations with encrypted password storage. Users can configure multiple accounts, each with its own IMAP and SMTP server details.

### Email Drafts

The `email_drafts` table provides draft management including To, CC, and BCC addresses, subject and body content, attachment metadata, and auto-save support for work in progress.

### Email Folders

The `email_folders` table handles folder organization with IMAP folder mapping, message counts, unread tracking, and hierarchical structure support for nested folders.

## BASIC Keywords for Email

### SEND MAIL

Send emails from BASIC scripts:

```basic
SEND MAIL "recipient@example.com", "Subject", "Email body content"

# With variables
let to = "user@example.com"
let subject = "Meeting Reminder"
let body = "Don't forget our meeting at 2 PM"
SEND MAIL to, subject, body
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
        SEND MAIL sender, "Re: " + subject, "I'll get back to you soon."
    }
}
```

## Email Operations

### Reading Emails

The system can connect to IMAP servers, fetch message headers, download full messages, search by various criteria, mark messages as read or unread, and move messages between folders.

### Sending Emails

SMTP operations include authentication with the mail server, sending plain text and HTML emails, reply and forward functionality, and bulk sending with configurable limits. Attachment support is planned for a future release.

## Security

### Password Storage

Email passwords are encrypted using AES-GCM and never stored in plaintext. Passwords are decrypted only when needed for authentication and memory is cleared after use to prevent credential leakage.

### Connection Security

All email connections require TLS/SSL encryption with proper certificate validation. Secure authentication methods are enforced, and plaintext transmission is never permitted.

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
    SEND MAIL email.from, "Ticket Created: " + email.subject, response
}
```

### Newsletter Distribution

```basic
# Send newsletter to subscribers
let subscribers = GET "subscribers.csv"
let newsletter = GET "newsletter.html"

FOR EACH subscriber IN subscribers {
    SEND MAIL subscriber.email, "Monthly Newsletter", newsletter
    WAIT 1  # Rate limiting
}
```

### Email-to-Task Conversion

```basic
# Convert emails to tasks
let task_emails = GET_EMAILS("tasks", "UNSEEN")

FOR EACH email IN task_emails {
    CREATE TASK email.subject, email.body, email.from
    MARK_AS_READ email
}
```

## Integration with Other Features

### With Calendar

Email integrates with the calendar system for meeting invitations, event reminders, and schedule updates sent via email notifications.

### With Tasks

Task integration enables task creation from emails, status updates delivered via email, and deadline reminders sent to responsible parties.

### With Knowledge Base

Knowledge base integration supports email archival for compliance, searchable email history, and providing email context for bot conversations.

## Limitations

### Current Limitations

The current implementation does not support attachment handling, provides only basic HTML email support, lacks email templates, has limited filtering options, and does not support OAuth2 authentication, requiring app-specific passwords instead.

### Rate Limiting

Provider-specific rate limits apply to all email operations. Implement delays between sends to avoid throttling, monitor for rate limit errors, and use batch operations wisely to stay within provider limits.

## Email Provider Setup

### Gmail Configuration

To configure Gmail, first enable 2-factor authentication on your Google account. Then generate an app-specific password for botserver to use. Enable IMAP access in Gmail settings. Use `imap.gmail.com` on port 993 for IMAP and `smtp.gmail.com` on port 587 for SMTP.

### Outlook/Office 365

For Outlook or Office 365, enable IMAP in your account settings. If 2FA is enabled, generate an app password. Use `outlook.office365.com` on port 993 for IMAP and `smtp.office365.com` on port 587 for SMTP.

### Custom Email Servers

For custom email servers, configure the appropriate server addresses, port numbers, security settings including TLS or SSL requirements, and the authentication method supported by your server.

## Error Handling

### Connection Errors

```basic
# Handle email errors
status = SEND MAIL recipient, subject, body
IF status = "sent" THEN
    TALK "Email sent successfully"
ELSE
    TALK "Failed to send email: " + status
    # Log error for admin
END IF
```

### Common Issues

Common email issues include authentication failures from incorrect credentials, network timeouts when servers are slow to respond, server unavailable errors during outages, quota exceeded errors when hitting send limits, and invalid address errors for malformed recipients.

## Best Practices

Use app-specific passwords rather than primary account passwords to limit security exposure. Respect provider rate limits by implementing appropriate delays between operations. Implement retry logic for transient failures to ensure delivery. Validate email addresses before sending to catch format errors early. Monitor usage by tracking sent and received counts. Encrypt sensitive data in storage and transit. Maintain an audit trail by logging all email operations.

## Monitoring

### Metrics to Track

Key metrics include emails sent and received, failed operations and their causes, connection failures, processing time for email operations, and queue size when batching sends.

### Health Checks

Regular health checks should verify IMAP connectivity, SMTP availability, account validity and credential freshness, and folder synchronization status.

## Summary

Email integration in botserver enables powerful email-based automation and communication. Through IMAP/SMTP protocols and BASIC script integration, bots can manage email workflows, automate responses, and integrate email with other bot features for comprehensive communication automation.