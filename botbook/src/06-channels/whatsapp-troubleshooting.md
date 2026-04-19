# Troubleshooting Guide

This comprehensive guide helps you diagnose and resolve common issues when integrating WhatsApp Business API with Twilio phone numbers in General Bots.

## Table of Contents

- [Diagnostic Tools](#diagnostic-tools)
- [Verification Issues](#verification-issues)
- [Webhook Problems](#webhook-problems)
- [API Errors](#api-errors)
- [Message Delivery Failures](#message-delivery-failures)
- [Twilio-Specific Issues](#twilio-specific-issues)
- [Meta-Specific Issues](#meta-specific-issues)
- [Performance Issues](#performance-issues)
  - [LLM Hallucination Loop](#issue-llm-hallucination-loop)
- [Security Issues](#security-issues)

## Diagnostic Tools

### Essential Commands

```bash
# Test webhook connectivity
curl -X POST https://your-webhook-url/webhooks/whatsapp \
  -H "Content-Type: application/json" \
  -d '{"test": true}'

# Check Meta API status
curl https://developers.facebook.com/status/

# Verify Twilio number configuration
twilio phone-numbers:list +553322980098

# Test Meta API connectivity
curl -X GET "https://graph.facebook.com/v18.0/1158433381968079" \
  -H "Authorization: Bearer EAAQdlso6aM8BOwl..."
```

### Log Locations

```bash
# General Bots logs
tail -f .gbot/logs/bot.log

# Webhook server logs (Node.js)
pm2 logs whatsapp-webhook

# Webhook server logs (Python)
journalctl -u whatsapp-webhook -f

# Twilio debugger
https://console.twilio.com/us1/develop/monitor/debugger

# Meta webhook debugging
https://developers.facebook.com/apps/YOUR_APP_ID/webhooks/
```

## Verification Issues

### Issue: Phone Number Verification Fails

**Symptoms:**
- Meta cannot verify your Twilio number
- "Verification failed" error after call completes
- No verification code received

**Diagnosis:**

```bash
# Check Twilio number has Voice capability
twilio phone-numbers:info +553322980098

# Test incoming call handling
# Call your Twilio number from another phone
# Check if webhook is triggered
```

**Solutions:**

1. **Verify Voice Webhook Configuration**
   ```bash
   # Check webhook URL is correct
   curl -X POST https://your-domain.com/twilio/voice \
     -d "CallSid=CA123&From=+1234567890&To=+553322980098"
   
   # Expected: TwiML XML response
   ```

2. **Verify Verification Method**
   - Ensure "Phone Call" is selected (not SMS)
   - Twilio numbers don't support SMS for verification
   - Meta must call your number

3. **Check TwiML Response**
   ```xml
   <!-- Your webhook must return valid TwiML -->
   <?xml version="1.0" encoding="UTF-8"?>
   <Response>
     <Gather action="https://your-domain.com/twilio/gather" 
             method="POST" 
             numDigits="6">
       <Say>Please enter your verification code.</Say>
     </Gather>
   </Response>
   ```

4. **Test Webhook Locally**
   ```bash
   # Use ngrok for local testing
   ngrok http 3000
   
   # Update Twilio webhook to ngrok URL
   # Test with actual Meta verification call
   ```

### Issue: Verification Code Not Captured

**Symptoms:**
- Call completes but no code received
- Email not forwarded
- Code not logged

**Diagnosis:**

```bash
# Check webhook server logs
tail -f /var/log/whatsapp-webhook/app.log

# Verify Gather action is configured
# Test DTMF capture with a test call
```

**Solutions:**

1. **Implement Email Forwarding**
   ```javascript
   // Add to your gather handler
   app.post('/twilio/gather', (req, res) => {
     const code = req.body.Digits;
     
     // Send email
     sendEmail({
       to: 'your-email@example.com',
       subject: 'Verification Code',
       body: `Code: ${code}`
     });
     
     // Also log it
     console.log('Verification Code:', code);
     
     res.type('text/xml');
     res.send('<Response><Say>Thank you</Say></Response>');
   });
   ```

2. **Use Voicemail Fallback**
   ```xml
   <Redirect>
     https://twimlets.com/voicemail?Email=your-email@example.com&Transcribe=true
   </Redirect>
   ```

3. **Add Logging**
   ```basic
   REM BASIC logging
   ON WEBHOOK POST TO "/twilio/gather" DO
     LET CODE$ = GET FORM VALUE "Digits"
     LOG "Verification Code: " + CODE$
     PRINT "Code logged"
   END ON
   ```

## Webhook Problems

### Issue: Webhook Verification Fails

**Symptoms:**
- Meta webhook setup shows "Verification failed"
- "Challenge mismatch" error
- 403 Forbidden response

**Diagnosis:**

```bash
# Test webhook verification endpoint
curl "https://your-domain.com/webhooks/whatsapp?hub.verify_token=YOUR_TOKEN&hub.challenge=CHALLENGE"

# Expected response: The challenge string
```

**Solutions:**

1. **Match Verify Token Exactly**
   ```javascript
   // In config.csv
   whatsapp-webhook-verify-token,4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY
   
   // In your code
   const VERIFY_TOKEN = '4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY';
   
   app.get('/webhooks/whatsapp', (req, res) => {
     const token = req.query['hub.verify_token'];
     if (token === VERIFY_TOKEN) {
       res.send(req.query['hub.challenge']);
     } else {
       res.sendStatus(403);
     }
   });
   ```

2. **Return Challenge Correctly**
   - Must return the challenge as plain text
   - Must respond with 200 OK status
   - Must not include HTML or JSON formatting

3. **Check URL Accessibility**
   ```bash
   # Ensure URL is publicly accessible
   curl -v https://your-domain.com/webhooks/whatsapp
   
   # Check firewall rules
   sudo ufw status
   
   # Verify SSL certificate
   openssl s_client -connect your-domain.com:443
   ```

### Issue: Webhook Not Receiving Messages

**Symptoms:**
- Webhook endpoint configured but no messages received
- Messages appear in Meta inbox but not in bot
- No webhook logs

**Diagnosis:**

```bash
# Check webhook subscriptions in Meta dashboard
# Navigate to: WhatsApp > API Setup > Webhook > Subscriptions

# Verify webhook field subscription
curl -X GET "https://graph.facebook.com/v18.0/YOUR_APP_ID/subscriptions" \
  -H "Authorization: Bearer EAAQdlso6aM8BOwl..."
```

**Solutions:**

1. **Subscribe to Messages Field**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > API Setup > Webhook
   2. Click "Manage" webhook fields
   3. Subscribe to: messages
   4. Save changes
   ```

2. **Check Webhook URL**
   ```javascript
   // Ensure correct webhook path
   app.post('/webhooks/whatsapp', (req, res) => {
     console.log('Webhook received:', JSON.stringify(req.body, null, 2));
     res.status(200).send('OK');
   });
   ```

3. **Verify Response Time**
   - Webhook must respond within 3 seconds
   - Process heavy operations asynchronously
   - Return 200 OK immediately

   ```javascript
   app.post('/webhooks/whatsapp', (req, res) => {
     // Acknowledge immediately
     res.status(200).send('OK');
     
     // Process asynchronously
     setImmediate(() => {
       processWebhook(req.body);
     });
   });
   ```

## API Errors

### Issue: 401 Unauthorized

**Symptoms:**
- API calls return 401 status
- "Invalid access token" error
- Messages fail to send

**Diagnosis:**

```bash
# Test access token validity
curl -X GET "https://graph.facebook.com/v18.0/me" \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"

# Expected: JSON with app ID and name
# 401 means token is invalid or expired
```

**Solutions:**

1. **Generate New Access Token**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > API Setup
   2. Click "Temporary Access Token"
   3. Copy and update config.csv
   4. Restart bot
   ```

2. **Check Token Format**
   ```csv
   # config.csv
   # Token must start with EAAQ...
   whatsapp-api-key,EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr
   ```

3. **Verify Token Permissions**
   ```
   In Meta Dashboard:
   1. Go to App Review > Permissions and Features
   2. Ensure "WhatsApp" permission is granted
   3. Check for any additional required permissions
   ```

### Issue: 470 Message Rate Limit

**Symptoms:**
- Messages fail with 470 error
- "Rate limit exceeded" message
- Bulk sending stops working

**Diagnosis:**

```bash
# Check API rate limits
# WhatsApp Business API: 1000 messages/second per WABA

# Monitor message queue
tail -f .gbot/logs/message-queue.log
```

**Solutions:**

1. **Implement Rate Limiting**
   ```javascript
   class RateLimiter {
     constructor(maxRequests = 50, timeWindow = 60) {
       this.maxRequests = maxRequests;
       this.timeWindow = timeWindow;
       this.requests = [];
     }
     
     async send(whatsapp, to, message) {
       while (!this.canMakeRequest()) {
         await this.sleep(1000);
       }
       this.recordRequest();
       return await whatsapp.sendText(to, message);
     }
     
     canMakeRequest() {
       const now = Date.now();
       this.requests = this.requests.filter(t => now - t < this.timeWindow * 1000);
       return this.requests.length < this.maxRequests;
     }
     
     recordRequest() {
       this.requests.push(Date.now());
     }
     
     sleep(ms) {
       return new Promise(resolve => setTimeout(resolve, ms));
     }
   }
   
   const rateLimiter = new RateLimiter(50, 60);
   await rateLimiter.send(whatsapp, to, message);
   ```

2. **Use Message Queuing**
   ```basic
   REM BASIC message queuing
   SUB SEND WHATSAPP WITH RATE LIMIT TO NUMBER$, MESSAGE$
     ADD TO QUEUE "whatsapp-outbound", NUMBER$ + "|" + MESSAGE$
   END SUB
   ```

3. **Monitor Usage**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > API Usage
   2. Monitor message volume
   3. Check for rate limit warnings
   ```

### Issue: Invalid Phone Number Format

**Symptoms:**
- "Invalid phone number" error
- Messages not delivered
- 400 Bad Request

**Diagnosis:**

```bash
# Test phone number format
# Correct format: 5511999999999 (no +, no spaces, no dashes)
```

**Solutions:**

1. **Format Phone Numbers Correctly**
   ```javascript
   function formatPhoneNumber(phone) {
     // Remove all non-digits
     let cleaned = phone.replace(/\D/g, '');
     
     // Remove leading + or 00 if present
     cleaned = cleaned.replace(/^(\+|00)/, '');
     
     // Ensure country code is present
     if (!cleaned.startsWith('55') && cleaned.length === 11) {
       cleaned = '55' + cleaned;
     }
     
     return cleaned;
   }
   
   const formatted = formatPhoneNumber('+55 (11) 99999-9999');
   // Result: 5511999999999
   ```

2. **Validate Before Sending**
   ```basic
   REM BASIC validation
   SUB SEND VALIDATED WHATSAPP NUMBER$, MESSAGE$
     LET CLEANED$ = ""
     FOR I = 1 TO LEN(NUMBER$)
       LET CH$ = MID$(NUMBER$, I, 1)
       IF CH$ >= "0" AND CH$ <= "9" THEN
         CLEANED$ = CLEANED$ + CH$
       END IF
     NEXT I
     
     IF LEN(CLEANED$) < 10 OR LEN(CLEANED$) > 15 THEN
       LOG "Invalid phone number: " + NUMBER$
       EXIT SUB
     END IF
     
     SEND WHATSAPP TO CLEANED$ WITH MESSAGE$
   END SUB
   ```

## Message Delivery Failures

### Issue: Messages Not Delivered (24-Hour Window)

**Symptoms:**
- Messages work when user messages first
- Fail after 24 hours of inactivity
- "24-hour window" error

**Diagnosis:**

```bash
# Check last message timestamp
# Meta allows business-initiated messages only within 24 hours of last user message
```

**Solutions:**

1. **Use Message Templates**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > Message Templates
   2. Create and submit template for approval
   3. Use template for business-initiated messages
   ```

2. **Send Template Message**
   ```javascript
   async function sendTemplate(to, templateName, parameters = []) {
     const response = await axios.post(
       `${baseURL}/${phoneNumberId}/messages`,
       {
         messaging_product: 'whatsapp',
         to: to,
         type: 'template',
         template: {
           name: templateName,
           language: { code: 'pt_BR' },
           components: [{
             type: 'body',
             parameters: parameters.map(p => ({
               type: 'text',
               text: p
             }))
           }]
         }
       },
       { headers: { 'Authorization': `Bearer ${accessToken}` } }
     );
     return response.data;
   }
   ```

3. **Track Last Interaction**
   ```javascript
   // Store last message time
   const lastInteraction = new Map();
   
   async function sendMessage(to, message) {
     const lastTime = lastInteraction.get(to);
     const now = Date.now();
     const hoursSince = (now - lastTime) / (1000 * 60 * 60);
     
     if (hoursSince > 24) {
       // Use template
       await sendTemplate(to, 'reengagement_template', []);
     } else {
       // Use regular message
       await sendText(to, message);
     }
     
     lastInteraction.set(to, now);
   }
   ```

### Issue: Media Messages Fail to Send

**Symptoms:**
- Text messages work, media fails
- "Media upload failed" error
- Images not delivered

**Diagnosis:**

```bash
# Test media URL accessibility
curl -I https://your-media-url.com/image.jpg

# Expected: 200 OK with Content-Type: image/jpeg
```

**Solutions:**

1. **Verify Media URL**
   - Must be publicly accessible
   - Must use HTTPS
   - Must return correct Content-Type
   - Should be under 5MB for images

2. **Upload Media First**
   ```javascript
   async function uploadMedia(mediaUrl) {
     const response = await axios.post(
       `${baseURL}/${phoneNumberId}/media`,
       {
         file: mediaUrl,
         type: 'image/jpeg'
       },
       { headers: { 'Authorization': `Bearer ${accessToken}` } }
     );
     return response.data.id;
   }
   
   async function sendImageWithUpload(to, imageUrl, caption) {
     const mediaId = await uploadMedia(imageUrl);
     await sendMediaById(to, mediaId, caption);
   }
   ```

3. **Use Approved Media Hosts**
   - Meta's media servers (preferred)
   - AWS S3 with public access
   - CloudFront CDN
   - Avoid self-hosted media for reliability

## Twilio-Specific Issues

### Issue: TwiML Bin Not Working

**Symptoms:**
- TwiML Bin returns 404
- Invalid TwiML error
- Webhook not triggered

**Diagnosis:**

```bash
# Test TwiML Bin URL
curl -X POST https://handler.twilio.com/twiml/EH123...
```

**Solutions:**

1. **Validate TwiML Syntax**
   ```xml
   <!-- Correct TwiML structure -->
   <?xml version="1.0" encoding="UTF-8"?>
   <Response>
     <Gather action="https://your-domain.com/gather" method="POST">
       <Say voice="alice">Enter your code</Say>
     </Gather>
   </Response>
   ```

2. **Use Custom Webhook Instead**
   ```javascript
   // Replace TwiML Bin with custom server
   app.post('/twilio/voice', (req, res) => {
     const twiml = new twilio.twiml.VoiceResponse();
     const gather = twiml.gather({
       action: '/twilio/gather',
       method: 'POST'
     });
     gather.say('Enter your code');
     res.type('text/xml');
     res.send(twiml.toString());
   });
   ```

### Issue: Call Not Forwarded to Email

**Symptoms:**
- Voicemail not received
- Email not sent
- Transcription not working

**Diagnosis:**

```bash
# Check TwiML voicemail URL
https://twimlets.com/voicemail?Email=your-email@example.com
```

**Solutions:**

1. **Implement Custom Voicemail**
   ```javascript
   app.post('/twilio/gather', (req, res) => {
     const code = req.body.Digits;
     
     // Send email with code
     transporter.sendMail({
       from: 'bot@example.com',
       to: 'your-email@example.com',
       subject: 'WhatsApp Verification Code',
       text: `Your code is: ${code}`
     });
     
     // Also send via SMS as backup
     client.messages.create({
       to: '+5511999999999',
       from: process.env.TWILIO_NUMBER,
       body: `Code: ${code}`
     });
     
     res.type('text/xml');
     res.send('<Response><Say>Code sent to email</Say></Response>');
   });
   ```

2. **Add Multiple Notification Channels**
   - Email (primary)
   - SMS backup
   - Database logging
   - Webhook notification

## Meta-Specific Issues

### Issue: Phone Number Not Approved

**Symptoms:**
- Number shows "Not Verified"
- Cannot send messages
- "Number quality" error

**Diagnosis:**

```bash
# Check number status in Meta Dashboard
# WhatsApp Accounts > Phone Numbers > Select Number
```

**Solutions:**

1. **Verify Number Quality**
   - Number must be from reputable provider
   - Avoid VOIP numbers
   - Use local numbers for target country

2. **Complete Business Verification**
   ```
   In Meta Dashboard:
   1. Go to Business Settings > Business Verification
   2. Submit business documents
   3. Wait for approval (7-14 days)
   ```

3. **Request Higher Limits**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > API Settings
   2. Request increased messaging limits
   3. Provide business justification
   ```

### Issue: Webhook Signature Verification Fails

**Symptoms:**
- All webhooks rejected
- "Invalid signature" errors
- Security check failures

**Diagnosis:**

```bash
# Check X-Hub-Signature-256 header
curl -X POST https://your-domain.com/webhooks/whatsapp \
  -H "X-Hub-Signature-256: sha256=..." \
  -d '{"test": true}'
```

**Solutions:**

1. **Implement Signature Verification**
   ```javascript
   const crypto = require('crypto');
   
   function verifySignature(payload, signature, appSecret) {
     const expectedSignature = 'sha256=' + 
       crypto.createHmac('sha256', appSecret)
         .update(payload)
         .digest('hex');
     return crypto.timingSafeEqual(
       Buffer.from(signature),
       Buffer.from(expectedSignature)
     );
   }
   
   app.post('/webhooks/whatsapp', (req, res) => {
     const signature = req.headers['x-hub-signature-256'];
     const payload = JSON.stringify(req.body);
     
     if (!verifySignature(payload, signature, APP_SECRET)) {
       return res.status(403).send('Invalid signature');
     }
     
     // Process webhook
     processWebhook(req.body);
     res.status(200).send('OK');
   });
   ```

2. **Get App Secret**
   ```
   In Meta Dashboard:
   1. Go to App Settings > Basic
   2. Copy App Secret
   3. Store securely in environment variable
   ```

## Performance Issues

### Issue: Slow Webhook Response

**Symptoms:**
- Webhooks timeout
- Meta shows "Webhook slow" warning
- Messages delayed

**Diagnosis:**

```bash
# Measure webhook response time
time curl -X POST https://your-domain.com/webhooks/whatsapp \
  -H "Content-Type: application/json" \
  -d '{"object":"whatsapp_business_account","entry":[]}'
```

**Solutions:**

1. **Optimize Webhook Handler**
   ```javascript
   app.post('/webhooks/whatsapp', (req, res) => {
     // Acknowledge immediately
     res.status(200).send('OK');
     
     // Process asynchronously
     setImmediate(() => {
       processMessageAsync(req.body);
     });
   });
   
   async function processMessageAsync(data) {
     // Heavy processing here
     await handleMessage(data);
   }
   ```

2. **Use Worker Queues**
   ```javascript
   const { Queue } = require('bull');
   
   const webhookQueue = new Queue('webhook-processing', {
     redis: { host: 'localhost', port: 6379 }
   });
   
   app.post('/webhooks/whatsapp', async (req, res) => {
     await webhookQueue.add('process', req.body);
     res.status(200).send('OK');
   });
   
   webhookQueue.process('process', async (job) => {
     await handleMessage(job.data);
   });
   ```

3. **Monitor Response Times**
   ```javascript
   const responseTime = require('response-time');
   
   app.use(responseTime((req, res, time) => {
     console.log(`${req.method} ${req.path} ${time}ms`);
   }));
   ```

### Issue: High Memory Usage

**Symptoms:**
- Bot crashes with out of memory
- Slow response times
- High server load

**Diagnosis:**

```bash
# Check memory usage
node --inspect app.js
# Open chrome://inspect in Chrome

# Monitor process
pm2 monit
```

**Solutions:**

1. **Limit Conversation State**
   ```javascript
   // Don't store unlimited conversation history
   const MAX_HISTORY = 50;
   const conversations = new Map();
   
   function addToConversation(phone, message) {
     let history = conversations.get(phone) || [];
     history.push({ message, time: Date.now() });
     
     if (history.length > MAX_HISTORY) {
       history = history.slice(-MAX_HISTORY);
     }
     
     conversations.set(phone, history);
   }
   ```

2. **Use Redis for State**
   ```javascript
   const redis = require('redis');
   const client = redis.createClient();
   
   async function setState(phone, key, value) {
     await client.hset(`conversation:${phone}`, key, JSON.stringify(value));
   }
   
   async function getState(phone, key) {
     const value = await client.hget(`conversation:${phone}`, key);
     return value ? JSON.parse(value) : null;
   }
   ```

3. **Implement Cleanup**
   ```javascript
   // Clean up old conversations
   setInterval(() => {
     const now = Date.now();
     const MAX_AGE = 24 * 60 * 60 * 1000; // 24 hours
     
     for (const [phone, data] of conversations.entries()) {
       if (now - data.lastActivity > MAX_AGE) {
         conversations.delete(phone);
       }
     }
   }, 60 * 60 * 1000); // Every hour
   ```

### Issue: LLM Hallucination Loop

**Symptoms:**
- Bot sends repeated content endlessly
- Same token/phrase appears multiple times (e.g., "GBJ2KP GBJ2KP GBJ2KP...")
- Stream never completes
- Logs show rapid repeated patterns

**Diagnosis:**

```bash
# Check logs for hallucination detection
grep -E "hallucination (detected|loop)" botserver.log

# Look for identical token repetition
grep "identical token repeated" botserver.log
```

**Solutions:**

1. **Built-in Hallucination Detector**
   General Bots includes automatic detection for LLM hallucination loops in the LLM layer:
   - Works for all channels (Web, WhatsApp, Telegram, etc.)
   - Detects identical tokens repeated 10+ times
   - Detects patterns repeated 5+ times consecutively
   - Detects patterns appearing 8+ times in recent 500 chars
   - Automatically stops stream and sends accumulated content

2. **Detection Thresholds**
   ```rust
   // Configured in botserver/src/llm/hallucination_detector.rs
   HallucinationConfig {
       min_text_length: 50,           // Minimum text before detection
       pattern_lengths: [3,4,5,6,8,10,15,20],  // Pattern sizes to check
       consecutive_threshold: 5,      // Consecutive repetitions to trigger
       occurrence_threshold: 8,       // Total occurrences in window to trigger
       recent_text_window: 500,       // Window size for occurrence counting
       identical_token_threshold: 10, // Identical tokens to trigger
   }
   ```

3. **Monitor Detection in Logs**
   ```bash
   # Look for hallucination warnings (works for all channels)
   grep -E "hallucination (detected|loop)" botserver.log

   # Example log output:
   # WARN LLM hallucation detected: identical token repeated 10 times: "GBJ2KP"
   # WARN LLM hallucination loop detected: pattern "XYZ123"
   # WARN WA hallucination detected: Some("XYZ123"), stopping stream
   ```

4. **Reduce Hallucination Risk**
   - Use clear, specific system prompts
   - Set appropriate temperature (0.7-0.9 for chat)
   - Limit max tokens in responses
   - Use well-structured prompts with examples

5. **Customize Detection (Advanced)**
   ```rust
   // Adjust thresholds in botserver/src/llm/hallucination_detector.rs
   let config = HallucinationConfig {
       identical_token_threshold: 8,  // Lower for more aggressive detection
       consecutive_threshold: 4,      // Fewer repetitions to trigger
       ..Default::default()
   };
   let mut detector = HallucinationDetector::new(config);
   ```

## Security Issues

### Issue: Access Token Exposed

**Symptoms:**
- Token found in logs
- Token in version control
- Unauthorized API usage

**Solutions:**

1. **Use Environment Variables**
   ```bash
   # .env file (add to .gitignore)
   WHATSAPP_API_KEY=EAAQdlso6aM8BOwl...
   WHATSAPP_PHONE_NUMBER_ID=1158433381968079
   ```

   ```javascript
   // config.js
   module.exports = {
     whatsapp: {
       apiKey: process.env.WHATSAPP_API_KEY,
       phoneNumberId: process.env.WHATSAPP_PHONE_NUMBER_ID
     }
   };
   ```

2. **Secure Config File**
   ```bash
   # Set proper permissions
   chmod 600 .gbot/config.csv
   chown bot-user:bot-group .gbot/config.csv
   ```

3. **Rotate Tokens Regularly**
   ```
   In Meta Dashboard:
   1. Go to WhatsApp > API Setup
   2. Generate new temporary token
   3. Update config.csv
   4. Restart bot
   5. Invalidate old token
   ```

### Issue: Webhook Abuse

**Symptoms:**
- Excessive webhook calls
- Spam messages
- High server load

**Solutions:**

1. **Rate Limit Webhooks**
   ```javascript
   const rateLimit = require('express-rate-limit');
   
   const webhookLimiter = rateLimit({
     windowMs: 60 * 1000, // 1 minute
     max: 100, // 100 requests per minute
     keyGenerator: (req) => {
       return req.body.entry?.[0]?.changes?.[0]?.value?.messages?.[0]?.from || 'unknown';
     }
   });
   
   app.post('/webhooks/whatsapp', webhookLimiter, (req, res) => {
     // Process webhook
   });
   ```

2. **Validate Payload**
   ```javascript
   function validateWebhookPayload(data) {
     if (!data.object === 'whatsapp_business_account') {
       return false;
     }
     
     if (!data.entry || !Array.isArray(data.entry)) {
       return false;
     }
     
     return true;
   }
   
   app.post('/webhooks/whatsapp', (req, res) => {
     if (!validateWebhookPayload(req.body)) {
       return res.status(400).send('Invalid payload');
     }
     
     // Process webhook
   });
   ```

3. **Monitor for Anomalies**
   ```javascript
   const messageCounts = new Map();
   
   function checkForAbuse(phoneNumber) {
     const count = messageCounts.get(phoneNumber) || 0;
     messageCounts.set(phoneNumber, count + 1);
     
     if (count > 100) {
       console.warn(`Potential abuse from ${phoneNumber}`);
       // Block or throttle
     }
   }
   ```

## Getting Help

If you're still experiencing issues after following this guide:

1. **Check Community Resources**
   - [General Bots Discord](https://discord.gg/general-bots)
   - [Meta for Developers Forum](https://developers.facebook.com/community/)
   - [Twilio Community](https://www.twilio.com/help/faq)

2. **Enable Debug Logging**
   ```basic
   REM Enable detailed logging
   SET DEBUG MODE TO "verbose"
   LOG ALL WEBHOOK REQUESTS
   LOG ALL API RESPONSES
   ```

3. **Collect Diagnostic Information**
   ```bash
   # Export logs
   tar -czf whatsapp-debug-$(date +%Y%m%d).tar.gz \
     .gbot/logs/ \
     /var/log/whatsapp-webhook/
   
   # Include configuration (redact sensitive data)
   # Include error messages
   # Include timestamps
   ```

4. **Create Support Ticket**
   - Include diagnostic tarball
   - Describe expected vs actual behavior
   - List steps to reproduce
   - Include error messages

For the latest troubleshooting information, see [Webhook Configuration Guide](./webhooks.md) or [Code Examples](./examples.md).