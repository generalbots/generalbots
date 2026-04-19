# Dialog Basics

BASIC dialogs in General Bots are designed for the LLM era - you write tools and context setters, not complex conversation flows.

## Core Concepts

| Concept | Description |
|---------|-------------|
| **LLM Tools** | BASIC scripts that become callable tools for the LLM |
| **Context** | SET CONTEXT provides knowledge to the LLM |
| **Suggestions** | ADD SUGGESTION guides conversations |
| **Memory** | GET/SET BOT/USER MEMORY for persistent data |

## LLM-First Example

```basic
' Load context from memory
resume = GET BOT MEMORY "announcements"
context = GET BOT MEMORY "company_info"

' Give LLM the context it needs
SET CONTEXT "announcements" AS resume
SET CONTEXT "company" AS context

' Guide the conversation
CLEAR SUGGESTIONS
ADD SUGGESTION "announcements" AS "Show me this week's updates"
ADD SUGGESTION "company" AS "Tell me about the company"

' Start conversation
TALK "What would you like to know?"
```

## Creating LLM Tools

Instead of parsing user input, create tools the LLM can call:

```basic
' update-summary.bas - A tool the LLM can invoke
PARAM topic AS STRING LIKE "Q4 Results" DESCRIPTION "Topic to summarize"
PARAM length AS STRING LIKE "brief" DESCRIPTION "brief or detailed"

DESCRIPTION "Creates a summary of the requested topic"

data = GET BOT MEMORY topic
summary = LLM "Summarize this " + length + ": " + data
TALK summary
```

## Traditional vs LLM Approach

| Traditional | LLM + BASIC |
|-------------|-------------|
| Parse user input manually | LLM understands naturally |
| Complex IF/ELSE trees | Tools with PARAMs |
| Validate every field | LLM handles validation |
| Design conversation flows | LLM manages conversation |

## Tool Pattern Example

```basic
' schedule-appointment.bas
PARAM service AS STRING LIKE "consultation" DESCRIPTION "Type of appointment"
PARAM date AS DATE LIKE "tomorrow at 3pm" DESCRIPTION "Preferred date/time"

DESCRIPTION "Schedules an appointment and sends confirmation"

appointment = GET "api/appointments/available" WITH service, date
IF appointment.available THEN
  SET BOT MEMORY "last_appointment" AS appointment.id
  SEND EMAIL TO user.email WITH appointment.details
  TALK "Scheduled your " + service + " for " + date
ELSE
  alternatives = GET "api/appointments/suggest" WITH service, date
  TALK "That time isn't available. Alternatives: " + alternatives
END IF
```

## Best Practices

| Do | Don't |
|----|-------|
| Write focused tools | Create complex conversation flows |
| Use context wisely | Micromanage the LLM |
| Trust the LLM | Parse user input manually |
| Use suggestions | Force rigid paths |

## See Also

- [Keywords Reference](./keywords.md) - Complete keyword list
- [Chapter Overview](./README.md) - Philosophy and introduction
- [Templates](./templates.md) - Real-world examples