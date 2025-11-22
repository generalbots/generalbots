# ADD_SUGGESTION

Add conversational suggestions or quick reply options for user interactions.

## Syntax

```basic
ADD_SUGGESTION text AS value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | String | Display text shown to the user |
| `value` | String | Value sent when suggestion is clicked |

## Description

The `ADD_SUGGESTION` keyword adds quick reply buttons or suggestion chips to the conversation interface. These provide users with:

- Quick action buttons
- Common response options
- Guided conversation paths
- Menu-like interactions
- Context-aware suggestions

Suggestions appear as clickable elements in supported channels (web, WhatsApp, Teams, etc.).

## Examples

### Basic Suggestions
```basic
ADD_SUGGESTION "Yes" AS "confirm"
ADD_SUGGESTION "No" AS "decline"
ADD_SUGGESTION "Maybe later" AS "postpone"
answer = HEAR "Would you like to proceed?"
```

### Dynamic Suggestions from Data
```basic
departments = ["Sales", "Support", "Billing", "Technical"]
FOR EACH dept IN departments
    ADD_SUGGESTION dept AS dept
NEXT
selection = HEAR "Which department do you need?"
```

### Context-Based Suggestions
```basic
IF user_type = "new" THEN
    ADD_SUGGESTION "Get Started" AS "onboarding"
    ADD_SUGGESTION "View Tutorial" AS "tutorial"
    ADD_SUGGESTION "Contact Support" AS "help"
ELSE
    ADD_SUGGESTION "Check Status" AS "status"
    ADD_SUGGESTION "New Request" AS "create"
    ADD_SUGGESTION "View History" AS "history"
END IF
```

### Menu System
```basic
' Main menu
CLEAR_SUGGESTIONS
ADD_SUGGESTION "Products" AS "menu_products"
ADD_SUGGESTION "Services" AS "menu_services"
ADD_SUGGESTION "Support" AS "menu_support"
ADD_SUGGESTION "About Us" AS "menu_about"

choice = HEAR "What would you like to know about?"

IF choice = "menu_products" THEN
    CLEAR_SUGGESTIONS
    ADD_SUGGESTION "Pricing" AS "product_pricing"
    ADD_SUGGESTION "Features" AS "product_features"
    ADD_SUGGESTION "Compare" AS "product_compare"
    ADD_SUGGESTION "Back" AS "menu_main"
END IF
```

## Channel Support

| Channel | Display Type | Limitations |
|---------|-------------|-------------|
| Web | Buttons/Chips | Up to 10 suggestions |
| WhatsApp | Reply Buttons | Max 3 buttons |
| Teams | Hero Cards | Unlimited |
| Slack | Block Actions | Up to 5 per block |
| SMS | Numbered List | Text-only fallback |

## Suggestion Persistence

Suggestions remain active until:
- User clicks one
- `CLEAR_SUGGESTIONS` is called
- New suggestions replace them
- Conversation ends
- Timeout occurs (configurable)

## Styling and Appearance

Suggestions adapt to channel capabilities:
- **Rich Channels**: Styled buttons with icons
- **Basic Channels**: Numbered text options
- **Voice**: Read as options list

## Return Value

The keyword itself returns `true` if suggestion was added successfully, `false` otherwise.

When user clicks a suggestion, the value is returned to the next `HEAR` command.

## Memory Management

Suggestions are stored in session cache:
- Each session maintains its own suggestion list
- Cleared automatically on session end
- Can store up to 50 suggestions per session

## Best Practices

1. **Clear Before Adding**: Always clear old suggestions when changing context
2. **Limit Options**: Keep to 3-5 suggestions for better UX
3. **Descriptive Text**: Make suggestion text clear and actionable
4. **Meaningful Values**: Use values that are easy to handle in code
5. **Provide Escape**: Always include a "Back" or "Cancel" option
6. **Mobile First**: Consider mobile screen sizes

## Advanced Features

### Suggestion Groups
```basic
' Group related suggestions
ADD_SUGGESTION "Small ($10)" AS "size:small"
ADD_SUGGESTION "Medium ($15)" AS "size:medium"
ADD_SUGGESTION "Large ($20)" AS "size:large"
```

### Conditional Display
```basic
IF has_permission("admin") THEN
    ADD_SUGGESTION "Admin Panel" AS "admin"
END IF
```

### Localized Suggestions
```basic
language = GET_USER_LANGUAGE()
IF language = "es" THEN
    ADD_SUGGESTION "Sí" AS "yes"
    ADD_SUGGESTION "No" AS "no"
ELSE
    ADD_SUGGESTION "Yes" AS "yes"
    ADD_SUGGESTION "No" AS "no"
END IF
```

## Error Handling

- Silently fails if suggestion limit exceeded
- Logs warning if value is too long
- Falls back to text input if channel doesn't support suggestions

## Related Keywords

- [CLEAR_SUGGESTIONS](./keyword-clear-suggestions.md) - Remove all suggestions
- [HEAR](./keyword-hear.md) - Wait for user input (including suggestion clicks)
- [TALK](./keyword-talk.md) - Send message with suggestions
- [FORMAT](./keyword-format.md) - Format suggestion text

## Implementation

Located in `src/basic/keywords/add_suggestion.rs`

Uses Redis cache for storage when available, falls back to in-memory storage.