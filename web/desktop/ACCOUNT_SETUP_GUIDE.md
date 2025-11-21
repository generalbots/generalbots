# Account Setup Quick Guide

## 🚀 Quick Start

### Step 1: Run Database Migration

First, apply the new database migration to add user account tables:

```bash
cd botserver
diesel migration run
```

This creates the following tables:
- `user_email_accounts` - Store email credentials
- `email_drafts` - Save email drafts
- `email_folders` - Cache folder structure
- `user_preferences` - User settings
- `user_login_tokens` - Session management

### Step 2: Start the Server

Make sure the `email` feature is enabled (it should be by default):

```bash
cargo run --features email
```

Or if already built:

```bash
./target/release/botserver
```

### Step 3: Access Account Settings

1. Open your browser to `http://localhost:8080`
2. Click on the user avatar or settings icon
3. Navigate to "Account Settings"

## 📧 Adding Your First Email Account

### For Gmail Users

1. **Generate App Password** (Required for Gmail)
   - Go to Google Account settings
   - Security → 2-Step Verification
   - App passwords → Generate new password
   - Copy the 16-character password

2. **Add Account in BotServer**
   - Go to Account Settings → Email Accounts tab
   - Click "Add Account"
   - Fill in:
     ```
     Email: your-email@gmail.com
     Display Name: Your Name
     IMAP Server: imap.gmail.com
     IMAP Port: 993
     SMTP Server: smtp.gmail.com
     SMTP Port: 587
     Username: your-email@gmail.com
     Password: [paste app password]
     ```
   - Check "Set as primary email account"
   - Click "Add Account"

3. **Test Connection**
   - Click "Test" button
   - Should show "Connection successful"

### For Outlook/Office 365 Users

```
Email: your-email@outlook.com
IMAP Server: outlook.office365.com
IMAP Port: 993
SMTP Server: smtp.office365.com
SMTP Port: 587
Username: your-email@outlook.com
Password: [your password]
```

### For Yahoo Mail Users

**Important:** Yahoo requires app-specific password

1. Go to Yahoo Account Security
2. Generate app password
3. Use these settings:

```
Email: your-email@yahoo.com
IMAP Server: imap.mail.yahoo.com
IMAP Port: 993
SMTP Server: smtp.mail.yahoo.com
SMTP Port: 587
Username: your-email@yahoo.com
Password: [app-specific password]
```

### For Custom IMAP/SMTP Servers

```
Email: your-email@domain.com
IMAP Server: mail.domain.com
IMAP Port: 993
SMTP Server: mail.domain.com
SMTP Port: 587
Username: your-email@domain.com (or just username)
Password: [your password]
```

## 📬 Using the Mail Client

### Reading Emails

1. Navigate to Mail section (📧 icon)
2. Your emails will load automatically
3. Click on any email to read it
4. Use folders (Inbox, Sent, Drafts, etc.) to navigate

### Sending Emails

1. Click "Compose" button (✏️)
2. Fill in:
   - To: recipient@example.com
   - Subject: Your subject
   - Body: Your message
3. Click "Send"

### Multiple Accounts

If you have multiple email accounts:
1. Account dropdown appears in mail toolbar
2. Select account to view its emails
3. Composing email uses currently selected account

## 🔧 Troubleshooting

### "Failed to connect to IMAP server"

**Possible causes:**
- Incorrect server address or port
- Firewall blocking connection
- Need to enable IMAP in email provider settings
- Using regular password instead of app password

**Solutions:**
- Verify IMAP server address from your provider
- Check if IMAP is enabled in your email settings
- Use app-specific password for Gmail/Yahoo
- Try port 143 with STARTTLS if 993 fails

### "Authentication failed"

**Causes:**
- Wrong username or password
- Need app-specific password
- 2FA not configured properly

**Solutions:**
- Double-check username (often full email address)
- Generate app-specific password
- Ensure 2FA is enabled before generating app password

### "Failed to send email"

**Causes:**
- SMTP server/port incorrect
- Authentication issues
- Rate limiting

**Solutions:**
- Verify SMTP settings
- Try port 587 (STARTTLS) or 465 (SSL)
- Check if sender email matches account
- Wait and retry if rate limited

### "No emails loading"

**Causes:**
- Mailbox is empty
- Wrong folder name
- IMAP connection issue

**Solutions:**
- Try different folders (INBOX, Sent)
- Click refresh button
- Test connection in Account Settings
- Check account is marked as active

## 🔒 Security Notes

### Current Implementation

⚠️ **IMPORTANT**: Current password encryption uses base64 encoding, which is **NOT SECURE** for production use. This is temporary for development.

### For Production Deployment

You **MUST** implement proper encryption before deploying to production:

1. **Replace base64 with AES-256-GCM encryption**
   - Update `encrypt_password()` and `decrypt_password()` functions
   - Use a strong encryption key from environment variable
   - Never commit encryption keys to version control

2. **Use HTTPS/TLS**
   - All communication must be encrypted in transit
   - Configure reverse proxy (nginx/Apache) with SSL certificate

3. **Implement rate limiting**
   - Limit login attempts
   - Limit email sending rate
   - Protect against brute force attacks

4. **Use JWT tokens for authentication**
   - Implement proper session management
   - Token refresh mechanism
   - Secure token storage

5. **Regular security audits**
   - Review code for vulnerabilities
   - Update dependencies
   - Monitor for suspicious activity

## 📊 Account Management Features

### Profile Settings
- Update display name
- Change phone number
- View account creation date

### Security Settings
- Change password
- View active sessions
- Revoke sessions on other devices

### Drive Settings
- View storage usage
- Configure auto-sync
- Enable offline mode

## 🆘 Getting Help

### Check Logs

Server logs show detailed error messages:
```bash
# View recent logs
tail -f nohup.out

# Or if running in foreground
# Logs appear in terminal
```

### API Testing

Test the API directly:
```bash
# List accounts
curl http://localhost:8080/api/email/accounts

# Add account
curl -X POST http://localhost:8080/api/email/accounts/add \
  -H "Content-Type: application/json" \
  -d '{"email":"test@gmail.com",...}'
```

### Database Inspection

Check database directly:
```bash
psql -d botserver_dev -c "SELECT * FROM user_email_accounts;"
```

## ✅ Verification Checklist

- [ ] Database migration completed successfully
- [ ] Server starts with `email` feature enabled
- [ ] Can access Account Settings page
- [ ] Can add email account
- [ ] Connection test passes
- [ ] Can see emails in Mail client
- [ ] Can send email successfully
- [ ] Can compose and save drafts
- [ ] Multiple accounts work (if applicable)

## 📚 Further Reading

- See `MULTI_USER_SYSTEM.md` for technical details
- See `REST_API.md` for API documentation
- See `TESTING.md` for testing procedures

## 🎯 Next Steps

After basic setup:
1. Configure additional email accounts
2. Explore Drive functionality
3. Set up automated tasks (future)
4. Customize preferences
5. **Implement proper security for production**

---

Need help? Check the logs, review error messages, and consult the troubleshooting section above.