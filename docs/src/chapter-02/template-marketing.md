# Marketing Automation Template (marketing.gbai)

A General Bots template for marketing campaign management, content creation, and multi-channel broadcast messaging.

---

## Overview

The Marketing template provides marketing automation capabilities including campaign management, content ideation, image generation, social media posting, and WhatsApp broadcast messaging. It enables marketing teams to create, schedule, and deliver campaigns through conversational AI.

## Features

- **Campaign Management** - Create and organize marketing campaigns
- **Content Ideation** - AI-assisted content idea generation
- **Image Generation** - AI-powered marketing visuals
- **Social Media Posting** - Direct posting to Instagram and other platforms
- **WhatsApp Broadcasts** - Mass messaging with template support
- **Contact Segmentation** - Target specific audience segments
- **Template Compliance** - META-approved template validation
- **Broadcast Logging** - Track delivery and engagement

---

## Package Structure

```
marketing.gbai/
├── marketing.gbdialog/
│   ├── add-new-idea.bas       # Content ideation tool
│   ├── broadcast.bas          # WhatsApp broadcast messaging
│   ├── get-image.bas          # AI image generation
│   ├── post-to-instagram.bas  # Instagram posting
│   ├── poster.bas             # Marketing poster creation
│   └── campaigns/             # Campaign templates
└── marketing.gbot/
    └── config.csv             # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `add-new-idea.bas` | Generate and save marketing content ideas |
| `broadcast.bas` | Send WhatsApp broadcasts to contact lists |
| `get-image.bas` | Generate marketing images with AI |
| `post-to-instagram.bas` | Post content to Instagram |
| `poster.bas` | Create marketing posters and visuals |

---

## WhatsApp Broadcast

The `broadcast.bas` script enables mass WhatsApp messaging with template support:

### Parameters

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `message` | STRING | Message with variables | `"Olá {name}, confira nossas novidades!"` |
| `template_file` | FILE | Header image for template | `header.jpg` |
| `list_file` | FILE | Contact list with phone numbers | `contacts.xlsx` |
| `filter` | STRING | Optional filter condition | `"Perfil=VIP"` |

### Template Compliance

The system validates messages for META WhatsApp template approval:

```basic
report = LLM "Esta mensagem será aprovada pelo WhatsApp META como Template? 
              Responda OK se sim, ou explique o problema: " + message

IF report <> "OK" THEN
    TALK "Atenção: " + report
END IF
```

### Broadcast Execution

```basic
IF filter THEN
    list = FIND list_file, filter
ELSE
    list = FIND list_file
END IF

index = 1
sent = 0

DO WHILE index < UBOUND(list)
    row = list[index]
    
    SEND TEMPLATE TO row.telefone, template_file
    
    WAIT 0.1  ' Rate limiting
    
    ' Log delivery
    WITH logEntry
        timestamp = NOW()
        phone = row.telefone
        name = row.name
        status = "sent"
    END WITH
    
    SAVE "broadcast_log.csv", logEntry
    
    sent = sent + 1
    index = index + 1
LOOP

TALK "Broadcast enviado para " + sent + " contatos."
```

---

## Contact List Format

The contact file should include:

| Column | Required | Description |
|--------|----------|-------------|
| `telefone` | Yes | Phone number with country code |
| `name` | No | Contact name for personalization |
| `email` | No | Email address |
| `Perfil` | No | Segment tag for filtering |

### Example contacts.xlsx

```csv
telefone,name,email,Perfil
+5511999999999,João Silva,joao@email.com,VIP
+5511888888888,Maria Santos,maria@email.com,Regular
+5511777777777,Pedro Costa,pedro@email.com,VIP
```

---

## Sample Conversations

### Creating a Broadcast

```
User: Send a broadcast to VIP customers

Bot:  I'll help you create a broadcast. What's the message?

User: Olá {name}, aproveite 20% de desconto esta semana!

Bot:  Checking template compliance...
      ✅ Message approved for WhatsApp template.
      
      Do you have a header image?

User: Yes, use promo-banner.jpg

Bot:  Which contact list should I use?

User: contacts.xlsx

Bot:  📤 Broadcast Preview:
      
      Message: Olá {name}, aproveite 20% de desconto esta semana!
      Image: promo-banner.jpg
      Recipients: 127 VIP customers
      
      Send now or schedule?

User: Send now

Bot:  📤 Broadcasting...
      ✅ Broadcast complete! Sent to 127 contacts.
      
      View delivery report in Analytics.
```

### Content Ideation

```
User: I need content ideas for our new product launch

Bot:  What product are you launching?

User: Wireless headphones

Bot:  What platform are you targeting?

User: Instagram

Bot:  Here are 5 content ideas for your wireless headphones launch:

      1. "Experience Sound Like Never Before 🎧"
         Key message: Premium audio quality
         CTA: Shop now, link in bio
         Hashtags: #WirelessFreedom #AudioExperience #NewLaunch
      
      2. "Cut the Cord, Not the Quality"
         Key message: Freedom without compromise
         CTA: Pre-order today
         Hashtags: #WirelessAudio #TechLife #MusicLovers
      
      3. "Your Music. Your Freedom."
         Key message: Lifestyle integration
         CTA: Discover more
         Hashtags: #OnTheGo #Wireless #Premium
      
      4. "30 Hours of Pure Sound"
         Key message: Battery life
         CTA: Learn more
         Hashtags: #AllDayBattery #WirelessHeadphones
      
      5. "Studio Sound, Everywhere"
         Key message: Professional quality
         CTA: Experience now
         Hashtags: #StudioQuality #Audiophile #TechGear
      
      Would you like me to generate images for any of these?
```

### Image Generation

```
User: Generate an image for idea 1

Bot:  Generating marketing image...
      
      Prompt: "Premium wireless headphones floating in space with 
      sound waves emanating, modern minimalist style, product 
      photography, dark background with purple accent lighting"
      
      [Generated Image]
      
      ✅ Image generated! Would you like to:
      1. Post to Instagram
      2. Use in a broadcast
      3. Generate variations
      4. Download

User: Post to Instagram

Bot:  What caption should I use?

User: Experience Sound Like Never Before 🎧 #WirelessFreedom #NewLaunch

Bot:  ✅ Posted to Instagram!
      
      Post ID: 12345678
      Time: May 15, 2025 at 2:30 PM
      
      Track engagement in Analytics.
```

---

## Audience Segmentation

### Filter Examples

| Filter | Description |
|--------|-------------|
| `Perfil=VIP` | VIP customers only |
| `Perfil=Regular AND compras>5` | Regular customers with 5+ purchases |
| `ultima_compra>2024-01-01` | Recent purchasers |
| `cidade=São Paulo` | Location-based |
| `interesse=electronics` | Interest-based |

### Building Segments

```basic
' Create reusable segments
WITH segment
    id = "SEG-" + FORMAT(GUID())
    name = "VIP São Paulo"
    filter = "Perfil=VIP AND cidade=São Paulo"
    count = COUNT(FIND "contacts.xlsx", filter)
    created_at = NOW()
END WITH

SAVE "segments.csv", segment
```

---

## Configuration

Configure in `marketing.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Brand Color` | Primary brand color | `#FF5722` |
| `Logo File` | Company logo path | `logo.png` |
| `Instagram Account` | Connected IG account | `@mycompany` |
| `WhatsApp Business ID` | WA Business account | `123456789` |
| `Default Template` | Default broadcast template | `marketing_update` |
| `Rate Limit` | Messages per second | `10` |
| `Max Broadcast Size` | Maximum recipients | `1000` |

---

## Analytics & Reporting

### Broadcast Analytics

```basic
' Get broadcast statistics
broadcast_id = "BROADCAST-20240115-1234"
logs = FIND "broadcast_log.csv", "broadcast_id = '" + broadcast_id + "'"

total_sent = COUNT(logs)
delivered = COUNT(FIND logs, "status = 'delivered'")
read = COUNT(FIND logs, "status = 'read'")
clicked = COUNT(FIND logs, "status = 'clicked'")

TALK "📊 Broadcast Report"
TALK "Total Sent: " + total_sent
TALK "Delivered: " + delivered + " (" + (delivered/total_sent*100) + "%)"
TALK "Read: " + read + " (" + (read/total_sent*100) + "%)"
TALK "Clicked: " + clicked + " (" + (clicked/total_sent*100) + "%)"
```

---

## Customization

### Adding Campaign Types

```basic
' campaign-email.bas
PARAM subject AS STRING DESCRIPTION "Email subject line"
PARAM body AS STRING DESCRIPTION "Email body content"
PARAM list_file AS FILE DESCRIPTION "Contact list"
PARAM filter AS STRING DESCRIPTION "Segment filter" OPTIONAL

DESCRIPTION "Send email marketing campaign"

IF filter THEN
    contacts = FIND list_file, filter
ELSE
    contacts = FIND list_file
END IF

FOR EACH contact IN contacts
    personalized_body = REPLACE(body, "{name}", contact.name)
    SEND EMAIL contact.email, subject, personalized_body
    
    WITH log
        campaign_id = campaign_id
        contact_email = contact.email
        sent_at = NOW()
        status = "sent"
    END WITH
    
    SAVE "email_campaign_log.csv", log
NEXT

TALK "Email campaign sent to " + UBOUND(contacts) + " recipients."
```

### Social Media Scheduling

```basic
' schedule-post.bas
PARAM platform AS STRING LIKE "instagram" DESCRIPTION "Social platform"
PARAM content AS STRING DESCRIPTION "Post content"
PARAM image AS FILE DESCRIPTION "Post image" OPTIONAL
PARAM schedule_time AS STRING DESCRIPTION "When to post"

DESCRIPTION "Schedule social media post"

WITH scheduled_post
    id = "POST-" + FORMAT(GUID())
    platform = platform
    content = content
    image = image
    scheduled_for = schedule_time
    status = "scheduled"
    created_at = NOW()
END WITH

SAVE "scheduled_posts.csv", scheduled_post

SET SCHEDULE schedule_time, "execute-scheduled-post.bas"

TALK "Post scheduled for " + schedule_time + " on " + platform
```

---

## Best Practices

1. **Template compliance** - Always validate templates before broadcast
2. **Segment wisely** - Target relevant audiences to improve engagement
3. **Rate limiting** - Respect platform rate limits to avoid blocks
4. **Personalization** - Use variables for personalized messages
5. **A/B testing** - Test different messages with small segments first
6. **Timing** - Schedule broadcasts for optimal engagement times
7. **Tracking** - Monitor delivery and engagement metrics
8. **Opt-out handling** - Honor unsubscribe requests immediately

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Template rejected | Non-compliant content | Review META guidelines |
| Low delivery rate | Invalid phone numbers | Clean contact list |
| Broadcast blocked | Rate limit exceeded | Reduce sending speed |
| Images not generating | Invalid prompt | Simplify prompt text |
| Instagram post failed | Auth expired | Reconnect account |

---

## Compliance Notes

- Ensure recipients have opted in to receive marketing messages
- Honor unsubscribe requests within 24 hours
- Follow META WhatsApp Business policies
- Comply with GDPR/LGPD data protection requirements
- Keep records of consent for audit purposes

---

## Use Cases

- **Product Launches** - Announce new products to customers
- **Promotions** - Send special offers and discounts
- **Events** - Promote webinars, sales, and events
- **Newsletters** - Regular customer communications
- **Re-engagement** - Win back inactive customers
- **Social Media** - Automated content posting

---

## Related Templates

- [CRM](./template-crm.md) - Customer relationship management
- [Contacts](./template-crm-contacts.md) - Contact list management
- [Broadcast](./template-broadcast.md) - General message broadcasting
- [Analytics](./template-analytics.md) - Marketing analytics

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide