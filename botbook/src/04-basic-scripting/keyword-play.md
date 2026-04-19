# PLAY

Open a content projector/player to display various media types including videos, images, documents, and presentations.

## Syntax

```basic
' Basic playback
PLAY file_or_url

' With options
PLAY file_or_url WITH OPTIONS options_string
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file_or_url` | String | Yes | Path to file or URL to display |
| `options_string` | String | No | Comma-separated playback options |

## Supported Options

| Option | Description |
|--------|-------------|
| `autoplay` | Start playback automatically |
| `loop` | Loop content continuously |
| `fullscreen` | Open in fullscreen mode |
| `muted` | Start with audio muted |
| `controls` | Show playback controls |
| `nocontrols` | Hide playback controls |

## Supported Content Types

### Video

| Extension | Format |
|-----------|--------|
| `.mp4` | MPEG-4 Video |
| `.webm` | WebM Video |
| `.ogg` | Ogg Video |
| `.mov` | QuickTime |
| `.avi` | AVI Video |
| `.mkv` | Matroska |
| `.m4v` | M4V Video |

### Audio

| Extension | Format |
|-----------|--------|
| `.mp3` | MP3 Audio |
| `.wav` | WAV Audio |
| `.flac` | FLAC Audio |
| `.aac` | AAC Audio |
| `.m4a` | M4A Audio |
| `.ogg` | Ogg Audio |

### Images

| Extension | Format |
|-----------|--------|
| `.jpg` `.jpeg` | JPEG Image |
| `.png` | PNG Image |
| `.gif` | GIF (animated) |
| `.webp` | WebP Image |
| `.svg` | SVG Vector |
| `.bmp` | Bitmap |

### Documents

| Extension | Format |
|-----------|--------|
| `.pdf` | PDF Document |
| `.docx` `.doc` | Word Document |
| `.pptx` `.ppt` | PowerPoint |
| `.xlsx` `.xls` | Excel Spreadsheet |
| `.odt` | OpenDocument Text |
| `.odp` | OpenDocument Presentation |

### Code

| Extension | Language |
|-----------|----------|
| `.rs` | Rust |
| `.py` | Python |
| `.js` `.ts` | JavaScript/TypeScript |
| `.java` | Java |
| `.go` | Go |
| `.rb` | Ruby |
| `.md` | Markdown |
| `.html` | HTML |

## Examples

### Play a Video

```basic
' Play a video file
PLAY "training-video.mp4"

' Play with autoplay and loop
PLAY "background.mp4" WITH OPTIONS "autoplay,loop,muted"

' Play from URL
PLAY "https://example.com/videos/demo.mp4"
```

### Display an Image

```basic
' Show an image
PLAY "product-photo.jpg"

' Show image fullscreen
PLAY "banner.png" WITH OPTIONS "fullscreen"
```

### Show a Presentation

```basic
' Display PowerPoint presentation
PLAY "quarterly-report.pptx"

' Fullscreen presentation mode
PLAY "sales-deck.pptx" WITH OPTIONS "fullscreen"
```

### Display a Document

```basic
' Show PDF document
PLAY "contract.pdf"

' Show Word document
PLAY "proposal.docx"
```

### Interactive Training Module

```basic
TALK "Welcome to the training module!"
TALK "Let's start with an introduction video."

PLAY "intro-video.mp4" WITH OPTIONS "controls"

HEAR ready AS TEXT "Type 'continue' when you're ready to proceed:"

IF LOWER(ready) = "continue" THEN
    TALK "Great! Now let's review the key concepts."
    PLAY "concepts-slides.pptx"
    
    HEAR understood AS TEXT "Did you understand the concepts? (yes/no)"
    
    IF LOWER(understood) = "yes" THEN
        TALK "Excellent! Here's your certificate."
        PLAY "certificate.pdf"
    ELSE
        TALK "Let's review the material again."
        PLAY "concepts-detailed.mp4"
    END IF
END IF
```

### Product Showcase

```basic
' Show product images in sequence
products = FIND "products", "featured=true"

FOR EACH product IN products
    TALK "Now showing: " + product.name
    PLAY product.image_path
    WAIT 3000  ' Wait 3 seconds between images
NEXT
```

### Code Review

```basic
' Display code for review
TALK "Let's review the implementation:"
PLAY "src/main.rs"

HEAR feedback AS TEXT "Any comments on this code?"
INSERT "code_reviews", file_path, feedback, NOW()
```

### Audio Playback

```basic
' Play audio message
TALK "Here's a voice message from your team:"
PLAY "team-message.mp3" WITH OPTIONS "controls"

' Play background music
PLAY "ambient.mp3" WITH OPTIONS "autoplay,loop,muted"
```

### Dynamic Content Display

```basic
' Display content based on file type
HEAR file_name AS TEXT "Enter the file name to display:"

file_ext = LOWER(RIGHT(file_name, 4))

IF file_ext = ".mp4" OR file_ext = "webm" THEN
    PLAY file_name WITH OPTIONS "controls,autoplay"
ELSE IF file_ext = ".pdf" THEN
    PLAY file_name
ELSE IF file_ext = ".jpg" OR file_ext = ".png" THEN
    PLAY file_name WITH OPTIONS "fullscreen"
ELSE
    TALK "Unsupported file type"
END IF
```

### Embedded Video from URL

```basic
' Play YouTube video (via embed URL)
PLAY "https://www.youtube.com/embed/dQw4w9WgXcQ"

' Play Vimeo video
PLAY "https://player.vimeo.com/video/123456789"
```

### Onboarding Flow

```basic
' Multi-step onboarding with media
TALK "Welcome to our platform! Let's get you started."

' Step 1: Welcome video
TALK "First, watch this quick introduction:"
PLAY "onboarding/welcome.mp4" WITH OPTIONS "controls"

HEAR step1_done AS TEXT "Press Enter when done..."

' Step 2: Feature overview
TALK "Here's an overview of our key features:"
PLAY "onboarding/features.pptx"

HEAR step2_done AS TEXT "Press Enter when done..."

' Step 3: Quick start guide
TALK "Finally, here's your quick start guide:"
PLAY "onboarding/quickstart.pdf"

TALK "You're all set! 🎉"
```

### Error Handling

```basic
' Check if file exists before playing
file_path = "presentation.pptx"

IF FILE_EXISTS(file_path) THEN
    PLAY file_path
ELSE
    TALK "Sorry, the file could not be found."
    TALK "Please check the file path and try again."
END IF
```

## Player Behavior

### Web Interface

When used in the web interface, PLAY opens a modal overlay with:
- Appropriate player for the content type
- Close button to dismiss
- Optional playback controls
- Fullscreen toggle

### WhatsApp/Messaging Channels

On messaging channels, PLAY sends the file directly:
- Videos/images: Sent as media messages
- Documents: Sent as file attachments
- URLs: Sent as links with preview

### Desktop Application

In the desktop app, PLAY uses the native media player or viewer appropriate for the content type.

## File Locations

Files can be referenced from:

| Location | Example |
|----------|---------|
| Bot's .gbdrive | `documents/report.pdf` |
| User's folder | `users/john@email.com/uploads/photo.jpg` |
| Absolute URL | `https://cdn.example.com/video.mp4` |
| Relative path | `./assets/logo.png` |

## Limitations

- Maximum file size depends on channel (WhatsApp: 16MB for media, 100MB for documents)
- Some formats may require conversion for web playback
- Streaming large files requires adequate bandwidth
- Protected/DRM content is not supported

## See Also

- [SEND FILE](./keyword-send-mail.md) - Send files as attachments
- [TALK](./keyword-talk.md) - Display text messages
- [UPLOAD](./keyword-upload.md) - Upload files to storage
- [DOWNLOAD](./keyword-download.md) - Download files from URLs

## Implementation

The PLAY keyword is implemented in `src/basic/keywords/play.rs` with content type detection and appropriate player selection for each media format.