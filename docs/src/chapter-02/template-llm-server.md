# LLM Server Template (llm-server.gbai)

A General Bots template for deploying LLM-powered web services that process orders and requests via API endpoints.

---

## Overview

The LLM Server template transforms General Bots into a headless API service that processes structured requests using LLM intelligence. It's designed for integrating AI-powered order processing, chatbot backends, and automated response systems into existing applications.

## Features

- **REST API Endpoints** - HTTP endpoints for bot interaction
- **Order Processing** - Structured JSON responses for orders
- **Product Catalog Integration** - Dynamic product menu from CSV
- **System Prompt Configuration** - Customizable AI behavior
- **Session Management** - Track conversations across requests
- **Operator Support** - Multi-operator/tenant architecture

---

## Package Structure

```
llm-server.gbai/
├── llm-server.gbdata/      # Data files
│   └── products.csv        # Product catalog
├── llm-server.gbdialog/
│   └── start.bas           # Main dialog with system prompt
├── llm-server.gbkb/        # Knowledge base
└── llm-server.gbot/
    └── config.csv          # Bot configuration
```

---

## API Endpoints

### Start a Session

```http
POST https://{host}/{botId}/dialogs/start
Content-Type: application/x-www-form-urlencoded

operator=123
userSystemId=999
```

**Response:**
```json
{
  "pid": "1237189231897",
  "conversationId": "abc123",
  "status": "started"
}
```

### Send a Message

```http
POST https://{host}/api/dk/messageBot
Content-Type: application/x-www-form-urlencoded

pid=1237189231897
text=I want a banana
```

**Response:**
```json
{
  "orderedItems": [
    {
      "item": {
        "id": 102,
        "price": 0.30,
        "name": "Banana",
        "quantity": 1,
        "notes": ""
      }
    }
  ],
  "userId": "123",
  "accountIdentifier": "TableA",
  "deliveryTypeId": 2
}
```

---

## Configuration

### System Prompt

The `start.bas` defines the AI behavior:

```basic
PARAM operator AS number LIKE 12312312 DESCRIPTION "Operator code."
DESCRIPTION It is a WebService of GB.

products = FIND "products.csv"

BEGIN SYSTEM PROMPT

You are a chatbot assisting a store attendant in processing orders. Follow these rules:

1. **Order Format**: Each order must include the product name, the table number, and the customer's name.

2. **Product Details**: The available products are:
   
   ${TOYAML(products)}

3. **JSON Response**: For each order, return a valid RFC 8259 JSON object containing:
   - product name
   - table number

4. **Guidelines**:
   - Do **not** engage in conversation.
   - Return the response in plain text JSON format only.

END SYSTEM PROMPT
```

### Product Catalog

Create `products.csv` in the `llm-server.gbdata` folder:

```csv
id,name,price,category,description
101,Apple,0.50,Fruit,Fresh red apple
102,Banana,0.30,Fruit,Ripe yellow banana
103,Orange,0.40,Fruit,Juicy orange
201,Milk,1.20,Dairy,1 liter whole milk
202,Cheese,2.50,Dairy,200g cheddar
```

### Bot Configuration

Configure in `llm-server.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `LLM Provider` | AI model provider | `openai` |
| `LLM Model` | Specific model | `gpt-4` |
| `Max Tokens` | Response length limit | `500` |
| `Temperature` | Response creativity | `0.3` |
| `API Mode` | Enable API mode | `true` |

---

## Usage Examples

### cURL Examples

**Start Session:**
```bash
curl -X POST https://api.example.com/llmservergbot/dialogs/start \
  -d "operator=123" \
  -d "userSystemId=999"
```

**Send Order:**
```bash
curl -X POST https://api.example.com/api/dk/messageBot \
  -d "pid=1237189231897" \
  -d "text=I need 2 apples and 1 milk"
```

### JavaScript Integration

```javascript
async function startBotSession(operator, userId) {
  const response = await fetch('https://api.example.com/llmservergbot/dialogs/start', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ operator, userSystemId: userId })
  });
  return response.json();
}

async function sendMessage(pid, text) {
  const response = await fetch('https://api.example.com/api/dk/messageBot', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ pid, text })
  });
  return response.json();
}

// Usage
const session = await startBotSession('123', '999');
const order = await sendMessage(session.pid, 'I want a banana');
console.log(order.orderedItems);
```

### Python Integration

```python
import requests

class LLMServerClient:
    def __init__(self, base_url, operator):
        self.base_url = base_url
        self.operator = operator
        self.pid = None
    
    def start_session(self, user_id):
        response = requests.post(
            f"{self.base_url}/llmservergbot/dialogs/start",
            data={"operator": self.operator, "userSystemId": user_id}
        )
        self.pid = response.json()["pid"]
        return self.pid
    
    def send_message(self, text):
        response = requests.post(
            f"{self.base_url}/api/dk/messageBot",
            data={"pid": self.pid, "text": text}
        )
        return response.json()

# Usage
client = LLMServerClient("https://api.example.com", "123")
client.start_session("999")
order = client.send_message("I need 2 bananas")
print(order)
```

---

## Response Format

### Order Response Structure

```json
{
  "orderedItems": [
    {
      "item": {
        "id": 102,
        "price": 0.30,
        "name": "Banana",
        "sideItems": [],
        "quantity": 2,
        "notes": "ripe ones please"
      }
    }
  ],
  "userId": "123",
  "accountIdentifier": "Table5",
  "deliveryTypeId": 2
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `orderedItems` | Array | List of ordered items |
| `item.id` | Number | Product ID from catalog |
| `item.price` | Number | Unit price |
| `item.name` | String | Product name |
| `item.sideItems` | Array | Additional items |
| `item.quantity` | Number | Order quantity |
| `item.notes` | String | Special instructions |
| `userId` | String | Operator identifier |
| `accountIdentifier` | String | Table/customer identifier |
| `deliveryTypeId` | Number | Delivery method |

---

## Customization

### Custom Response Format

Modify the system prompt for different output structures:

```basic
BEGIN SYSTEM PROMPT
Return responses as JSON with this structure:
{
  "intent": "order|question|complaint",
  "entities": [...extracted entities...],
  "response": "...",
  "confidence": 0.0-1.0
}
END SYSTEM PROMPT
```

### Adding Validation

```basic
' Validate order before returning
order = LLM_RESPONSE

IF NOT order.orderedItems THEN
    RETURN {"error": "No items in order", "suggestion": "Please specify products"}
END IF

FOR EACH item IN order.orderedItems
    product = FIND "products.csv", "id = " + item.item.id
    IF NOT product THEN
        RETURN {"error": "Invalid product ID: " + item.item.id}
    END IF
NEXT

RETURN order
```

### Multi-Language Support

```basic
PARAM language AS STRING LIKE "en" DESCRIPTION "Response language"

BEGIN SYSTEM PROMPT
Respond in ${language} language.
Available products: ${TOYAML(products)}
Return JSON format only.
END SYSTEM PROMPT
```

---

## Error Handling

### Common Error Responses

```json
{
  "error": "session_expired",
  "message": "Please start a new session",
  "code": 401
}
```

```json
{
  "error": "invalid_request",
  "message": "Missing required parameter: text",
  "code": 400
}
```

```json
{
  "error": "product_not_found",
  "message": "Product 'pizza' is not in our catalog",
  "code": 404
}
```

---

## Best Practices

1. **Keep prompts focused** - Single-purpose system prompts work better
2. **Validate responses** - Always validate LLM output before returning
3. **Handle edge cases** - Plan for invalid products, empty orders
4. **Monitor usage** - Track API calls and response times
5. **Rate limiting** - Implement rate limits for production
6. **Secure endpoints** - Use authentication for production APIs
7. **Log requests** - Maintain audit logs for debugging

---

## Deployment

### Environment Variables

```bash
LLM_PROVIDER=openai
LLM_API_KEY=sk-...
LLM_MODEL=gpt-4
API_RATE_LIMIT=100
SESSION_TIMEOUT=3600
```

### Docker Deployment

```dockerfile
FROM generalbots/server:latest
COPY llm-server.gbai /app/packages/
ENV API_MODE=true
EXPOSE 4242
CMD ["npm", "start"]
```

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Empty responses | System prompt too restrictive | Adjust prompt guidelines |
| Invalid JSON | LLM hallucination | Add JSON validation examples |
| Session expired | Timeout reached | Implement session refresh |
| Wrong products | Catalog not loaded | Verify products.csv path |
| Slow responses | Large catalog | Optimize product filtering |

---

## Use Cases

- **Restaurant Ordering** - Process food orders via API
- **Retail POS Integration** - AI-powered point of sale
- **Chatbot Backend** - Headless chatbot for web/mobile apps
- **Voice Assistant Backend** - Process voice-to-text commands
- **Order Automation** - Automate order entry from various sources

---

## Related Templates

- [LLM Tools](./template-llm-tools.md) - LLM with tool/function calling
- [Store](./template-store.md) - Full e-commerce with order processing
- [API Client](./template-api-client.md) - API integration examples

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide