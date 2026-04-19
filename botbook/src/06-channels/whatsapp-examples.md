# Code Examples for WhatsApp Integration

This page provides practical code examples for integrating WhatsApp Business API with General Bots, covering common use cases and implementations in multiple languages.

## Table of Contents

- [BASIC Examples](#basic-examples)
- [Node.js Examples](#nodejs-examples)
- [Python Examples](#python-examples)
- [Common Use Cases](#common-use-cases)
- [Advanced Scenarios](#advanced-scenarios)

## BASIC Examples

### Simple Message Sender

```basic
REM Send a simple text message via WhatsApp
SEND WHATSAPP TO "+5511999999999" WITH "Hello from General Bots!"
```

### Interactive Bot Response

```basic
REM Handle incoming WhatsApp messages
ON WHATSAPP MESSAGE RECEIVED
  REM Get message details
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  LET MESSAGE_ID$ = GET WHATSAPP MESSAGE ID
  
  REM Log the incoming message
  LOG "Received from " + SENDER$ + ": " + MESSAGE$
  
  REM Process based on message content
  IF INSTR(UCASE$(MESSAGE$), "HELP") > 0 THEN
    SEND HELP MENU TO SENDER$
  ELSEIF INSTR(UCASE$(MESSAGE$), "STATUS") > 0 THEN
    SEND STATUS UPDATE TO SENDER$
  ELSE
    SEND DEFAULT RESPONSE TO SENDER$
  END IF
END ON

SUB SEND HELP MENU TO NUMBER$
  LET MENU$ = "🤖 *Bot Menu*" + CHR$(10)
  MENU$ = MENU$ + CHR$(10) + "1. 📊 Status - Check system status"
  MENU$ = MENU$ + CHR$(10) + "2. 🌐 Weather - Get weather info"
  MENU$ = MENU$ + CHR$(10) + "3. 📧 Contact - Get support contact"
  MENU$ = MENU$ + CHR$(10) + CHR$(10) + "Reply with a number or keyword"
  
  SEND WHATSAPP TO NUMBER$ WITH MENU$
END SUB

SUB SEND STATUS UPDATE TO NUMBER$
  LET STATUS$ = "✅ *System Status*" + CHR$(10)
  STATUS$ = STATUS$ + CHR$(10) + "🔹 Bot: Online"
  STATUS$ = STATUS$ + CHR$(10) + "🔹 Uptime: " + GET SYSTEM UPTIME$()
  STATUS$ = STATUS$ + CHR$(10) + "🔹 Memory: " + GET MEMORY USAGE$()
  
  SEND WHATSAPP TO NUMBER$ WITH STATUS$
END SUB

SUB SEND DEFAULT RESPONSE TO NUMBER$
  LET RESPONSE$ = "👋 Hello! I'm your General Bot assistant."
  RESPONSE$ = RESPONSE$ + CHR$(10) + CHR$(10) + "Type *help* to see available commands."
  
  SEND WHATSAPP TO NUMBER$ WITH RESPONSE$
END SUB
```

### Message with Formatting

```basic
REM Send message with rich text formatting
SEND WHATSAPP TO "+5511999999999" WITH "*Bold text* and _italics_ and ~strikethrough~"
```

### Send Location

```basic
REM Send a location message
SEND WHATSAPP TO "+5511999999999" WITH LOCATION AT "-23.5505,-46.6333" NAMED "São Paulo" WITH ADDRESS "São Paulo, Brazil"
```

### Send Media (Image)

```basic
REM Send an image from URL
SEND WHATSAPP TO "+5511999999999" WITH IMAGE FROM "https://example.com/image.jpg" AND CAPTION "Check this out!"
```

### Interactive Menu with Button Response

```basic
REM Create an interactive menu
REM Note: Interactive templates must be pre-approved by Meta

REM Send a list message
SEND WHATSAPP TO "+5511999999999" WITH LIST "Choose an option" WITH HEADER "Main Menu" AND ITEMS "Status,Help,Contact,About"
```

## Node.js Examples

### Configuration Setup

```javascript
// config.js
module.exports = {
  whatsapp: {
    apiKey: process.env.WHATSAPP_API_KEY || 'EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr',
    phoneNumberId: process.env.WHATSAPP_PHONE_NUMBER_ID || '1158433381968079',
    wabaId: process.env.WHATSAPP_WABA_ID || '390727550789228',
    verifyToken: process.env.WHATSAPP_VERIFY_TOKEN || '4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY',
    apiVersion: 'v18.0'
  }
};
```

### Send Text Message

```javascript
// whatsapp-client.js
const axios = require('axios');
const config = require('./config');

class WhatsAppClient {
  constructor() {
    this.baseURL = `https://graph.facebook.com/${config.whatsapp.apiVersion}`;
    this.phoneNumberId = config.whatsapp.phoneNumberId;
    this.accessToken = config.whatsapp.apiKey;
  }

  async sendText(to, message) {
    try {
      const response = await axios.post(
        `${this.baseURL}/${this.phoneNumberId}/messages`,
        {
          messaging_product: 'whatsapp',
          to: to.replace(/[^\d]/g, ''), // Remove non-digits
          type: 'text',
          text: {
            body: message
          }
        },
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return response.data;
    } catch (error) {
      console.error('Error sending message:', error.response?.data || error.message);
      throw error;
    }
  }

  async sendFormattedText(to, message) {
    // WhatsApp formatting: *bold*, _italics_, ~strikethrough~, ```monospace```
    return await this.sendText(to, message);
  }
}

module.exports = WhatsAppClient;
```

### Send Media Message

```javascript
// Extend WhatsAppClient with media methods
class WhatsAppClientWithMedia extends WhatsAppClient {
  async sendImage(to, imageUrl, caption = '') {
    try {
      const response = await axios.post(
        `${this.baseURL}/${this.phoneNumberId}/messages`,
        {
          messaging_product: 'whatsapp',
          to: to.replace(/[^\d]/g, ''),
          type: 'image',
          image: {
            link: imageUrl,
            caption: caption
          }
        },
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return response.data;
    } catch (error) {
      console.error('Error sending image:', error.response?.data || error.message);
      throw error;
    }
  }

  async sendDocument(to, documentUrl, filename, caption = '') {
    try {
      const response = await axios.post(
        `${this.baseURL}/${this.phoneNumberId}/messages`,
        {
          messaging_product: 'whatsapp',
          to: to.replace(/[^\d]/g, ''),
          type: 'document',
          document: {
            link: documentUrl,
            filename: filename,
            caption: caption
          }
        },
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return response.data;
    } catch (error) {
      console.error('Error sending document:', error.response?.data || error.message);
      throw error;
    }
  }

  async sendLocation(to, latitude, longitude, name, address) {
    try {
      const response = await axios.post(
        `${this.baseURL}/${this.phoneNumberId}/messages`,
        {
          messaging_product: 'whatsapp',
          to: to.replace(/[^\d]/g, ''),
          type: 'location',
          location: {
            latitude: latitude,
            longitude: longitude,
            name: name,
            address: address
          }
        },
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
            'Content-Type': 'application/json'
          }
        }
      );

      return response.data;
    } catch (error) {
      console.error('Error sending location:', error.response?.data || error.message);
      throw error;
    }
  }
}

module.exports = WhatsAppClientWithMedia;
```

### Webhook Handler (Express)

```javascript
// webhook-handler.js
const express = require('express');
const WhatsAppClient = require('./whatsapp-client');

const app = express();
app.use(express.json());

const whatsapp = new WhatsAppClient();

// Webhook verification (GET request)
app.get('/webhooks/whatsapp', (req, res) => {
  const mode = req.query['hub.mode'];
  const token = req.query['hub.verify_token'];
  const challenge = req.query['hub.challenge'];

  const VERIFY_TOKEN = '4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY';

  if (mode === 'subscribe' && token === VERIFY_TOKEN) {
    console.log('✅ Webhook verified');
    res.status(200).send(challenge);
  } else {
    console.log('❌ Webhook verification failed');
    res.sendStatus(403);
  }
});

// Webhook message handler (POST request)
app.post('/webhooks/whatsapp', async (req, res) => {
  try {
    const data = req.body;

    if (data.object === 'whatsapp_business_account') {
      for (const entry of data.entry) {
        for (const change of entry.changes) {
          if (change.field === 'messages') {
            const message = change.value.messages[0];
            await handleMessage(message);
          }
        }
      }
    }

    res.status(200).send('OK');
  } catch (error) {
    console.error('❌ Webhook error:', error);
    res.status(500).send('Error');
  }
});

async function handleMessage(message) {
  const from = message.from;
  const body = message.text.body;
  const messageId = message.id;
  const timestamp = message.timestamp;

  console.log(`📩 Message from ${from}: ${body}`);

  // Process the message
  const response = await generateResponse(body);

  // Send reply
  await whatsapp.sendText(from, response);

  // Mark as read (optional)
  await markAsRead(messageId);
}

async function generateResponse(userMessage) {
  const lowerMessage = userMessage.toLowerCase();

  if (lowerMessage.includes('help') || lowerMessage === '1') {
    return `🤖 *Bot Menu*

1. 📊 Status - Check system status
2. 🌐 Weather - Get weather info  
3. 📧 Contact - Get support contact

Reply with a number or keyword`;
  } else if (lowerMessage.includes('status') || lowerMessage === '2') {
    return `✅ *System Status*

🔹 Bot: Online
🔹 Uptime: ${process.uptime().toFixed(2)}s
🔹 Memory: ${(process.memoryUsage().heapUsed / 1024 / 1024).toFixed(2)}MB

Type *help* for more options.`;
  } else {
    return `👋 Hello! I'm your General Bot assistant.

Type *help* to see available commands.

You said: ${userMessage}`;
  }
}

async function markAsRead(messageId) {
  try {
    await axios.post(
      `https://graph.facebook.com/${config.whatsapp.apiVersion}/${messageId}`,
      {
        messaging_product: 'whatsapp',
        status: 'read'
      },
      {
        headers: {
          'Authorization': `Bearer ${config.whatsapp.apiKey}`,
          'Content-Type': 'application/json'
        }
      }
    );
  } catch (error) {
    console.error('Error marking as read:', error.message);
  }
}

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`🚀 WhatsApp webhook server running on port ${PORT}`);
});
```

### Interactive Bot with Conversation State

```javascript
// conversation-bot.js
const WhatsAppClient = require('./whatsapp-client');

class ConversationBot {
  constructor() {
    this.whatsapp = new WhatsAppClient();
    this.conversations = new Map(); // Store conversation state per user
  }

  async handleMessage(from, message) {
    // Get or create conversation state
    let state = this.conversations.get(from) || {
      step: 'initial',
      data: {}
    };

    // Process based on current step
    const response = await this.processStep(state, message);
    
    // Update state
    this.conversations.set(from, state);

    // Send response
    await this.whatsapp.sendText(from, response);
  }

  async processStep(state, message) {
    const lowerMessage = message.toLowerCase();

    switch (state.step) {
      case 'initial':
        if (lowerMessage.includes('order')) {
          state.step = 'awaiting_product';
          return '🛒 *Order Process*\n\nWhat product would you like to order?';
        } else if (lowerMessage.includes('support')) {
          state.step = 'awaiting_issue';
          return '🎫 *Support*\n\nPlease describe your issue:';
        } else {
          return '👋 Welcome!\n\nReply with:\n• "order" to place an order\n• "support" for help';
        }

      case 'awaiting_product':
        state.data.product = message;
        state.step = 'awaiting_quantity';
        return `📦 Product: *${message}*\n\nHow many would you like?`;

      case 'awaiting_quantity':
        const quantity = parseInt(message);
        if (isNaN(quantity) || quantity <= 0) {
          return '❌ Please enter a valid number.';
        }
        state.data.quantity = quantity;
        state.step = 'confirm_order';
        return `📋 *Order Summary*\n\nProduct: ${state.data.product}\nQuantity: ${quantity}\n\nReply "confirm" to place order or "cancel" to start over.`;

      case 'confirm_order':
        if (lowerMessage === 'confirm') {
          // Process order
          const orderId = await this.placeOrder(state.data);
          state.step = 'initial';
          state.data = {};
          return `✅ Order placed successfully!\n\nOrder ID: ${orderId}\n\nThank you for your business!`;
        } else if (lowerMessage === 'cancel') {
          state.step = 'initial';
          state.data = {};
          return '❌ Order cancelled.\n\nReply with "order" to start over.';
        } else {
          return 'Please reply "confirm" or "cancel".';
        }

      case 'awaiting_issue':
        state.data.issue = message;
        state.step = 'confirm_ticket';
        return `📝 *Issue Description*\n\n${message}\n\nReply "confirm" to submit ticket or "cancel" to discard.`;

      case 'confirm_ticket':
        if (lowerMessage === 'confirm') {
          const ticketId = await this.createTicket(state.data);
          state.step = 'initial';
          state.data = {};
          return `✅ Support ticket created!\n\nTicket ID: ${ticketId}\n\nOur team will review your issue shortly.`;
        } else if (lowerMessage === 'cancel') {
          state.step = 'initial';
          state.data = {};
          return '❌ Ticket cancelled.\n\nReply with "support" to start over.';
        } else {
          return 'Please reply "confirm" or "cancel".';
        }

      default:
        state.step = 'initial';
        return '👋 Welcome!\n\nReply with:\n• "order" to place an order\n• "support" for help';
    }
  }

  async placeOrder(data) {
    // Implement order placement logic
    return 'ORD-' + Date.now();
  }

  async createTicket(data) {
    // Implement ticket creation logic
    return 'TKT-' + Date.now();
  }
}

module.exports = ConversationBot;
```

## Python Examples

### Configuration Setup

```python
# config.py
import os
from dataclasses import dataclass

@dataclass
class WhatsAppConfig:
    api_key: str = os.getenv('WHATSAPP_API_KEY', 'EAAQdlso6aM8BOwlhc3yM6bbJkGyibQPGJd87zFDHtfaFoJDJPohMl2c5nXs4yYuuHwoXJWx0rQKo0VXgTwThPYzqLEZArOZBhCWPBUpq7YlkEJXFAgB6ZAb3eoUzZAMgNZCZA1sg11rT2G8e1ZAgzpRVRffU4jmMChc7ybcyIwbtGOPKZAXKcNoMRfUwssoLhDWr')
    phone_number_id: str = os.getenv('WHATSAPP_PHONE_NUMBER_ID', '1158433381968079')
    waba_id: str = os.getenv('WHATSAPP_WABA_ID', '390727550789228')
    verify_token: str = os.getenv('WHATSAPP_VERIFY_TOKEN', '4qIogZadggQ.BEoMeciXIdl_MlkV_1DTx8Z_i0bYPxtSJwKSbH0FKlY')
    api_version: str = 'v18.0'

config = WhatsAppConfig()
```

### WhatsApp Client

```python
# whatsapp_client.py
import requests
from typing import Dict, Optional
from config import config

class WhatsAppClient:
    def __init__(self):
        self.base_url = f"https://graph.facebook.com/{config.api_version}"
        self.phone_number_id = config.phone_number_id
        self.access_token = config.api_key
        self.headers = {
            'Authorization': f'Bearer {self.access_token}',
            'Content-Type': 'application/json'
        }

    def send_text(self, to: str, message: str) -> Dict:
        """Send a text message via WhatsApp"""
        url = f"{self.base_url}/{self.phone_number_id}/messages"
        
        # Clean phone number (remove non-digits)
        to = ''.join(filter(str.isdigit, to))
        
        payload = {
            'messaging_product': 'whatsapp',
            'to': to,
            'type': 'text',
            'text': {
                'body': message
            }
        }
        
        response = requests.post(url, json=payload, headers=self.headers)
        response.raise_for_status()
        return response.json()

    def send_image(self, to: str, image_url: str, caption: str = '') -> Dict:
        """Send an image message"""
        url = f"{self.base_url}/{self.phone_number_id}/messages"
        
        to = ''.join(filter(str.isdigit, to))
        
        payload = {
            'messaging_product': 'whatsapp',
            'to': to,
            'type': 'image',
            'image': {
                'link': image_url,
                'caption': caption
            }
        }
        
        response = requests.post(url, json=payload, headers=self.headers)
        response.raise_for_status()
        return response.json()

    def send_location(self, to: str, latitude: float, longitude: float, 
                      name: str, address: str) -> Dict:
        """Send a location message"""
        url = f"{self.base_url}/{self.phone_number_id}/messages"
        
        to = ''.join(filter(str.isdigit, to))
        
        payload = {
            'messaging_product': 'whatsapp',
            'to': to,
            'type': 'location',
            'location': {
                'latitude': latitude,
                'longitude': longitude,
                'name': name,
                'address': address
            }
        }
        
        response = requests.post(url, json=payload, headers=self.headers)
        response.raise_for_status()
        return response.json()

    def mark_as_read(self, message_id: str) -> Dict:
        """Mark a message as read"""
        url = f"{self.base_url}/{message_id}"
        
        payload = {
            'messaging_product': 'whatsapp',
            'status': 'read'
        }
        
        response = requests.post(url, json=payload, headers=self.headers)
        response.raise_for_status()
        return response.json()
```

### Flask Webhook Handler

```python
# webhook_handler.py
from flask import Flask, request, jsonify
import logging
from whatsapp_client import WhatsAppClient
from config import config

app = Flask(__name__)
whatsapp = WhatsAppClient()
logging.basicConfig(level=logging.INFO)

@app.route('/webhooks/whatsapp', methods=['GET'])
def verify_webhook():
    """Verify webhook with Meta"""
    mode = request.args.get('hub.mode')
    token = request.args.get('hub.verify_token')
    challenge = request.args.get('hub.challenge')

    VERIFY_TOKEN = config.verify_token

    if mode == 'subscribe' and token == VERIFY_TOKEN:
        logging.info("✅ Webhook verified")
        return challenge, 200
    else:
        logging.error("❌ Webhook verification failed")
        return 'Forbidden', 403

@app.route('/webhooks/whatsapp', methods=['POST'])
def webhook_handler():
    """Handle incoming WhatsApp messages"""
    try:
        data = request.get_json()

        if data.get('object') == 'whatsapp_business_account':
            for entry in data.get('entry', []):
                for change in entry.get('changes', []):
                    if change.get('field') == 'messages':
                        message = change['value']['messages'][0]
                        handle_message(message)

        return 'OK', 200
    except Exception as e:
        logging.error(f"❌ Webhook error: {e}")
        return 'Error', 500

def handle_message(message):
    """Process incoming message"""
    from_number = message['from']
    body = message['text']['body']
    message_id = message['id']

    logging.info(f"📩 Message from {from_number}: {body}")

    # Generate response
    response = generate_response(body)

    # Send reply
    whatsapp.send_text(from_number, response)

    # Mark as read
    whatsapp.mark_as_read(message_id)

def generate_response(user_message: str) -> str:
    """Generate bot response based on user input"""
    lower_message = user_message.lower()

    if 'help' in lower_message or lower_message == '1':
        return """🤖 *Bot Menu*

1. 📊 Status - Check system status
2. 🌐 Weather - Get weather info
3. 📧 Contact - Get support contact

Reply with a number or keyword"""

    elif 'status' in lower_message or lower_message == '2':
        return f"""✅ *System Status*

🔹 Bot: Online
🔹 Uptime: Active
🔹 Version: 1.0.0

Type *help* for more options."""

    else:
        return f"""👋 Hello! I'm your General Bot assistant.

Type *help* to see available commands.

You said: {user_message}"""

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
```

## Common Use Cases

### 1. Order Confirmation Bot

```basic
REM Order confirmation workflow
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  
  IF LEFT$(MESSAGE$, 5) = "ORDER" THEN
    REM Extract order ID (format: ORDER 12345)
    LET ORDER_ID$ = MID$(MESSAGE$, 7)
    
    REM Fetch order details from database
    LET ORDER_DETAILS$ = GET ORDER DETAILS ORDER_ID$
    
    REM Send confirmation
    SEND WHATSAPP TO SENDER$ WITH "✅ Order " + ORDER_ID$ + " confirmed!" + CHR$(10) + ORDER_DETAILS$
  ELSE
    SEND WHATSAPP TO SENDER$ WITH "Send 'ORDER <ID>' to check your order status"
  END IF
END ON
```

### 2. Weather Information Bot

```javascript
// Node.js weather bot
async function handleWeatherRequest(location) {
  try {
    // Call weather API
    const weatherData = await fetchWeatherData(location);
    
    // Format response
    const response = `🌤️ *Weather in ${location}*

Temperature: ${weatherData.main.temp}°C
Condition: ${weatherData.weather[0].description}
Humidity: ${weatherData.main.humidity}%
Wind: ${weatherData.wind.speed} m/s

Have a great day!`;

    return response;
  } catch (error) {
    return '❌ Unable to fetch weather data. Please try again.';
  }
}

// In webhook handler
if (lowerMessage.includes('weather')) {
  const location = lowerMessage.replace('weather', '').trim() || 'São Paulo';
  const response = await handleWeatherRequest(location);
  await whatsapp.sendText(from, response);
}
```

### 3. Support Ticket System

```python
# Python support ticket bot
class SupportBot:
    def __init__(self):
        self.whatsapp = WhatsAppClient()
        self.tickets = {}
    
    def handle_support_request(self, from_number, issue):
        # Create ticket
        ticket_id = f"TKT-{int(time.time())}"
        self.tickets[ticket_id] = {
            'from': from_number,
            'issue': issue,
            'status': 'open',
            'created_at': datetime.now()
        }
        
        # Send confirmation
        response = f"""🎫 *Support Ticket Created*

Ticket ID: {ticket_id}
Status: Open
Issue: {issue}

Our team will review your request shortly.
Reply with 'STATUS {ticket_id}' to check updates."""
        
        self.whatsapp.send_text(from_number, response)
    
    def check_ticket_status(self, from_number, ticket_id):
        if ticket_id in self.tickets:
            ticket = self.tickets[ticket_id]
            response = f"""📋 *Ticket Status*

ID: {ticket_id}
Status: {ticket['status'].title()}
Created: {ticket['created_at'].strftime('%Y-%m-%d %H:%M')}

Issue: {ticket['issue']}"""
        else:
            response = f"❌ Ticket {ticket_id} not found."
        
        self.whatsapp.send_text(from_number, response)
```

### 4. Appointment Scheduling

```basic
REM Appointment scheduling bot
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  
  IF LEFT$(MESSAGE$, 5) = "BOOK " THEN
    REM Parse date (format: BOOK 2024-01-15 14:00)
    LET DATE_STR$ = MID$(MESSAGE$, 6)
    
    REM Check availability
    IF CHECK AVAILABILITY DATE_STR$ THEN
      REM Book appointment
      LET CONFIRMATION$ = BOOK APPOINTMENT SENDER$, DATE_STR$
      
      REM Send confirmation
      SEND WHATSAPP TO SENDER$ WITH "📅 Appointment Confirmed!" + CHR$(10) + CONFIRMATION$
    ELSE
      SEND WHATSAPP TO SENDER$ WITH "❌ Date not available. Please choose another time."
    END IF
  END IF
END ON
```

### 5. Poll/Survey Bot

```javascript
// Node.js survey bot
class SurveyBot {
  constructor() {
    this.surveys = {};
    this.responses = {};
  }

  async createSurvey(from, question, options) {
    const surveyId = `SURV-${Date.now()}`;
    
    this.surveys[surveyId] = {
      question,
      options,
      responses: []
    };

    let message = `📊 *Survey*\n\n${question}\n\n`;
    options.forEach((opt, i) => {
      message += `${i + 1}. ${opt}\n`;
    });
    message += '\nReply with the option number to vote.';

    await this.whatsapp.sendText(from, message);
    return surveyId;
  }

  async handleResponse(from, surveyId, choice) {
    if (this.surveys[surveyId]) {
      const survey = this.surveys[surveyId];
      
      if (choice >= 1 && choice <= survey.options.length) {
        survey.responses.push({
          from,
          choice,
          timestamp: new Date()
        });

        const selectedOption = survey.options[choice - 1];
        await this.whatsapp.sendText(from, `✅ You voted for: ${selectedOption}`);
      } else {
        await this.whatsapp.sendText(from, '❌ Invalid option. Please try again.');
      }
    }
  }
}
```

### 6. Broadcast Message Sender

```python
# Python broadcast sender
class BroadcastSender:
    def __init__(self):
        self.whatsapp = WhatsAppClient()
    
    def send_broadcast(self, recipients, message):
        """Send message to multiple recipients"""
        success_count = 0
        failed_count = 0
        
        for recipient in recipients:
            try:
                self.whatsapp.send_text(recipient, message)
                success_count += 1
                time.sleep(1)  # Rate limiting
            except Exception as e:
                logging.error(f"Failed to send to {recipient}: {e}")
                failed_count += 1
        
        return {
            'total': len(recipients),
            'success': success_count,
            'failed': failed_count
        }

# Usage
broadcast = BroadcastSender()
recipients = ['+5511999999999', '+5511888888888', '+5511777777777']
message = "📢 *Important Announcement*\n\nThis is a broadcast message to all subscribers."

result = broadcast.send_broadcast(recipients, message)
print(f"Sent: {result['success']}/{result['total']}")
```

### 7. File Download Bot

```basic
REM File download and send bot
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  
  IF LEFT$(MESSAGE$, 4) = "GET " THEN
    REM Extract filename
    LET FILENAME$ = MID$(MESSAGE$, 5)
    
    REM Check if file exists
    IF FILE EXISTS("documents/" + FILENAME$) THEN
      REM Get file URL
      LET FILE_URL$ = "https://your-domain.com/files/" + FILENAME$
      
      REM Send document
      SEND WHATSAPP TO SENDER$ WITH DOCUMENT FROM FILE_URL$ NAMED FILENAME$
    ELSE
      SEND WHATSAPP TO SENDER$ WITH "❌ File not found: " + FILENAME$
    END IF
  END IF
END ON
```

### 8. E-commerce Product Catalog

```javascript
// Product catalog bot
const products = [
  { id: 1, name: 'Wireless Headphones', price: 299.90, image: 'https://example.com/headphones.jpg' },
  { id: 2, name: 'Smart Watch', price: 499.90, image: 'https://example.com/watch.jpg' },
  { id: 3, name: 'Bluetooth Speaker', price: 199.90, image: 'https://example.com/speaker.jpg' }
];

async function showProductCatalog(to) {
  let message = '🛍️ *Product Catalog*\n\n';
  
  products.forEach(product => {
    message += `${product.id}. *${product.name}*\n`;
    message += `   Price: R$ ${product.price.toFixed(2)}\n\n`;
  });
  
  message += 'Reply with product number to see details.';
  
  await whatsapp.sendText(to, message);
}

async function showProductDetails(to, productId) {
  const product = products.find(p => p.id === productId);
  
  if (product) {
    await whatsapp.sendImage(to, product.image, `📦 *${product.name}*\n\nPrice: R$ ${product.price.toFixed(2)}\n\nReply 'BUY ${productId}' to purchase.`);
  } else {
    await whatsapp.sendText(to, '❌ Product not found.');
  }
}

// In webhook handler
if (lowerMessage === 'catalog' || lowerMessage === 'products') {
  await showProductCatalog(from);
} else if (lowerMessage.startsWith('product ')) {
  const productId = parseInt(lowerMessage.split(' ')[1]);
  await showProductDetails(from, productId);
}
```

### 9. Daily Digest/Notification Bot

```python
# Daily digest scheduler
import schedule
import time

class DigestBot:
    def __init__(self):
        self.whatsapp = WhatsAppClient()
        self.subscribers = set()
    
    def subscribe(self, number):
        self.subscribers.add(number)
        return "✅ Subscribed to daily digest!"
    
    def unsubscribe(self, number):
        self.subscribers.discard(number)
        return "❌ Unsubscribed from daily digest."
    
    def send_daily_digest(self):
        digest = self.generate_digest()
        
        for subscriber in self.subscribers:
            try:
                self.whatsapp.send_text(subscriber, digest)
            except Exception as e:
                logging.error(f"Failed to send digest to {subscriber}: {e}")
    
    def generate_digest(self):
        # Generate daily digest content
        return f"""📰 *Daily Digest*

📅 {datetime.now().strftime('%Y-%m-%d')}

• Weather: Sunny, 25°C
• News: 5 new articles
• Events: 2 upcoming

Have a great day! 🌟"""
    
    def start_scheduler(self):
        schedule.every().day.at("09:00").do(self.send_daily_digest)
        
        while True:
            schedule.run_pending()
            time.sleep(60)
```

### 10. Multi-language Support Bot

```basic
REM Multi-language bot
ON WHATSAPP MESSAGE RECEIVED
  LET SENDER$ = GET WHATSAPP SENDER NUMBER
  LET MESSAGE$ = GET WHATSAPP MESSAGE BODY
  
  REM Get user's preferred language (default: English)
  LET LANG$ = GET USER LANGUAGE SENDER$
  
  REM Check for language change command
  IF LEFT$(MESSAGE$, 3) = "SET " THEN
    LET NEW_LANG$ = UPPER$(MID$(MESSAGE$, 5))
    
    IF NEW_LANG$ = "EN" OR NEW_LANG$ = "ES" OR NEW_LANG$ = "PT" THEN
      SET USER LANGUAGE SENDER$ TO NEW_LANG$
      SEND WHATSAPP TO SENDER$ WITH GET TEXT "language_set" IN NEW_LANG$
    ELSE
      SEND WHATSAPP TO SENDER$ WITH "❌ Unsupported language. Use: EN, ES, or PT"
    END IF
  ELSE
    REM Process message in user's language
    LET RESPONSE$ = PROCESS MESSAGE MESSAGE$ IN LANG$
    SEND WHATSAPP TO SENDER$ WITH RESPONSE$
  END IF
END ON

REM Language text retrieval
FUNCTION GET TEXT$ KEY$, LANG$
  IF LANG$ = "EN" THEN
    IF KEY$ = "language_set" THEN RETURN "Language set to English ✅"
    IF KEY$ = "welcome" THEN RETURN "Welcome! How can I help you today?"
  ELSEIF LANG$ = "ES" THEN
    IF KEY$ = "language_set" THEN RETURN "Idioma establecido en Español ✅"
    IF KEY$ = "welcome" THEN RETURN "¡Bienvenido! ¿Cómo puedo ayudarte hoy?"
  ELSEIF LANG$ = "PT" THEN
    IF KEY$ = "language_set" THEN RETURN "Idioma definido para Português ✅"
    IF KEY$ = "welcome" THEN RETURN "Bem-vindo! Como posso ajudá-lo hoje?"
  END IF
  
  RETURN "Text not found"
END FUNCTION
```

## Advanced Scenarios

### Conversation Flow Management

```javascript
// Advanced conversation state machine
class ConversationFlow {
  constructor() {
    this.flows = {
      'sales': {
        'initial': async (from, input, state) => {
          state.products = await getProducts();
          return this.formatProductList(state.products);
        },
        'selecting_product': async (from, input, state) => {
          const product = state.products.find(p => p.id === parseInt(input));
          if (product) {
            state.selectedProduct = product;
            state.step = 'confirming_purchase';
            return `You selected: ${product.name}\nPrice: $${product.price}\n\nConfirm purchase? (yes/no)`;
          }
          return 'Invalid product. Please select a valid number.';
        },
        'confirming_purchase': async (from, input, state) => {
          if (input.toLowerCase() === 'yes') {
            const order = await createOrder(from, state.selectedProduct);
            state.step = 'completed';
            return `✅ Order created!\nOrder ID: ${order.id}\n\nThank you for your purchase!`;
          } else if (input.toLowerCase() === 'no') {
            state.step = 'initial';
            return this.formatProductList(state.products);
          }
          return 'Please reply "yes" or "no".';
        }
      }
    };
  }

  async processFlow(flowName, from, input, state) {
    const flow = this.flows[flowName];
    if (!flow) return 'Flow not found';

    const handler = flow[state.step] || flow['initial'];
    return await handler(from, input, state);
  }

  formatProductList(products) {
    let message = '🛍️ *Products*\n\n';
    products.forEach((p, i) => {
      message += `${i + 1}. ${p.name} - $${p.price}\n`;
    });
    message += '\nReply with the product number to purchase.';
    return message;
  }
}
```

### Interactive Buttons and Lists

```python
# Interactive message templates (must be pre-approved in Meta)
class InteractiveMessages:
    @staticmethod
    def send_list_message(to, header, body, options):
        """Send an interactive list message"""
        sections = [{
            'title': header,
            'rows': [{'id': str(i), 'title': opt, 'description': ''} 
                    for i, opt in enumerate(options)]
        }]

        payload = {
            'messaging_product': 'whatsapp',
            'to': to,
            'type': 'interactive',
            'interactive': {
                'type': 'list',
                'header': {
                    'type': 'text',
                    'text': header
                },
                'body': {
                    'text': body
                },
                'action': {
                    'button': 'Select',
                    'sections': sections
                }
            }
        }

        return send_interactive(payload)

    @staticmethod
    def send_button_message(to, text, buttons):
        """Send an interactive button message"""
        payload = {
            'messaging_product': 'whatsapp',
            'to': to,
            'type': 'interactive',
            'interactive': {
                'type': 'button',
                'body': {
                    'text': text
                },
                'action': {
                    'buttons': [
                        {
                            'type': 'reply',
                            'reply': {'id': f'btn_{i}', 'title': btn}
                        } for i, btn in enumerate(buttons)
                    ]
                }
            }
        }

        return send_interactive(payload)
```

### Media Upload and Management

```javascript
// Media management utilities
class MediaManager {
  constructor(whatsappClient) {
    this.whatsapp = whatsappClient;
    this.mediaCache = new Map();
  }

  async uploadMedia(url, mediaType = 'image') {
    // Check cache first
    if (this.mediaCache.has(url)) {
      return this.mediaCache.get(url);
    }

    try {
      // Upload to Meta servers
      const response = await axios.post(
        `${this.baseURL}/${this.phone_number_id}/media`,
        {
          file: url,
          type: mediaType
        },
        {
          headers: {
            'Authorization': `Bearer ${this.accessToken}`
          }
        }
      );

      const mediaId = response.data.id;
      this.mediaCache.set(url, mediaId);
      
      return mediaId;
    } catch (error) {
      console.error('Media upload failed:', error);
      throw error;
    }
  }

  async sendCachedImage(to, imageUrl, caption = '') {
    const mediaId = await this.uploadMedia(imageUrl);
    
    return await this.whatsapp.send({
      to,
      type: 'image',
      image: {
        id: mediaId,
        caption
      }
    });
  }
}
```

### Rate Limiting and Queue Management

```python
# Rate limiter for WhatsApp API
from datetime import datetime, timedelta
import time

class RateLimiter:
    def __init__(self, max_requests=1000, time_window=60):
        self.max_requests = max_requests
        self.time_window = time_window
        self.requests = []
        self.queue = []
    
    def can_make_request(self):
        now = datetime.now()
        cutoff = now - timedelta(seconds=self.time_window)
        
        # Remove old requests
        self.requests = [r for r in self.requests if r > cutoff]
        
        return len(self.requests) < self.max_requests
    
    def record_request(self):
        self.requests.append(datetime.now())
    
    async def send_with_limit(self, send_func, *args, **kwargs):
        if not self.can_make_request():
            wait_time = self.time_window - (datetime.now() - self.requests[0]).total_seconds()
            if wait_time > 0:
                time.sleep(wait_time)
        
        self.record_request()
        return await send_func(*args, **kwargs)

# Usage
rate_limiter = RateLimiter(max_requests=50, time_window=60)

async def send_message_safe(to, message):
    await rate_limiter.send_with_limit(whatsapp.send_text, to, message)
```

## Testing Examples

### Unit Tests for WhatsApp Client

```javascript
// whatsapp-client.test.js
const WhatsAppClient = require('./whatsapp-client');
const nock = require('nock');

describe('WhatsAppClient', () => {
  let client;
  
  beforeEach(() => {
    client = new WhatsAppClient();
  });

  test('sendText should send message successfully', async () => {
    nock('https://graph.facebook.com')
      .post('/v18.0/1158433381968079/messages')
      .reply(200, {
        messaging_product: 'whatsapp',
        contacts: [{ input: '5511999999999', wa_id: '5511999999999' }],
        messages: [{ id: 'wamid.example' }]
      });

    const result = await client.sendText('+5511999999999', 'Test message');
    
    expect(result.messaging_product).toBe('whatsapp');
    expect(result.messages[0].id).toBeDefined();
  });

  test('sendText should handle errors', async () => {
    nock('https://graph.facebook.com')
      .post('/v18.0/1158433381968079/messages')
      .reply(400, {
        error: {
          message: 'Invalid phone number',
          type: 'WhatsAppApiError'
        }
      });

    await expect(
      client.sendText('invalid', 'Test message')
    ).rejects.toThrow();
  });
});
```

### Integration Tests with Twilio

```python
# twilio_integration_test.py
import pytest
from twilio_integration import TwilioWebhookHandler

def test_voice_webhook_verification():
    handler = TwilioWebhookHandler()
    
    # Mock Twilio request
    request_data = {
        'CallSid': 'CA123',
        'From': '+1234567890',
        'To': '+553322980098',
        'CallStatus': 'ringing'
    }
    
    # Process webhook
    response = handler.handle_voice_call(request_data)
    
    # Verify TwiML response
    assert '<?xml version="1.0" encoding="UTF-8"?>' in response
    assert '<Gather' in response
    assert '<Say' in response

def test_verification_code_capture():
    handler = TwilioWebhookHandler()
    
    # Simulate DTMF input
    digits = '123456'
    response = handler.handle_gather({'Digits': digits})
    
    # Verify code was captured
    assert handler.verification_code == digits
    assert 'Thank you' in response
```

## Best Practices Summary

1. **Always handle errors gracefully** - Implement retry logic and error notifications
2. **Validate input** - Check phone numbers, message lengths, and media URLs
3. **Use conversation state** - Track user progress through multi-step flows
4. **Implement rate limiting** - Respect API limits to avoid throttling
5. **Log everything** - Track messages, errors, and user interactions
6. **Secure your webhooks** - Verify signatures and use HTTPS
7. **Test thoroughly** - Use unit tests, integration tests, and manual testing
8. **Monitor performance** - Track response times and success rates
9. **Handle timeouts** - Set appropriate timeouts for API calls
10. **Provide feedback** - Always acknowledge user actions with confirmations

For more information on webhook configuration, see [Webhook Configuration Guide](./webhooks.md).