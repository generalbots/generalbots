# CARD

Creates beautiful Instagram-style social media posts by combining AI-generated images with optimized text overlays.

## Syntax

```basic
CARD image_prompt, text_prompt TO variable
CARD image_prompt, text_prompt, style TO variable
CARD image_prompt, text_prompt, style, count TO variable
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `image_prompt` | String | Yes | Description of the image to generate |
| `text_prompt` | String | Yes | Theme or topic for the text overlay |
| `style` | String | No | Visual style preset (default: "modern") |
| `count` | Number | No | Number of cards to generate (default: 1, max: 10) |

## Available Styles

| Style | Description |
|-------|-------------|
| `modern` | Trendy Instagram aesthetic with clean design |
| `minimal` | Simple composition with negative space, muted colors |
| `vibrant` | Bold colors, high saturation, energetic feel |
| `dark` | Moody atmosphere with dramatic lighting |
| `light` | Bright, airy, soft pastel colors |
| `gradient` | Smooth color gradients, abstract backgrounds |
| `polaroid` | Vintage style with warm, nostalgic tones |
| `magazine` | Editorial, high-fashion professional look |
| `story` | Optimized for Instagram Stories (9:16 ratio) |
| `carousel` | Consistent style for multi-image posts |

## Return Value

Returns an object (or array of objects when count > 1) containing:

```json
{
  "image_path": "/path/to/generated/card.png",
  "image_url": "https://storage.example.com/card.png",
  "text_content": "The generated overlay text",
  "hashtags": ["#nature", "#inspiration", "#photography"],
  "caption": "A ready-to-use caption for the post",
  "style": "modern",
  "dimensions": [1080, 1080]
}
```

## Examples

### Basic Usage

```basic
REM Generate a single motivational post
CARD "sunrise over calm ocean", "morning motivation" TO post

TALK "Your post is ready!"
TALK "Caption: " + post.caption
TALK "Hashtags: " + JOIN(post.hashtags, " ")
```

### With Style Selection

```basic
REM Create a minimalist product showcase
CARD "elegant watch on marble surface", "luxury timepiece", "minimal" TO watch_post

REM Create an energetic fitness post  
CARD "person running at sunset", "fitness motivation", "vibrant" TO fitness_post
```

### Carousel Generation

```basic
REM Generate 5 cards for a carousel post about healthy eating
CARD "colorful healthy meals", "nutrition tips", "modern", 5 TO carousel

FOR EACH card IN carousel
    TALK "Card " + index + ": " + card.text_content
    UPLOAD card.image_path TO "instagram/carousel/"
NEXT
```

### Story Format

```basic
REM Create Instagram Story content
CARD "coffee shop interior", "monday vibes", "story" TO story_post

TALK "Story dimensions: " + story_post.dimensions[0] + "x" + story_post.dimensions[1]
```

## Sample Conversation

<div class="chat-conversation">
<div class="chat-message user">
<div class="chat-bubble">Create an Instagram post about our new summer collection</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">I'll create a beautiful post for your summer collection! Let me generate that for you...</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
✨ Your Instagram post is ready!

**Text Overlay:** "Summer Vibes Only"

**Caption:** Embrace the warmth of the season with our stunning new summer collection. ☀️

**Hashtags:** #SummerCollection #FashionVibes #SummerStyle #NewArrivals #SummerFashion #OOTD #StyleInspo

**Image:** A vibrant beach scene with flowing summer dresses

Would you like me to create more variations or adjust the style?
</div>
</div>

<div class="chat-message user">
<div class="chat-bubble">Yes, make 3 more with a minimal style for a carousel</div>
</div>

<div class="chat-message bot">
<div class="chat-bubble">
Perfect! I've created 3 additional cards with a minimal aesthetic:

📸 **Card 1:** "Less is More" - Clean white background with single garment
📸 **Card 2:** "Effortless Style" - Soft neutral tones
📸 **Card 3:** "Simply Summer" - Minimal composition with plant accents

All 4 images are now ready for your carousel post. The consistent minimal style will create a cohesive feed aesthetic.

Shall I upload them to your drafts?
</div>
</div>
</div>

## Implementation Details

The CARD keyword performs the following steps:

1. **Text Generation**: Uses LLM to create optimized overlay text based on the prompt
2. **Image Generation**: Creates the base image using AI image generation
3. **Style Application**: Applies color filters and effects based on the selected style
4. **Text Overlay**: Adds the generated text with proper positioning and shadows
5. **Social Content**: Generates relevant hashtags and a ready-to-use caption

## Image Dimensions

| Format | Dimensions | Use Case |
|--------|------------|----------|
| Square | 1080 × 1080 | Feed posts |
| Portrait | 1080 × 1350 | Feed posts (more visibility) |
| Story | 1080 × 1920 | Stories and Reels |
| Landscape | 1080 × 566 | Link previews |

## Best Practices

1. **Be Specific with Image Prompts**: "golden retriever playing in autumn leaves" works better than just "dog"

2. **Keep Text Prompts Thematic**: Focus on the message, not the exact words - the LLM will optimize

3. **Match Style to Brand**: Use consistent styles across posts for brand recognition

4. **Use Carousel for Stories**: Generate multiple related cards to create engaging carousel posts

5. **Review Hashtags**: The generated hashtags are suggestions - customize for your audience

## Error Handling

```basic
TRY
    CARD "abstract art", "creativity unleashed", "vibrant" TO art_post
    
    IF art_post.image_path = "" THEN
        TALK "Image generation failed, please try again"
    ELSE
        TALK "Post created successfully!"
    END IF
CATCH error
    TALK "Error creating card: " + error.message
END TRY
```

## Related Keywords

- [GENERATE IMAGE](./keyword-generate-image.md) - Generate images without text overlay
- [UPLOAD](./keyword-upload.md) - Upload generated cards to storage
- [POST TO SOCIAL](./keyword-post-to-social.md) - Publish directly to social media
- [CREATE DRAFT](./keyword-create-draft.md) - Save as draft for review

## See Also

- [Social Media Keywords](./keywords-social-media.md)
- [Image Processing](./keywords-image-processing.md)