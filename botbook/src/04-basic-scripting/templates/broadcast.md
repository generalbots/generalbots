# Broadcast Template

The broadcast template enables mass messaging to contact lists, perfect for announcements, marketing campaigns, and bulk notifications through WhatsApp and other channels.

## Topic: Mass Messaging & Announcements

This template is perfect for:
- Company-wide announcements
- Marketing campaigns
- Customer notifications
- Event reminders
- Newsletter distribution

## The Code

```basic
PARAM message AS STRING LIKE "Hello {name}, how are you?" DESCRIPTION "Message to broadcast, supports {name} and {mobile} variables"
PARAM listfile AS STRING LIKE "broadcast.csv" DESCRIPTION "CSV file with contacts (name, mobile columns)"
PARAM filter AS STRING LIKE "status=active" DESCRIPTION "Filter condition for contact list" OPTIONAL

DESCRIPTION "Send broadcast message to a list of contacts from CSV file"

IF NOT listfile THEN
    listfile = "broadcast.csv"
END IF

IF filter THEN
    list = FIND listfile, filter
ELSE
    list = FIND listfile
END IF

IF UBOUND(list) = 0 THEN
    TALK "No contacts found in " + listfile
    RETURN 0
END IF

index = 1
sent = 0

DO WHILE index < UBOUND(list)
    row = list[index]

    msg = REPLACE(message, "{name}", row.name)
    msg = REPLACE(msg, "{mobile}", row.mobile)

    TALK TO row.mobile, msg
    WAIT 5

    WITH logEntry
        timestamp = NOW()
        user = USERNAME
        from = FROM
        mobile = row.mobile
        name = row.name
        status = "sent"
    END WITH

    SAVE "Log.xlsx", logEntry

    sent = sent + 1
    index = index + 1
LOOP

TALK "Broadcast sent to " + sent + " contacts."

RETURN sent
```

## Sample Dialogs

These conversations show how the broadcast template works in real-world scenarios.

### Dialog 1: Simple Broadcast

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Broadcast Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Send a broadcast to all customers</p>
      <div class="wa-time">10:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Broadcast Bot</div>
      <p>📢 <strong>Broadcast Setup</strong></p>
      <p>What message would you like to send?</p>
      <p>You can use these variables:</p>
      <p>• <code>{name}</code> - Contact's name</p>
      <p>• <code>{mobile}</code> - Contact's phone</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Hello {name}, we have exciting news! Our new product line is now available. Check it out at our store!</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Preview:</strong></p>
      <p>"Hello John, we have exciting news! Our new product line is now available. Check it out at our store!"</p>
      <p>📊 <strong>Recipients:</strong> 156 contacts</p>
      <p>Type <strong>SEND</strong> to start broadcast.</p>
      <div class="wa-time">10:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>SEND</p>
      <div class="wa-time">10:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚀 Broadcasting started...</p>
      <p>Progress: 0/156</p>
      <div class="wa-time">10:02</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Broadcast Complete!</strong></p>
      <p>📤 Sent: 154 messages</p>
      <p>❌ Failed: 2 messages</p>
      <p>📋 Log saved to Log.xlsx</p>
      <div class="wa-time">10:15</div>
    </div>
  </div>
</div>

### Dialog 2: Filtered Broadcast

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Broadcast Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Send message only to VIP customers</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎯 <strong>Filtered Broadcast</strong></p>
      <p>I found 42 VIP customers.</p>
      <p>What message would you like to send to them?</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Dear {name}, as a valued VIP customer, you're invited to our exclusive preview sale this weekend. Use code VIP20 for 20% off!</p>
      <div class="wa-time">14:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Preview:</strong></p>
      <p>"Dear Maria, as a valued VIP customer, you're invited to our exclusive preview sale this weekend. Use code VIP20 for 20% off!"</p>
      <p>📊 <strong>Recipients:</strong> 42 VIP contacts</p>
      <p>Type <strong>SEND</strong> to start broadcast.</p>
      <div class="wa-time">14:31</div>
    </div>
  </div>
</div>

### Dialog 3: No Contacts Found

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Broadcast Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Broadcast to inactive users</p>
      <div class="wa-time">16:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>⚠️ No contacts found in broadcast.csv matching filter "status=inactive"</p>
      <p>Would you like to:</p>
      <p>1. Try a different filter</p>
      <p>2. Send to all contacts</p>
      <p>3. Upload a new contact list</p>
      <div class="wa-time">16:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `PARAM` | Define input parameters with descriptions |
| `DESCRIPTION` | Tool description for AI |
| `FIND` | Query contacts from CSV file |
| `REPLACE` | Substitute variables in message template |
| `TALK TO` | Send message to specific phone number |
| `WAIT` | Delay between messages (rate limiting) |
| `SAVE` | Log each message to spreadsheet |
| `RETURN` | Return count of sent messages |

## How It Works

1. **Load Contacts**: `FIND` retrieves contacts from CSV with optional filter
2. **Validate List**: Checks if contacts were found
3. **Loop Through Contacts**: Iterates through each contact
4. **Personalize Message**: `REPLACE` substitutes {name} and {mobile}
5. **Send Message**: `TALK TO` delivers to each phone number
6. **Rate Limiting**: `WAIT 5` pauses 5 seconds between messages
7. **Log Operation**: Each send is recorded in Log.xlsx
8. **Report Results**: Returns total messages sent

## Contact List Format

Your CSV file should have these columns:

```csv
name,mobile,status,segment
John Smith,+5511999999999,active,regular
Maria Garcia,+5521888888888,active,vip
Carlos Santos,+5531777777777,inactive,regular
Ana Lima,+5541666666666,active,vip
```

| Column | Required | Description |
|--------|----------|-------------|
| `name` | Yes | Contact's display name |
| `mobile` | Yes | Phone in international format |
| `status` | No | For filtering (active/inactive) |
| `segment` | No | For targeting (vip/regular) |

## Customization Ideas

### Add Message Templates

```basic
ADD TOOL "broadcast"
ADD TOOL "list-templates"
ADD TOOL "create-template"

' Load saved templates
templates = FIND "message_templates.csv"

TALK "Available templates:"
FOR EACH template IN templates
    TALK "• " + template.name + ": " + LEFT(template.message, 50) + "..."
NEXT

TALK "Which template would you like to use?"
HEAR templateName

selected = FIND "message_templates.csv", "name = '" + templateName + "'"
message = selected.message
```

### Add Scheduling

```basic
PARAM schedule_time AS STRING LIKE "2025-01-20 09:00" DESCRIPTION "When to send (optional)"

IF schedule_time THEN
    SET SCHEDULE schedule_time
    
    ' Store broadcast details for later
    SET BOT MEMORY "scheduled_message", message
    SET BOT MEMORY "scheduled_list", listfile
    SET BOT MEMORY "scheduled_filter", filter
    
    TALK "📅 Broadcast scheduled for " + schedule_time
    TALK "I'll send to " + UBOUND(list) + " contacts at that time."
    RETURN 0
END IF
```

### Add Progress Updates

```basic
total = UBOUND(list)
checkpoints = [25, 50, 75, 100]

DO WHILE index <= total
    ' ... send message ...
    
    ' Check progress
    percent = INT((index / total) * 100)
    IF INARRAY(percent, checkpoints) THEN
        TALK "📊 Progress: " + percent + "% (" + index + "/" + total + ")"
    END IF
    
    index = index + 1
LOOP
```

### Add Opt-Out Handling

```basic
' Check if contact has opted out
optouts = FIND "optouts.csv"

DO WHILE index <= UBOUND(list)
    row = list[index]
    
    ' Skip opted-out contacts
    IF FIND("optouts.csv", "mobile = '" + row.mobile + "'") THEN
        WITH logEntry
            mobile = row.mobile
            status = "skipped-optout"
        END WITH
        SAVE "Log.xlsx", logEntry
        index = index + 1
        CONTINUE
    END IF
    
    ' ... send message ...
LOOP
```

### Add Media Support

```basic
PARAM image AS STRING LIKE "promo.jpg" DESCRIPTION "Image to include (optional)"

IF image THEN
    msg = msg + "\n[Image: " + image + "]"
    TALK TO row.mobile, msg, image
ELSE
    TALK TO row.mobile, msg
END IF
```

## Best Practices

### Message Content

1. **Personalize**: Always use `{name}` for a personal touch
2. **Be Concise**: Keep messages short and clear
3. **Clear CTA**: Include a clear call-to-action
4. **Identify Yourself**: Make sure recipients know who's messaging

### Compliance

1. **Consent Required**: Only message contacts who opted in
2. **Easy Opt-Out**: Include unsubscribe instructions
3. **Respect Hours**: Don't send late at night
4. **Honor Limits**: WhatsApp has daily messaging limits

### Performance

1. **Rate Limiting**: Keep 5+ second delays to avoid blocks
2. **Batch Processing**: For large lists, consider batching
3. **Error Handling**: Log and handle failed sends
4. **Monitor Results**: Check logs for delivery issues

## Logging Structure

The Log.xlsx file tracks all broadcast activity:

| Column | Description |
|--------|-------------|
| timestamp | When message was sent |
| user | Who initiated the broadcast |
| from | Sender identifier |
| mobile | Recipient phone number |
| name | Recipient name |
| status | sent/failed/skipped |
| error | Error message if failed |

## Related Templates

- [announcements.bas](./announcements.md) - Company announcements system
- [whatsapp.bas](./whatsapp.md) - WhatsApp-specific features
- [store.bas](./store.md) - E-commerce with customer notifications

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