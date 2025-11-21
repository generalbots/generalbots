REM General Bots: GET IMAGE Keyword for Marketing Template
REM Generates images based on text descriptions using free APIs

PARAM prompt AS string LIKE "A beautiful sunset over mountains"
DESCRIPTION "Generate or fetch an image based on a text description for marketing purposes"

REM Try multiple free image sources

REM Option 1: Use Unsplash for high-quality stock photos
REM Extract keywords from prompt
keywords = REPLACE(prompt, " ", ",")
unsplash_url = "https://source.unsplash.com/1080x1080/?" + keywords

TALK "🎨 Generating image for: " + prompt
TALK "📸 Using Unsplash source..."

REM Download the image
image_file = DOWNLOAD unsplash_url

IF image_file THEN
    TALK "✅ Image generated successfully!"
    SEND FILE image_file
    RETURN image_file
END IF

REM Option 2: Fallback to Picsum (Lorem Picsum) for random images
picsum_url = "https://picsum.photos/1080/1080"
image_file = DOWNLOAD picsum_url

IF image_file THEN
    TALK "✅ Image generated successfully!"
    SEND FILE image_file
    RETURN image_file
END IF

REM Option 3: Generate a placeholder with text
placeholder_url = "https://via.placeholder.com/1080x1080/4A90E2/FFFFFF/?text=" + REPLACE(prompt, " ", "+")
image_file = DOWNLOAD placeholder_url

IF image_file THEN
    TALK "✅ Placeholder image generated!"
    SEND FILE image_file
    RETURN image_file
END IF

TALK "❌ Could not generate image"
RETURN NULL
