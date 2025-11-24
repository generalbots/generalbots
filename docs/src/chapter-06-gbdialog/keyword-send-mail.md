# SEND MAIL

Send email messages with optional attachments and HTML formatting.

## Syntax

```basic
SEND MAIL to, subject, body
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `to` | String | Recipient email address(es), comma-separated for multiple |
| `subject` | String | Email subject line |
| `body` | String | Email body (plain text or HTML) |

## Description

The `SEND MAIL` keyword sends emails using the SMTP configuration defined in `config.csv`. It supports:

- Plain text and HTML emails
- Multiple recipients
- CC and BCC (via extended syntax)
- File attachments
- Email templates
- Delivery tracking

## Configuration

Email settings in `config.csv`:

```csv
name,value
email-from,noreply@example.com
email-server,smtp.example.com
email-port,587
email-user,smtp-user@example.com
email-pass,smtp-password
```

## Examples

### Simple Text Email
```basic
SEND MAIL "user@example.com", "Welcome!", "Thank you for signing up."
```

### Multiple Recipients
```basic
recipients = "john@example.com, jane@example.com, bob@example.com"
SEND MAIL recipients, "Team Update", "Meeting tomorrow at 3 PM"
```

### HTML Email
```basic
body = "<h1>Welcome!</h1><p>Thank you for joining us.</p>"
body = body + "<ul><li>Step 1: Complete profile</li>"
body = body + "<li>Step 2: Verify email</li></ul>"
SEND MAIL "user@example.com", "Getting Started", body
```

### Dynamic Content
```basic
order_id = GET "order_id"
subject = "Order #" + order_id + " Confirmation"
body = "Hello " + user_name + ", your order has been confirmed."
SEND MAIL user_email, subject, body
```

### With Error Handling
```basic
email = HEAR "Enter your email address:"
IF email CONTAINS "@" AND email CONTAINS "." THEN
    SEND MAIL email, "Verification Code", "Your code is: 123456"
    TALK "Email sent successfully!"
ELSE
    TALK "Invalid email address"
END IF
```

### Notification System
```basic
' Send notification to admin when issue detected
admin_email = GET BOT MEMORY "admin_email"
IF error_detected THEN
    subject = "Bot Alert"
    body = "Issue detected at " + NOW()
    SEND MAIL admin_email, subject, body
END IF
```

### Bulk Email with Personalization
```basic
subscribers = GET "subscribers.list"
FOR EACH email IN subscribers
    body = "Dear " + username + ", here's your weekly update..."
    SEND MAIL email, "Weekly Newsletter", body
    WAIT 1  ' Rate limiting
NEXT
```

## Extended Syntax

### With CC and BCC
```basic
' Using structured format
email_data = {
    "to": "primary@example.com",
    "cc": "copy@example.com",
    "bcc": "hidden@example.com",
    "subject": "Report",
    "body": "Please review attached report."
}
SEND MAIL email_data
```

### With Attachments
```basic
' Attach file from drive
email_data = {
    "to": "user@example.com",
    "subject": "Invoice",
    "body": "Please find invoice attached.",
    "attachments": ["invoice.pdf"]
}
SEND MAIL email_data
```

### Using Templates
```basic
' Load and fill template
template = LOAD_TEMPLATE "welcome_email"
template = REPLACE(template, "{{name}}", user_name)
template = REPLACE(template, "{{date}}", TODAY())
SEND MAIL user_email, "Welcome!", template
```

## Email Validation

Always validate email addresses before sending:

```basic
email = HEAR "Your email:"
IF email CONTAINS "@" AND email CONTAINS "." THEN
    parts = SPLIT(email, "@")
    IF LENGTH(parts) = 2 THEN
        domain = parts[1]
        IF domain CONTAINS "." THEN
            SEND MAIL email, "Test", "This is a test"
        ELSE
            TALK "Please enter a valid email"
        END IF
    ELSE
        TALK "Please enter a valid email"
    END IF
ELSE
    TALK "Please enter a valid email"
END IF
```

## Delivery Status

Check email delivery status:

```basic
status = SEND MAIL "user@example.com", "Test", "Message"
IF status = "sent" THEN
    TALK "Email delivered successfully"
ELSE IF status = "queued" THEN
    TALK "Email queued for delivery"
ELSE
    TALK "Email delivery failed: " + status
END IF
```

## Rate Limiting

Implement rate limiting to avoid spam:

```basic
last_sent = GET BOT MEMORY "last_email_time"
IF TIME_DIFF(NOW(), last_sent) < 60 THEN
    TALK "Please wait before sending another email"
ELSE
    SEND MAIL email, subject, body
    SET BOT MEMORY "last_email_time", NOW()
END IF
```

## Error Handling

Common error scenarios:

```basic
' Check email format before sending
IF recipient CONTAINS "@" AND recipient CONTAINS "." THEN
    status = SEND MAIL recipient, subject, body
    IF status = "sent" THEN
        TALK "Email sent successfully"
    ELSE IF status = "smtp_error" THEN
        TALK "Email server is unavailable"
    ELSE IF status = "auth_error" THEN
        TALK "Email authentication failed"
        LOG "Check SMTP credentials in config.csv"
    ELSE
        TALK "Failed to send email: " + status
    END IF
ELSE
    TALK "The email address is invalid"
END IF
```

## Best Practices

1. **Validate Recipients**: Always validate email addresses
2. **Rate Limit**: Implement delays for bulk emails
3. **Handle Failures**: Use try-catch for error handling
4. **Log Attempts**: Keep records of sent emails
5. **Test Configuration**: Verify SMTP settings before production
6. **Use Templates**: Maintain consistent formatting
7. **Respect Privacy**: Use BCC for multiple recipients
8. **Include Unsubscribe**: Add opt-out links for marketing emails

## Security Considerations

- Never log email passwords
- Use environment variables for sensitive data
- Implement SPF, DKIM, and DMARC for deliverability
- Sanitize user input in email bodies
- Use TLS/SSL for SMTP connections

## Troubleshooting

### Email Not Sending

1. Check SMTP configuration in `config.csv`
2. Verify firewall allows port 587/465
3. Test credentials manually
4. Check email server logs

### Authentication Failed

Check SMTP configuration:
1. Verify credentials in `config.csv`
2. Ensure SMTP server allows your connection
3. Check if port 587/465 is open
4. Verify TLS/SSL settings match server requirements

### Emails Going to Spam

- Set proper FROM address
- Include text version with HTML
- Avoid spam trigger words
- Configure domain authentication (SPF/DKIM)

## Related Keywords

- [GET](./keyword-get.md) - Retrieve user data for emails
- [FORMAT](./keyword-format.md) - Format email content
- [WAIT](./keyword-wait.md) - Rate limiting between emails
- [SET SCHEDULE](./keyword-set-schedule.md) - Schedule email sending

## Implementation

Located in `src/basic/keywords/send_mail.rs`

The implementation uses:
- `lettre` crate for SMTP
- Async email sending
- Connection pooling for performance
- Retry logic for failed attempts
- HTML sanitization for security
