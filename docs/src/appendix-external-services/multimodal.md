# Multimodal Module

Image, video, and audio generation with vision/captioning capabilities.

## Overview

The multimodal module connects to BotModels server for AI-powered media generation and analysis.

## BASIC Keywords

| Keyword | Purpose |
|---------|---------|
| `IMAGE` | Generate image from text prompt |
| `VIDEO` | Generate video from text prompt |
| `AUDIO` | Generate speech audio from text |
| `SEE` | Describe/caption an image or video |

## IMAGE

Generate an image from a text prompt:

```basic
url = IMAGE "A sunset over mountains with a lake"
TALK "Here's your image: " + url
```

Timeout: 300 seconds (5 minutes)

## VIDEO

Generate a video from a text prompt:

```basic
url = VIDEO "A cat playing with a ball of yarn"
TALK "Here's your video: " + url
```

Timeout: 600 seconds (10 minutes)

## AUDIO

Generate speech audio from text:

```basic
url = AUDIO "Welcome to our service. How can I help you today?"
PLAY url
```

## SEE

Get a description of an image or video:

```basic
description = SEE "path/to/image.jpg"
TALK "I see: " + description
```

## Configuration

Add to `config.csv`:

```csv
botmodels-enabled,true
botmodels-host,localhost
botmodels-port,5000
botmodels-api-key,your-api-key
botmodels-use-https,false
```

### Image Generation Config

```csv
botmodels-image-model,stable-diffusion
botmodels-image-steps,20
botmodels-image-width,512
botmodels-image-height,512
```

### Video Generation Config

```csv
botmodels-video-model,text2video
botmodels-video-frames,16
botmodels-video-fps,8
```

## BotModels Client

Rust API for direct integration:

```rust
let client = BotModelsClient::from_state(&state, &bot_id);

if client.is_enabled() {
    let image_url = client.generate_image("A beautiful garden").await?;
    let description = client.describe_image("path/to/photo.jpg").await?;
}
```

### Available Methods

| Method | Description |
|--------|-------------|
| `generate_image(prompt)` | Create image from text |
| `generate_video(prompt)` | Create video from text |
| `generate_audio(text)` | Create speech audio |
| `describe_image(path)` | Get image caption |
| `describe_video(path)` | Get video description |
| `speech_to_text(audio_path)` | Transcribe audio |
| `health_check()` | Check BotModels server status |

## Response Structures

### GenerationResponse

```json
{
    "status": "success",
    "file_path": "/path/to/generated/file.png",
    "generation_time": 12.5,
    "error": null
}
```

### DescribeResponse

```json
{
    "description": "A golden retriever playing fetch in a park",
    "confidence": 0.92
}
```

## Requirements

- BotModels server running (separate service)
- GPU recommended for generation tasks
- Sufficient disk space for generated media

## See Also

- [NVIDIA Module](./nvidia.md) - GPU monitoring
- [PLAY Keyword](../chapter-06-gbdialog/keyword-play.md) - Play generated audio