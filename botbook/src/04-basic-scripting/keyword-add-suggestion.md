# ADD SUGGESTION / CLEAR SUGGESTIONS Keywords

Display quick-reply suggestion buttons to users during conversations.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `ADD SUGGESTION` | Add a suggestion button |
| `CLEAR SUGGESTIONS` | Remove all suggestions |

## ADD SUGGESTION

```basic
ADD SUGGESTION "Yes"
ADD SUGGESTION "No"
ADD SUGGESTION "Maybe later"
```

With action data:

```basic
ADD SUGGESTION "View Order", "action:view_order"
ADD SUGGESTION "Track Package", "action:track"
```

## CLEAR SUGGESTIONS

```basic
CLEAR SUGGESTIONS
```

## Example: Product Selection

```basic
TALK "What type of product are you interested in?"

ADD SUGGESTION "Electronics"
ADD SUGGESTION "Clothing"
ADD SUGGESTION "Home & Garden"
ADD SUGGESTION "Books"

HEAR choice
CLEAR SUGGESTIONS

TALK "Great! Let me show you our " + choice + " collection."
```

## Example: Confirmation Flow

```basic
TALK "Your order total is $99.00. Would you like to proceed?"

ADD SUGGESTION "Confirm Order"
ADD SUGGESTION "Modify Cart"
ADD SUGGESTION "Cancel"

HEAR response
CLEAR SUGGESTIONS
```

## Behavior

- Suggestions appear as clickable buttons in supported channels
- Clicking a suggestion sends its text as user input
- Suggestions persist until cleared or new ones are added
- Maximum suggestions varies by channel (typically 3-10)

## Channel Support

| Channel | Supported | Max Buttons |
|---------|-----------|-------------|
| WhatsApp | ✅ | 3 |
| Telegram | ✅ | 8 |
| Web Chat | ✅ | 10 |
| SMS | ❌ | N/A |

## See Also

- [TALK](./keyword-talk.md)
- [HEAR](./keyword-hear.md)
- [CARD](./keyword-card.md)