# Quick Reference Guide

Essential commands, configurations, and code snippets for WhatsApp Business API integration with Twilio.

## Configuration Keys

### Required config.csv Entries

```csv
# Core Configuration
whatsapp-enabled,true
whatsapp-api-key,EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr
whatsapp-phone-number-id,1158433381968079
whatsapp-business-account-id,390727550789228
whatsapp-webhook-verify-token,4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY
whatsapp-application-id,323250907549153

# Optional: Advanced Settings
whatsapp-webhook-url,https://your-domain.com/webhooks/whatsapp
whatsapp-timeout,30000
whatsapp-retry-attempts,3
whatsapp-rate-limit,50
whatsapp-from-number,+553322980098
```

### Environment Variables

```bash
# Meta WhatsApp Configuration
export WHATSAPP_API_KEY="EAAQdlso6aM8BOwl..."
export WHATSAPP_PHONE_NUMBER_ID="1158433381968079"
export WHATSAPP_WABA_ID="390727550789228"
export WHATSAPP_VERIFY_TOKEN="4qIogZadggQ.BEoMeci..."
export WHATSAPP_APPLICATION_ID="323250907549153"
export WHATSAPP_APP_SECRET="your_app_secret_here"

# Twilio Configuration
export TWILIO_ACCOUNT_SID="ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export TWILIO_AUTH_TOKEN="your_auth_token_here"
export TWILIO_PHONE_NUMBER="+553322980098"

# Webhook Configuration
export WEBHOOK_URL="https://your-domain.com/webhooks/whatsapp"
export WEBHOOK_PORT="3000"
```

## API Endpoints

### Meta WhatsApp API

```bash
# Base URL
https://graph.facebook.com/v18.0

# Send Message
POST /{phone-number-id}/messages

# Get Phone Number Info
GET /{phone-number-id}

# Mark as Read
POST /{message-id}

# Upload Media
POST /{phone-number-id}/media

# Get Message Templates
GET /{waba-id}/message_templates
```

### Twilio API

```bash
# Base URL
https://api.twilio.com/2010-04-01

# Incoming Call Webhook
POST /twilio/voice

# Gather Handler
POST /twilio/gather

# Get Call Logs
GET /Accounts/{AccountSid}/Calls.json
```

## Common API Requests

### Send Text Message

```bash
curl -X POST \
  'https://graph.facebook.com/v18.0/1158433381968079/messages' \
  -H 'Authorization: Bearer EAAQdlso6aM8BOwl...' \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "to": "5511999999999",
    "type": "text",
    "text": {
      "body": "Hello from General Bots!"
    }
  }'
```

### Send Image

```bash
curl -X POST \
  'https://graph.facebook.com/v18.0/1158433381968079/messages' \
  -H 'Authorization: Bearer EAAQdlso6aM8BOwl...' \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "to": "5511999999999",
    "type": "image",
    "image": {
      "link": "https://example.com/image.jpg",
      "caption": "Check this out!"
    }
  }'
```

### Send Template

```bash
curl -X POST \
  'https://graph.facebook.com/v18.0/1158433381968079/messages' \
  -H 'Authorization: Bearer EAAQdlso6aM8BOwl...' \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "to": "5511999999999",
    "type": "template",
    "template": {
      "name": "hello_world",
      "language": {
        "code": "en_US"
      }
    }
  }'
```

### Mark as Read

```bash
curl -X POST \
  'https://graph.facebook.com/v18.0/wamid.HBgLNTE1OTk5OTk5OTk5FQIAERgSMzg1QTlCNkE2RTlFRTdFNDdF' \
  -H 'Authorization: Bearer EAAQdlso6aM8BOwl...' \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "status": "read"
  }'
```

## BASIC Keywords

### Sending Messages

```basic
REM Send simple text
SEND WHATSAPP TO "+5511999999999" WITH "Hello!"

REM Send with formatting
SEND WHATSAPP TO "+5511999999999" WITH "*Bold* and _italics_"

REM Send location
SEND WHATSAPP TO "+5511999999999" WITH LOCATION AT "-23.5505,-46.6333"

REM Send image
SEND WHATSAPP TO "+5511999999999" WITH IMAGE FROM "https://example.com/img.jpg"

REM Send document
SEND WHATSAPP TO "+5511999999999" WITH DOCUMENT FROM "https://example.com/doc.pdf"
```

### Receiving Messages

```basic
REM Handle incoming messages
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  LET ID$ = GET WHATSAPP MESSAGE ID
  LET TIMESTAMP$ = GET WHATSAPP MESSAGE TIMESTAMP
  
  LOG "From: " + SENDER$ + " Msg: " + MESSAGE$
  
  REM Process message
  PROCESS MESSAGE SENDER$, MESSAGE$
END ON
```

## Error Codes

### Common HTTP Status Codes

| Code | Meaning | Solution |
|------|---------|----------|
| 200 | Success | Request completed |
| 400 | Bad Request | Check request body |
| 401 | Unauthorized | Verify access token |
| 403 | Forbidden | Check permissions |
| 404 | Not Found | Verify phone number ID |
| 429 | Rate Limited | Implement backoff |
| 500 | Server Error | Retry later |

### WhatsApp-Specific Errors

```json
{
  "error": {
    "message": "Invalid parameter",
    "type": "WhatsAppApiError",
    "code": 130426,
    "error_data": {
      "details": "Phone number ID is required"
    }
  }
}
```

| Error Code | Description | Solution |
|------------|-------------|----------|
| 130426 | Invalid parameter | Check phone number format |
| 130472 | Rate limit hit | Reduce message frequency |
| 131056 | 24-hour window | Use message templates |
| 131047 | Media upload failed | Check media URL |
| 131053 | Template not approved | Submit template for review |

## Phone Number Formatting

### Correct Formats

```javascript
// Remove all non-digits
function formatPhone(phone) {
  return phone.replace(/\D/g, '');
}

// Examples
Input:  +55 (11) 99999-9999
Output: 5511999999999

Input:  +1-555-123-4567
Output: 15551234567

Input:  55 11 99999 9999
Output: 5511999999999
```

### Country Codes

| Country | Code | Example |
|---------|------|---------|
| Brazil | 55 | 5511999999999 |
| USA | 1 | 15551234567 |
| Mexico | 52 | 5215512345678 |
| Argentina | 54 | 5491112345678 |
| Portugal | 351 | 351912345678 |

## Webhook Payloads

### Incoming Message

```json
{
  "object": "whatsapp_business_account",
  "entry": [{
    "id": "390727550789228",
    "changes": [{
      "value": {
        "messaging_product": "whatsapp",
        "metadata": {
          "display_phone_number": "+553322980098",
          "phone_number_id": "1158433381968079"
        },
        "contacts": [{
          "profile": {
            "name": "John Doe"
          },
          "wa_id": "5511999999999"
        }],
        "messages": [{
          "from": "5511999999999",
          "id": "wamid.HBgLNTE1OTk5OTk5OTk5FQIAERgSMzg1QTlCNkE2RTlFRTdFNDdF",
          "timestamp": "1704067200",
          "text": {
            "body": "Hello bot!"
          },
          "type": "text"
        }]
      },
      "field": "messages"
    }]
  }]
}
```

### Message Status

```json
{
  "object": "whatsapp_business_account",
  "entry": [{
    "id": "390727550789228",
    "changes": [{
      "value": {
        "status": "sent",
        "id": "wamid.HBgLNTE1OTk5OTk5OTk5FQIAERgSMzg1QTlCNkE2RTlFRTdFNDdF",
        "timestamp": "1704067201"
      },
      "field": "message_template_status_update"
    }]
  }]
}
```

## TwiML Examples

### Gather Verification Code

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Gather action="https://your-domain.com/twilio/gather" 
          method="POST" 
          numDigits="6"
          timeout="10">
    <Say voice="alice" language="pt-BR">
      Please enter your verification code followed by the pound sign.
    </Say>
  </Gather>
  <Redirect>
    https://twimlets.com/voicemail?Email=your-email@example.com
  </Redirect>
</Response>
```

### Simple Voicemail

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Say>Please leave your message after the tone.</Say>
  <Record action="https://your-domain.com/twilio/recording" 
          method="POST" 
          maxLength="30" />
  <Say>Thank you for your message.</Say>
</Response>
```

## Diagnostic Commands

### Test Connectivity

```bash
# Test webhook endpoint
curl -X POST https://your-domain.com/webhooks/whatsapp \
  -H "Content-Type: application/json" \
  -d '{"test": true}'

# Test Meta API
curl -X GET "https://graph.facebook.com/v18.0/1158433381968079" \
  -H "Authorization: Bearer EAAQdlso6aM8BOwl..."

# Test Twilio webhook
curl -X POST https://your-domain.com/twilio/voice \
  -d "CallSid=CA123&From=+1234567890&To=+553322980098"

# Check SSL certificate
openssl s_client -connect your-domain.com:443
```

### Monitor Logs

```bash
# General Bots logs
tail -f .gbot/logs/bot.log

# Webhook server logs (PM2)
pm2 logs whatsapp-webhook

# System logs
journalctl -u whatsapp-webhook -f

# Twilio debugger
# https://console.twilio.com/us1/develop/monitor/debugger

# Meta webhook status
# https://developers.facebook.com/apps/YOUR_APP_ID/webhooks/
```

## Rate Limits

### WhatsApp Business API

| Tier | Messages/Day | Messages/Second |
|------|--------------|-----------------|
| Tier 1 | 1,000 | 1 |
| Tier 2 | 10,000 | 5 |
| Tier 3 | 100,000 | 50 |
| Tier 4 | Unlimited | 1,000 |

### Rate Limiting Implementation

```javascript
const rateLimiter = {
  requests: [],
  maxRequests: 50,
  timeWindow: 60000, // 1 minute
  
  canMakeRequest() {
    const now = Date.now();
    this.requests = this.requests.filter(t => now - t < this.timeWindow);
    return this.requests.length < this.maxRequests;
  },
  
  recordRequest() {
    this.requests.push(Date.now());
  }
};
```

## Formatting Syntax

### Text Formatting

```basic
REM Bold text
*bold text*

REM Italics
_italics_

REM Strikethrough
~strikethrough~

REM Monospace
```monospace```

REM Combined
*_bold and italic_*

REM Line breaks
Line 1
Line 2
```

### Message Examples

```basic
REM Formatted menu
SEND WHATSAPP TO "+5511999999999" WITH "🤖 *Bot Menu*" + CHR$(10) + CHR$(10) + "1. 📊 Status" + CHR$(10) + "2. 🌐 Weather" + CHR$(10) + "3. 📧 Contact"

REM Address
SEND WHATSAPP TO "+5511999999999" WITH "📍 *Address:*" + CHR$(10) + "123 Main St" + CHR$(10) + "São Paulo, SP" + CHR$(10) + "Brazil"

REM Code snippet
SEND WHATSAPP TO "+5511999999999" WITH "```bash" + CHR$(10) + "npm install" + CHR$(10) + "```"
```

## URLs

### Meta Platforms

```
Meta for Developers:
https://developers.facebook.com/

Meta Business Suite:
https://business.facebook.com/

WhatsApp Business API:
https://developers.facebook.com/docs/whatsapp/

Webhook Configuration:
https://developers.facebook.com/apps/YOUR_APP_ID/webhooks/

Message Templates:
https://business.facebook.com/latest/wa/manage/message-templates/
```

### Twilio Platforms

```
Twilio Console:
https://console.twilio.com/

Twilio Debugger:
https://console.twilio.com/us1/develop/monitor/debugger

TwiML Bins:
https://console.twilio.com/us1/develop/twiml/bins

Phone Numbers:
https://console.twilio.com/us1/develop/phone-numbers/manage/active
```

### General Bots

```
Documentation:
https://botbook.general-bots.com/

Community Discord:
https://discord.gg/general-bots

GitHub Repository:
https://github.com/general-bots/general-bots
```

## Quick Troubleshooting

### Issue: Verification Code Not Received

```bash
# Check Twilio webhook is configured
twilio phone-numerals:info +553322980098

# Test webhook manually
curl -X POST https://your-domain.com/twilio/voice \
  -d "CallSid=CA123&From=+1234567890"

# Verify voice capability is enabled
# In Twilio Console: Phone Numbers > Active Numbers > Your Number
# Check "Voice" is enabled
```

### Issue: Messages Not Sending

```bash
# Verify access token
curl -X GET "https://graph.facebook.com/v18.0/me" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Check phone number ID
curl -X GET "https://graph.facebook.com/v18.0/1158433381968079" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Test message format
# Ensure phone number: 5511999999999 (no +, no spaces)
```

### Issue: Webhook Not Receiving Messages

```bash
# Verify webhook subscription
curl -X GET "https://graph.facebook.com/v18.0/YOUR_APP_ID/subscriptions" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Test webhook endpoint
curl -X POST https://your-domain.com/webhooks/whatsapp \
  -H "Content-Type: application/json" \
  -d '{"object":"whatsapp_business_account","entry":[]}'

# Check webhook is subscribed to "messages" field
# In Meta Dashboard: WhatsApp > API Setup > Webhook > Manage
```

## Code Snippets

### Validate Phone Number

```javascript
function validatePhoneNumber(phone) {
  // Remove non-digits
  const cleaned = phone.replace(/\D/g, '');
  
  // Check length (10-15 digits)
  if (cleaned.length < 10 || cleaned.length > 15) {
    return false;
  }
  
  // Check if all digits
  if (!/^\d+$/.test(cleaned)) {
    return false;
  }
  
  return cleaned;
}
```

### Format WhatsApp Message

```javascript
function formatMessage(template, variables) {
  let message = template;
  
  for (const [key, value] of Object.entries(variables)) {
    message = message.replace(new RegExp(`{{${key}}}`, 'g'), value);
  }
  
  return message;
}

// Usage
const template = 'Hello {{name}}, your order {{orderId}} is confirmed!';
const variables = { name: 'John', orderId: '12345' };
const message = formatMessage(template, variables);
// Result: "Hello John, your order 12345 is confirmed!"
```

### Retry Logic

```javascript
async function sendWithRetry(whatsapp, to, message, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await whatsapp.sendText(to, message);
    } catch (error) {
      if (attempt === maxRetries) {
        throw error;
      }
      
      // Exponential backoff
      const delay = Math.pow(2, attempt) * 1000;
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}
```

## Environment Checklist

### Development

```bash
# Node.js
npm install express twilio body-parser axios

# Python
pip install flask requests twilio

# BASIC
REM No installation required
```

### Production

```bash
# Reverse proxy (nginx)
apt install nginx

# Process manager (PM2)
npm install -g pm2

# SSL certificate (Let's Encrypt)
apt install certbot python3-certbot-nginx
```

## Meta Console URLs

### Direct Access

Replace with your IDs:

```
WhatsApp Settings:
https://business.facebook.com/latest/settings/whatsapp_account/?business_id=312254061496740&selected_asset_id=303621682831134&selected_asset_type=whatsapp-business-account

Webhook Configuration:
https://developers.facebook.com/apps/323250907549153/webhooks/

Message Templates:
https://business.facebook.com/latest/wa/manage/message-templates/?waba_id=390727550789228

API Usage:
https://developers.facebook.com/apps/323250907549153/usage/
```

---

**For detailed documentation:** See [README.md](./README.md)  
**For troubleshooting:** See [troubleshooting.md](./troubleshooting.md)  
**For code examples:** See [examples.md](./examples.md)  
**For webhook setup:** See [webhooks.md](./webhooks.md)