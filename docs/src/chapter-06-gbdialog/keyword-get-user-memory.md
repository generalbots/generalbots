# GET USER MEMORY

Retrieves data stored at the user level, accessible across sessions and bots. This is the companion to `SET USER MEMORY` for reading persistent user data.

## Syntax

```basic
value = GET USER MEMORY("key")
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `key` | String | The identifier for the stored value |

## Returns

The stored value, or empty string (`""`) if the key doesn't exist.

## Description

`GET USER MEMORY` retrieves persistent data associated with a specific user. This data:

- **Persists across sessions** - Available when user returns days/weeks later
- **Persists across bots** - Accessible from any bot the user interacts with
- **Returns original type** - Objects, arrays, strings, numbers preserved
- **Returns empty on miss** - No error if key doesn't exist

## Examples

### Basic Usage

```basic
' Retrieve user preferences
language = GET USER MEMORY("language")
timezone = GET USER MEMORY("timezone")
theme = GET USER MEMORY("theme")

TALK "Your settings: " + language + ", " + timezone + ", " + theme
```

### Check If User Is Returning

```basic
' Personalized greeting based on stored name
name = GET USER MEMORY("name")

IF name = "" THEN
    TALK "Hello! I don't think we've met. What's your name?"
    HEAR name
    SET USER MEMORY "name", name
ELSE
    TALK "Welcome back, " + name + "! How can I help you today?"
END IF
```

### Retrieve Complex Objects

```basic
' Get stored user profile
profile = GET USER MEMORY("profile")

IF profile <> "" THEN
    TALK "Hello " + profile.name + "!"
    TALK "Your plan: " + profile.plan
    TALK "Member since: " + profile.signupDate
ELSE
    TALK "Please complete your profile first."
END IF
```

### Cross-Bot Data Access

```basic
' Support bot accessing sales data
lastPurchase = GET USER MEMORY("lastPurchase")

IF lastPurchase <> "" THEN
    TALK "I can see your recent order #" + lastPurchase.orderId
    TALK "Purchased on: " + lastPurchase.date
    TALK "Amount: $" + lastPurchase.amount
    TALK "How can I help with this order?"
ELSE
    TALK "I don't see any recent purchases. How can I help?"
END IF
```

### Retrieve User Facts for AI Context

```basic
' Load user facts into context for personalization
occupation = GET USER MEMORY("fact_occupation")
interests = GET USER MEMORY("fact_interests")
company = GET USER MEMORY("fact_company")

IF occupation <> "" THEN
    SET CONTEXT "user_occupation" AS occupation
END IF

IF interests <> "" THEN
    SET CONTEXT "user_interests" AS interests
END IF

' Now AI responses will be personalized based on these facts
```

### Default Values Pattern

```basic
' Get with fallback to default
language = GET USER MEMORY("language")
IF language = "" THEN
    language = "en-US"
END IF

' Or use inline default
theme = GET USER MEMORY("theme")
IF theme = "" THEN theme = "light"

TALK "Using language: " + language + ", theme: " + theme
```

### Session Continuity

```basic
' Resume conversation from previous session
lastTopic = GET USER MEMORY("lastTopic")
lastQuestion = GET USER MEMORY("lastQuestion")

IF lastTopic <> "" THEN
    TALK "Last time we were discussing " + lastTopic
    TALK "You asked: " + lastQuestion
    TALK "Would you like to continue from there?"
    HEAR continueChoice AS BOOLEAN
    
    IF continueChoice THEN
        ' Resume previous conversation
        SET CONTEXT "topic" AS lastTopic
    END IF
END IF
```

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`SET USER MEMORY`](./keyword-set-user-memory.md) | Store user-level persistent data |
| [`GET BOT MEMORY`](./keyword-get-bot-memory.md) | Retrieve bot-level data |
| [`SET BOT MEMORY`](./keyword-set-bot-memory.md) | Store data at bot level |
| [`USER FACTS`](./keyword-user-facts.md) | Get all stored user facts |

## Comparison: User Memory vs Bot Memory

| Aspect | User Memory | Bot Memory |
|--------|-------------|------------|
| **Scope** | Per user, across all bots | Per bot, across all users |
| **Use case** | User preferences, profile | Bot state, counters |
| **Access** | Any bot can read/write | Only owning bot |
| **Example** | `language`, `name`, `timezone` | `totalOrders`, `lastDeployed` |

## Error Handling

```basic
' GET USER MEMORY never throws - returns empty on missing key
value = GET USER MEMORY("nonexistent_key")
' value = ""

' Always check for empty before using
data = GET USER MEMORY("important_data")
IF data = "" THEN
    TALK "Data not found. Please provide it."
    ' Handle missing data case
ELSE
    ' Use the data
END IF
```

## Best Practices

1. **Always check for empty** - Keys may not exist for new users
2. **Use consistent key naming** - `user_name` vs `userName` vs `name`
3. **Document your keys** - Keep track of what data you're storing
4. **Handle missing gracefully** - New users won't have stored data
5. **Don't assume structure** - Stored objects might have missing fields

## See Also

- [Memory Management](../chapter-11-features/memory-management.md) - Complete memory architecture
- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Cross-bot data sharing
- [User Context](../chapter-12-auth/user-system-context.md) - User vs system context