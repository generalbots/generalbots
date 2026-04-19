# Webhook Configuration Guide

This guide provides detailed instructions for configuring webhooks for both Twilio (voice call handling) and Meta (WhatsApp message handling) in your General Bots integration.

## Overview

The integration requires two separate webhook configurations:

1. **Twilio Voice Webhook** - Handles incoming verification calls and captures verification codes
2. **Meta WhatsApp Webhook** - Receives incoming WhatsApp messages and status updates

## Twilio Webhook Configuration

### Purpose

The Twilio webhook is critical during the initial phone number verification phase. Since Twilio numbers don't support SMS verification, Meta must call your number and read a 6-digit code. Your webhook must:

1. Answer the incoming call from Meta
2. Capture the audio or DTMF tones (key presses)
3. Forward the verification code to your email or logging system

### Webhook URL Structure

```
POST https://your-domain.com/twilio/voice
```

### Required HTTP Headers

Twilio sends these headers with every webhook request:

| Header | Description | Example |
|--------|-------------|---------|
| `X-Twilio-Signature` | Request signature for security | `RCYmLs...` |
| `Content-Type` | Always `application/x-www-form-urlencoded` | - |

### Request Body Parameters

When a call comes in, Twilio POSTs these parameters:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `CallSid` | Unique call identifier | `CA1234567890ABCDEF1234567890ABCDEF` |
| `From` | Caller's phone number | `+1234567890` (Meta's verification number) |
| `To` | Your Twilio number | `+553322980098` |
| `CallStatus` | Current call status | `ringing` |
| `Direction` | Call direction | `inbound` |

### TwiML Response Format

Your webhook must respond with TwiML (Twilio Markup Language) XML:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Gather action="https://your-domain.com/twilio/gather" method="POST" numDigits="6">
    <Say voice="alice" language="pt-BR">
      Please enter your verification code followed by the pound sign.
    </Say>
  </Gather>
  <Redirect>https://twimlets.com/voicemail?Email=your-email@example.com</Redirect>
</Response>
```

### Implementation Examples

#### Node.js/Express

```javascript
const express = require('express');
const twilio = require('twilio');
const app = express();

app.post('/twilio/voice', (req, res) => {
  const twiml = new twilio.twiml.VoiceResponse();
  
  const gather = twiml.gather({
    action: '/twilio/gather',
    method: 'POST',
    numDigits: 6,
    timeout: 10
  });
  
  gather.say({ 
    voice: 'alice', 
    language: 'pt-BR' 
  }, 'Please enter your verification code followed by the pound key.');
  
  // Fallback to voicemail if no input
  twiml.redirect('https://twimlets.com/voicemail?Email=your-email@example.com');
  
  res.type('text/xml');
  res.send(twiml.toString());
});

app.post('/twilio/gather', (req, res) => {
  const verificationCode = req.body.Digits;
  
  console.log('WhatsApp Verification Code:', verificationCode);
  
  // Send email notification
  sendEmail({
    to: 'your-email@example.com',
    subject: 'WhatsApp Verification Code',
    body: `Your verification code is: ${verificationCode}`
  });
  
  const twiml = new twilio.twiml.VoiceResponse();
  twiml.say('Thank you. Your code has been received.');
  
  res.type('text/xml');
  res.send(twiml.toString());
});

app.listen(3000, () => {
  console.log('Twilio webhook server running on port 3000');
});
```

#### Python/Flask

```python
from flask import Flask, request, Response
from twilio.twiml.voice_response import VoiceResponse, Gather
import smtplib

app = Flask(__name__)

@app.route('/twilio/voice', methods=['POST'])
def voice_webhook():
    response = VoiceResponse()
    
    gather = Gather(
        action='/twilio/gather',
        method='POST',
        num_digits=6,
        timeout=10
    )
    gather.say(
        'Please enter your verification code followed by the pound key.',
        voice='alice',
        language='pt-BR'
    )
    response.append(gather)
    
    # Fallback to voicemail
    response.redirect('https://twimlets.com/voicemail?Email=your-email@example.com')
    
    return Response(str(response), mimetype='text/xml')

@app.route('/twilio/gather', methods=['POST'])
def gather_webhook():
    verification_code = request.form.get('Digits')
    
    print(f'WhatsApp Verification Code: {verification_code}')
    
    # Send email notification
    send_email(
        to='your-email@example.com',
        subject='WhatsApp Verification Code',
        body=f'Your verification code is: {verification_code}'
    )
    
    response = VoiceResponse()
    response.say('Thank you. Your code has been received.')
    
    return Response(str(response), mimetype='text/xml')

def send_email(to, subject, body):
    # Implement email sending logic
    pass

if __name__ == '__main__':
    app.run(port=3000)
```

#### BASIC (General Bots)

```basic
REM Twilio Voice Webhook Handler
ON WEBHOOK POST TO "/twilio/voice" DO
  REM Create TwiML response
  LET TWIML$ = "<?xml version=""1.0"" encoding=""UTF-8""?>"
  TWIML$ = TWIML$ + "<Response>"
  TWIML$ = TWIML$ + "<Gather action=""https://your-domain.com/twilio/gather"" method=""POST"" numDigits=""6"">"
  TWIML$ = TWIML$ + "<Say voice=""alice"" language=""pt-BR"">"
  TWIML$ = TWIML$ + "Please enter your verification code followed by the pound sign."
  TWIML$ = TWIML$ + "</Say>"
  TWIML$ = TWIML$ + "</Gather>"
  TWIML$ = TWIML$ + "<Redirect>https://twimlets.com/voicemail?Email=your-email@example.com</Redirect>"
  TWIML$ = TWIML$ + "</Response>"
  
  REM Set response content type
  SET RESPONSE HEADER "Content-Type" TO "text/xml"
  PRINT TWIML$
END ON

REM Gather Handler (receives the DTMF input)
ON WEBHOOK POST TO "/twilio/gather" DO
  REM Get the digits entered
  LET CODE$ = GET FORM VALUE "Digits"
  
  REM Log the verification code
  LOG "WhatsApp Verification Code: " + CODE$
  
  REM Send email notification
  SEND MAIL TO "your-email@example.com" WITH SUBJECT "WhatsApp Verification Code" AND BODY "Your verification code is: " + CODE$
  
  REM Create confirmation TwiML
  LET TWIML$ = "<?xml version=""1.0"" encoding=""UTF-8""?>"
  TWIML$ = TWIML$ + "<Response>"
  TWIML$ = TWIML$ + "<Say>Thank you. Your code has been received.</Say>"
  TWIML$ = TWIML$ + "</Response>"
  
  SET RESPONSE HEADER "Content-Type" TO "text/xml"
  PRINT TWIML$
END ON
```

### Configuring Twilio

1. **Navigate to your phone number**
   - Go to Twilio Console > Phone Numbers > Active Numbers
   - Click on your purchased number

2. **Configure Voice Webhook**
   - Find "Voice & Fax" section
   - Set "A Call Comes In" to your webhook URL
   - Select HTTP POST method
   - Example: `https://your-domain.com/twilio/voice`

3. **Save changes**
   - Click "Save" to apply the configuration

### Webhook Security

Verify that requests come from Twilio:

```javascript
const twilio = require('twilio');
const client = twilio(process.env.TWILIO_ACCOUNT_SID, process.env.TWILIO_AUTH_TOKEN);

app.post('/twilio/voice', (req, res) => {
  const url = `https://${req.headers.host}${req.path}`;
  const signature = req.headers['x-twilio-signature'];
  
  if (client.validateRequest(url, req.body, signature)) {
    // Request is from Twilio, process it
    handleVoiceWebhook(req, res);
  } else {
    // Invalid signature
    res.status(403).send('Invalid signature');
  }
});
```

## Meta WhatsApp Webhook Configuration

### Purpose

The Meta webhook receives:
- Incoming WhatsApp messages from users
- Message delivery status updates
- Message read receipts
- Webhook verification requests

### Webhook URL Structure

```
POST https://your-domain.com/webhooks/whatsapp
```

### Required HTTP Headers

| Header | Description | Example |
|--------|-------------|---------|
| `X-Hub-Signature-256` | HMAC SHA-256 signature | `sha256=...` |

### Webhook Verification

When you first configure the webhook, Meta sends a GET request to verify your URL:

```
GET https://your-domain.com/webhooks/whatsapp?hub.verify_token=YOUR_TOKEN&hub.challenge=CHALLENGE_STRING
```

Your webhook must respond with the challenge:

```javascript
app.get('/webhooks/whatsapp', (req, res) => {
  const mode = req.query['hub.mode'];
  const token = req.query['hub.verify_token'];
  const challenge = req.query['hub.challenge'];
  
  const VERIFY_TOKEN = '4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY';
  
  if (mode === 'subscribe' && token === VERIFY_TOKEN) {
    console.log('Webhook verified');
    res.status(200).send(challenge);
  } else {
    res.sendStatus(403);
  }
});
```

### Message Payload Structure

Meta sends JSON payloads with message data:

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
            "body": "Hello, how can I help you?"
          },
          "type": "text"
        }]
      },
      "field": "messages"
    }]
  }]
}
```

### Implementation Examples

#### Node.js/Express

```javascript
app.post('/webhooks/whatsapp', (req, res) => {
  try {
    const data = req.body;
    
    // Check if this is a WhatsApp message
    if (data.object === 'whatsapp_business_account') {
      data.entry.forEach(entry => {
        entry.changes.forEach(change => {
          if (change.field === 'messages') {
            const message = change.value.messages[0];
            const from = message.from;
            const body = message.text.body;
            const messageId = message.id;
            
            console.log(`Received message from ${from}: ${body}`);
            
            // Process the message
            processWhatsAppMessage(from, body, messageId);
          }
        });
      });
    }
    
    res.status(200).send('OK');
  } catch (error) {
    console.error('Webhook error:', error);
    res.status(500).send('Error');
  }
});

async function processWhatsAppMessage(from, body, messageId) {
  // Implement your bot logic here
  const response = await generateResponse(body);
  
  // Send reply
  await sendWhatsAppMessage(from, response);
}
```

#### Python/Flask

```python
@app.route('/webhooks/whatsapp', methods=['POST'])
def whatsapp_webhook():
    try:
        data = request.get_json()
        
        if data.get('object') == 'whatsapp_business_account':
            for entry in data.get('entry', []):
                for change in entry.get('changes', []):
                    if change.get('field') == 'messages':
                        message = change['value']['messages'][0]
                        from_number = message['from']
                        body = message['text']['body']
                        message_id = message['id']
                        
                        print(f"Received message from {from_number}: {body}")
                        
                        # Process the message
                        process_whatsapp_message(from_number, body, message_id)
        
        return 'OK', 200
    except Exception as e:
        print(f'Webhook error: {e}')
        return 'Error', 500

def process_whatsapp_message(from_number, body, message_id):
    # Implement your bot logic here
    response = generate_response(body)
    
    # Send reply
    send_whatsapp_message(from_number, response)
```

#### BASIC (General Bots)

```basic
REM Meta WhatsApp Webhook Handler
ON WEBHOOK POST TO "/webhooks/whatsapp" DO
  REM Get the JSON payload
  LET PAYLOAD$ = GET REQUEST BODY
  
  REM Parse the JSON (requires JSON parser library)
  LET OBJ = PARSE JSON PAYLOAD$
  
  REM Check if this is a WhatsApp message
  IF GET JSON PATH OBJ, "object" = "whatsapp_business_account" THEN
    REM Get the message
    LET MESSAGE = GET JSON PATH OBJ, "entry[0].changes[0].value.messages[0]"
    
    REM Extract message details
    LET FROM$ = GET JSON PATH MESSAGE, "from"
    LET BODY$ = GET JSON PATH MESSAGE, "text.body"
    LET ID$ = GET JSON PATH MESSAGE, "id"
    
    REM Log the message
    LOG "WhatsApp message from " + FROM$ + ": " + BODY$
    
    REM Process the message asynchronously
    SPAWN PROCESS WHATSAPP MESSAGE FROM$, BODY$, ID$
  END IF
  
  REM Respond with 200 OK
  PRINT "OK"
  SET RESPONSE STATUS TO 200
END ON

REM Message processor
SUB PROCESS WHATSAPP MESSAGE FROM$, BODY$, ID$
  REM Generate a response
  LET RESPONSE$ = GENERATE RESPONSE BODY$
  
  REM Send the reply
  SEND WHATSAPP TO FROM$ WITH RESPONSE$
END SUB
```

### Configuring Meta

1. **Navigate to WhatsApp API Setup**
   - Go to Meta for Developers > Your App > WhatsApp > API Setup

2. **Edit Webhook**
   - Click "Edit" next to Webhook
   - Enter your webhook URL: `https://your-domain.com/webhooks/whatsapp`
   - Enter your Verify Token: `4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY`
   - Click "Verify and Save"

3. **Subscribe to Webhook Fields**
   - Subscribe to: `messages`
   - This ensures you receive all incoming messages

### Webhook Security

Implement signature verification:

```javascript
const crypto = require('crypto');

app.post('/webhooks/whatsapp', (req, res) => {
  const signature = req.headers['x-hub-signature-256'];
  const payload = JSON.stringify(req.body);
  const appSecret = 'YOUR_APP_SECRET'; // From Meta dashboard
  
  const expectedSignature = 'sha256=' + crypto
    .createHmac('sha256', appSecret)
    .update(payload)
    .digest('hex');
  
  if (signature !== expectedSignature) {
    console.error('Invalid webhook signature');
    return res.status(403).send('Invalid signature');
  }
  
  // Process the webhook
  processWebhook(req.body);
  res.status(200).send('OK');
});
```

## Testing Webhooks

### Using Ngrok for Local Development

1. **Install ngrok**
   ```bash
   npm install -g ngrok
   ```

2. **Start your local server**
   ```bash
   node server.js
   ```

3. **Start ngrok**
   ```bash
   ngrok http 3000
   ```

4. **Use the ngrok URL**
   - Your webhook URL: `https://abc123.ngrok.io/webhooks/whatsapp`

### Testing Twilio Webhook

Use Twilio's webhook debugger:

```bash
curl -X POST \
  'https://your-domain.com/twilio/voice' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'CallSid=CA123&From=+1234567890&To=+553322980098&CallStatus=ringing&Direction=inbound'
```

### Testing Meta Webhook

Use Meta's webhook testing tool:

```bash
curl -X POST \
  'https://your-domain.com/webhooks/whatsapp' \
  -H 'Content-Type: application/json' \
  -H 'X-Hub-Signature-256: sha256=...' \
  -d '{
    "object": "whatsapp_business_account",
    "entry": [{
      "id": "390727550789228",
      "changes": [{
        "value": {
          "messaging_product": "whatsapp",
          "messages": [{
            "from": "5511999999999",
            "text": {"body": "Test message"}
          }]
        },
        "field": "messages"
      }]
    }]
  }'
```

## Production Considerations

### High Availability

- Deploy webhooks behind a load balancer
- Implement retry logic for failed deliveries
- Use a message queue (RabbitMQ, Redis) for async processing
- Monitor webhook health and set up alerts

### Performance

- Respond to webhooks quickly (< 3 seconds)
- Process heavy operations asynchronously
- Use worker queues for message processing
- Implement rate limiting to prevent abuse

### Monitoring

- Log all webhook requests and responses
- Track delivery success rates
- Monitor response times
- Set up alerts for failures
- Use tools like Sentry, Datadog, or New Relic

## Troubleshooting

### Common Issues

**Problem: Webhook verification fails**
- Ensure verify token matches exactly
- Check that your endpoint returns the challenge
- Verify your URL is publicly accessible

**Problem: Messages not received**
- Check webhook logs for errors
- Verify subscription to `messages` field
- Ensure your server is online and responding

**Problem: Invalid signature errors**
- Verify your app secret is correct
- Check that you're computing the hash correctly
- Ensure you're using the raw request body

**Problem: Timeout errors**
- Optimize your webhook handler
- Move heavy processing to background jobs
- Increase server capacity if needed

### Debugging Tools

- **Twilio Debugger**: View all Twilio webhook attempts
- **Meta Webhook Debugging**: Enable in app settings
- **Ngrok Inspector**: Inspect requests in real-time
- **Webhook.site**: Test webhooks without a server

## Next Steps

- Set up persistent storage for message history
- Implement message queue for reliability
- Add webhook retry logic
- Configure monitoring and alerting
- Set up automated testing

For more information on webhook security, see [Security Considerations](./README.md#security-considerations).