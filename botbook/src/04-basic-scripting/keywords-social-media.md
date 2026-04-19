# Social Media Keywords

General Bots provides native social media integration through BASIC keywords for posting content, scheduling, retrieving metrics, and managing posts across multiple platforms.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Platform Support

Supported platforms include Instagram, Facebook, LinkedIn, and Twitter/X. Each platform requires appropriate API credentials configured in your bot's `config.csv`.

## POST TO

Publish content to one or more social media platforms.

### Single Platform

```basic
POST TO INSTAGRAM image, "Check out our new feature! #AI #Automation"
POST TO FACEBOOK image, caption
POST TO LINKEDIN image, caption
POST TO TWITTER image, caption
```

### Multiple Platforms

Post to several platforms simultaneously:

```basic
POST TO "instagram,facebook,linkedin" image, caption
```

The keyword returns a post ID that can be used for metrics retrieval or deletion.

### Example: Product Announcement

```basic
image = "/products/new-release.jpg"
caption = "Introducing our latest innovation! Available now. #NewProduct #Innovation"

post_id = POST TO "instagram,facebook" image, caption
SET BOT MEMORY "latest_post_id", post_id
TALK "Posted to Instagram and Facebook"
```

## POST TO ... AT (Scheduled)

Schedule posts for future publishing at a specific date and time.

```basic
POST TO INSTAGRAM AT "2025-02-01 10:00" image, caption
POST TO FACEBOOK AT "2025-02-15 09:00" image, "Coming soon!"
```

### Campaign Scheduling

```basic
' Schedule a week of posts
images = LIST "/campaign/week1/"
dates = ["2025-02-03 09:00", "2025-02-04 09:00", "2025-02-05 09:00"]

FOR i = 0 TO LEN(images) - 1
    POST TO "instagram,facebook" AT dates[i] images[i].path, captions[i]
NEXT i

TALK "Campaign scheduled: " + LEN(images) + " posts"
```

## GET METRICS

Retrieve engagement metrics for published posts.

### Platform-Specific Metrics

```basic
' Instagram metrics
metrics = GET INSTAGRAM METRICS "post-id"
TALK "Likes: " + metrics.likes + ", Comments: " + metrics.comments

' Facebook metrics
fb_metrics = GET FACEBOOK METRICS "post-id"
TALK "Shares: " + fb_metrics.shares + ", Reactions: " + fb_metrics.reactions

' LinkedIn metrics
li_metrics = GET LINKEDIN METRICS "post-id"
TALK "Impressions: " + li_metrics.impressions

' Twitter metrics
tw_metrics = GET TWITTER METRICS "post-id"
TALK "Retweets: " + tw_metrics.retweets + ", Likes: " + tw_metrics.likes
```

### Metrics Report

```basic
SET SCHEDULE "every monday at 9am"

post_id = GET BOT MEMORY "latest_post_id"
metrics = GET INSTAGRAM METRICS post_id

WITH report
    .post_id = post_id
    .likes = metrics.likes
    .comments = metrics.comments
    .reach = metrics.reach
    .engagement_rate = ROUND((metrics.likes + metrics.comments) / metrics.reach * 100, 2)
    .report_date = NOW()
END WITH

SEND MAIL TO "marketing@company.com" SUBJECT "Weekly Social Report" BODY report
```

## GET POSTS

List posts from a platform.

```basic
' Get all Instagram posts
posts = GET INSTAGRAM POSTS
FOR EACH post IN posts
    TALK post.id + ": " + post.caption
NEXT post

' Get Facebook posts
fb_posts = GET FACEBOOK POSTS
```

## DELETE POST

Remove a scheduled or published post.

```basic
DELETE POST "post-id"
TALK "Post removed"
```

### Conditional Deletion

```basic
' Delete posts with low engagement
posts = GET INSTAGRAM POSTS
FOR EACH post IN posts
    metrics = GET INSTAGRAM METRICS post.id
    IF metrics.likes < 10 AND DATEDIFF("day", post.created_at, NOW()) > 30 THEN
        DELETE POST post.id
        TALK "Deleted low-engagement post: " + post.id
    END IF
NEXT post
```

## Campaign Examples

### Welcome Campaign

```basic
ON FORM SUBMIT "signup"
    name = fields.name
    email = fields.email
    
    ' Welcome email immediately
    SEND TEMPLATE "welcome", "email", email, #{name: name}
    
    ' Schedule social proof post
    IF fields.share_permission = "yes" THEN
        caption = "Welcome to our community, " + name + "! 🎉 #NewMember #Community"
        POST TO INSTAGRAM AT DATEADD(NOW(), 1, "hour") "/templates/welcome-card.png", caption
    END IF
END ON
```

### Social Media Campaign

```basic
' social-campaign.bas
SET SCHEDULE "every day at 10am"

' Rotate through content library
content_index = GET BOT MEMORY "content_index"
IF content_index = "" THEN content_index = 0

content_library = [
    #{image: "/content/tip1.png", caption: "Pro tip: Automate your workflows! #Productivity"},
    #{image: "/content/tip2.png", caption: "Save hours every week with automation #Efficiency"},
    #{image: "/content/tip3.png", caption: "Let AI handle the repetitive tasks #AI #Automation"}
]

current = content_library[content_index MOD LEN(content_library)]
post_id = POST TO "instagram,linkedin" current.image, current.caption

SET BOT MEMORY "content_index", content_index + 1
SET BOT MEMORY "last_post_id", post_id

TALK "Posted content #" + (content_index + 1)
```

### Engagement Monitoring

```basic
SET SCHEDULE "every 6 hours"

posts = GET INSTAGRAM POSTS
total_engagement = 0
post_count = 0

FOR EACH post IN posts
    IF DATEDIFF("day", post.created_at, NOW()) <= 7 THEN
        metrics = GET INSTAGRAM METRICS post.id
        total_engagement = total_engagement + metrics.likes + metrics.comments
        post_count = post_count + 1
    END IF
NEXT post

avg_engagement = IIF(post_count > 0, ROUND(total_engagement / post_count, 0), 0)

IF avg_engagement < 50 THEN
    SEND MAIL TO "marketing@company.com" SUBJECT "Low Engagement Alert" BODY "Average engagement this week: " + avg_engagement
END IF
```

## Configuration

Add social media credentials to your bot's `config.csv`:

```csv
key,value
instagram-access-token,your-instagram-token
instagram-account-id,your-account-id
facebook-access-token,your-facebook-token
facebook-page-id,your-page-id
linkedin-access-token,your-linkedin-token
linkedin-organization-id,your-org-id
twitter-api-key,your-api-key
twitter-api-secret,your-api-secret
twitter-access-token,your-access-token
twitter-access-secret,your-access-secret
```

## Best Practices

**Schedule posts strategically.** Analyze your audience engagement patterns and post when your followers are most active.

**Use hashtags effectively.** Include relevant hashtags but avoid overloading—3 to 5 well-chosen tags typically perform better than 30 generic ones.

**Monitor metrics regularly.** Set up scheduled reports to track engagement trends and adjust your content strategy.

**Handle rate limits gracefully.** Social platforms enforce API rate limits. Space out bulk operations and implement retry logic.

**Store post IDs.** Save post identifiers in BOT MEMORY for later metrics retrieval or deletion.

```basic
post_id = POST TO INSTAGRAM image, caption
SET BOT MEMORY "post_" + FORMAT(NOW(), "yyyyMMdd"), post_id
```

## See Also

- [SET SCHEDULE](./keyword-set-schedule.md) - Automate posting schedules
- [Template Variables](./template-variables.md) - Dynamic content in captions
- [SEND TEMPLATE](./keywords.md) - Multi-channel messaging
- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Store post tracking data