# Channel Setup Guide

This guide provides step-by-step instructions for connecting General Bots to various messaging platforms and social media channels.

## Overview

General Bots supports multiple communication channels:

| Category | Channels |
|----------|----------|
| Social Media | Instagram, Facebook, Twitter/X, LinkedIn, Bluesky, Threads, TikTok, YouTube, Pinterest, Reddit, Snapchat |
| Messaging | WhatsApp, Telegram, Discord, Microsoft Teams, WeChat, Twilio SMS |
| Email | Gmail, Outlook, Custom SMTP |

## Configuration Methods

Channels can be configured through:

1. **Settings UI** - Navigate to Settings → Accounts
2. **config.csv** - Add credentials to your bot's configuration
3. **Environment Variables** - Set system-wide defaults
4. **API** - Programmatic configuration

## Social Media Channels

### Instagram

**Requirements:**
- Instagram Business or Creator account
- Facebook Page connected to Instagram
- Facebook Developer App

**Setup Steps:**

1. Create a Facebook Developer App at developers.facebook.com
2. Add Instagram Graph API product
3. Connect your Instagram Business account
4. Generate long-lived access token

**Configuration:**

```csv
key,value
instagram-access-token,your-long-lived-token
instagram-account-id,your-instagram-account-id
instagram-page-id,your-facebook-page-id
```

**Permissions Required:**
- `instagram_basic`
- `instagram_content_publish`
- `instagram_manage_comments`
- `instagram_manage_insights`

**Rate Limits:**
- 25 posts per 24 hours
- 200 API calls per hour

---

### Facebook

**Requirements:**
- Facebook Page
- Facebook Developer App

**Setup Steps:**

1. Create a Facebook Developer App
2. Add Facebook Login and Pages API products
3. Generate Page Access Token
4. Request page_publish permissions

**Configuration:**

```csv
key,value
facebook-access-token,your-page-access-token
facebook-page-id,your-page-id
facebook-app-id,your-app-id
facebook-app-secret,your-app-secret
```

**Permissions Required:**
- `pages_manage_posts`
- `pages_read_engagement`
- `pages_show_list`

---

### Twitter / X

**Requirements:**
- Twitter Developer Account
- Elevated or Basic API access

**Setup Steps:**

1. Apply for Twitter Developer Account at developer.twitter.com
2. Create a Project and App
3. Generate API keys and access tokens
4. Enable OAuth 1.0a with read and write permissions

**Configuration:**

```csv
key,value
twitter-api-key,your-api-key
twitter-api-secret,your-api-secret
twitter-access-token,your-access-token
twitter-access-secret,your-access-token-secret
twitter-bearer-token,your-bearer-token
```

**Rate Limits:**
- Free tier: 1,500 tweets/month
- Basic tier: 3,000 tweets/month
- Pro tier: 300,000 tweets/month

---

### LinkedIn

**Requirements:**
- LinkedIn Page
- LinkedIn Marketing Developer Platform access

**Setup Steps:**

1. Create app at linkedin.com/developers
2. Request Marketing Developer Platform access
3. Add products: Share on LinkedIn, Marketing Developer Platform
4. Generate OAuth 2.0 access token

**Configuration:**

```csv
key,value
linkedin-client-id,your-client-id
linkedin-client-secret,your-client-secret
linkedin-access-token,your-access-token
linkedin-organization-id,your-organization-urn
```

**Permissions Required:**
- `w_organization_social`
- `r_organization_social`
- `rw_organization_admin`

---

### Bluesky

**Requirements:**
- Bluesky account
- App password (not your main password)

**Setup Steps:**

1. Log into Bluesky
2. Go to Settings → App Passwords
3. Create new app password
4. Note your handle (e.g., yourname.bsky.social)

**Configuration:**

```csv
key,value
bluesky-handle,yourname.bsky.social
bluesky-app-password,xxxx-xxxx-xxxx-xxxx
```

**Rate Limits:**
- 1,666 posts per day
- 5,000 API calls per hour

---

### Threads

**Requirements:**
- Threads account (linked to Instagram)
- Meta Developer App with Threads API access

**Setup Steps:**

1. Use your existing Meta Developer App
2. Add Threads API product
3. Generate Threads access token
4. Link your Threads account

**Configuration:**

```csv
key,value
threads-access-token,your-threads-token
threads-user-id,your-threads-user-id
```

**Rate Limits:**
- 250 posts per 24 hours

---

### TikTok

**Requirements:**
- TikTok Business Account
- TikTok Developer App

**Setup Steps:**

1. Register at developers.tiktok.com
2. Create an app
3. Apply for Content Posting API access
4. Complete OAuth flow to get access token

**Configuration:**

```csv
key,value
tiktok-client-key,your-client-key
tiktok-client-secret,your-client-secret
tiktok-access-token,your-access-token
tiktok-refresh-token,your-refresh-token
```

**Permissions Required:**
- `video.upload`
- `video.publish`

**Notes:**
- Videos must be uploaded before publishing
- Supports direct upload or URL pull

---

### YouTube

**Requirements:**
- YouTube Channel
- Google Cloud Project

**Setup Steps:**

1. Create project at console.cloud.google.com
2. Enable YouTube Data API v3
3. Create OAuth 2.0 credentials
4. Complete OAuth consent screen setup
5. Generate refresh token

**Configuration:**

```csv
key,value
youtube-client-id,your-client-id
youtube-client-secret,your-client-secret
youtube-refresh-token,your-refresh-token
youtube-channel-id,your-channel-id
```

**Permissions Required:**
- `youtube.upload`
- `youtube.force-ssl`

**Quota:**
- 10,000 units per day (default)
- Video upload: 1,600 units

---

### Pinterest

**Requirements:**
- Pinterest Business Account
- Pinterest Developer App

**Setup Steps:**

1. Apply at developers.pinterest.com
2. Create an app
3. Request access to Pins API
4. Generate access token

**Configuration:**

```csv
key,value
pinterest-app-id,your-app-id
pinterest-app-secret,your-app-secret
pinterest-access-token,your-access-token
```

**Rate Limits:**
- 1,000 API calls per minute
- 200 Pins per hour

---

### Reddit

**Requirements:**
- Reddit Account
- Reddit Developer App

**Setup Steps:**

1. Go to reddit.com/prefs/apps
2. Create script or web application
3. Note client ID (under app name) and secret

**Configuration:**

```csv
key,value
reddit-client-id,your-client-id
reddit-client-secret,your-client-secret
reddit-username,your-username
reddit-password,your-password
reddit-user-agent,GeneralBots/1.0
```

**Rate Limits:**
- 60 requests per minute
- 10 posts per subreddit per day

---

### Snapchat

**Requirements:**
- Snapchat Business Account
- Snap Marketing API access

**Setup Steps:**

1. Create account at business.snapchat.com
2. Apply for Marketing API access
3. Create OAuth app
4. Generate access token

**Configuration:**

```csv
key,value
snapchat-client-id,your-client-id
snapchat-client-secret,your-client-secret
snapchat-access-token,your-access-token
snapchat-refresh-token,your-refresh-token
snapchat-ad-account-id,your-ad-account-id
```

---

## Messaging Channels

### WhatsApp Business

**Requirements:**
- WhatsApp Business Account
- Meta Business verification

**Setup Steps:**

1. Set up Meta Business Suite
2. Add WhatsApp to your Meta Developer App
3. Configure webhook URL
4. Get phone number ID and access token

**Configuration:**

```csv
key,value
whatsapp-phone-number-id,your-phone-number-id
whatsapp-access-token,your-access-token
whatsapp-business-account-id,your-business-account-id
whatsapp-webhook-verify-token,your-verify-token
```

**Webhook URL:** `https://your-domain.com/api/channels/whatsapp/webhook`

---

### Telegram

**Requirements:**
- Telegram Bot (created via BotFather)

**Setup Steps:**

1. Message @BotFather on Telegram
2. Send `/newbot` and follow prompts
3. Copy the bot token
4. Set webhook URL

**Configuration:**

```csv
key,value
telegram-bot-token,your-bot-token
telegram-webhook-secret,your-webhook-secret
```

**Set Webhook:**

```bash
curl -X POST "https://api.telegram.org/bot<token>/setWebhook" \
  -d "url=https://your-domain.com/api/channels/telegram/webhook"
```

---

### Discord

**Requirements:**
- Discord Server (with admin permissions)
- Discord Developer Application

**Setup Steps:**

1. Create app at discord.com/developers/applications
2. Create a Bot
3. Copy bot token
4. Generate invite URL with required permissions
5. Add bot to your server

**Configuration:**

```csv
key,value
discord-bot-token,your-bot-token
discord-application-id,your-application-id
discord-guild-id,your-server-id
discord-channel-id,default-channel-id
```

**Required Bot Permissions:**
- Send Messages
- Embed Links
- Attach Files
- Read Message History

---

### Microsoft Teams

**Requirements:**
- Microsoft 365 account
- Azure Active Directory app registration

**Setup Steps:**

1. Register app in Azure Portal
2. Add Teams permissions
3. Create Teams app manifest
4. Upload to Teams Admin Center

**Configuration:**

```csv
key,value
teams-app-id,your-app-id
teams-app-password,your-app-password
teams-tenant-id,your-tenant-id
```

---

### WeChat

**Requirements:**
- WeChat Official Account (Service or Subscription)
- ICP license (for China operations)

**Setup Steps:**

1. Register at mp.weixin.qq.com
2. Complete verification
3. Configure server settings
4. Get AppID and AppSecret

**Configuration:**

```csv
key,value
wechat-app-id,your-app-id
wechat-app-secret,your-app-secret
wechat-token,your-verification-token
wechat-encoding-aes-key,your-encoding-key
```

---

### Twilio SMS

**Requirements:**
- Twilio Account
- Phone number with SMS capability

**Setup Steps:**

1. Create account at twilio.com
2. Get Account SID and Auth Token
3. Purchase or port a phone number
4. Configure webhook URL

**Configuration:**

```csv
key,value
twilio-account-sid,your-account-sid
twilio-auth-token,your-auth-token
twilio-phone-number,+1234567890
twilio-messaging-service-sid,optional-messaging-service
```

**Webhook URL:** `https://your-domain.com/api/channels/twilio/webhook`

---

## Email Channels

### Gmail / Google Workspace

**Setup Steps:**

1. Enable Gmail API in Google Cloud Console
2. Create OAuth 2.0 credentials
3. Complete consent screen
4. Generate refresh token

**Configuration:**

```csv
key,value
gmail-client-id,your-client-id
gmail-client-secret,your-client-secret
gmail-refresh-token,your-refresh-token
gmail-user-email,your-email@gmail.com
```

---

### Outlook / Office 365

**Setup Steps:**

1. Register app in Azure Portal
2. Add Mail.Send permission
3. Grant admin consent
4. Generate access token

**Configuration:**

```csv
key,value
outlook-client-id,your-client-id
outlook-client-secret,your-client-secret
outlook-tenant-id,your-tenant-id
outlook-user-email,your-email@company.com
```

---

### Custom SMTP

**Configuration:**

```csv
key,value
smtp-host,smtp.example.com
smtp-port,587
smtp-username,your-username
smtp-password,your-password
smtp-from-address,noreply@example.com
smtp-from-name,Your Company
smtp-security,tls
```

---

## Testing Your Configuration

### Verify Connection

```http
GET /api/channels/{channel}/status
Authorization: Bearer <token>
```

Response:

```json
{
  "channel": "instagram",
  "connected": true,
  "account": "@youraccount",
  "permissions": ["read", "write"],
  "rate_limit_remaining": 180,
  "last_sync": "2025-01-21T10:00:00Z"
}
```

### Send Test Message

```http
POST /api/channels/{channel}/test
Authorization: Bearer <token>
Content-Type: application/json

{
  "message": "Hello from General Bots!"
}
```

---

## Troubleshooting

### Common Issues

**Invalid Token**
- Tokens expire; regenerate and update
- Check token permissions match requirements

**Rate Limited**
- Reduce posting frequency
- Implement exponential backoff
- Consider upgrading API tier

**Webhook Not Receiving**
- Verify URL is publicly accessible
- Check SSL certificate is valid
- Confirm webhook is registered with platform

**Permission Denied**
- Review required permissions list
- Reauthorize with correct scopes
- Check app review status

### Debug Logging

Enable channel debug logging:

```bash
CHANNEL_DEBUG=true
CHANNEL_LOG_LEVEL=debug
```

---

## Security Best Practices

1. **Store secrets securely** - Use environment variables or secret management
2. **Rotate tokens regularly** - Set reminders for token refresh
3. **Use webhook signatures** - Validate incoming webhook requests
4. **Limit permissions** - Request only needed scopes
5. **Monitor usage** - Track API calls and rate limits
6. **Audit access** - Review connected accounts periodically

---

## Related Topics

- [POST TO Keyword](../06-gbdialog/keyword-post-to.md) - Posting from BASIC
- [Social Media Keywords](../06-gbdialog/keywords-social-media.md) - Full social media reference
- [Multi-Channel Architecture](./channels.md) - System design
- [Accounts Settings](../10-configuration-deployment/accounts.md) - UI configuration