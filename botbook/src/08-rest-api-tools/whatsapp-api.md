# WhatsApp API

> **Send and receive WhatsApp messages via the WhatsApp Business API integration.**

---

## Base URL

```
/api/whatsapp
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. The webhook verification endpoint is exempt from authentication.

---

## Overview

The WhatsApp API provides integration with the WhatsApp Business Platform (Meta). It enables:

- Receiving inbound messages via webhooks
- Sending outbound text messages
- Monitoring connection status
- Managing active WhatsApp sessions
- Automatic session management with user lookup and creation
- Audio message transcription (when available)

> **Feature Flag:** This API is only available when the `whatsapp` feature is enabled at compile time (included in default features). The integration follows the same ChannelState dependency injection pattern as Telegram, Instagram, and Microsoft Teams channels.

---

## Endpoints

### Webhook (Receive Messages)

**`POST /api/whatsapp/webhook`**

Receive incoming WhatsApp messages and status updates from Meta. This endpoint is called by the WhatsApp Business Platform whenever a message is delivered or a status changes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| body | object | Yes | WhatsApp webhook payload (see Meta documentation) |

**Request Body (Meta Verification):**
```json
{
  "object": "whatsapp_business_account",
  "entry": [...]
}
```

**Response:**
```json
{
  "status": "ok"
}
```

---

### Webhook Verification

**`GET /api/whatsapp/webhook`**

Verify the webhook URL with Meta during setup. Meta sends a `GET` request with `hub.mode`, `hub.verify_token`, and `hub.challenge` query parameters. The endpoint responds with the challenge value to confirm ownership.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| hub.mode | string | Yes | Must be `"subscribe"` |
| hub.verify_token | string | Yes | Verification token configured in Meta |
| hub.challenge | string | Yes | Challenge string to echo back |

**Response:**
```
ok
```

---

### Send Message

**`POST /api/whatsapp/send`**

Send a text message to a WhatsApp user.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| to | string | Yes | Recipient phone number (E.164 format, e.g., `+5511999990000`) |
| message | string | Yes | Text message content |

**Request Body:**
```json
{
  "to": "+5511999990000",
  "message": "Hello from BotServer!"
}
```

**Response:**
```json
{
  "status": "ok",
  "to": "+5511999990000"
}
```

---

### Get Status

**`GET /api/whatsapp/status`**

Check the current status of the WhatsApp integration.

**Response:**
```json
{
  "status": "ok"
}
```

---

### Get Sessions

**`GET /api/whatsapp/sessions`**

List all active WhatsApp sessions.

**Response:**
```json
{
  "sessions": []
}
```

---

## Examples

### Send a Message

```bash
curl -X POST http://localhost:8080/api/whatsapp/send \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to": "+5511999990000",
    "message": "Your order #1234 has been shipped!"
  }'
```

### Check WhatsApp Status

```bash
curl -X GET http://localhost:8080/api/whatsapp/status \
  -H "Authorization: Bearer $TOKEN"
```

### List Active Sessions

```bash
curl -X GET http://localhost:8080/api/whatsapp/sessions \
  -H "Authorization: Bearer $TOKEN"
```

### Test Webhook Verification

```bash
curl -X GET "http://localhost:8080/api/whatsapp/webhook?hub.mode=subscribe&hub.verify_token=MY_TOKEN&hub.challenge=CHALLENGE_VALUE"
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request (invalid phone number or payload) |
| 401 | Unauthorized |
| 500 | Internal Server Error |

---

## Configuration

The WhatsApp integration requires the following credentials in the bot's `config.csv`:

| Key | Description |
|-----|-------------|
| `whatsapp-api-key` | API key from Meta Business Suite |
| `whatsapp-verify-token` | Custom token for webhook verification |
| `whatsapp-phone-number-id` | Phone Number ID from Meta |
| `whatsapp-business-account-id` | Business Account ID from Meta |

---

## See Also

- [Conversations API](./conversations-api.md) — Chat and message handling
- [Email API](./email-api.md) — Email channel integration
- [Notifications API](./notifications-api.md) — Multi-channel notifications
