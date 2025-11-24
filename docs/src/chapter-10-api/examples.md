# API Examples

This section provides practical examples of using the BotServer REST API in various programming languages and scenarios.

## Authentication Examples

### Getting a Session Token

**JavaScript/TypeScript:**
```javascript
// Note: Authentication is handled through Zitadel OAuth flow
// This is a simplified example
async function authenticate() {
  // Redirect to Zitadel login
  window.location.href = '/auth/login';
  
  // After OAuth callback, session token is set
  // Use it for subsequent requests
}
```

**cURL:**
```bash
# Session validation
curl -X GET http://localhost:8080/auth/validate \
  -H "Authorization: Bearer YOUR_SESSION_TOKEN"
```

## Group Management Examples

### Creating a Group

**JavaScript:**
```javascript
async function createGroup() {
  const response = await fetch('/api/groups/create', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer YOUR_TOKEN'
    },
    body: JSON.stringify({
      name: 'Engineering Team',
      description: 'Software developers'
    })
  });
  
  const group = await response.json();
  console.log('Created group:', group);
}
```

**Python:**
```python
import requests

def create_group():
    url = "http://localhost:8080/api/groups/create"
    headers = {
        "Authorization": "Bearer YOUR_TOKEN",
        "Content-Type": "application/json"
    }
    data = {
        "name": "Engineering Team",
        "description": "Software developers"
    }
    
    response = requests.post(url, json=data, headers=headers)
    return response.json()
```

### Adding Group Members

**JavaScript:**
```javascript
async function addMember(groupId, userId) {
  const response = await fetch(`/api/groups/${groupId}/members/add`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer YOUR_TOKEN'
    },
    body: JSON.stringify({
      user_id: userId,
      role: 'member'
    })
  });
  
  return response.json();
}
```

## Admin API Examples

### Getting System Status

**cURL:**
```bash
curl -X GET http://localhost:8080/api/admin/system/status \
  -H "Authorization: Bearer ADMIN_TOKEN"
```

**Go:**
```go
package main

import (
    "net/http"
    "io/ioutil"
    "fmt"
)

func getSystemStatus(token string) {
    client := &http.Client{}
    req, _ := http.NewRequest("GET", 
        "http://localhost:8080/api/admin/system/status", nil)
    req.Header.Add("Authorization", "Bearer " + token)
    
    resp, err := client.Do(req)
    if err != nil {
        panic(err)
    }
    defer resp.Body.Close()
    
    body, _ := ioutil.ReadAll(resp.Body)
    fmt.Println(string(body))
}
```

### Creating a Backup

**JavaScript:**
```javascript
async function createBackup() {
  const response = await fetch('/api/admin/backup/create', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer ADMIN_TOKEN'
    },
    body: JSON.stringify({
      backup_type: 'full',
      include_data: true,
      include_config: true
    })
  });
  
  const backup = await response.json();
  console.log('Backup created:', backup.id);
  console.log('Download URL:', backup.download_url);
}
```

## WebSocket Communication

### Real-Time Chat

**JavaScript:**
```javascript
class BotChat {
  constructor(sessionId) {
    this.sessionId = sessionId;
    this.ws = null;
  }
  
  connect() {
    this.ws = new WebSocket('ws://localhost:8080/ws');
    
    this.ws.onopen = () => {
      console.log('Connected to bot');
    };
    
    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };
    
    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }
  
  sendMessage(content) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({
        type: 'message',
        content: content,
        session_id: this.sessionId
      }));
    }
  }
  
  handleMessage(message) {
    console.log('Bot response:', message.content);
    
    if (message.suggestions) {
      console.log('Suggestions:', message.suggestions);
    }
  }
}

// Usage
const chat = new BotChat('session-123');
chat.connect();
chat.sendMessage('Hello bot!');
```

## Error Handling

### Handling API Errors

**JavaScript:**
```javascript
async function apiCall(url, options = {}) {
  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': 'Bearer YOUR_TOKEN',
        'Content-Type': 'application/json',
        ...options.headers
      }
    });
    
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || `HTTP ${response.status}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error('API Error:', error);
    
    // Handle specific error codes
    if (error.code === 'RATE_LIMITED') {
      // Wait and retry
      await new Promise(resolve => setTimeout(resolve, 1000));
      return apiCall(url, options);
    }
    
    throw error;
  }
}
```

### Rate Limit Handling

**Python:**
```python
import time
import requests

class APIClient:
    def __init__(self, base_url, token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {token}',
            'Content-Type': 'application/json'
        }
    
    def request(self, method, endpoint, **kwargs):
        url = f"{self.base_url}{endpoint}"
        
        response = requests.request(
            method, url, 
            headers=self.headers, 
            **kwargs
        )
        
        # Check rate limit headers
        remaining = response.headers.get('X-RateLimit-Remaining')
        if remaining and int(remaining) < 10:
            reset_time = int(response.headers.get('X-RateLimit-Reset', 0))
            sleep_time = max(reset_time - time.time(), 0)
            print(f"Rate limit approaching, sleeping {sleep_time}s")
            time.sleep(sleep_time)
        
        response.raise_for_status()
        return response.json()
```

## Pagination Examples

### Iterating Through Paginated Results

**JavaScript:**
```javascript
async function* getAllGroups(token) {
  let offset = 0;
  const limit = 20;
  
  while (true) {
    const response = await fetch(
      `/api/groups/list?limit=${limit}&offset=${offset}`,
      {
        headers: { 'Authorization': `Bearer ${token}` }
      }
    );
    
    const data = await response.json();
    
    for (const group of data.groups) {
      yield group;
    }
    
    if (data.groups.length < limit) {
      break;  // No more pages
    }
    
    offset += limit;
  }
}

// Usage
for await (const group of getAllGroups(token)) {
  console.log(group.name);
}
```

## Integration Patterns

### Webhook Handler

**Node.js/Express:**
```javascript
const express = require('express');
const app = express();

app.post('/webhook/botserver', express.json(), (req, res) => {
  const event = req.body;
  
  switch(event.type) {
    case 'user.created':
      handleUserCreated(event.data);
      break;
    case 'conversation.completed':
      handleConversationCompleted(event.data);
      break;
    default:
      console.log('Unknown event type:', event.type);
  }
  
  res.status(200).send('OK');
});

function handleUserCreated(userData) {
  console.log('New user:', userData);
  // Process new user
}

function handleConversationCompleted(conversationData) {
  console.log('Conversation completed:', conversationData);
  // Process completed conversation
}
```

## Best Practices

1. **Always handle errors gracefully** - Network failures happen
2. **Respect rate limits** - Implement exponential backoff
3. **Use environment variables** for API tokens
4. **Log API interactions** for debugging
5. **Cache responses** when appropriate
6. **Use connection pooling** for multiple requests
7. **Implement timeout handling** for long operations

## Testing API Calls

### Using Postman

1. Import the API collection (when available)
2. Set environment variables for:
   - `base_url`: http://localhost:8080
   - `token`: Your session token
3. Run requests individually or as collection

### Unit Testing API Calls

**JavaScript/Jest:**
```javascript
describe('Groups API', () => {
  test('should create a group', async () => {
    const mockFetch = jest.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ 
          id: 'group-123', 
          name: 'Test Group' 
        })
      })
    );
    global.fetch = mockFetch;
    
    const result = await createGroup('Test Group');
    
    expect(mockFetch).toHaveBeenCalledWith(
      '/api/groups/create',
      expect.objectContaining({
        method: 'POST'
      })
    );
    expect(result.id).toBe('group-123');
  });
});
```

## Summary

These examples demonstrate common patterns for interacting with the BotServer API. Remember to:
- Handle authentication properly through Zitadel
- Check response status codes
- Parse error responses
- Implement proper error handling
- Use appropriate HTTP methods
- Follow REST conventions

For more specific endpoint documentation, refer to the individual API sections in this chapter.