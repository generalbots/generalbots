# How To: Write Your First Dialog

> **Tutorial 5 of the BASIC Dialogs Series**
>
> *Create a simple conversation script in 20 minutes*

---

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                                                                 │   │
│   │     📝  WRITE YOUR FIRST DIALOG                                 │   │
│   │                                                                 │   │
│   │     ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   │   │
│   │     │  Step   │───▶│  Step   │───▶│  Step   │───▶│  Step   │   │   │
│   │     │   1     │    │   2     │    │   3     │    │   4     │   │   │
│   │     │ Create  │    │  Write  │    │  Test   │    │ Enhance │   │   │
│   │     │  File   │    │  Code   │    │ Dialog  │    │  Logic  │   │   │
│   │     └─────────┘    └─────────┘    └─────────┘    └─────────┘   │   │
│   │                                                                 │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Objective

By the end of this tutorial, you will have:
- Created a `.bas` dialog file
- Written code using TALK and HEAR keywords
- Used conditional logic (IF/THEN/ELSE)
- Stored and retrieved user information
- Tested your dialog in the chat interface

---

## Time Required

⏱️ **20 minutes**

---

## Prerequisites

Before you begin, make sure you have:

- [ ] A working bot (see [Create Your First Bot](./create-first-bot.md))
- [ ] Access to the Designer or Drive app
- [ ] Basic understanding of the chat interface

---

## What is a Dialog?

A **dialog** is a conversation script written in BASIC that controls how your bot talks with users. Think of it like a script for a play — you write what the bot should say and how it should respond to the user.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        HOW DIALOGS WORK                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│    User says: "Hello"                                                   │
│         │                                                               │
│         ▼                                                               │
│    ┌─────────────────┐                                                 │
│    │  Dialog Script  │  ◄── Your BASIC code runs here                  │
│    │  (greeting.bas) │                                                 │
│    └────────┬────────┘                                                 │
│             │                                                           │
│             ▼                                                           │
│    Bot says: "Hi there! What's your name?"                              │
│         │                                                               │
│         ▼                                                               │
│    User says: "Sarah"                                                   │
│         │                                                               │
│         ▼                                                               │
│    Bot says: "Nice to meet you, Sarah!"                                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Create the Dialog File

### 1.1 Open the Drive App

Click the **Apps Menu** (⋮⋮⋮) and select **Drive**.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📁 Drive                                                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  📂 mycompany.gbai                                                      │
│     ├── 📂 mycompany.gbdialog    ◄── Dialog files go here              │
│     ├── 📂 mycompany.gbot                                               │
│     ├── 📂 mycompany.gbkb                                               │
│     └── 📂 mycompany.gbdrive                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Navigate to the Dialog Folder

Double-click **mycompany.gbai**, then **mycompany.gbdialog**.

### 1.3 Create a New File

Click **New File** (or press Ctrl+N) and name it:

```
greeting.bas
```

⚠️ **Warning**: The file must end with `.bas` to be recognized as a dialog.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           New File                                [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  File Name:                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ greeting.bas                                                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Location: mycompany.gbai / mycompany.gbdialog /                       │
│                                                                         │
│                    ┌──────────┐  ┌──────────────────┐                  │
│                    │  Cancel  │  │  Create  ──►     │                  │
│                    └──────────┘  └──────────────────┘                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

✅ **Checkpoint**: You should see `greeting.bas` in your dialog folder.

---

## Step 2: Write the Dialog Code

### 2.1 Open the File for Editing

Double-click `greeting.bas` to open it in the editor.

### 2.2 Write Your First Line

Type the following code:

```basic
TALK "Hello! Welcome to our service. 👋"
```

This is the simplest possible dialog — the bot just says one thing.

### 2.3 Add User Input

Now let's ask for the user's name:

```basic
TALK "Hello! Welcome to our service. 👋"
TALK "What is your name?"
HEAR name
TALK "Nice to meet you, " + name + "!"
```

Let's break this down:

| Line | What It Does |
|------|--------------|
| `TALK "..."` | Bot displays a message |
| `HEAR name` | Bot waits for user input, stores it in `name` |
| `"..." + name + "..."` | Combines text with the variable |

### 2.4 The Complete First Dialog

Here's your complete `greeting.bas`:

```basic
' ============================================
' GREETING DIALOG
' A friendly welcome conversation
' ============================================

' Greet the user
TALK "Hello! Welcome to our service. 👋"

' Ask for their name
TALK "What is your name?"
HEAR name

' Respond with their name
TALK "Nice to meet you, " + name + "!"
TALK "How can I help you today?"
```

💡 **Tip**: Lines starting with `'` are comments — they're ignored by the bot but help you understand the code.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📝 greeting.bas                                              [Save] ⌘S │
├─────────────────────────────────────────────────────────────────────────┤
│  1 │ ' ============================================                     │
│  2 │ ' GREETING DIALOG                                                  │
│  3 │ ' A friendly welcome conversation                                  │
│  4 │ ' ============================================                     │
│  5 │                                                                    │
│  6 │ ' Greet the user                                                   │
│  7 │ TALK "Hello! Welcome to our service. 👋"                           │
│  8 │                                                                    │
│  9 │ ' Ask for their name                                               │
│ 10 │ TALK "What is your name?"                                          │
│ 11 │ HEAR name                                                          │
│ 12 │                                                                    │
│ 13 │ ' Respond with their name                                          │
│ 14 │ TALK "Nice to meet you, " + name + "!"                             │
│ 15 │ TALK "How can I help you today?"                                   │
│    │                                                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.5 Save the File

Press **Ctrl+S** or click the **Save** button.

✅ **Checkpoint**: Your dialog file is saved and ready to test.

---

## Step 3: Test Your Dialog

### 3.1 Open Chat

Click the **Apps Menu** (⋮⋮⋮) and select **Chat**.

### 3.2 Trigger the Dialog

Type the command to run your dialog:

```
/greeting
```

Or simply type something that matches "greeting" — the system will recognize it.

### 3.3 Have the Conversation

Watch your dialog run:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  💬 Chat                                                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  👤 You                                                     │   │
│      │  /greeting                                                  │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  🤖 Bot                                                     │   │
│      │  Hello! Welcome to our service. 👋                          │   │
│      │  What is your name?                                         │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  👤 You                                                     │   │
│      │  Sarah                                                      │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  🤖 Bot                                                     │   │
│      │  Nice to meet you, Sarah!                                   │   │
│      │  How can I help you today?                                  │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Type your message...                                            [↑]   │
└─────────────────────────────────────────────────────────────────────────┘
```

✅ **Checkpoint**: Your dialog runs and responds correctly!

---

## Step 4: Enhance with Logic

Now let's make our dialog smarter with conditional logic.

### 4.1 Add Input Validation

Update your dialog to handle different types of input:

```basic
' ============================================
' GREETING DIALOG (Enhanced)
' A friendly welcome with input validation
' ============================================

TALK "Hello! Welcome to our service. 👋"
TALK "What is your name?"
HEAR name

' Check if name was provided
IF name = "" THEN
    TALK "I didn't catch your name. That's okay!"
    name = "friend"
END IF

TALK "Nice to meet you, " + name + "!"
```

### 4.2 Add Menu Options

Let's give the user choices:

```basic
' ============================================
' GREETING DIALOG (Full Version)
' Welcome with menu options
' ============================================

TALK "Hello! Welcome to our service. 👋"
TALK "What is your name?"
HEAR name

IF name = "" THEN
    name = "friend"
END IF

TALK "Nice to meet you, " + name + "!"
TALK ""
TALK "How can I help you today?"
TALK "1. Learn about our services"
TALK "2. Contact support"
TALK "3. Check my account"
TALK ""
TALK "Please type 1, 2, or 3:"

HEAR choice

SELECT CASE choice
    CASE "1"
        TALK "Great! We offer AI-powered automation for businesses."
        TALK "Would you like to schedule a demo?"
    CASE "2"
        TALK "I'll connect you with our support team."
        TALK "Please describe your issue:"
        HEAR issue
        TALK "Thank you. A support agent will contact you about: " + issue
    CASE "3"
        TALK "To check your account, I'll need to verify your identity."
        TALK "Please enter your email address:"
        HEAR email
        TALK "Looking up account for: " + email
    CASE ELSE
        TALK "I didn't understand that choice."
        TALK "Please type 1, 2, or 3 next time."
END SELECT

TALK ""
TALK "Is there anything else I can help with, " + name + "?"
```

### 4.3 Understanding SELECT CASE

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      SELECT CASE EXPLAINED                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│    User types: "2"                                                      │
│         │                                                               │
│         ▼                                                               │
│    ┌─────────────────────────────────────────────────────────────┐     │
│    │  SELECT CASE choice                                          │     │
│    │    ┌─────────────┐                                          │     │
│    │    │ CASE "1"    │──▶ Skip (not matched)                    │     │
│    │    └─────────────┘                                          │     │
│    │    ┌─────────────┐                                          │     │
│    │    │ CASE "2"  ★ │──▶ EXECUTE! ───▶ "I'll connect you..."   │     │
│    │    └─────────────┘                                          │     │
│    │    ┌─────────────┐                                          │     │
│    │    │ CASE "3"    │──▶ Skip (not checked after match)        │     │
│    │    └─────────────┘                                          │     │
│    │    ┌─────────────┐                                          │     │
│    │    │ CASE ELSE   │──▶ Skip (only runs if nothing matched)   │     │
│    │    └─────────────┘                                          │     │
│    │  END SELECT                                                  │     │
│    └─────────────────────────────────────────────────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Step 5: Remember User Information

### 5.1 Store User Data

Use `SET USER MEMORY` to remember information between conversations:

```basic
' After getting the name
SET USER MEMORY "name", name

' Later, in another dialog, retrieve it:
savedName = GET USER MEMORY "name"

IF savedName <> "" THEN
    TALK "Welcome back, " + savedName + "!"
ELSE
    TALK "Hello! I don't think we've met before."
END IF
```

### 5.2 Store Bot-Wide Data

Use `SET BOT MEMORY` for data that applies to all users:

```basic
' Store a bot-wide counter
visitorCount = GET BOT MEMORY "visitor_count"
IF visitorCount = "" THEN
    visitorCount = 0
END IF
visitorCount = visitorCount + 1
SET BOT MEMORY "visitor_count", visitorCount

TALK "You are visitor number " + visitorCount + " today!"
```

---

## Complete Example: Support Request Dialog

Here's a complete, practical dialog you can use as a template:

```basic
' ============================================
' SUPPORT REQUEST DIALOG
' Collects support ticket information
' ============================================

' Check if we know this user
userName = GET USER MEMORY "name"

IF userName = "" THEN
    TALK "Hello! I'm here to help you create a support request."
    TALK "First, what's your name?"
    HEAR userName
    SET USER MEMORY "name", userName
ELSE
    TALK "Welcome back, " + userName + "!"
END IF

' Get contact information
TALK "What email should we use to contact you?"
HEAR AS email
email

IF email = "" THEN
    TALK "I'll need an email to send you updates."
    HEAR AS email
    email
END IF

' Get issue category
TALK ""
TALK "What type of issue are you experiencing?"
TALK ""
TALK "1. 🔧 Technical problem"
TALK "2. 💳 Billing question"
TALK "3. 📦 Order status"
TALK "4. ❓ General question"
TALK ""

HEAR category

SELECT CASE category
    CASE "1"
        categoryName = "Technical"
        TALK "I'm sorry you're having technical difficulties."
    CASE "2"
        categoryName = "Billing"
        TALK "I can help with billing questions."
    CASE "3"
        categoryName = "Orders"
        TALK "Let me check on your order."
    CASE ELSE
        categoryName = "General"
        TALK "I'll make sure the right team sees this."
END SELECT

' Get description
TALK ""
TALK "Please describe your issue in detail:"
HEAR description

' Get urgency
TALK ""
TALK "How urgent is this?"
TALK "1. 🔴 Critical - I can't work"
TALK "2. 🟡 High - Affecting my work"
TALK "3. 🟢 Normal - When you get a chance"
HEAR urgency

SELECT CASE urgency
    CASE "1"
        urgencyLevel = "Critical"
    CASE "2"
        urgencyLevel = "High"
    CASE ELSE
        urgencyLevel = "Normal"
END SELECT

' Confirm ticket
TALK ""
TALK "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TALK "📋 SUPPORT REQUEST SUMMARY"
TALK "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TALK "Name: " + userName
TALK "Email: " + email
TALK "Category: " + categoryName
TALK "Urgency: " + urgencyLevel
TALK "Issue: " + description
TALK "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TALK ""
TALK "Should I submit this request? (yes/no)"

HEAR confirm

IF confirm = "yes" OR confirm = "Yes" OR confirm = "YES" THEN
    ' Here you would typically save to a database
    ' For now, just confirm
    TALK "✅ Your support request has been submitted!"
    TALK "Ticket ID: SR-" + FORMAT(NOW, "yyyyMMddHHmm")
    TALK "You'll receive a confirmation email at " + email
    TALK "Our team typically responds within 24 hours."
ELSE
    TALK "No problem! Your request was not submitted."
    TALK "Feel free to start over when you're ready."
END IF

TALK ""
TALK "Is there anything else I can help with?"
```

---

## 🎉 Congratulations!

You've written your first dialog! Here's what you learned:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│    ✓ Created a .bas dialog file                                         │
│    ✓ Used TALK to display messages                                      │
│    ✓ Used HEAR to get user input                                        │
│    ✓ Combined text with variables                                       │
│    ✓ Used IF/THEN/ELSE for decisions                                    │
│    ✓ Used SELECT CASE for menus                                         │
│    ✓ Stored data with SET USER MEMORY                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Troubleshooting

### Problem: Dialog doesn't start

**Cause**: File name or location is incorrect.

**Solution**:
1. Verify file ends with `.bas`
2. Confirm file is in the `.gbdialog` folder
3. Check there are no syntax errors

### Problem: "Unexpected token" error

**Cause**: Syntax error in your code.

**Solution**:
1. Check all strings have opening and closing quotes
2. Verify IF statements have matching END IF
3. Ensure SELECT CASE has END SELECT

### Problem: Variable is empty

**Cause**: User skipped the HEAR prompt.

**Solution**:
1. Add validation: `IF variable = "" THEN`
2. Provide a default value
3. Ask again if needed

### Problem: Bot doesn't remember data

**Cause**: Not using memory keywords correctly.

**Solution**:
1. Use `SET USER MEMORY "key", value` to save
2. Use `GET USER MEMORY "key"` to retrieve
3. Ensure key names match exactly (case-sensitive)

---

## Quick Reference

### Essential Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `TALK` | Display message | `TALK "Hello!"` |
| `HEAR` | Get user input | `HEAR name` |
| `HEAR AS type` | Get typed input | `HEAR AS email emailVar` |
| `SET` | Set variable | `SET x = 5` |
| `IF/THEN/ELSE` | Conditional | `IF x > 5 THEN ... END IF` |
| `SELECT CASE` | Menu choice | `SELECT CASE x ... END SELECT` |
| `SET USER MEMORY` | Save user data | `SET USER MEMORY "key", value` |
| `GET USER MEMORY` | Load user data | `x = GET USER MEMORY "key"` |
| `SET BOT MEMORY` | Save bot data | `SET BOT MEMORY "key", value` |
| `GET BOT MEMORY` | Load bot data | `x = GET BOT MEMORY "key"` |

### Common Patterns

**Greeting with memory:**
```basic
name = GET USER MEMORY "name"
IF name = "" THEN
    TALK "What's your name?"
    HEAR name
    SET USER MEMORY "name", name
ELSE
    TALK "Welcome back, " + name + "!"
END IF
```

**Menu with validation:**
```basic
TALK "Choose: 1, 2, or 3"
HEAR choice
IF choice < "1" OR choice > "3" THEN
    TALK "Invalid choice, using default."
    choice = "1"
END IF
```

**Loop for retries:**
```basic
attempts = 0
valid = FALSE
WHILE valid = FALSE AND attempts < 3
    TALK "Enter your email:"
    HEAR AS email input
    IF input <> "" THEN
        valid = TRUE
    END IF
    attempts = attempts + 1
WEND
```

---

## Next Steps

| Next Tutorial | What You'll Learn |
|---------------|-------------------|
| [Store User Information](./store-user-info.md) | Advanced memory patterns |
| [Call External APIs](./call-external-apis.md) | Connect to web services |
| [Send Automated Messages](./send-automated.md) | Scheduled broadcasts |

---

## Best Practices

1. **Comment your code** — Use `'` for explanations
2. **Validate all input** — Never assume users type correctly
3. **Provide defaults** — Handle empty responses gracefully
4. **Use clear prompts** — Tell users exactly what to type
5. **Confirm important actions** — Ask before submitting forms
6. **Use spaces in keywords** — `SET BOT MEMORY` not `SET_BOT_MEMORY`
7. **Test thoroughly** — Try all menu options and edge cases

---

*Tutorial 5 of 30 • [Back to How-To Index](./README.md) • [Next: Store User Information →](./store-user-info.md)*