# WhatsApp Channel Configuration

This guide covers setting up WhatsApp Business API as a communication channel for your General Bots deployment, enabling bots to interact with users on WhatsApp.

---

## Overview

WhatsApp integration allows your bot to:
- Receive and respond to WhatsApp messages
- Send rich media (images, documents, audio, video)
- Use interactive buttons and lists
- Send template messages for notifications
- Handle group conversations

---

## Quick Start

### Minimal Configuration

```csv
name,value
whatsapp-api-key,your_access_token
whatsapp-phone-number-id,your_phone_number_id
whatsapp-verify-token,your_webhook_verify_token
```

---

## Prerequisites

Before configuring WhatsApp, you need:

1. **Meta Business Account** at [business.facebook.com](https://business.facebook.com)
2. **WhatsApp Business Account** linked to Meta Business
3. **Phone Number** registered with WhatsApp Business API
4. **General Bots Server** accessible via HTTPS with valid SSL

---

## Configuration Parameters

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `whatsapp-api-key` | Access token from Meta Business | `EAABs...ZDZd` |
| `whatsapp-phone-number-id` | Phone number ID from WhatsApp Business | `123456789012345` |
| `whatsapp-verify-token` | Token for webhook verification | `my-secret-verify-token` |

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `whatsapp-business-account-id` | WhatsApp Business Account ID | Not set |
| `whatsapp-api-version` | Graph API version | `v17.0` |
| `whatsapp-webhook-url` | Custom webhook URL | `/webhooks/whatsapp` |

### Complete Example

```csv
name,value
whatsapp-api-key,EAABsBcDeFgHiJkLmNoPqRsTuVwXyZ123456789
whatsapp-phone-number-id,123456789012345
whatsapp-verify-token,my-super-secret-verify-token-2024
whatsapp-business-account-id,987654321098765
whatsapp-api-version,v17.0
```

---

## Meta Business Setup

### Step 1: Create Meta Business Account

1. Go to [business.facebook.com](https://business.facebook.com)
2. Click **Create Account**
3. Complete business verification

### Step 2: Create WhatsApp Business Account

1. In Meta Business Suite, go to **All tools** → **WhatsApp Manager**
2. Click **Get Started** or **Add WhatsApp Account**
3. Follow the setup wizard

### Step 3: Add Phone Number

1. In WhatsApp Manager, go to **Phone Numbers**
2. Click **Add Phone Number**
3. Verify the number via SMS or voice call
4. Note the **Phone Number ID** for configuration

### Step 4: Generate Access Token

1. Go to [developers.facebook.com](https://developers.facebook.com)
2. Create or select your app
3. Add **WhatsApp** product to your app
4. Go to **WhatsApp** → **API Setup**
5. Generate a **Permanent Access Token**:
   - Click **Add System User**
   - Grant `whatsapp_business_messaging` permission
   - Generate token

### Step 5: Configure Webhook

1. In your Meta app, go to **WhatsApp** → **Configuration**
2. Click **Edit** next to Webhook
3. Set **Callback URL**: `https://your-server.example.com/webhooks/whatsapp`
4. Set **Verify Token**: Same as `whatsapp-verify-token` in config.csv
5. Click **Verify and Save**
6. Subscribe to webhook fields:
   - `messages`
   - `message_deliveries` (optional)
   - `message_reads` (optional)

---

## BASIC Usage Examples

### Sending Text Messages

```basic
' Reply to WhatsApp message
TALK "Hello! How can I help you today?"
```

### Sending Images

```basic
' Send an image
image_url = "https://example.com/product.jpg"
CARD #{
    "type": "image",
    "url": image_url,
    "caption": "Here's the product you asked about"
}
```

### Sending Documents

```basic
' Generate and send a document
report = GENERATE PDF "templates/invoice.html", invoice_data, "temp/invoice.pdf"
DOWNLOAD report.url AS "Invoice.pdf"
```

### Interactive Buttons

```basic
' Send message with buttons
CARD #{
    "type": "interactive",
    "interactive": #{
        "type": "button",
        "body": #{
            "text": "Would you like to proceed with your order?"
        },
        "action": #{
            "buttons": [
                #{ "type": "reply", "reply": #{ "id": "confirm", "title": "✓ Confirm" } },
                #{ "type": "reply", "reply": #{ "id": "cancel", "title": "✗ Cancel" } }
            ]
        }
    }
}
```

### Interactive Lists

```basic
' Send a list selection
CARD #{
    "type": "interactive",
    "interactive": #{
        "type": "list",
        "header": #{
            "type": "text",
            "text": "Select a Category"
        },
        "body": #{
            "text": "Choose from our product categories:"
        },
        "action": #{
            "button": "View Categories",
            "sections": [
                #{
                    "title": "Electronics",
                    "rows": [
                        #{ "id": "phones", "title": "Phones", "description": "Smartphones and accessories" },
                        #{ "id": "laptops", "title": "Laptops", "description": "Notebooks and tablets" }
                    ]
                },
                #{
                    "title": "Home",
                    "rows": [
                        #{ "id": "furniture", "title": "Furniture", "description": "Tables, chairs, sofas" },
                        #{ "id": "appliances", "title": "Appliances", "description": "Kitchen and home appliances" }
                    ]
                }
            ]
        }
    }
}
```

### Template Messages

Template messages are pre-approved messages for notifications:

```basic
' Send a template message (must be approved by Meta)
SEND TEMPLATE "whatsapp", customer_phone, "order_confirmation", #{
    "1": order_id,
    "2": order_total,
    "3": delivery_date
}
```

---

## Message Templates

### Creating Templates

1. Go to WhatsApp Manager → **Message Templates**
2. Click **Create Template**
3. Choose category (Marketing, Utility, Authentication)
4. Define template with variables: `{{1}}`, `{{2}}`, etc.
5. Submit for approval

### Template Example

**Name:** `order_shipped`  
**Category:** Utility  
**Content:**
```
Hi {{1}}! 📦

Your order #{{2}} has been shipped!

Tracking number: {{3}}
Estimated delivery: {{4}}

Track your package: {{5}}
```

### Using Templates in BASIC

```basic
SEND TEMPLATE "whatsapp", customer.phone, "order_shipped", #{
    "1": customer.name,
    "2": order.id,
    "3": tracking_number,
    "4": estimated_delivery,
    "5": tracking_url
}
```

---

## Handling Media

### Receiving Media

```basic
' Check if message contains media
IF message.type = "image" THEN
    image_url = message.image.url
    TALK "I received your image. Let me analyze it..."
    ' Process image
ELSE IF message.type = "document" THEN
    doc_url = message.document.url
    doc_name = message.document.filename
    TALK "I received " + doc_name + ". Processing..."
END IF
```

### Sending Different Media Types

```basic
' Audio
CARD #{ "type": "audio", "url": "https://example.com/message.mp3" }

' Video
CARD #{ "type": "video", "url": "https://example.com/demo.mp4", "caption": "Product demo" }

' Location
CARD #{ 
    "type": "location", 
    "latitude": -23.5505, 
    "longitude": -46.6333,
    "name": "Our Store",
    "address": "123 Main Street"
}

' Contact
CARD #{
    "type": "contacts",
    "contacts": [
        #{
            "name": #{ "formatted_name": "Support Team" },
            "phones": [#{ "phone": "+15551234567", "type": "WORK" }]
        }
    ]
}
```

---

## Rate Limits and Best Practices

### Official Meta Rate Limits (Per Recipient)

**Base Rate**: 1 message per 6 seconds per recipient (0.17 messages/second)
- Equals ~10 messages per minute or 600 per hour per recipient
- Exceeding this triggers error code **131056**

**Burst Allowance**: Up to 45 messages in a 6-second burst
- "Borrows" from future quota
- After burst, must wait equivalent time at normal rate
- Example: 20-message burst requires ~2-minute cooldown

**Retry Strategy**: When rate limit hit (error 131056):
- Wait 4^X seconds before retry (X starts at 0)
- X increments by 1 after each failure
- Retry sequence: 1s, 4s, 16s, 64s, 256s

**Implementation**: BotServer automatically manages rate limiting via Redis queue
- Messages are queued and sent at compliant rate
- Per-recipient tracking prevents violations
- Automatic exponential backoff on 131056 errors

### Daily Messaging Limits (Tier-Based)

| Tier | Messages/24h | How to Upgrade |
|------|--------------|----------------|
| Tier 1 | 1,000 | Verify business |
| Tier 2 | 10,000 | Good quality rating |
| Tier 3 | 100,000 | Sustained quality |
| Tier 4 | Unlimited | Top quality rating |

### Quality Rating

Maintain quality by:
- Responding within 24 hours
- Avoiding spam complaints
- Using templates appropriately
- Providing value in conversations

### 24-Hour Window

- **User-initiated**: Free-form messages allowed for 24 hours after user message
- **Business-initiated**: Only template messages outside the 24-hour window

```basic
' Check if within messaging window
IF within_24h_window THEN
    TALK "Here's your update: " + update_text
ELSE
    ' Use template for out-of-window message
    SEND TEMPLATE "whatsapp", phone, "update_notification", #{ "1": update_text }
END IF
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

TALK "Processing your request..."

IF ERROR THEN
    error_msg = ERROR MESSAGE
    
    IF INSTR(error_msg, "131056") > 0 THEN
        PRINT "Rate limit exceeded - message queued for automatic retry"
        ' BotServer handles retry automatically
    ELSE IF INSTR(error_msg, "invalid phone") > 0 THEN
        PRINT "Invalid phone number format"
    ELSE IF INSTR(error_msg, "not registered") > 0 THEN
        PRINT "User not on WhatsApp"
    ELSE
        PRINT "WhatsApp error: " + error_msg
    END IF
END IF
```

### Common Errors

| Error Code | Meaning | Solution |
|------------|---------|----------|
| 130472 | User's number is part of an experiment | The user is in a WhatsApp beta test or Meta is testing new features on their account. Message delivery is temporarily blocked during these tests. Wait and retry later. |
| 131026 | Message undeliverable | The phone number may be invalid, the user may have blocked your business number, deleted their WhatsApp account, or there are temporary network issues. This is usually a permanent error for that specific number. |
| 131030 | User not on WhatsApp | Verify phone number |
| 131047 | Re-engagement required | Send template message |
| 131048 | Spam rate limit | Reduce message frequency |
| 131049 | Message not delivered to maintain healthy ecosystem engagement | You are sending too many messages too quickly, or WhatsApp detected spam-like behavior. This is a temporary rate limit - reduce your sending frequency and implement proper throttling. |
| 131051 | Unsupported message type | Check message format |
| 131056 | Rate limit exceeded | Sending too fast to same recipient (>1 msg/6s). BotServer automatically retries with exponential backoff. |
| 131052 | Media download failed | Verify media URL |
| 132000 | Template not found | Check template name |
| 132001 | Template paused | Resume in WhatsApp Manager |
| 133010 | Phone not registered | Complete phone verification |

---

## Webhook Security

### Validating Requests

General Bots automatically validates webhook signatures. Ensure your `whatsapp-verify-token` is kept secret.

### Recommended Setup

```csv
name,value
whatsapp-verify-token,a-very-long-random-string-that-is-hard-to-guess
whatsapp-validate-signature,true
whatsapp-app-secret,your_app_secret_from_meta
```

---

## Testing

### Test Numbers

Use Meta's test phone numbers during development:

1. In App Dashboard, go to **WhatsApp** → **API Setup**
2. Use the **Test Phone Number** provided
3. Add your phone as a test recipient

### Sending Test Messages

```basic
' Test message to verified number
test_phone = "+15551234567"  ' Must be in test recipients list
SEND SMS test_phone, "Test message from General Bots"
```

---

## Production Checklist

- [ ] Business verification completed
- [ ] Phone number verified and registered
- [ ] Permanent access token generated
- [ ] Webhook configured and verified
- [ ] Message templates approved
- [ ] SSL certificate valid
- [ ] Error handling implemented
- [ ] Rate limiting considered
- [ ] Quality guidelines reviewed

---

## Troubleshooting

### Webhook Not Receiving Messages

1. Verify callback URL is HTTPS with valid SSL
2. Check verify token matches configuration
3. Ensure webhook fields are subscribed
4. Check server logs for incoming requests

### Messages Not Sending

1. Verify access token is valid and not expired
2. Check phone number ID is correct
3. Ensure recipient is on WhatsApp
4. Check for rate limit errors

### Template Messages Failing

1. Verify template is approved
2. Check template name is correct
3. Ensure all variables are provided
4. Verify language code matches

---

## Related Documentation

- [Teams Configuration](./teams-channel.md) — Microsoft Teams setup
- [SMS Configuration](./sms-providers.md) — SMS provider configuration
- [Universal Messaging](../04-basic-scripting/universal-messaging.md) — Multi-channel messaging
- [SEND TEMPLATE Keyword](../04-basic-scripting/keyword-send-template.md) — Template messaging
- [CARD Keyword](../04-basic-scripting/keyword-card.md) — Rich card messages

---

## Summary

WhatsApp Business API integration enables your General Bots deployment to communicate with billions of WhatsApp users. Configure the required parameters from Meta Business, set up webhooks, create message templates for notifications, and your bot is ready to engage customers on their preferred messaging platform. Follow quality guidelines and respect the 24-hour messaging window to maintain a high quality rating.