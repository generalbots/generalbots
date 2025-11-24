# Bot Authentication

General Bots implements bot authentication through session-based mechanisms, where bots are associated with user sessions and authenticated through the system.

## Overview

Bot authentication in General Bots:
- Bots are registered in the database with unique IDs
- Sessions link users to specific bots
- No separate bot authentication - bots operate within user sessions
- Bot access controlled through user permissions

## Bot Registration

### Database Storage

Bots are stored in the `bots` table:
- `id` - UUID primary key
- `name` - Bot name
- `org_id` - Organization association
- `created_at` - Registration timestamp
- `updated_at` - Last modification

### Bot Configuration

Configuration stored in `bot_configuration` table:
- Bot-specific settings
- Key-value pairs from config.csv
- Runtime parameters
- Feature flags

## Session-Based Bot Access

### How It Works

1. User authenticates (via Directory Service)
2. User selects bot to interact with
3. Session created linking user + bot
4. All operations scoped to that bot

### Session Structure

```rust
// user_sessions table
{
    id: UUID,
    user_id: UUID,      // User reference
    bot_id: UUID,       // Bot reference
    session_token: String,
    expires_at: Timestamp
}
```

## Bot Isolation

### Data Isolation

Each bot has isolated:
- Message history
- Bot memories
- Knowledge bases
- Configuration
- Drive bucket

### Cross-Bot Protection

- Sessions locked to single bot
- Cannot access other bot's data
- Queries filtered by bot_id
- Storage segregated by bucket

## Bot Discovery

### Listing Available Bots

Users can access bots based on:
- Organization membership
- Direct assignment
- Public availability
- Role-based access

### Bot Selection

When starting conversation:
1. List available bots
2. User chooses bot
3. Session created for that bot
4. Bot context loaded

## Bot Lifecycle

### Creation

Bots created during bootstrap:
1. Template found in `templates/`
2. Bot registered in database
3. Configuration loaded
4. Resources uploaded to drive storage
5. Knowledge base indexed

### Activation

Bot becomes active when:
- Registration complete
- Configuration valid
- Resources available
- No critical errors

### Updates

Bot updates involve:
- Configuration changes
- Script modifications
- Knowledge base updates
- No authentication changes needed

## Bot Permissions

### Access Control

Bot access determined by:
- User's organization
- User's role
- Bot visibility settings
- Explicit assignments

### Permission Levels

- **Public**: Anyone can access
- **Organization**: Org members only
- **Private**: Specific users only
- **Admin**: Administrators only

## Configuration

### Bot Identity

Each bot has in config.csv:
```csv
name,value
Bot Name,Customer Service Bot
Bot ID,auto-generated
Organization,org-123
```

### Access Configuration

```csv
name,value
Access Level,organization
Allowed Roles,user;admin
Max Sessions,100
```

## Security Considerations

### No Bot Credentials

Important: Bots don't have:
- Passwords
- API keys
- Authentication tokens
- Login credentials

All authentication through user sessions.

### Bot Impersonation Prevention

- Bots cannot authenticate independently
- Always require user context
- Audit trail through sessions
- No bot-to-bot communication

## API Access

### Bot Operations via API

All bot operations require user authentication:

```javascript
// User must be authenticated first
fetch('/api/bot/message', {
  headers: {
    'Authorization': 'Bearer USER_SESSION_TOKEN'
  },
  body: JSON.stringify({
    bot_id: 'bot-123',
    message: 'Hello'
  })
})
```

### No Bot API Keys

Bots don't have separate API authentication:
- No bot tokens
- No bot API keys  
- No service accounts for bots
- All through user sessions

## Multi-Bot Scenarios

### Switching Bots

Users can switch between bots:
1. End current bot session
2. Start new session with different bot
3. Context switches to new bot
4. History preserved separately

### Concurrent Bot Access

Users can have multiple bot sessions:
- Different session IDs
- Separate contexts
- Independent conversations
- Isolated data access

## Bot Monitoring

### Authentication Metrics

Track bot access patterns:
- Sessions per bot
- User engagement
- Access attempts
- Permission denials

### Audit Logging

Log bot interactions:
- Session creation
- Bot selection
- Configuration changes
- Access violations

## Best Practices

1. **Use Organizations**: Group bots by organization
2. **Configure Access Levels**: Set appropriate visibility
3. **Monitor Usage**: Track bot access patterns
4. **Regular Audits**: Review bot permissions
5. **Document Bots**: Maintain bot documentation
6. **Test Isolation**: Verify data segregation

## Common Issues

### Bot Not Accessible

Causes:
- User not in organization
- Insufficient permissions
- Bot not activated
- Configuration error

### Session Errors

Issues:
- Expired session
- Invalid bot ID
- Concurrent session limit
- Database connection

## Implementation Notes

### No Separate Auth Module

Bot authentication is integrated into:
- Session management
- User authentication
- Database queries
- Not a separate system

### Future Considerations

Potential enhancements:
- Bot API tokens (not implemented)
- Service accounts (not implemented)
- Bot-to-bot communication (not implemented)
- Webhook authentication (not implemented)

## Summary

Bot authentication in General Bots is inherently tied to user authentication. Bots don't authenticate independently but operate within the context of authenticated user sessions. This design ensures security through user-based access control while maintaining simplicity in the authentication model.