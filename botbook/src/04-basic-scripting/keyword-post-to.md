# POST TO

Publish content to social media platforms and messaging channels. Supports text, images, videos, and multi-platform posting.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Syntax

```bas
POST TO platform content
POST TO platform content, caption
POST TO platform image, caption
POST TO "platform1,platform2" image, caption
POST TO platform AT "datetime" content
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| platform | String | Target platform name or comma-separated list |
| content | String/File | Text content, image path, or video path |
| caption | String | Optional caption or message text |
| datetime | String | Optional scheduled time (ISO 8601 format) |

## Supported Platforms

| Platform | Identifier | Content Types |
|----------|------------|---------------|
| Instagram | `instagram` | Images, Videos, Carousels |
| Facebook | `facebook` | Text, Images, Videos, Links |
| LinkedIn | `linkedin` | Text, Images, Articles |
| Twitter/X | `twitter` | Text, Images, Videos |
| Bluesky | `bluesky` | Text, Images |
| Threads | `threads` | Text, Images |
| Discord | `discord` | Text, Images, Embeds |
| TikTok | `tiktok` | Videos |
| YouTube | `youtube` | Videos, Community Posts |
| Pinterest | `pinterest` | Images (Pins) |
| Reddit | `reddit` | Text, Links, Images |
| WeChat | `wechat` | Text, Images, Articles |
| Snapchat | `snapchat` | Images, Videos |

## Basic Usage

### Text Post

```bas
POST TO BLUESKY "Just launched our new AI assistant! 🚀"
POST TO TWITTER "Check out our latest update"
POST TO LINKEDIN "Excited to announce our partnership with..."
```

### Image Post

```bas
image = "/marketing/product-launch.jpg"
caption = "Introducing our newest feature! #AI #Innovation"

POST TO INSTAGRAM image, caption
```

### Video Post

```bas
video = "/videos/tutorial.mp4"
description = "Quick tutorial on getting started"

POST TO TIKTOK video, description
POST TO YOUTUBE video, "How to automate your workflow"
```

## Multi-Platform Posting

Post to multiple platforms simultaneously:

```bas
image = "/content/announcement.png"
caption = "Big news! We're expanding to new markets 🌍"

POST TO "instagram,facebook,linkedin,twitter" image, caption
```

### Platform-Specific Content

```bas
image = "/promo/sale.jpg"

POST TO INSTAGRAM image, "Summer sale! 50% off everything ☀️ #Sale #Summer"
POST TO LINKEDIN image, "We're offering exclusive discounts to our business partners."
POST TO TWITTER image, "FLASH SALE: 50% off for the next 24 hours! 🔥"
```

## Scheduled Posting

Schedule posts for future publication:

```bas
POST TO INSTAGRAM AT "2025-02-14 09:00" image, "Happy Valentine's Day! ❤️"
POST TO FACEBOOK AT "2025-03-01 10:00:00" image, "March is here!"
```

### Campaign Scheduling

```bas
posts = [
    #{date: "2025-02-01 09:00", caption: "Week 1: Introduction"},
    #{date: "2025-02-08 09:00", caption: "Week 2: Deep Dive"},
    #{date: "2025-02-15 09:00", caption: "Week 3: Advanced Tips"},
    #{date: "2025-02-22 09:00", caption: "Week 4: Conclusion"}
]

FOR EACH post IN posts
    POST TO "instagram,linkedin" AT post.date "/campaign/week" + i + ".png", post.caption
NEXT
```

## Platform-Specific Features

### Instagram

```bas
POST TO INSTAGRAM image, caption
POST TO INSTAGRAM video, caption
POST TO INSTAGRAM [image1, image2, image3], "Carousel post!"
```

### Facebook

```bas
POST TO FACEBOOK text
POST TO FACEBOOK image, caption
POST TO FACEBOOK link, description
```

### LinkedIn

```bas
POST TO LINKEDIN text
POST TO LINKEDIN image, caption
POST TO LINKEDIN article_url, commentary
```

### Twitter/X

```bas
POST TO TWITTER text
POST TO TWITTER image, text
POST TO TWITTER [image1, image2, image3, image4], text
```

### Bluesky

```bas
POST TO BLUESKY text
POST TO BLUESKY image, text
POST TO BLUESKY [image1, image2, image3, image4], text
```

### Threads

```bas
POST TO THREADS text
POST TO THREADS image, caption
```

### Discord

```bas
POST TO DISCORD text
POST TO DISCORD image, message

embed = #{
    title: "New Release",
    description: "Version 2.0 is here!",
    color: 5814783,
    fields: [
        #{name: "Features", value: "New dashboard, API improvements"},
        #{name: "Download", value: "https://example.com/download"}
    ]
}
POST TO DISCORD embed
```

### TikTok

```bas
video = "/videos/dance.mp4"
POST TO TIKTOK video, "Check out this tutorial! #Tutorial #HowTo"
```

### YouTube

```bas
video = "/videos/full-tutorial.mp4"
POST TO YOUTUBE video, "Complete Guide to Automation"

POST TO YOUTUBE "community", "What topics should we cover next? 🤔"
```

### Pinterest

```bas
POST TO PINTEREST image, "DIY Home Decor Ideas | Save for later!"
POST TO PINTEREST image, caption, board_name
```

### Reddit

```bas
POST TO REDDIT "r/programming", "title", "text content"
POST TO REDDIT "r/pics", "Check this out!", image
```

### WeChat

```bas
POST TO WECHAT text
POST TO WECHAT article_title, article_content, cover_image
```

### Snapchat

```bas
POST TO SNAPCHAT image
POST TO SNAPCHAT video
```

## Return Value

Returns a post identifier that can be used for metrics or deletion:

```bas
post_id = POST TO INSTAGRAM image, caption
SET BOT MEMORY "latest_instagram_post", post_id

metrics = GET INSTAGRAM METRICS post_id
TALK "Post received " + metrics.likes + " likes"
```

## Examples

### Daily Content Automation

```bas
SET SCHEDULE "every day at 10am"

day_of_week = WEEKDAY(NOW())

content = [
    #{image: "/content/monday.png", caption: "Monday motivation! 💪"},
    #{image: "/content/tuesday.png", caption: "Tech Tuesday tip 🔧"},
    #{image: "/content/wednesday.png", caption: "Midweek check-in ✅"},
    #{image: "/content/thursday.png", caption: "Throwback Thursday 📸"},
    #{image: "/content/friday.png", caption: "Friday feels! 🎉"}
]

IF day_of_week >= 1 AND day_of_week <= 5 THEN
    today = content[day_of_week - 1]
    POST TO "instagram,facebook,linkedin" today.image, today.caption
END IF
```

### Product Launch Campaign

```bas
launch_image = "/products/new-product.jpg"
launch_date = "2025-03-15 09:00"

POST TO INSTAGRAM AT launch_date launch_image, "🚀 IT'S HERE! Our most requested feature is now live. Link in bio! #Launch #NewProduct"

POST TO TWITTER AT launch_date launch_image, "🚀 The wait is over! Check out our newest feature: [link] #ProductLaunch"

POST TO LINKEDIN AT launch_date launch_image, "We're excited to announce the launch of our newest innovation. After months of development, we're proud to share..."

POST TO FACEBOOK AT launch_date launch_image, "🎉 Big announcement! We just launched something amazing. Head to our website to learn more!"

TALK "Launch campaign scheduled for " + launch_date
```

### Customer Testimonial Posts

```bas
ON FORM SUBMIT "testimonial"
    IF fields.share_permission = "yes" THEN
        quote = fields.testimonial
        name = fields.name
        company = fields.company
        
        caption = "\"" + quote + "\" - " + name + ", " + company + " ⭐️ #CustomerLove #Testimonial"
        
        template_image = FILL "testimonial-template.png", #{quote: quote, name: name, company: company}
        
        POST TO "linkedin,twitter" template_image, caption
        
        TALK "Thank you for sharing your experience!"
    END IF
END ON
```

### Engagement-Based Reposting

```bas
SET SCHEDULE "every sunday at 8pm"

posts = GET INSTAGRAM POSTS
best_post = NULL
best_engagement = 0

FOR EACH post IN posts
    IF DATEDIFF("day", post.created_at, NOW()) <= 90 THEN
        metrics = GET INSTAGRAM METRICS post.id
        engagement = metrics.likes + metrics.comments * 2
        
        IF engagement > best_engagement THEN
            best_engagement = engagement
            best_post = post
        END IF
    END IF
NEXT

IF best_post != NULL THEN
    POST TO "facebook,linkedin" best_post.image, "In case you missed it: " + best_post.caption
END IF
```

## Configuration

Add platform credentials to your bot's `config.csv`:

```csv
key,value
bluesky-handle,your.handle.bsky.social
bluesky-app-password,your-app-password
threads-access-token,your-threads-token
discord-bot-token,your-discord-token
discord-channel-id,your-channel-id
tiktok-access-token,your-tiktok-token
youtube-api-key,your-youtube-key
pinterest-access-token,your-pinterest-token
reddit-client-id,your-reddit-client-id
reddit-client-secret,your-reddit-secret
wechat-app-id,your-wechat-app-id
wechat-app-secret,your-wechat-secret
snapchat-access-token,your-snapchat-token
```

## Error Handling

```bas
ON ERROR RESUME NEXT

post_id = POST TO INSTAGRAM image, caption

IF ERROR THEN
    error_msg = ERROR MESSAGE
    SEND MAIL TO "admin@company.com" SUBJECT "Post Failed" BODY error_msg
    TALK "Sorry, I couldn't post right now. The team has been notified."
ELSE
    TALK "Posted successfully!"
    SET BOT MEMORY "last_post", post_id
END IF

CLEAR ERROR
```

## Best Practices

**Tailor content per platform.** What works on LinkedIn may not work on TikTok. Adjust tone, length, and format accordingly.

**Respect rate limits.** Platforms enforce posting limits. Space out posts and avoid bulk operations.

**Use scheduling wisely.** Analyze when your audience is most active and schedule posts accordingly.

**Store post IDs.** Save identifiers for later metrics retrieval or content management.

**Handle errors gracefully.** Network issues and API changes happen. Implement proper error handling.

**Test before campaigns.** Always test posts to a single platform before launching multi-platform campaigns.

## See Also

- [Social Media Keywords](./keywords-social-media.md) - Platform metrics and management
- [SET SCHEDULE](./keyword-set-schedule.md) - Automate posting schedules
- [FILL](./keyword-fill.md) - Generate images from templates
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Store post tracking data
- [ON ERROR](./keyword-on-error.md) - Error handling