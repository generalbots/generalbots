# How To: Create Your First Bot 🟡 BETA

> **Tutorial 1 of the Getting Started Series**
>
> *Follow these simple steps to create a working bot in 10 minutes*

---

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │                                                                 │   │
│   │     🤖  CREATE YOUR FIRST BOT                                   │   │
│   │                                                                 │   │
│   │     ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   │   │
│   │     │  Step   │───▶│  Step   │───▶│  Step   │───▶│  Step   │   │   │
│   │     │   1     │    │   2     │    │   3     │    │   4     │   │   │
│   │     │ Access  │    │ Create  │    │Configure│    │  Test   │   │   │
│   │     │ Suite   │    │  Bot    │    │  Bot    │    │  Bot    │   │   │
│   │     └─────────┘    └─────────┘    └─────────┘    └─────────┘   │   │
│   │                                                                 │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Objective

By the end of this tutorial, you will have:
- Created a new bot instance
- Configured basic settings
- Written a simple greeting
- Tested your bot by talking to it

---

## Time Required

⏱️ **10 minutes**

---

## Prerequisites

Before you begin, make sure you have:

- [ ] Access to General Bots Suite (URL provided by your administrator)
- [ ] A web browser (Chrome, Firefox, Safari, or Edge)
- [ ] Administrator or Bot Creator permissions

---

## Step 1: Access the Suite

### 1.1 Open Your Browser

Launch your preferred web browser by clicking its icon.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  🌐 Browser                                                     [─][□][×]│
├─────────────────────────────────────────────────────────────────────────┤
│  ← → ↻  │ https://your-company.bot:9000                          │ ☆ │  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                          Loading...                                     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Navigate to Your General Bots URL

Type your General Bots address in the address bar and press **Enter**.

💡 **Tip**: Your URL will look something like:
- `http://localhost:9000` (development)
- `https://bots.yourcompany.com` (production)
- `https://app.pragmatismo.cloud` (cloud hosted)

### 1.3 Log In (If Required)

If you see a login screen:
1. Enter your **username** or **email**
2. Enter your **password**
3. Click **Sign In**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                    ┌────────────────────────────┐                       │
│                    │    🤖 General Bots         │                       │
│                    │                            │                       │
│                    │  Username:                 │                       │
│                    │  ┌────────────────────┐    │                       │
│                    │  │ admin@company.com  │    │                       │
│                    │  └────────────────────┘    │                       │
│                    │                            │                       │
│                    │  Password:                 │                       │
│                    │  ┌────────────────────┐    │                       │
│                    │  │ ••••••••••••       │    │                       │
│                    │  └────────────────────┘    │                       │
│                    │                            │                       │
│                    │  ┌────────────────────┐    │                       │
│                    │  │     Sign In  ──►   │    │                       │
│                    │  └────────────────────┘    │                       │
│                    │                            │                       │
│                    └────────────────────────────┘                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

✅ **Checkpoint**: You should now see the General Bots Suite interface.

---

## Step 2: Create a New Bot

### 2.1 Open the Apps Menu

Click the **nine-dot grid icon** (⋮⋮⋮) in the top-right corner of the screen.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  🤖 General Bots                                    [⋮⋮⋮] ◄── Click here │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                                                                         │
```

### 2.2 Select "Sources"

From the apps menu that appears, click **Sources**.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                         ┌───────────────────┐                           │
│                         │   💬 Chat         │                           │
│                         │   📁 Drive        │                           │
│                         │   ✓  Tasks        │                           │
│                         │   ✉  Mail         │                           │
│                         │   📝 Paper        │                           │
│                         │   📊 Analytics    │                           │
│                         │ ▶ 📋 Sources ◀───┼─── Click here             │
│                         │   🎨 Designer     │                           │
│                         │   ⚙️  Settings     │                           │
│                         └───────────────────┘                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Click "New Bot"

In the Sources application, locate and click the **New Bot** button.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Sources                                                                │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │  Templates   │  │   Prompts    │  │    Bots      │ ◄── Active Tab   │
│  └──────────────┘  └──────────────┘  └──────────────┘                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Your Bots                              ┌─────────────────┐             │
│  ─────────                              │  ➕ New Bot     │ ◄── Click  │
│                                         └─────────────────┘             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  No bots yet. Create your first bot!                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.4 Enter Bot Details

A dialog box will appear. Fill in the following fields:

| Field | What to Enter | Example |
|-------|---------------|---------|
| **Bot Name** | A unique identifier (no spaces) | `mycompany` |
| **Display Name** | Friendly name shown to users | `My Company Assistant` |
| **Description** | What your bot does | `Helps employees find information` |
| **Template** | Starting point (select from dropdown) | `default` |

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Create New Bot                           [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Bot Name *                                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ mycompany                                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  ⚠️ Use lowercase letters, numbers, and hyphens only                    │
│                                                                         │
│  Display Name *                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ My Company Assistant                                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Description                                                            │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Helps employees find information and complete tasks             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Template                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ default                                                     [▼] │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│                    ┌──────────┐  ┌──────────────────┐                  │
│                    │  Cancel  │  │  Create Bot ──►  │                  │
│                    └──────────┘  └──────────────────┘                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.5 Click "Create Bot"

Click the **Create Bot** button to create your bot.

💡 **Tip**: The bot creation process takes a few seconds. You'll see a progress indicator.

✅ **Checkpoint**: Your new bot should appear in the bot list.

---

## Step 3: Configure Basic Settings

### 3.1 Open Bot Settings

Click on your new bot to select it, then click **Settings** (or the ⚙️ icon).

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Your Bots                                                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  🤖 mycompany                                              [⚙️]  │◄──│
│  │     My Company Assistant                                        │   │
│  │     Status: ● Active                                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                                                    │
                                                          Click the ⚙️ icon
```

### 3.2 Set the Welcome Message

Find the **Welcome Message** field and enter a friendly greeting:

```
Welcome Message:

┌─────────────────────────────────────────────────────────────────────────┐
│ Hello! 👋 I'm your Company Assistant. I can help you with:             │
│                                                                         │
│ • Finding documents and information                                     │
│ • Answering questions about policies                                    │
│ • Creating tasks and reminders                                          │
│                                                                         │
│ How can I help you today?                                               │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Configure AI Model (Optional)

If you have API keys for AI services, configure them:

| Setting | Description | Example Value |
|---------|-------------|---------------|
| **LLM Provider** | AI service to use | `anthropic` |
| **Model** | Specific model | `claude-sonnet-4.5` |
| **API Key** | Your API key | `sk-...` |

⚠️ **Warning**: Keep your API keys secret. Never share them.

### 3.4 Save Settings

Click the **Save** button to save your configuration.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Bot Settings                                                     [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  [General]  [AI Model]  [Channels]  [Advanced]                         │
│                                                                         │
│  ─────────────────────────────────────────────────────────────────     │
│                                                                         │
│                                              ┌────────────────────┐     │
│                                              │    💾 Save         │◄────│
│                                              └────────────────────┘     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
                                                         Click Save
```

✅ **Checkpoint**: Your settings are saved. The bot is ready to test.

---

## Step 4: Test Your Bot

### 4.1 Open Chat

Click the **Chat** app from the Apps Menu (⋮⋮⋮).

### 4.2 Select Your Bot

If you have multiple bots, select yours from the bot dropdown:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  💬 Chat                              [mycompany           ▼]           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  🤖 My Company Assistant                                    │   │
│      │                                                             │   │
│      │  Hello! 👋 I'm your Company Assistant. I can help          │   │
│      │  you with:                                                  │   │
│      │                                                             │   │
│      │  • Finding documents and information                        │   │
│      │  • Answering questions about policies                       │   │
│      │  • Creating tasks and reminders                             │   │
│      │                                                             │   │
│      │  How can I help you today?                                  │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Type your message...                                            [↑]   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Send a Test Message

Type a simple message and press **Enter**:

```
You: Hello!
```

### 4.4 Verify the Response

Your bot should respond! If it does, congratulations — your bot is working!

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  👤 You                                                     │   │
│      │  Hello!                                                     │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
│      ┌─────────────────────────────────────────────────────────────┐   │
│      │  🤖 My Company Assistant                                    │   │
│      │  Hello! How can I assist you today?                         │   │
│      └─────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

✅ **Checkpoint**: Your bot responds to messages. Setup complete!

---

## 🎉 Congratulations!

You have successfully created your first bot! Here's what you accomplished:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│    ✓ Accessed General Bots Suite                                        │
│    ✓ Created a new bot instance                                         │
│    ✓ Configured basic settings                                          │
│    ✓ Tested the bot with a conversation                                 │
│                                                                         │
│    Your bot "mycompany" is now ready to use!                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Troubleshooting

### Problem: "Create Bot" button is disabled

**Cause**: Required fields are empty or invalid.

**Solution**: 
1. Check that Bot Name contains only lowercase letters, numbers, and hyphens
2. Ensure Display Name is not empty
3. Verify a template is selected

### Problem: Bot doesn't respond

**Cause**: AI model not configured or API key invalid.

**Solution**:
1. Open bot settings
2. Verify AI model configuration
3. Check that API key is correct
4. Ensure you have API credits remaining

### Problem: "Permission denied" error

**Cause**: Your account doesn't have bot creation rights.

**Solution**:
1. Contact your administrator
2. Request "Bot Creator" or "Administrator" role

### Problem: Page won't load

**Cause**: Network or server issue.

**Solution**:
1. Check your internet connection
2. Try refreshing the page (F5 or Ctrl+R)
3. Clear browser cache
4. Contact your system administrator

---

## What You Learned

In this tutorial, you learned:

| Concept | Description |
|---------|-------------|
| **Bot Instance** | A unique bot with its own configuration |
| **Bot Name** | Technical identifier used internally |
| **Display Name** | Friendly name shown to users |
| **Template** | Pre-built starting point for your bot |
| **Welcome Message** | First message users see |

---

## Next Steps

Now that you have a working bot, continue learning:

| Next Tutorial | What You'll Learn |
|---------------|-------------------|
| [Your First Conversation](./first-conversation.md) | Understanding how conversations work |
| [Add Knowledge Base Documents](./add-kb-documents.md) | Teaching your bot from files |
| [Write Your First Dialog](./write-first-dialog.md) | Creating custom conversation flows |

---

## Quick Reference

### Bot Naming Rules

- ✅ `mycompany` — Good
- ✅ `hr-assistant` — Good
- ✅ `support2024` — Good
- ❌ `My Company` — No spaces
- ❌ `HR_Bot` — No underscores
- ❌ `Support@2024` — No special characters

### Essential Settings Checklist

- [ ] Bot Name (unique identifier)
- [ ] Display Name (user-friendly)
- [ ] Welcome Message (first impression)
- [ ] AI Model (for responses)
- [ ] Language (for localization)

---

*Tutorial 1 of 30 • [Back to How-To Index](./README.md) • [Next: Your First Conversation →](./first-conversation.md)*