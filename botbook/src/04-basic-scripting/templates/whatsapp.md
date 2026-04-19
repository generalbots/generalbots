# WhatsApp Template

The WhatsApp template provides specialized tools for WhatsApp Business API integration, including template message sending, task creation, and WhatsApp-specific features.

## Topic: WhatsApp Business Integration

This template is perfect for:
- WhatsApp Business API integration
- Template message campaigns
- WhatsApp-based customer service
- Automated WhatsApp notifications
- Task management via WhatsApp

## The Code: send.bas

```basic
PARAM phone AS PHONE LIKE "122233333333" DESCRIPTION "WhatsApp phone number with country code"
PARAM template AS STRING LIKE "newsletter-zap.txt" DESCRIPTION "Template file name to send"
PARAM variables AS OBJECT LIKE "{name: 'John'}" DESCRIPTION "Template variables for personalization" OPTIONAL

DESCRIPTION "Send a WhatsApp template message to a phone number"

SEND TEMPLATE TO phone, template, variables

WITH log
    timestamp = NOW()
    phoneNumber = phone
    templateFile = template
    status = "sent"
END WITH

SAVE "whatsapp_log.csv", log

TALK "WhatsApp message sent to " + phone

RETURN phone
```

## Sample Dialogs

These conversations show how the WhatsApp template works in real-world scenarios.

### Dialog 1: Sending a Template Message

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💬</div>
    <div class="wa-header-info">
      <div class="wa-header-name">WhatsApp Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Send the newsletter to +5511999999999</p>
      <div class="wa-time">10:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">WhatsApp Bot</div>
      <p>📤 Sending template message...</p>
      <p>📱 Phone: +5511999999999</p>
      <p>📄 Template: newsletter-zap.txt</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ WhatsApp message sent to +5511999999999</p>
      <p>📋 Logged to whatsapp_log.csv</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>
</div>

### Dialog 2: Creating a Task via WhatsApp

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💬</div>
    <div class="wa-header-info">
      <div class="wa-header-name">WhatsApp Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Create a task: Call client about proposal</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Task Created</strong></p>
      <p>📋 Call client about proposal</p>
      <p>📅 Due: Today</p>
      <p>👤 Assigned to: You</p>
      <p>I'll remind you later!</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

### Dialog 3: Personalized Template with Variables

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💬</div>
    <div class="wa-header-info">
      <div class="wa-header-name">WhatsApp Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Send welcome message to Maria at +5521888888888</p>
      <div class="wa-time">09:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📤 Sending personalized template...</p>
      <p>📱 Phone: +5521888888888</p>
      <p>📄 Template: welcome.txt</p>
      <p>👤 Variables: {name: "Maria"}</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ WhatsApp message sent!</p>
      <p>Message preview:</p>
      <p><em>"Hello Maria! Welcome to our service. We're excited to have you with us!"</em></p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `PARAM` | Define input parameters for the tool |
| `DESCRIPTION` | Tool description for AI understanding |
| `SEND TEMPLATE TO` | Send WhatsApp template message |
| `WITH/END WITH` | Create structured log object |
| `SAVE` | Log message to CSV file |
| `TALK` | Confirm action to user |
| `RETURN` | Return result |

## Template Structure

```
whatsapp.gbai/
├── whatsapp.gbdialog/
│   ├── send.bas           # Send template messages
│   └── create-task.bas    # Create tasks via WhatsApp
├── whatsapp.gbkb/
│   ├── articles/          # Knowledge base articles
│   │   └── newsletter-zap.txt
│   └── images/            # Media files
└── whatsapp.gbot/
    └── config.csv         # Bot configuration
```

## Create Task Tool: create-task.bas

```basic
PARAM title AS STRING LIKE "Call client" DESCRIPTION "Task title"
PARAM due_date AS DATE LIKE "2025-01-20" DESCRIPTION "Due date" OPTIONAL
PARAM priority AS STRING LIKE "medium" DESCRIPTION "Priority: high, medium, low" OPTIONAL

DESCRIPTION "Create a task from WhatsApp conversation"

IF NOT due_date THEN
    due_date = NOW()
END IF

IF NOT priority THEN
    priority = "medium"
END IF

WITH task
    id = "TASK-" + FORMAT(RANDOM(10000, 99999))
    taskTitle = title
    dueDate = due_date
    taskPriority = priority
    createdBy = FROM
    createdAt = NOW()
    status = "pending"
END WITH

SAVE "tasks.csv", task

CREATE TASK title, priority, FROM

TALK "✅ Task created: " + title
TALK "📅 Due: " + FORMAT(due_date, "MMM DD, YYYY")
TALK "⚡ Priority: " + priority

RETURN task.id
```

## WhatsApp Template Messages

### Understanding Template Messages

WhatsApp Business API requires pre-approved templates for initiating conversations. Templates can include:

- **Text**: Plain text with optional variables
- **Media**: Images, documents, videos
- **Buttons**: Quick reply or call-to-action buttons
- **Headers**: Text, image, document, or video headers

### Template File Format

Create templates in the `.gbkb/articles/` folder:

```
newsletter-zap.txt
---
Hello {{1}}!

Here's your weekly newsletter:

📰 Top Stories This Week
{{2}}

🎯 Don't miss our special offer!
{{3}}

Reply STOP to unsubscribe.
```

### Variables in Templates

Variables are placeholders replaced with actual values:

| Variable | Description | Example |
|----------|-------------|---------|
| `{{1}}` | First parameter | Customer name |
| `{{2}}` | Second parameter | Content body |
| `{{3}}` | Third parameter | Offer details |

## Customization Ideas

### Add Bulk Messaging

```basic
PARAM template AS STRING DESCRIPTION "Template to send"
PARAM contacts_file AS STRING LIKE "contacts.csv" DESCRIPTION "CSV file with contacts"

DESCRIPTION "Send template to multiple contacts"

contacts = FIND contacts_file

sent = 0
failed = 0

FOR EACH contact IN contacts
    variables = {
        "name": contact.name,
        "company": contact.company
    }
    
    result = SEND TEMPLATE TO contact.phone, template, variables
    
    IF result THEN
        sent = sent + 1
    ELSE
        failed = failed + 1
    END IF
    
    WAIT 2  ' Rate limiting
NEXT

TALK "📊 Bulk send complete!"
TALK "✅ Sent: " + sent
TALK "❌ Failed: " + failed
```

### Add Message Status Tracking

```basic
' After sending
message_id = SEND TEMPLATE TO phone, template, variables

' Store for tracking
WITH messageRecord
    id = message_id
    phone = phone
    template = template
    status = "sent"
    sentAt = NOW()
END WITH

SAVE "message_status.csv", messageRecord

' Webhook handler for status updates
ON WEBHOOK "whatsapp_status"
    status = webhook_data.status
    message_id = webhook_data.message_id
    
    UPDATE "message_status.csv" SET status = status WHERE id = message_id
    
    IF status = "delivered" THEN
        TALK "✅ Message " + message_id + " delivered"
    ELSE IF status = "read" THEN
        TALK "👀 Message " + message_id + " read"
    ELSE IF status = "failed" THEN
        TALK "❌ Message " + message_id + " failed"
    END IF
END ON
```

### Add Interactive Buttons

```basic
PARAM phone AS PHONE DESCRIPTION "Recipient phone number"

DESCRIPTION "Send message with quick reply buttons"

template_with_buttons = {
    "template": "order_confirmation",
    "buttons": [
        {"type": "quick_reply", "text": "Track Order"},
        {"type": "quick_reply", "text": "Contact Support"},
        {"type": "quick_reply", "text": "View Details"}
    ]
}

SEND TEMPLATE TO phone, template_with_buttons

TALK "Message with buttons sent to " + phone
```

### Add Media Messages

```basic
PARAM phone AS PHONE DESCRIPTION "Recipient phone number"
PARAM image_url AS STRING DESCRIPTION "URL of image to send"
PARAM caption AS STRING DESCRIPTION "Image caption" OPTIONAL

DESCRIPTION "Send WhatsApp message with image"

' Send image with caption
SEND MEDIA TO phone, image_url, caption

WITH log
    timestamp = NOW()
    phone = phone
    mediaType = "image"
    mediaUrl = image_url
    caption = caption
    status = "sent"
END WITH

SAVE "whatsapp_media_log.csv", log

TALK "📷 Image sent to " + phone
```

## WhatsApp Business API Best Practices

### Message Timing

1. **Session Messages**: Free-form messages within 24-hour window after user message
2. **Template Messages**: Pre-approved templates for initiating conversations
3. **Rate Limits**: Respect WhatsApp's messaging limits

### Template Approval

1. Submit templates via WhatsApp Business Manager
2. Wait for approval (usually 24-48 hours)
3. Use approved templates only
4. Follow content guidelines (no promotional content in utility templates)

### Phone Number Format

Always use international format without `+` or spaces:
- ✅ `5511999999999` (Brazil)
- ✅ `14155551234` (USA)
- ❌ `+55 11 99999-9999`
- ❌ `(11) 99999-9999`

### Compliance

1. **Opt-in Required**: Only message users who have opted in
2. **Opt-out Handling**: Honor STOP/unsubscribe requests immediately
3. **Business Verification**: Complete WhatsApp business verification
4. **Quality Rating**: Maintain high quality rating to avoid restrictions

## Logging Structure

The `whatsapp_log.csv` tracks all messages:

| Column | Description |
|--------|-------------|
| timestamp | When message was sent |
| phoneNumber | Recipient phone number |
| templateFile | Template used |
| variables | Personalization variables |
| status | sent/delivered/read/failed |
| messageId | WhatsApp message ID |

## Error Handling

```basic
result = SEND TEMPLATE TO phone, template, variables

IF NOT result THEN
    ' Log the failure
    WITH errorLog
        timestamp = NOW()
        phone = phone
        template = template
        error = "Send failed"
    END WITH
    
    SAVE "whatsapp_errors.csv", errorLog
    
    TALK "❌ Failed to send message to " + phone
    TALK "Please verify the phone number and try again."
    RETURN NULL
END IF
```

## Related Templates

- [broadcast.bas](./broadcast.md) - Mass messaging to contact lists
- [store.bas](./store.md) - E-commerce with WhatsApp notifications
- [bank.bas](./bank.md) - Banking notifications via WhatsApp

---

<style>
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
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>