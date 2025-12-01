# Conversation Examples Style Guide

> **Standard format for displaying bot-user conversations in documentation**

## Overview

All conversation examples in General Bots documentation use a WhatsApp-style chat format. This provides a consistent, familiar, and readable way to show bot interactions.

## CSS Include

The styling is defined in `/assets/wa-chat.css`. Include it in your mdBook or HTML output.

---

## Basic Structure

```html
<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Bot message here</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>User message here</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>
```

---

## Message Types

### Bot Message

```html
<div class="wa-message bot">
  <div class="wa-bubble">
    <p>Hello! How can I help you today?</p>
    <div class="wa-time">10:30</div>
  </div>
</div>
```

### User Message

```html
<div class="wa-message user">
  <div class="wa-bubble">
    <p>What meetings do I have today?</p>
    <div class="wa-time">10:31</div>
  </div>
</div>
```

---

## Formatting Within Messages

### Multiple Paragraphs

```html
<div class="wa-bubble">
  <p>You have 2 meetings scheduled:</p>
  <p>• 2:00 PM - Team Standup (30 min)</p>
  <p>• 4:00 PM - Project Review (1 hour)</p>
  <div class="wa-time">10:31</div>
</div>
```

### Bold Text

```html
<p><strong>Name:</strong> John Smith</p>
<p><strong>Email:</strong> john@example.com</p>
```

### Emoji Usage

Emojis are encouraged to make conversations more expressive:

| Purpose | Emoji Examples |
|---------|----------------|
| Success | ✅ ✓ 🎉 |
| Warning | ⚠️ ⚡ |
| Error | ❌ 🔴 |
| Info | ℹ️ 📋 |
| File | 📄 📁 📎 |
| Calendar | 📅 🗓️ |
| Email | 📧 ✉️ |
| Person | 👤 👥 |
| Time | 🕐 ⏱️ |

### File Attachments

```html
<div class="wa-message user">
  <div class="wa-bubble">
    <p>Here's the report</p>
    <p>📎 quarterly-report.pdf</p>
    <div class="wa-time">10:32</div>
  </div>
</div>
```

### Action Buttons (visual representation)

```html
<p>[📧 Send] [✏️ Edit] [🗑 Discard]</p>
```

---

## Complete Example

```html
<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Schedule a meeting with Sarah tomorrow at 2pm</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Meeting scheduled!</p>
      <p>👥 Meeting with Sarah</p>
      <p>📅 Tomorrow at 2:00 PM</p>
      <p>⏱️ Duration: 1 hour</p>
      <p>Invitation sent to Sarah.</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>
```

**Rendered Output:**

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Schedule a meeting with Sarah tomorrow at 2pm</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Meeting scheduled!</p>
      <p>👥 Meeting with Sarah</p>
      <p>📅 Tomorrow at 2:00 PM</p>
      <p>⏱️ Duration: 1 hour</p>
      <p>Invitation sent to Sarah.</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

---

## Variants

### Full Width

Add `wa-full-width` class for wider conversations:

```html
<div class="wa-chat wa-full-width">
  ...
</div>
```

### Compact

Add `wa-compact` class for tighter spacing:

```html
<div class="wa-chat wa-compact">
  ...
</div>
```

### Hide Timestamps

Add `wa-no-time` class to hide timestamps:

```html
<div class="wa-chat wa-no-time">
  ...
</div>
```

---

## Best Practices

1. **Keep messages concise** - Break long bot responses into multiple paragraphs
2. **Use consistent timestamps** - Use realistic times (10:30, 10:31, etc.)
3. **Start with user context** - Show what the user asked before the bot response
4. **Include visual feedback** - Use emojis for status (✅, ❌, 📋)
5. **Show realistic flows** - Include multi-turn conversations when appropriate
6. **Use semantic formatting** - Bold for labels, lists for options

---

## Files Using This Format

This format is used throughout the documentation:

- `chapter-02/template-crm-contacts.md`
- `chapter-04-gbui/apps/*.md`
- `chapter-06-gbdialog/basic-vs-automation-tools.md`
- And many more...

---

## See Also

- [UI Structure](../chapter-04-gbui/ui-structure.md)
- [Chat App Documentation](../chapter-04-gbui/apps/chat.md)

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:500px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-message{margin-bottom:10px}
.wa-message.user{text-align:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;display:inline-block;text-align:left}
.wa-message.bot .wa-bubble{background-color:#fff;display:inline-block}
.wa-bubble{padding:8px 12px;border-radius:8px;box-shadow:0 1px .5px rgba(0,0,0,.13);max-width:85%}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
</style>