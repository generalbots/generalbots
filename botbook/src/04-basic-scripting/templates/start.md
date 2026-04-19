# Start Template

The start template is the simplest possible bot - a greeting flow that demonstrates the core interaction pattern of BASIC: greeting users and responding to their input.

## Topic: Basic Greeting & Help Flow

This template is perfect for:
- Learning BASIC fundamentals
- Simple FAQ bots
- Quick demos
- Starting point for more complex bots

## The Code

```basic
REM Basic greeting and help flow
SET user_name = "Guest"

TALK "Hello, " + user_name + "! How can I help you today?"
HEAR user_input

IF user_input = "help" THEN
    TALK "Sure, I can assist with account info, orders, or support."
ELSE
    TALK "Sorry, I didn't understand. Type 'help' for options."
END IF
```

## Sample Dialogs

These conversations show how the start template works in real-world scenarios.

### Dialog 1: User Asks for Help

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Helper Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Helper Bot</div>
      <p>Hello, Guest! How can I help you today? 👋</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>help</p>
      <div class="wa-time">09:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Sure, I can assist with account info, orders, or support. 📋</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

### Dialog 2: Unknown Input

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Helper Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Helper Bot</div>
      <p>Hello, Guest! How can I help you today? 👋</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>what's the weather?</p>
      <div class="wa-time">11:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Sorry, I didn't understand. Type 'help' for options.</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>help</p>
      <div class="wa-time">11:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Sure, I can assist with account info, orders, or support. 📋</p>
      <div class="wa-time">11:31</div>
    </div>
  </div>
</div>

### Dialog 3: Personalized Greeting (Enhanced Version)

When you add user detection, the experience improves:

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Smart Helper</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Smart Helper</div>
      <p>Hello, Maria! 👋 How can I help you today?</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need help with my order</p>
      <div class="wa-time">14:20 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Of course, Maria! I found your recent order #12345.</p>
      <p>📦 <strong>Status:</strong> Shipped</p>
      <p>🚚 <strong>Delivery:</strong> Tomorrow by 6pm</p>
      <p>Is there anything specific about this order?</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `SET` | Assign a value to a variable |
| `TALK` | Send a message to the user |
| `HEAR` | Wait for and capture user input |
| `IF/ELSE` | Conditional branching based on input |

## How It Works

1. **Variable Setup**: `SET` creates a variable to hold the user's name
2. **Greeting**: `TALK` sends the welcome message
3. **Input Capture**: `HEAR` waits for user response
4. **Response Logic**: `IF/ELSE` determines what to say back

## Enhanced Version

Here's the same template enhanced with LLM for natural understanding:

```basic
REM Smart greeting flow with LLM
SET user_name = "Guest"

TALK "Hello, " + user_name + "! How can I help you today?"
HEAR user_input

' Let LLM understand intent
intent = LLM "Classify this user message into one category: help, account, orders, support, other. Message: " + user_input

SWITCH intent
    CASE "help"
        TALK "I can assist with account info, orders, or support."
    CASE "account"
        TALK "Let me pull up your account information..."
    CASE "orders"
        TALK "I'll check on your recent orders..."
    CASE "support"
        TALK "Connecting you with our support team..."
    DEFAULT
        response = LLM "Respond helpfully to: " + user_input
        TALK response
END SWITCH
```

## Customization Ideas

### Add User Detection

```basic
' Get user info if available
user_name = GET BOT MEMORY "user_" + user_id + "_name"
IF user_name = "" THEN
    TALK "Hi there! What's your name?"
    HEAR user_name
    SET BOT MEMORY "user_" + user_id + "_name", user_name
END IF

TALK "Welcome back, " + user_name + "!"
```

### Add Quick Reply Buttons

```basic
ADD SUGGESTION "Account Info"
ADD SUGGESTION "My Orders"
ADD SUGGESTION "Get Support"
TALK "What would you like help with?"
HEAR choice
```

### Add Time-Based Greeting

```basic
hour = HOUR(NOW())
IF hour < 12 THEN
    greeting = "Good morning"
ELSE IF hour < 18 THEN
    greeting = "Good afternoon"
ELSE
    greeting = "Good evening"
END IF

TALK greeting + ", " + user_name + "!"
```

## Related Templates

- [enrollment.bas](./enrollment.md) - Multi-step data collection
- [auth.bas](./auth.md) - User authentication patterns

---

<style>
/* Inline WhatsApp Chat Styles for this page */
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-system{text-align:center;margin:15px 0;clear:both}
.wa-system span{background-color:#e1f2fb;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>