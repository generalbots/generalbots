# Email API

The Email API provides endpoints for email operations including sending, receiving, and managing email accounts.

## Status

**Not Implemented** - Email functionality exists in BotServer through the email module and BASIC keywords, but REST API endpoints are not yet implemented.

## Current Email Functionality

Email features are available through:

1. **BASIC Script Keywords**
   ```basic
   SEND MAIL "recipient@example.com", "Subject", "Body"
   ```

2. **Email Module** (when feature enabled)
   - IMAP/SMTP integration
   - Database storage for email accounts
   - Draft management

## Planned Endpoints

### Send Email

**POST** `/api/email/send` (Planned)

Send an email message.

### List Emails

**GET** `/api/email/inbox` (Planned)

Retrieve inbox messages.

### Get Email

**GET** `/api/email/:id` (Planned)

Get specific email details.

### Delete Email

**DELETE** `/api/email/:id` (Planned)

Delete an email message.

### Email Accounts

**GET** `/api/email/accounts` (Planned)

List configured email accounts.

**POST** `/api/email/accounts` (Planned)

Add new email account.

## Current Implementation

Email functionality requires:

1. **Feature Flag**
   ```bash
   cargo build --features email
   ```

2. **Environment Configuration**
   ```bash
   EMAIL_IMAP_SERVER=imap.gmail.com
   EMAIL_IMAP_PORT=993
   EMAIL_USERNAME=your-email@example.com
   EMAIL_PASSWORD=your-app-password
   EMAIL_SMTP_SERVER=smtp.gmail.com
   EMAIL_SMTP_PORT=587
   ```

3. **BASIC Scripts**
   - Use SEND MAIL keyword
   - Process emails in automation

## Database Schema

Email data is stored in:
- `user_email_accounts` - Email account configurations
- `email_drafts` - Draft emails
- `email_folders` - Folder organization

## Using Email Today

To use email functionality:

1. **Enable Feature**
   - Build with email feature flag
   - Configure IMAP/SMTP settings

2. **Use in BASIC Scripts**
   ```basic
   # Send email
   SEND MAIL "user@example.com", "Hello", "Message body"
   
   # Automated email processing
   let emails = GET_EMAILS("INBOX", "UNSEEN")
   FOR EACH email IN emails {
       # Process email
   }
   ```

## Future Implementation

When implemented, the Email API will provide:
- RESTful email operations
- Attachment handling
- Template management
- Batch operations
- Webhook notifications

## Summary

While email functionality exists in BotServer through the email module and BASIC scripting, REST API endpoints are not yet implemented. Use BASIC scripts for email automation until the API is available.