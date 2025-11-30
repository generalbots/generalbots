# Template Examples

Templates are pre-built BASIC scripts that demonstrate common use cases and patterns. Each template includes complete code, explanations, and **interactive WhatsApp-style sample dialogs** showing how the bot behaves in real conversations.

## Available Templates

### 🚀 [start.bas](./templates/start.md)
**Topic: Basic Greeting & Help Flow**

The simplest possible bot - learn BASIC fundamentals with a greeting flow that demonstrates `SET`, `TALK`, `HEAR`, and `IF/ELSE`.

Perfect for:
- Learning BASIC syntax
- Quick demos
- Starting point for new bots

---

### 📋 [enrollment.bas](./templates/enrollment.md)
**Topic: User Registration & Data Collection**

A complete data collection workflow that gathers user information step-by-step, validates inputs, confirms details, and saves the data.

Perfect for:
- Customer onboarding
- Event registrations
- Lead capture forms
- Survey collection

---

### 🔐 [auth.bas](./templates/auth.md)
**Topic: Authentication Patterns**

Secure user authentication flows including login, registration, password reset, and session management.

Perfect for:
- User login systems
- Account verification
- Password recovery
- Session handling

---

## Template Structure

Each template documentation includes:

1. **Topic Description** - What the template is for
2. **The Code** - Complete, working BASIC script
3. **Sample Dialogs** - WhatsApp-style conversations showing real interactions
4. **Keywords Used** - Quick reference of BASIC keywords
5. **Customization Ideas** - Ways to extend the template

## Using Templates

### Method 1: Copy and Customize

Copy the template code into your `.gbdialog` folder and modify it:

```basic
' Copy start.bas and customize
SET user_name = "Guest"
TALK "Hello, " + user_name + "! Welcome to My Company."
HEAR user_input
' ... add your logic
```

### Method 2: Include Templates

Use the `INCLUDE` keyword to use templates as building blocks:

```basic
INCLUDE "templates/auth.bas"

' Now use auth functions
CALL authenticate_user()
```

### Method 3: Use as Reference

Study the templates to learn patterns, then write your own:

```basic
' Learned from enrollment.bas pattern
PARAM name AS string LIKE "John Doe"
DESCRIPTION "User's full name"

TALK "What's your name?"
HEAR name
' ... continue with your logic
```

## More Templates

The `templates/` directory contains 20+ ready-to-use bot configurations:

| Template | Description |
|----------|-------------|
| `default.gbai` | Basic bot with weather, email, and calculation tools |
| `edu.gbai` | Educational bot for course management |
| `crm.gbai` | Customer relationship management |
| `announcements.gbai` | Broadcast messaging system |
| `whatsapp.gbai` | WhatsApp Business integration |
| `store.gbai` | E-commerce bot |
| `healthcare` | Healthcare appointment scheduling |
| `hr` | Human resources assistant |
| `finance` | Financial services bot |
| `marketing.gbai` | Marketing automation |
| `reminder.gbai` | Task and reminder management |
| `backup.gbai` | Automated backup workflows |
| `crawler.gbai` | Web crawling and data extraction |

## Related

- [BASIC vs n8n/Zapier/Make](./basic-vs-automation-tools.md) - Why BASIC beats drag-and-drop tools
- [Keywords Reference](./keywords.md) - Complete keyword documentation
- [Consolidated Examples](./examples-consolidated.md) - More code examples