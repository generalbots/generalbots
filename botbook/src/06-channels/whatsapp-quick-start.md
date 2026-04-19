# Quick Start Guide

Get your WhatsApp Business bot up and running in 30 minutes with this streamlined setup guide.

## Prerequisites Checklist

- [ ] Twilio account with $10+ credit
- [ ] Meta for Developers account
- [ ] Meta Business Suite account
- [ ] Publicly accessible webhook URL (use ngrok for testing)
- [ ] Basic command line knowledge

## 30-Minute Setup

### Step 1: Buy Twilio Number (5 minutes)

```bash
# Log into Twilio Console
# https://console.twilio.com/

# Navigate to: Phone Numbers > Buy a Number
# Select: Voice capability (required!)
# Purchase number
# Example: +553322980098
```

**Tip:** Choose a number from your target country for easier verification.

### Step 2: Create Meta App (5 minutes)

```bash
# Go to Meta for Developers
# https://developers.facebook.com/apps/

# Click: Create App > Business type
# App name: "My WhatsApp Bot"
# Add product: WhatsApp
# Create WhatsApp Business Account (WABA)
```

**Save these values:**
```
WABA ID:          390727550789228
Application ID:   323250907549153
Phone Number ID:  (after verification)
```

### Step 3: Configure Twilio Webhook (5 minutes)

**Option A: TwiML Bin (Fastest)**

```xml
<!-- Create TwiML Bin in Twilio Console -->
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Gather action="https://twimlets.com/voicemail?Email=your-email@example.com" method="POST">
    <Say voice="alice">Please enter your verification code.</Say>
  </Gather>
</Response>
```

**Option B: ngrok + Node.js (Recommended)**

```bash
# Install dependencies
npm install express twilio body-parser

# Create server.js
```

```javascript
const express = require('express');
const twilio = require('twilio');
const app = express();

app.use(require('body-parser').urlencoded({ extended: false }));

app.post('/twilio/voice', (req, res) => {
  const twiml = new twilio.twiml.VoiceResponse();
  twiml.redirect('https://twimlets.com/voicemail?Email=your-email@example.com');
  res.type('text/xml');
  res.send(twiml.toString());
});

app.listen(3000);
```

```bash
# Start ngrok
ngrok http 3000

# Update Twilio number webhook to:
# https://abc123.ngrok.io/twilio/voice
```

### Step 4: Verify Phone Number (5 minutes)

```bash
# In Meta Business Suite:
# 1. WhatsApp Accounts > Add Phone Number
# 2. Enter: +553322980098
# 3. Select: "Phone Call" (NOT SMS!)
# 4. Click: Verify

# Meta will call your Twilio number
# Check your email for the verification code
# Enter code in Meta dashboard
```

**Critical:** Select "Phone Call" verification - Twilio numbers don't support SMS!

### Step 5: Get API Credentials (3 minutes)

```bash
# In Meta for Developers:
# 1. Your App > WhatsApp > API Setup
# 2. Click: "Temporary Access Token"
# 3. Copy token (starts with EAAQ...)
# 4. Note Phone Number ID from URL
```

**Required credentials:**
```csv
whatsapp-api-key,EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr
whatsapp-phone-number-id,1158433381968079
whatsapp-business-account-id,390727550789228
whatsapp-webhook-verify-token,4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY
whatsapp-application-id,323250907549153
whatsapp-enabled,true
```

### Step 6: Configure Webhook (5 minutes)

```bash
# Start your webhook server
node server.js

# In Meta Developers:
# 1. WhatsApp > API Setup > Webhook > Edit
# 2. Webhook URL: https://your-domain.com/webhooks/whatsapp
# 3. Verify Token: 4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY
# 4. Click: Verify and Save
# 5. Subscribe to: messages
```

### Step 7: Configure General Bots (2 minutes)

```bash
# Edit .gbot/config.csv
```

```csv
key,value
whatsapp-enabled,true
whatsapp-api-key,EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr
whatsapp-phone-number-id,1158433381968079
whatsapp-business-account-id,390727550789228
whatsapp-webhook-verify-token,4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY
whatsapp-application-id,323250907549153
```

### Step 8: Test Your Bot (5 minutes)

```bash
# Send test message via API
curl -X POST \
  'https://graph.facebook.com/v18.0/1158433381968079/messages' \
  -H 'Authorization: Bearer EAAQdlso6aM8BOwl...' \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "to": "5511999999999",
    "type": "text",
    "text": {"body": "Hello from General Bots!"}
  }'

# Or use BASIC
```

```basic
REM Test your WhatsApp integration
SEND WHATSAPP TO "+5511999999999" WITH "Hello from General Bots!"
```

## Your First WhatsApp Bot

Create a simple echo bot:

```basic
REM Simple WhatsApp Echo Bot
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  
  LOG "Message from " + SENDER$ + ": " + MESSAGE$
  
  REM Echo back with acknowledgment
  SEND WHATSAPP TO SENDER$ WITH "You said: " + MESSAGE$
END ON
```

## Common First-Time Mistakes

❌ **Don't select SMS verification** - Use "Phone Call"
❌ **Don't hardcode tokens** - Use config.csv
❌ **Don't forget webhook subscriptions** - Subscribe to "messages"
❌ **Don't use + in phone numbers** - Format: 5511999999999
❌ **Don't ignore rate limits** - Max 1000 messages/second

## Next Steps

1. **Create message templates** for business-initiated conversations
2. **Set up persistent storage** for conversation history
3. **Implement retry logic** for failed messages
4. **Add monitoring** for webhook health
5. **Review security best practices**

## Need Help?

- 📖 [Full Documentation](./README.md)
- 🔧 [Troubleshooting Guide](./troubleshooting.md)
- 💻 [Code Examples](./examples.md)
- 🌐 [Webhook Configuration](./webhooks.md)
- 💬 [Community Discord](https://discord.gg/general-bots)

## Verification Checklist

- [ ] Twilio number purchased with Voice capability
- [ ] Meta app created with WhatsApp product
- [ ] Phone number verified via phone call
- [ ] Access token generated and saved
- [ ] Webhook configured and verified
- [ ] Webhook subscribed to "messages"
- [ ] config.csv updated with all credentials
- [ ] Test message sent successfully
- [ ] Incoming webhook received
- [ ] Bot replied to test message

✅ **All checked? Your WhatsApp bot is live!**

## Quick Reference: Essential Commands

```bash
# Test webhook connectivity
curl -X POST https://your-webhook.com/webhooks/whatsapp \
  -H "Content-Type: application/json" \
  -d '{"test":true}'

# Check Meta API status
curl https://developers.facebook.com/status/

# View Twilio call logs
# https://console.twilio.com/us1/develop/monitor/logs/calls

# Test access token
curl -X GET "https://graph.facebook.com/v18.0/me" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Monitor bot logs
tail -f .gbot/logs/bot.log
```

## Configuration Template

Copy this template and replace with your values:

```csv
# WhatsApp Business Configuration
whatsapp-enabled,true
whatsapp-api-key,YOUR_ACCESS_TOKEN_HERE
whatsapp-phone-number-id,YOUR_PHONE_NUMBER_ID_HERE
whatsapp-business-account-id,YOUR_WABA_ID_HERE
whatsapp-webhook-verify-token,YOUR_VERIFY_TOKEN_HERE
whatsapp-application-id,YOUR_APP_ID_HERE
whatsapp-from-number,+553322980098

# Optional: Advanced Settings
whatsapp-webhook-url,https://your-domain.com/webhooks/whatsapp
whatsapp-timeout,30000
whatsapp-retry-attempts,3
whatsapp-rate-limit,50
```

## Time-Saving Tips

💡 **Use ngrok for testing** - No need to deploy to test webhooks
💡 **Save all credentials immediately** - Tokens won't be shown again
💡 **Test with your own number first** - Verify everything works
💡 **Enable debug logging** - Troubleshoot issues faster
💡 **Set up monitoring early** - Catch problems before users do

---

**Estimated total time:** 30 minutes  
**Difficulty:** Intermediate  
**Cost:** ~$10/month (Twilio number + usage)

For detailed explanations, advanced configurations, and production deployment, see the [complete documentation](./README.md).