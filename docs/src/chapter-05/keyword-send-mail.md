# SEND_MAIL

Send email messages from within bot conversations.

## Syntax

```basic
SEND_MAIL to, subject, body
```

or with attachments:

```basic
SEND_MAIL to, subject, body, attachments
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `to` | String/Array | Email recipient(s). Single email or array of emails |
| `subject` | String | Email subject line |
| `body` | String | Email body content (HTML supported) |
| `attachments` | Array | Optional. File paths or URLs to attach |

## Description

The `SEND_MAIL` keyword sends emails through the configured SMTP service. It supports:

- Single or multiple recipients
- HTML formatted messages
- File attachments
- CC and BCC recipients (via extended syntax)
- Email templates
- Delivery tracking
- Bounce handling

## Examples

### Simple Email
```basic
SEND_MAIL "user@example.com", "Welcome", "Thanks for signing up!"
```

### Multiple Recipients
```basic
recipients = ["alice@example.com", "bob@example.com"]
SEND_MAIL recipients, "Team Update", "Meeting tomorrow at 3pm"
```

### HTML Email
```basic
body = "<h1>Invoice</h1><p>Amount due: <b>$100</b></p>"
SEND_MAIL customer_email, "Invoice #123", body
```

### Email with Attachments
```basic
files = ["report.pdf", "data.xlsx"]
SEND_MAIL manager_email, "Monthly Report", "Please find attached reports", files
```

### Template-Based Email
```basic
' Load email template
template = LOAD_TEMPLATE("welcome_email")
body = FORMAT(template, customer_name, account_id)
SEND_MAIL customer_email, "Welcome to Our Service", body
```

### Conditional Email
```basic
IF order_total > 1000 THEN
    subject = "Large Order Alert"
    body = "New order over $1000: Order #" + order_id
    SEND_MAIL "sales@example.com", subject, body
END IF
```

## Advanced Usage

### Email with CC and BCC
```basic
' Using extended parameters
email_data = CREATE_MAP()
email_data["to"] = "primary@example.com"
email_data["cc"] = ["manager@example.com", "team@example.com"]
email_data["bcc"] = "archive@example.com"
email_data["subject"] = "Project Status"
email_data["body"] = status_report

SEND_MAIL_EXTENDED email_data
```

### Bulk Email with Personalization
```basic
customers = GET_CUSTOMER_LIST()
FOR EACH customer IN customers
    subject = "Special Offer for " + customer.name
    body = FORMAT("Hi {name}, your discount code is {code}", 
                  customer.name, customer.discount_code)
    SEND_MAIL customer.email, subject, body
    WAIT 1  ' Rate limiting
NEXT
```

### Email with Dynamic Attachments
```basic
' Generate report
report = GENERATE_REPORT(date_range)
report_file = SAVE_TEMP_FILE(report, "report.pdf")

' Send with attachment
SEND_MAIL recipient, "Daily Report", "See attached", [report_file]

' Cleanup
DELETE_TEMP_FILE(report_file)
```

## Configuration

Email settings in `config.csv`:

```csv
smtpHost,smtp.gmail.com
smtpPort,587
smtpUser,bot@example.com
smtpPassword,app-password
smtpFrom,Bot <noreply@example.com>
smtpUseTls,true
```

## Return Value

Returns an email status object:
- `message_id`: Unique message identifier
- `status`: "sent", "queued", "failed"
- `timestamp`: Send timestamp
- `error`: Error message if failed

## Error Handling

Common errors and solutions:

| Error | Cause | Solution |
|-------|-------|----------|
| Authentication failed | Invalid credentials | Check SMTP password |
| Connection timeout | Network issue | Verify SMTP host/port |
| Invalid recipient | Bad email format | Validate email addresses |
| Attachment not found | Missing file | Check file paths |
| Size limit exceeded | Large attachments | Compress or link files |

## Email Validation

Emails are validated before sending:
- Format check (RFC 5322)
- Domain verification
- Blacklist checking
- Bounce history

## Delivery Tracking

Track email delivery:

```basic
result = SEND_MAIL recipient, subject, body
IF result.status = "sent" THEN
    LOG "Email sent: " + result.message_id
    TRACK_DELIVERY result.message_id
ELSE
    LOG "Email failed: " + result.error
    RETRY_EMAIL result
END IF
```

## Templates

Use predefined email templates:

```basic
' Templates stored in mybot.gbai/templates/
template = LOAD_EMAIL_TEMPLATE("invoice")
body = FILL_TEMPLATE(template, invoice_data)
SEND_MAIL customer_email, "Invoice", body
```

## Security Considerations

1. **Never hardcode credentials** - Use environment variables
2. **Validate recipients** - Prevent email injection
3. **Sanitize content** - Escape HTML in user input
4. **Rate limit sends** - Prevent spam
5. **Use authentication** - Enable SMTP auth
6. **Encrypt connections** - Use TLS/SSL
7. **Monitor bounces** - Handle invalid addresses
8. **Implement unsubscribe** - Honor opt-outs

## Best Practices

1. **Use templates**: Maintain consistent formatting
2. **Test emails**: Send test emails before production
3. **Handle failures**: Implement retry logic
4. **Log sends**: Keep audit trail
5. **Personalize content**: Use recipient name
6. **Mobile-friendly**: Use responsive HTML
7. **Plain text fallback**: Include text version
8. **Unsubscribe link**: Always provide opt-out
9. **SPF/DKIM**: Configure domain authentication
10. **Monitor reputation**: Track delivery rates

## Rate Limiting

Prevent overwhelming SMTP server:

```basic
email_queue = []
FOR EACH email IN email_queue
    SEND_MAIL email.to, email.subject, email.body
    WAIT 2  ' 2 second delay between sends
NEXT
```

## Attachments

Supported attachment types:
- Documents: PDF, DOC, DOCX, TXT
- Spreadsheets: XLS, XLSX, CSV
- Images: PNG, JPG, GIF
- Archives: ZIP, RAR
- Any file under size limit

Size limits:
- Single file: 10MB default
- Total attachments: 25MB default

## Integration

Integrates with:
- SMTP servers (Gmail, Outlook, SendGrid)
- Email tracking services
- Template engines
- File storage (MinIO)
- Analytics systems

## Performance

- Async sending for non-blocking operation
- Queue management for bulk sends
- Connection pooling for efficiency
- Automatic retry on failure

## Monitoring

Track email metrics:
- Send rate
- Delivery rate
- Bounce rate
- Open rate (if tracking enabled)
- Click rate (for links)

## Implementation

Located in `src/basic/keywords/send_mail.rs`

Uses the email module for SMTP operations and supports multiple email providers.