# ADD BOT Keywords

Dynamically add bots to a session with specific triggers, tools, or schedules.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `ADD BOT ... WITH TRIGGER` | Add bot activated by keyword |
| `ADD BOT ... WITH TOOLS` | Add bot with specific tools |
| `ADD BOT ... WITH SCHEDULE` | Add bot on a schedule |
| `REMOVE BOT` | Remove bot from session |

## ADD BOT WITH TRIGGER

```basic
ADD BOT "sales-bot" WITH TRIGGER "pricing"
```

When user mentions "pricing", sales-bot activates.

## ADD BOT WITH TOOLS

```basic
ADD BOT "data-bot" WITH TOOLS "database,spreadsheet,charts"
```

## ADD BOT WITH SCHEDULE

```basic
ADD BOT "report-bot" WITH SCHEDULE "0 9 * * MON"
```

Adds bot that runs every Monday at 9 AM (cron format).

## REMOVE BOT

```basic
REMOVE BOT "sales-bot"
```

## Example: Multi-Bot Setup

```basic
' Set up specialized bots for different topics
ADD BOT "orders-bot" WITH TRIGGER "order status, shipping, delivery"
ADD BOT "support-bot" WITH TRIGGER "help, problem, issue, broken"
ADD BOT "sales-bot" WITH TRIGGER "pricing, quote, purchase"

TALK "I've set up our specialist team. Just ask about orders, support, or sales!"
```

## See Also

- [DELEGATE TO BOT](./keyword-delegate-to-bot.md) - Includes A2A Protocol details