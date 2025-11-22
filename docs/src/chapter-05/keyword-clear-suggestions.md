# CLEAR_SUGGESTIONS

Remove all active suggestions from the current conversation interface.

## Syntax

```basic
CLEAR_SUGGESTIONS
```

## Parameters

None - this keyword takes no parameters.

## Description

The `CLEAR_SUGGESTIONS` keyword removes all quick reply buttons and suggestion chips from the conversation interface. Use this to:

- Reset the suggestion list before showing new options
- Clean up the interface after user selection
- Remove outdated suggestions when context changes
- Prevent confusion from lingering options
- Force text input instead of button selection

## Examples

### Basic Clear
```basic
' Show initial options
ADD_SUGGESTION "Yes" AS "yes"
ADD_SUGGESTION "No" AS "no"
answer = HEAR "Do you agree?"

' Clear after response
CLEAR_SUGGESTIONS
TALK "Thank you for your response"
```

### Menu Navigation
```basic
' Main menu
CLEAR_SUGGESTIONS
ADD_SUGGESTION "Products" AS "products"
ADD_SUGGESTION "Support" AS "support"
choice = HEAR "Select an option:"

' Clear and show submenu
CLEAR_SUGGESTIONS
IF choice = "products" THEN
    ADD_SUGGESTION "View All" AS "all"
    ADD_SUGGESTION "Categories" AS "categories"
    ADD_SUGGESTION "Back" AS "main"
END IF
```

### Conditional Clearing
```basic
IF conversation_stage = "complete" THEN
    CLEAR_SUGGESTIONS
    TALK "Thank you for using our service!"
ELSE
    ' Keep suggestions active
    TALK "Please select an option above"
END IF
```

### Error Recovery
```basic
retry_count = 0
DO
    CLEAR_SUGGESTIONS
    ADD_SUGGESTION "Retry" AS "retry"
    ADD_SUGGESTION "Cancel" AS "cancel"
    ADD_SUGGESTION "Help" AS "help"
    
    action = HEAR "Operation failed. What would you like to do?"
    retry_count = retry_count + 1
LOOP WHILE action = "retry" AND retry_count < 3

CLEAR_SUGGESTIONS
```

## Behavior

When `CLEAR_SUGGESTIONS` is called:

1. All suggestions are removed from UI immediately
2. Session cache is cleared of suggestion data
3. Channel-specific cleanup occurs
4. Interface returns to text input mode
5. No error if no suggestions exist

## Channel-Specific Behavior

| Channel | Effect |
|---------|--------|
| Web | Removes button/chip elements |
| WhatsApp | Clears reply keyboard |
| Teams | Removes card actions |
| Slack | Updates message blocks |
| Voice | Stops listing options |
| SMS | No visible effect |

## Performance

- Instant execution (< 1ms)
- Minimal memory impact
- No network calls required
- Thread-safe operation

## Best Practices

1. **Clear Before Adding**: Always clear before adding new suggestion sets
   ```basic
   CLEAR_SUGGESTIONS
   ADD_SUGGESTION "Option 1" AS "opt1"
   ADD_SUGGESTION "Option 2" AS "opt2"
   ```

2. **Clear After Selection**: Remove suggestions once user has chosen
   ```basic
   choice = HEAR "Select:"
   CLEAR_SUGGESTIONS
   TALK "You selected: " + choice
   ```

3. **Clear on Context Change**: Remove irrelevant suggestions
   ```basic
   IF topic_changed THEN
       CLEAR_SUGGESTIONS
   END IF
   ```

4. **Clear on Error**: Reset interface on failures
   ```basic
   ON ERROR
       CLEAR_SUGGESTIONS
       TALK "Something went wrong. Please type your request."
   END ON
   ```

## Common Patterns

### Wizard/Multi-Step Flow
```basic
' Step 1
CLEAR_SUGGESTIONS
ADD_SUGGESTION "Personal" AS "personal"
ADD_SUGGESTION "Business" AS "business"
account_type = HEAR "Account type?"

' Step 2
CLEAR_SUGGESTIONS
ADD_SUGGESTION "Basic" AS "basic"
ADD_SUGGESTION "Premium" AS "premium"
plan = HEAR "Select plan:"

' Complete
CLEAR_SUGGESTIONS
TALK "Setup complete!"
```

### Dynamic Menu System
```basic
FUNCTION show_menu(menu_name)
    CLEAR_SUGGESTIONS
    menu_items = GET_MENU_ITEMS(menu_name)
    FOR EACH item IN menu_items
        ADD_SUGGESTION item.label AS item.value
    NEXT
END FUNCTION
```

## Error Handling

- Never throws errors
- Safe to call multiple times
- No-op if no suggestions exist
- Handles concurrent calls gracefully

## Memory Management

Calling `CLEAR_SUGGESTIONS`:
- Frees suggestion cache memory
- Removes Redis entries if cached
- Cleans up UI resources
- Prevents memory leaks in long conversations

## Related Keywords

- [ADD_SUGGESTION](./keyword-add-suggestion.md) - Add quick reply options
- [HEAR](./keyword-hear.md) - Get user input from suggestions
- [TALK](./keyword-talk.md) - Display messages with suggestions

## Implementation

Located in `src/basic/keywords/add_suggestion.rs`

Shares implementation with `ADD_SUGGESTION` for efficient suggestion management.