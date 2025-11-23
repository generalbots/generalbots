# ADD SUGGESTION

Add conversational suggestions or quick reply options for user interactions.

## Syntax

```basic
ADD SUGGESTION text AS value
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | String | Display text shown to the user |
| `value` | String | Value sent when suggestion is clicked |

## Description

The `ADD SUGGESTION` keyword adds quick reply buttons or suggestion chips to the conversation interface. These provide users with:

- Quick action buttons
- Common response options
- Guided conversation paths
- Menu-like interactions
- Context-aware suggestions

Suggestions appear as clickable elements in supported channels (web, WhatsApp, Teams, etc.).

## Examples

### Basic Suggestions
```basic
ADD SUGGESTION "Yes" AS "confirm"
ADD SUGGESTION "No" AS "decline"
ADD SUGGESTION "Maybe later" AS "postpone"
answer = HEAR "Would you like to proceed?"
```

### Dynamic Suggestions from Data
```basic
departments = ["Sales", "Support", "Billing", "Technical"]
FOR EACH dept IN departments
    ADD SUGGESTION dept AS dept
NEXT
selection = HEAR "Which department do you need?"
```

### Context-Based Suggestions
```basic
IF user_type = "new" THEN
    ADD SUGGESTION "Get Started" AS "onboarding"
    ADD SUGGESTION "View Tutorial" AS "tutorial"
    ADD SUGGESTION "Contact Support" AS "help"
ELSE
    ADD SUGGESTION "Check Status" AS "status"
    ADD SUGGESTION "New Request" AS "create"
    ADD SUGGESTION "View History" AS "history"
END IF
```

### Menu System
```basic
' Main menu
CLEAR SUGGESTIONS
ADD SUGGESTION "Products" AS "menu_products"
ADD SUGGESTION "Services" AS "menu_services"
ADD SUGGESTION "Support" AS "menu_support"
ADD SUGGESTION "About Us" AS "menu_about"

choice = HEAR "What would you like to know about?"

IF choice = "menu_products" THEN
    CLEAR SUGGESTIONS
    ADD SUGGESTION "Pricing" AS "product_pricing"
    ADD SUGGESTION "Features" AS "product_features"
    ADD SUGGESTION "Compare" AS "product_compare"
    ADD SUGGESTION "Back" AS "menu_main"
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
- `CLEAR SUGGESTIONS` is called
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
ADD SUGGESTION "Small ($10)" AS "size:small"
ADD SUGGESTION "Medium ($15)" AS "size:medium"
ADD SUGGESTION "Large ($20)" AS "size:large"
```

### Conditional Display
```basic
IF has_permission("admin") THEN
    ADD SUGGESTION "Admin Panel" AS "admin"
END IF
```

### Localized Suggestions
```basic
language = GET_USER_LANGUAGE()
IF language = "es" THEN
    ADD SUGGESTION "Sí" AS "yes"
    ADD SUGGESTION "No" AS "no"
ELSE
    ADD SUGGESTION "Yes" AS "yes"
    ADD SUGGESTION "No" AS "no"
END IF
```

## Error Handling

- Silently fails if suggestion limit exceeded
- Logs warning if value is too long
- Falls back to text input if channel doesn't support suggestions

## Related Keywords

- [CLEAR SUGGESTIONS](./keyword-clear-suggestions.md) - Remove all suggestions
- [HEAR](./keyword-hear.md) - Wait for user input (including suggestion clicks)
- [TALK](./keyword-talk.md) - Send message with suggestions
- [FORMAT](./keyword-format.md) - Format suggestion text

## Implementation

Located in `src/basic/keywords/add_suggestion.rs`

Uses cache component for storage when available, falls back to in-memory storage.