# Microsoft Teams Channel Configuration

This guide covers setting up Microsoft Teams as a communication channel for your General Bots deployment, enabling bots to interact with users directly in Teams conversations.

---

## Overview

Microsoft Teams integration allows your bot to:
- Receive messages from Teams users
- Send responses back to Teams conversations
- Share files and documents
- Create adaptive cards for rich interactions
- Participate in group chats and channels

---

## Quick Start

### Minimal Configuration

```csv
name,value
teams-app-id,your-app-id
teams-app-password,your-app-password
teams-tenant-id,your-tenant-id
teams-bot-id,your-bot-id
```

---

## Prerequisites

Before configuring Teams, you need:

1. **Microsoft 365 Account** with admin access
2. **Azure Subscription** for bot registration
3. **General Bots Server** accessible via HTTPS
4. **Valid SSL Certificate** (Teams requires HTTPS)

---

## Configuration Parameters

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `teams-app-id` | Microsoft App ID from Azure Bot registration | `12345678-1234-1234-1234-123456789abc` |
| `teams-app-password` | Microsoft App Password (client secret) | `your-secret-password` |
| `teams-tenant-id` | Azure AD Tenant ID | `87654321-4321-4321-4321-cba987654321` |
| `teams-bot-id` | Bot's unique identifier in Teams | `28:12345678-1234-1234-1234-123456789abc` |

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `teams-service-url` | Teams service endpoint | `https://smba.trafficmanager.net` |
| `teams-app-name` | Display name in Teams | Bot name from config |
| `teams-notify-url` | Proactive message webhook | Not set |

### Complete Example

```csv
name,value
teams-app-id,12345678-1234-1234-1234-123456789abc
teams-app-password,MySecretAppPassword123!
teams-tenant-id,87654321-4321-4321-4321-cba987654321
teams-bot-id,28:12345678-1234-1234-1234-123456789abc
teams-service-url,https://smba.trafficmanager.net
teams-app-name,Support Bot
```

---

## Azure Bot Registration

### Step 1: Create Azure Bot Resource

1. Go to [Azure Portal](https://portal.azure.com)
2. Click **Create a resource**
3. Search for **Azure Bot**
4. Click **Create**

### Step 2: Configure Bot Settings

| Setting | Value |
|---------|-------|
| Bot handle | Your unique bot name |
| Subscription | Your Azure subscription |
| Resource group | Create new or use existing |
| Pricing tier | F0 (free) for testing, S1 for production |
| Microsoft App ID | Create new |

### Step 3: Get Credentials

After creation:

1. Go to your Bot resource → **Configuration**
2. Copy the **Microsoft App ID** → Use as `teams-app-id`
3. Click **Manage** next to Microsoft App ID
4. Go to **Certificates & secrets** → **New client secret**
5. Copy the secret value → Use as `teams-app-password`

### Step 4: Configure Messaging Endpoint

Set the messaging endpoint to your General Bots server:

```
https://your-server.example.com/api/messages/teams
```

### Step 5: Enable Teams Channel

1. In your Azure Bot resource, go to **Channels**
2. Click **Microsoft Teams**
3. Accept the terms of service
4. Click **Apply**

---

## Teams App Manifest

Create a Teams app manifest to install the bot in Teams.

### manifest.json

```json
{
    "$schema": "https://developer.microsoft.com/json-schemas/teams/v1.16/MicrosoftTeams.schema.json",
    "manifestVersion": "1.16",
    "version": "1.0.0",
    "id": "{{teams-app-id}}",
    "packageName": "com.example.generalbotsbot",
    "developer": {
        "name": "Your Company",
        "websiteUrl": "https://example.com",
        "privacyUrl": "https://example.com/privacy",
        "termsOfUseUrl": "https://example.com/terms"
    },
    "name": {
        "short": "Support Bot",
        "full": "General Bots Support Assistant"
    },
    "description": {
        "short": "AI-powered support assistant",
        "full": "An AI-powered support assistant that can answer questions, look up information, and help with common tasks."
    },
    "icons": {
        "color": "color.png",
        "outline": "outline.png"
    },
    "accentColor": "#1565C0",
    "bots": [
        {
            "botId": "{{teams-app-id}}",
            "scopes": ["personal", "team", "groupchat"],
            "supportsFiles": true,
            "isNotificationOnly": false,
            "commandLists": [
                {
                    "scopes": ["personal", "team", "groupchat"],
                    "commands": [
                        {
                            "title": "help",
                            "description": "Get help and available commands"
                        },
                        {
                            "title": "status",
                            "description": "Check system status"
                        }
                    ]
                }
            ]
        }
    ],
    "permissions": ["identity", "messageTeamMembers"],
    "validDomains": ["your-server.example.com"]
}
```

### Creating the App Package

1. Create a folder with:
   - `manifest.json`
   - `color.png` (192x192 pixels)
   - `outline.png` (32x32 pixels, transparent)
2. Zip the folder contents
3. Upload to Teams Admin Center or sideload

---

## Installing the Bot in Teams

### For Testing (Sideload)

1. Open Microsoft Teams
2. Go to **Apps** → **Manage your apps**
3. Click **Upload an app** → **Upload a custom app**
4. Select your zip file
5. Click **Add**

### For Organization-Wide Deployment

1. Go to [Teams Admin Center](https://admin.teams.microsoft.com)
2. Navigate to **Teams apps** → **Manage apps**
3. Click **Upload new app**
4. Upload your zip file
5. Configure policies to allow the app

---

## BASIC Usage Examples

### Sending Messages

```basic
' Reply to Teams message
TALK "Hello from General Bots!"
```

### Sending Adaptive Cards

```basic
' Create an adaptive card
card = #{
    "type": "AdaptiveCard",
    "version": "1.4",
    "body": [
        #{
            "type": "TextBlock",
            "size": "Large",
            "weight": "Bolder",
            "text": "Order Confirmation"
        },
        #{
            "type": "TextBlock",
            "text": "Order #" + order_id + " has been confirmed."
        },
        #{
            "type": "FactSet",
            "facts": [
                #{ "title": "Order Date", "value": FORMAT(NOW(), "MMMM DD, YYYY") },
                #{ "title": "Total", "value": "$" + FORMAT(total, "#,##0.00") }
            ]
        }
    ],
    "actions": [
        #{
            "type": "Action.OpenUrl",
            "title": "View Order",
            "url": "https://example.com/orders/" + order_id
        }
    ]
}

CARD card
```

### Sending Files

```basic
' Generate and send a report
report = GENERATE PDF "templates/report.html", report_data, "temp/report.pdf"

DOWNLOAD report.url AS "Monthly Report.pdf"
```

### Proactive Messages

```basic
' Send a proactive message to a user
' Requires stored conversation reference
SET HEADER "Authorization", "Bearer " + access_token

POST "https://smba.trafficmanager.net/apis/v3/conversations/" + conversation_id + "/activities",
    #{
        "type": "message",
        "text": "Your scheduled report is ready!"
    }
```

---

## Authentication

### Single Sign-On (SSO)

Enable Teams SSO for seamless authentication:

```csv
name,value
teams-app-id,your-app-id
teams-app-password,your-app-password
teams-sso-enabled,true
teams-sso-connection-name,AzureAD
```

### Getting User Identity

```basic
' The user's Teams identity is available in the session
user_name = user.name
user_email = user.email
user_id = user.id

TALK "Hello, " + user_name + "!"
```

---

## Multi-Tenant Configuration

For bots available to multiple organizations:

```csv
name,value
teams-app-id,your-app-id
teams-app-password,your-app-password
teams-tenant-id,common
teams-multi-tenant,true
```

When `teams-tenant-id` is set to `common`, the bot accepts connections from any Azure AD tenant.

---

## Handling Different Contexts

### Personal Chat

```basic
' Check if in personal chat
IF context.type = "personal" THEN
    TALK "This is a private conversation. How can I help?"
END IF
```

### Group Chat

```basic
' Check if in group chat
IF context.type = "groupchat" THEN
    TALK "I'm here to help the group. Mention me with @bot to get my attention."
END IF
```

### Team Channel

```basic
' Check if in team channel
IF context.type = "channel" THEN
    channel_name = context.channel.name
    TALK "Hello, " + channel_name + " channel!"
END IF
```

---

## Troubleshooting

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Teams not configured` | Missing required parameters | Set all required config values |
| `401 Unauthorized` | Invalid credentials | Verify app-id and app-password |
| `403 Forbidden` | Bot not authorized | Check Azure Bot channel config |
| `404 Not Found` | Wrong endpoint URL | Verify messaging endpoint |
| `Service URL mismatch` | Wrong service URL | Use URL from activity |

### Verifying Configuration

```basic
' Check if Teams is configured
IF teams_configured THEN
    TALK "Teams is ready!"
ELSE
    PRINT "Error: Teams adapter not configured"
    PRINT "Please set teams-app-id and teams-app-password in config.csv"
END IF
```

### Checking Connectivity

1. Ensure your server is accessible from the internet
2. Verify HTTPS with valid SSL certificate
3. Test the messaging endpoint: `https://your-server/api/messages/teams`
4. Check Azure Bot resource health in Azure Portal

### Logs

Enable detailed logging for troubleshooting:

```csv
name,value
log-level,debug
teams-log-activities,true
```

---

## Security Best Practices

### 1. Secure Credentials

Never commit credentials to version control:

```csv
# Use environment variable references
teams-app-password,${TEAMS_APP_PASSWORD}
```

### 2. Validate Tokens

General Bots automatically validates incoming tokens from Teams. Ensure your server clock is synchronized (NTP).

### 3. Limit Permissions

Request only necessary permissions in your app manifest:

```json
"permissions": ["identity"]
```

### 4. Use HTTPS

Teams requires HTTPS with a valid certificate. Self-signed certificates are not accepted.

---

## Related Documentation

- [WhatsApp Configuration](./whatsapp-channel.md) — WhatsApp channel setup
- [SMS Configuration](./sms-providers.md) — SMS provider configuration
- [Universal Messaging](../04-basic-scripting/universal-messaging.md) — Multi-channel messaging
- [Secrets Management](./secrets-management.md) — Secure credential storage
- [CARD Keyword](../04-basic-scripting/keyword-card.md) — Creating rich cards

---

## Summary

Microsoft Teams integration enables your General Bots deployment to interact with users in their familiar Teams environment. Configure the required parameters in `config.csv`, register your bot in Azure, create and install the Teams app manifest, and your bot is ready to chat. Use adaptive cards for rich interactions and leverage Teams SSO for seamless authentication.