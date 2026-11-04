# Multimodal Module 🟡 BETA

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
botmodels-port,8085
botmodels-api-key,your-api-key
botmodels-https,false
```

### Image Generation Config

```csv
image-generator-model,
image-generator-steps,4
image-generator-width,512
image-generator-height,512
```

### Video Generation Config

```csv
video-generator-model,
video-generator-frames,24
video-generator-fps,8
video-generator-width,320
video-generator-height,576
```

## BotModels Client

Rust API for direct integration:

```rust
let provider = MyConfigProvider::new(&bot_id);
let client = botmultimodal::BotModelsClient::from_provider_all(&provider, &bot_id);

if client.is_enabled() {
    let image_path = client.generate_image("A beautiful garden").await?;
    let description = client.describe_image("path/to/photo.jpg").await?;
}
```

`from_provider_all` reads the `botmodels-*` keys, then the image and video
generator settings, through the `ConfigProvider` trait. The client is also
available to BASIC keywords through `botbasic_ai`.

### Available Methods

| Method | Description |
|--------|-------------|
| `generate_image(prompt)` | Create image from text |
| `generate_video(prompt)` | Create video from text |
| `generate_audio(text)` | Create speech audio |
| `describe_image(path)` | Get image caption |
| `describe_video(path)` | Sample frames and get a video description |
| `speech_to_text(audio_path)` | Transcribe audio |

## Response Structures

### GenerationResponse

```json
{
    "status": "completed",
    "file_path": "/outputs/images/20261104_120000_ab12cd34.png",
    "generation_time": 12.5,
    "error": null,
    "timestamp": "2026-11-04T12:00:00"
}
```

### DescribeResponse

```json
{
    "description": "A golden retriever playing fetch in a park",
    "confidence": null,
    "generation_time": 5.6
}
```

The captioner does not emit a calibrated confidence, so the field is `null`
rather than a fabricated constant.

## Requirements

- BotModels process running as a local sidecar (default port 8085)
- A model tier that fits the host: `min` runs on CPU; `max` needs a GPU
- Sufficient disk space for generated media and model caches

## See Also

- [NVIDIA Module](./nvidia.md) - GPU monitoring
- [PLAY Keyword](../04-basic-scripting/keyword-play.md) - Play generated audio