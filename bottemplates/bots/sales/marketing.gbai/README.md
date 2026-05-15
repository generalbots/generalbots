# Marketing Automation Template (marketing.gbai)

A General Bots template for marketing campaign management, content creation, and multi-channel broadcast messaging.

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

## Package Structure

```
marketing.gbai/
├── README.md
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

## WhatsApp Broadcast

The `broadcast.bas` script enables mass WhatsApp messaging with template support:

### Parameters

| Parameter | Type | Description | Example |
|-----------|------|-------------|---------|
| `message` | STRING | Message with variables | `"Olá {name}, confira nossas novidades!"` |
| `template_file` | FILE | Header image for template | `header.jpg` |
| `list_file` | FILE | Contact list with phone numbers | `contacts.xlsx` |
| `filter` | STRING | Optional filter condition | `"Perfil=VIP"` |

### Usage

```basic
PARAM message AS STRING LIKE "Olá {name}, confira nossas novidades!" 
    DESCRIPTION "Message to broadcast, supports {name} and {telefone} variables"
PARAM template_file AS FILE LIKE "header.jpg" 
    DESCRIPTION "Header image file for the template"
PARAM list_file AS FILE LIKE "contacts.xlsx" 
    DESCRIPTION "File with contacts (must have telefone column)"
PARAM filter AS STRING LIKE "Perfil=VIP" 
    DESCRIPTION "Filter condition for contact list" OPTIONAL

DESCRIPTION "Send marketing broadcast message to a filtered contact list via WhatsApp template"
```

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

## Campaign Management

### Creating Campaigns

```
User: Create a new marketing campaign
Bot: What's the campaign name?
User: Black Friday 2024
Bot: What's the campaign objective?
User: Drive sales for electronics category
Bot: What's the target audience?
User: VIP customers who purchased in the last 6 months
Bot: ✅ Campaign created: Black Friday 2024
     Objective: Drive sales for electronics
     Audience: VIP customers (last 6 months)
```

### Campaign Structure

```basic
WITH campaign
    id = "CAMP-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
    name = campaign_name
    objective = objective
    audience = audience_filter
    status = "draft"
    created_at = NOW()
    created_by = GET SESSION "user_email"
END WITH

SAVE "campaigns.csv", campaign
```

## Content Ideation

### AI-Powered Ideas

```basic
' add-new-idea.bas
PARAM topic AS STRING DESCRIPTION "Topic or product for content ideas"
PARAM platform AS STRING LIKE "instagram" DESCRIPTION "Target platform"
PARAM tone AS STRING LIKE "professional" DESCRIPTION "Content tone" OPTIONAL

DESCRIPTION "Generate marketing content ideas using AI"

SET CONTEXT "You are a creative marketing specialist."

ideas = LLM "Generate 5 creative marketing content ideas for: 
             Topic: " + topic + "
             Platform: " + platform + "
             Tone: " + (tone OR "engaging") + "
             
             Include:
             - Headline
             - Key message
             - Call to action
             - Hashtag suggestions"

TALK ideas

' Save ideas for future reference
WITH idea_record
    id = FORMAT(GUID())
    topic = topic
    platform = platform
    ideas = ideas
    created_at = NOW()
END WITH

SAVE "content_ideas.csv", idea_record
```

## Image Generation

### AI Marketing Visuals

```basic
' get-image.bas
PARAM prompt AS STRING DESCRIPTION "Description of the image to generate"
PARAM style AS STRING LIKE "photorealistic" DESCRIPTION "Image style" OPTIONAL
PARAM size AS STRING LIKE "1080x1080" DESCRIPTION "Image dimensions" OPTIONAL

DESCRIPTION "Generate marketing images using AI"

full_prompt = prompt
IF style THEN
    full_prompt = full_prompt + ", " + style + " style"
END IF

image = GENERATE IMAGE full_prompt, size OR "1080x1080"

IF image THEN
    SEND FILE image, "Generated image for: " + prompt
    RETURN image
ELSE
    TALK "Failed to generate image. Please try a different prompt."
    RETURN NULL
END IF
```

## Social Media Posting

### Instagram Integration

```basic
' post-to-instagram.bas
PARAM image AS FILE DESCRIPTION "Image to post"
PARAM caption AS STRING DESCRIPTION "Post caption with hashtags"
PARAM schedule AS STRING DESCRIPTION "Schedule time (optional)" OPTIONAL

DESCRIPTION "Post content to Instagram"

' Validate caption length
IF LEN(caption) > 2200 THEN
    TALK "Caption too long. Maximum 2,200 characters."
    RETURN
END IF

IF schedule THEN
    ' Schedule for later
    SET SCHEDULE schedule, "execute-instagram-post.bas"
    SET BOT MEMORY "scheduled_post_" + FORMAT(GUID()), {
        "image": image,
        "caption": caption,
        "scheduled_for": schedule
    }
    TALK "Post scheduled for: " + schedule
ELSE
    ' Post immediately
    result = POST TO INSTAGRAM image, caption
    
    IF result.success THEN
        TALK "✅ Posted to Instagram successfully!"
        TALK "Post ID: " + result.post_id
    ELSE
        TALK "❌ Failed to post: " + result.error
    END IF
END IF
```

## Poster Creation

### Marketing Poster Generator

```basic
' poster.bas
PARAM headline AS STRING DESCRIPTION "Main headline text"
PARAM subheadline AS STRING DESCRIPTION "Supporting text" OPTIONAL
PARAM product_image AS FILE DESCRIPTION "Product image" OPTIONAL
PARAM template AS STRING LIKE "sale" DESCRIPTION "Poster template: sale, announcement, event"

DESCRIPTION "Create marketing posters with customizable templates"

' Generate poster using template
poster = GENERATE POSTER template WITH {
    "headline": headline,
    "subheadline": subheadline,
    "image": product_image,
    "brand_color": GET CONFIG "brand_color",
    "logo": GET CONFIG "logo_file"
}

SEND FILE poster, "Marketing poster: " + headline
RETURN poster
```

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

## Example Workflows

### Product Launch Campaign

```
User: Create a product launch broadcast
Bot: What product are you launching?
User: New wireless headphones
Bot: I'll help you create the campaign. Let me generate some content ideas...

     📝 Content Ideas:
     1. "Experience Sound Like Never Before 🎧"
     2. "Your Music. Your Freedom. Wireless."
     3. "Say Goodbye to Tangled Cables"
     
     Which headline do you prefer?

User: 1
Bot: Now let me generate a promotional image...
     [Generates AI image]
     
     Here's your promotional image. Should I create the broadcast?

User: Yes, send to VIP customers
Bot: ✅ Broadcast prepared:
     📱 Recipients: 234 VIP customers
     📝 Message: "Experience Sound Like Never Before 🎧"
     🖼️ Template with header image
     
     Send now or schedule?

User: Send now
Bot: 📤 Broadcasting...
     ✅ Broadcast complete! Sent to 234 contacts.
```

## Best Practices

1. **Template compliance** - Always validate templates before broadcast
2. **Segment wisely** - Target relevant audiences to improve engagement
3. **Rate limiting** - Respect platform rate limits to avoid blocks
4. **Personalization** - Use variables for personalized messages
5. **A/B testing** - Test different messages with small segments first
6. **Timing** - Schedule broadcasts for optimal engagement times
7. **Tracking** - Monitor delivery and engagement metrics
8. **Opt-out handling** - Honor unsubscribe requests immediately

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Template rejected | Non-compliant content | Review META guidelines |
| Low delivery rate | Invalid phone numbers | Clean contact list |
| Broadcast blocked | Rate limit exceeded | Reduce sending speed |
| Images not generating | Invalid prompt | Simplify prompt text |
| Instagram post failed | Auth expired | Reconnect account |

## Related Templates

- `crm.gbai` - Customer relationship management
- `contacts.gbai` - Contact list management
- `broadcast.gbai` - General message broadcasting
- `analytics.gbai` - Marketing analytics

## Use Cases

- **Product Launches** - Announce new products to customers
- **Promotions** - Send special offers and discounts
- **Events** - Promote webinars, sales, and events
- **Newsletters** - Regular customer communications
- **Re-engagement** - Win back inactive customers
- **Social Media** - Automated content posting

## Compliance Notes

- Ensure recipients have opted in to receive marketing messages
- Honor unsubscribe requests within 24 hours
- Follow META WhatsApp Business policies
- Comply with GDPR/LGPD data protection requirements
- Keep records of consent for audit purposes

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots