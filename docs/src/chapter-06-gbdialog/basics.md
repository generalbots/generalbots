# Dialog Basics

BASIC dialogs in General Bots are designed for the LLM era - you write tools and context setters, not complex conversation flows. The LLM handles the natural language understanding and conversation management.

## Core Concepts

* **LLM Tools** – BASIC scripts that become callable tools for the LLM
* **Context Management** – SET_CONTEXT to provide knowledge to the LLM
* **Suggestions** – Guide conversations with ADD_SUGGESTION
* **Memory** – GET_BOT_MEMORY/SET_BOT_MEMORY for persistent data
* **Simple Syntax** – English-like commands that anyone can write

## Modern LLM-First Example

Inspired by real production bots, here's how modern BASIC works:

```basic
' Load context from memory
resume = GET_BOT_MEMORY("announcements")
context = GET_BOT_MEMORY("company_info")

' Give LLM the context it needs
SET_CONTEXT "announcements" AS resume
SET_CONTEXT "company" AS context

' Guide the conversation
CLEAR_SUGGESTIONS
ADD_SUGGESTION "announcements" AS "Show me this week's updates"
ADD_SUGGESTION "company" AS "Tell me about the company"
ADD_SUGGESTION "general" AS "What services do you offer?"

' Start the conversation
TALK "I have the latest announcements and company information ready."
TALK "What would you like to know?"
```

## Creating LLM Tools

Instead of parsing user input, create tools the LLM can call:

```basic
' update-summary.bas - A tool the LLM can invoke
PARAM topic AS STRING LIKE "Q4 Results" DESCRIPTION "Topic to summarize"
PARAM length AS STRING LIKE "brief" DESCRIPTION "brief or detailed"

DESCRIPTION "Creates a summary of the requested topic"

' The tool logic is simple
data = GET_BOT_MEMORY(topic)
summary = LLM "Summarize this " + length + ": " + data
TALK summary
```

## Execution Flow

<svg width="800" height="380" viewBox="0 0 800 380" xmlns="http://www.w3.org/2000/svg" style="background: transparent;">
  <!-- Title -->
  <text x="400" y="25" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-size="18" font-weight="300" fill="currentColor" opacity="0.9">BASIC LLM Tool Execution Flow</text>
  
  <!-- Define gradients and filters -->
  <defs>
    <!-- Soft glow filter -->
    <filter id="glow">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feMerge>
        <feMergeNode in="coloredBlur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
    
    <!-- Beautiful gradients with colors -->
    <linearGradient id="userGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#667eea;stop-opacity:0.15" />
      <stop offset="100%" style="stop-color:#764ba2;stop-opacity:0.25" />
    </linearGradient>
    
    <linearGradient id="llmGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#06ffa5;stop-opacity:0.15" />
      <stop offset="100%" style="stop-color:#00d2ff;stop-opacity:0.25" />
    </linearGradient>
    
    <linearGradient id="toolGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#ffa500;stop-opacity:0.15" />
      <stop offset="100%" style="stop-color:#ff6b6b;stop-opacity:0.25" />
    </linearGradient>
    
    <linearGradient id="responseGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#4facfe;stop-opacity:0.15" />
      <stop offset="100%" style="stop-color:#00f2fe;stop-opacity:0.25" />
    </linearGradient>
    
    <!-- Arrow markers -->
    <marker id="arrow" markerWidth="15" markerHeight="15" refX="14" refY="7.5" orient="auto">
      <path d="M 0 0 L 15 7.5 L 0 15 L 5 7.5 Z" fill="currentColor" opacity="0.6"/>
    </marker>
  </defs>
  
  <!-- Background accent circles for depth -->
  <circle cx="110" cy="85" r="40" fill="#667eea" opacity="0.05"/>
  <circle cx="380" cy="85" r="60" fill="#00d2ff" opacity="0.05"/>
  <circle cx="630" cy="85" r="35" fill="#ffa500" opacity="0.05"/>
  <circle cx="630" cy="215" r="45" fill="#ff6b6b" opacity="0.05"/>
  
  <!-- Stage 1: User Input -->
  <rect x="50" y="60" width="120" height="50" fill="url(#userGrad)" stroke="#667eea" stroke-width="1.5" rx="12" opacity="0.9"/>
  <text x="110" y="90" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">USER</text>
  
  <!-- Elegant flow arrow 1 -->
  <path d="M 170 85 C 200 85, 210 85, 240 85" stroke="url(#llmGrad)" stroke-width="2.5" fill="none" marker-end="url(#arrow)" opacity="0.7" stroke-linecap="round"/>
  
  <!-- Stage 2: LLM Processing -->
  <rect x="240" y="60" width="280" height="50" fill="url(#llmGrad)" stroke="#00d2ff" stroke-width="1.5" rx="12" opacity="0.9"/>
  <text x="380" y="90" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">LLM + CONTEXT</text>
  
  <!-- Elegant flow arrow 2 -->
  <path d="M 520 85 C 540 85, 560 85, 580 85" stroke="url(#toolGrad)" stroke-width="2.5" fill="none" marker-end="url(#arrow)" opacity="0.7" stroke-linecap="round"/>
  
  <!-- Stage 3: Decision Diamond -->
  <path d="M 630 60 L 680 85 L 630 110 L 580 85 Z" fill="rgba(255,165,0,0.1)" stroke="#ffa500" stroke-width="1.5" opacity="0.9"/>
  <circle cx="630" cy="85" r="8" fill="#ffa500" opacity="0.4"/>
  
  <!-- Direct Path - graceful curve -->
  <path d="M 630 60 C 630 35, 500 35, 380 35 C 260 35, 130 35, 130 150" 
        stroke="#4facfe" stroke-width="2" fill="none" marker-end="url(#arrow)" 
        stroke-dasharray="6,3" opacity="0.4" stroke-linecap="round"/>
  
  <!-- Tool Path - smooth descent -->
  <path d="M 630 110 C 630 140, 630 150, 630 180" stroke="#ff6b6b" stroke-width="3" fill="none" marker-end="url(#arrow)" opacity="0.8" stroke-linecap="round"/>
  
  <!-- Stage 4: BASIC Tool -->
  <rect x="550" y="180" width="160" height="70" fill="url(#toolGrad)" stroke="#ff6b6b" stroke-width="1.5" rx="12" opacity="0.9"/>
  <text x="630" y="220" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">BASIC TOOL</text>
  
  <!-- Tool Return Path - smooth curve -->
  <path d="M 550 215 C 480 215, 420 200, 380 180" stroke="#4facfe" stroke-width="2.5" fill="none" marker-end="url(#arrow)" opacity="0.6" stroke-linecap="round"/>
  
  <!-- Stage 5: Response Generation -->
  <rect x="300" y="150" width="160" height="50" fill="url(#responseGrad)" stroke="#4facfe" stroke-width="1.5" rx="12" opacity="0.9"/>
  <text x="380" y="180" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">RESPONSE</text>
  
  <!-- Final Arrow -->
  <path d="M 300 175 C 270 175, 230 175, 200 175" stroke="url(#userGrad)" stroke-width="2.5" fill="none" marker-end="url(#arrow)" opacity="0.7" stroke-linecap="round"/>
  
  <!-- Stage 6: Bot Output -->
  <rect x="50" y="150" width="120" height="50" fill="url(#userGrad)" stroke="#764ba2" stroke-width="1.5" rx="12" opacity="0.9"/>
  <text x="110" y="180" text-anchor="middle" font-family="system-ui" font-size="13" font-weight="500" fill="currentColor">BOT</text>
  
  <!-- Memory Store - elegant and subtle -->
  <rect x="350" y="270" width="300" height="50" fill="rgba(150,150,150,0.05)" stroke="currentColor" stroke-width="1" stroke-dasharray="6,3" rx="12" opacity="0.3"/>
  <text x="500" y="300" text-anchor="middle" font-family="system-ui" font-size="11" font-weight="300" fill="currentColor" opacity="0.5">MEMORY</text>
  
  <!-- Memory connection - delicate -->
  <path d="M 630 250 L 630 270" stroke="currentColor" stroke-width="1" stroke-dasharray="3,3" fill="none" opacity="0.2"/>
  
  <!-- Bottom padding for complete view -->
  <rect x="0" y="350" width="800" height="30" fill="transparent"/>
</svg>

## Key Differences from Traditional Chatbots

| Traditional Approach | LLM + BASIC Approach |
|---------------------|---------------------|
| Parse user input manually | LLM understands naturally |
| Complex IF/ELSE trees | Tools with PARAMs |
| Validate every field | LLM handles validation |
| Design conversation flows | LLM manages conversation |
| Handle errors explicitly | LLM provides graceful responses |

## Best Practices for LLM Era

* **Write Tools, Not Flows** – Create reusable tools the LLM can invoke
* **Use Context Wisely** – Load relevant knowledge with SET_CONTEXT
* **Trust the LLM** – Don't micromanage conversation flow
* **Keep Tools Focused** – Each tool should do one thing well
* **Use Suggestions** – Guide users without forcing paths

## Real-World Pattern

From production bots - a practical tool pattern:

```basic
' schedule-appointment.bas - A real business tool
PARAM service AS STRING LIKE "consultation" DESCRIPTION "Type of appointment"
PARAM date AS DATE LIKE "tomorrow at 3pm" DESCRIPTION "Preferred date and time"
PARAM notes AS STRING DESCRIPTION "Additional notes or requirements"

DESCRIPTION "Schedules an appointment and sends confirmation"

' Simple tool logic - LLM handles the conversation
appointment = GET "api/appointments/available" WITH service, date
IF appointment.available THEN
  SET_BOT_MEMORY "last_appointment" AS appointment.id
  SEND EMAIL TO user.email WITH appointment.details
  TALK "Perfect! I've scheduled your " + service + " for " + date
  TALK "Confirmation sent to your email"
ELSE
  TALK "That time isn't available. Let me suggest alternatives..."
  alternatives = GET "api/appointments/suggest" WITH service, date
  TALK alternatives
END IF
```

The LLM naturally guides the conversation, understands context like "tomorrow" or "next week", and calls this tool when all information is gathered.

## Summary

BASIC in General Bots isn't about controlling conversation flow - it's about providing tools and context that LLMs can use intelligently. Write simple tools, let AI handle the complexity.