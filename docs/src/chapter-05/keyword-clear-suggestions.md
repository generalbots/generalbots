# CLEAR SUGGESTIONS

Remove all active suggestions from the current conversation interface.

## Syntax

```basic
CLEAR SUGGESTIONS
```

## Parameters

This keyword takes no parameters.

## Description

The `CLEAR SUGGESTIONS` keyword removes all quick reply buttons and suggestion chips from the user interface. This is useful when you want to:

- Reset the conversation interface
- Change context and provide new options
- Remove outdated suggestions
- Clean up the interface after user selection

## Examples

### Basic Usage
```basic
CLEAR SUGGESTIONS
TALK "All suggestions have been cleared"
```

### Replace Suggestions
```basic
' Clear old suggestions before adding new ones
CLEAR SUGGESTIONS
ADD SUGGESTION "Option A" AS "a"
ADD SUGGESTION "Option B" AS "b"
ADD SUGGESTION "Option C" AS "c"
```

### Context Change
```basic
' Initial menu
ADD SUGGESTION "Sales" AS "sales"
ADD SUGGESTION "Support" AS "support"
choice = HEAR "Select department:"

' Clear and show department-specific options
CLEAR SUGGESTIONS
IF choice = "sales" THEN
    ADD SUGGESTION "Pricing" AS "pricing"
    ADD SUGGESTION "Demo" AS "demo"
ELSE IF choice = "support" THEN
    ADD SUGGESTION "Technical" AS "tech"
    ADD SUGGESTION "Billing" AS "billing"
END IF
```

### Clean Interface
```basic
' Show suggestions for initial choice
ADD SUGGESTION "Yes" AS "yes"
ADD SUGGESTION "No" AS "no"
answer = HEAR "Would you like to continue?"

' Clear suggestions after selection
CLEAR SUGGESTIONS
IF answer = "yes" THEN
    TALK "Great! Let's proceed..."
ELSE
    TALK "No problem. Goodbye!"
END IF
```

### Multi-Level Menu
```basic
FUNCTION ShowMainMenu()
    CLEAR SUGGESTIONS
    ADD SUGGESTION "Products" AS "products"
    ADD SUGGESTION "Services" AS "services"
    ADD SUGGESTION "About" AS "about"
    ADD SUGGESTION "Exit" AS "exit"
END FUNCTION

FUNCTION ShowProductMenu()
    CLEAR SUGGESTIONS
    ADD SUGGESTION "Catalog" AS "catalog"
    ADD SUGGESTION "Search" AS "search"
    ADD SUGGESTION "Back" AS "main"
END FUNCTION

' Usage
ShowMainMenu()
choice = HEAR "What would you like to explore?"

IF choice = "products" THEN
    ShowProductMenu()
ELSE IF choice = "exit" THEN
    CLEAR SUGGESTIONS
    TALK "Thank you for visiting!"
END IF
```

## Channel Support

The effect varies by communication channel:

| Channel | Behavior |
|---------|----------|
| Web | Removes all button/chip elements |
| WhatsApp | Clears reply buttons |
| Teams | Removes suggested actions |
| Slack | Clears interactive elements |
| SMS | No effect (text only) |

## Best Practices

1. **Clear Before Adding**: Always clear old suggestions before adding new ones
   ```basic
   CLEAR SUGGESTIONS
   ' Add new suggestions here
   ```

2. **Clear After Selection**: Remove suggestions after user makes a choice
   ```basic
   choice = HEAR "Select an option:"
   CLEAR SUGGESTIONS
   ' Process choice
   ```

3. **Error Handling**: Clear suggestions when errors occur
   ```basic
   TRY
       ' Some operation
   CATCH
       CLEAR SUGGESTIONS
       TALK "An error occurred. Please type your request."
   END TRY
   ```

4. **Timeout Handling**: Clear stale suggestions
   ```basic
   ON TIMEOUT
       CLEAR SUGGESTIONS
       TALK "Session timed out. Please start over."
   END ON
   ```

## Session Management

- Suggestions are session-specific
- Clearing affects only current user's interface
- Suggestions automatically clear when session ends
- No persistence across conversations

## Memory Usage

Clearing suggestions:
- Frees memory allocated for suggestion storage
- Reduces session state size
- Improves performance in long conversations

## Return Value

Returns `true` if suggestions were successfully cleared, `false` if there were no suggestions to clear.

## Error Handling

The keyword handles errors gracefully:
- Succeeds silently if no suggestions exist
- Logs warnings for channel communication errors
- Never throws exceptions

## Use Cases

### Wizard/Step Navigation
```basic
' Step 1
CLEAR SUGGESTIONS
ADD SUGGESTION "Next" AS "step2"
ADD SUGGESTION "Cancel" AS "cancel"

' Step 2
CLEAR SUGGESTIONS
ADD SUGGESTION "Back" AS "step1"
ADD SUGGESTION "Next" AS "step3"
ADD SUGGESTION "Cancel" AS "cancel"
```

### Dynamic Menus
```basic
options = GET_AVAILABLE_OPTIONS(user_role)
CLEAR SUGGESTIONS
FOR EACH option IN options
    ADD SUGGESTION option.label AS option.value
NEXT
```

### Conversation Reset
```basic
FUNCTION ResetConversation()
    CLEAR SUGGESTIONS
    CLEAR KB ALL
    SET CONTEXT ""
    TALK "Let's start fresh. How can I help you?"
END FUNCTION
```

## Related Keywords

- [ADD SUGGESTION](./keyword-add-suggestion.md) - Add new suggestions
- [HEAR](./keyword-hear.md) - Wait for user input with suggestions
- [TALK](./keyword-talk.md) - Send messages with suggestions

## Implementation

Located in `src/basic/keywords/clear_suggestions.rs`

The implementation:
- Maintains suggestion list in session cache
- Sends clear command to channel adapter
- Updates UI state asynchronously
- Handles multiple channel types