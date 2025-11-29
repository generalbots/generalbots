# Multimodal Configuration Guide

This document describes how to configure botserver to use the botmodels service for image, video, audio generation, and vision/captioning capabilities.

## Overview

The multimodal feature connects botserver to botmodels - a Python-based service similar to llama.cpp but for multimodal AI tasks. This enables BASIC scripts to generate images, videos, audio, and analyze visual content.

## Configuration Keys

Add the following configuration to your bot's `config.csv` file:

### Image Generator Settings

| Key | Default | Description |
|-----|---------|-------------|
| `image-generator-model` | - | Path to the image generation model (e.g., `../../../../data/diffusion/sd_turbo_f16.gguf`) |
| `image-generator-steps` | `4` | Number of inference steps for image generation |
| `image-generator-width` | `512` | Output image width in pixels |
| `image-generator-height` | `512` | Output image height in pixels |
| `image-generator-gpu-layers` | `20` | Number of layers to offload to GPU |
| `image-generator-batch-size` | `1` | Batch size for generation |

### Video Generator Settings

| Key | Default | Description |
|-----|---------|-------------|
| `video-generator-model` | - | Path to the video generation model (e.g., `../../../../data/diffusion/zeroscope_v2_576w`) |
| `video-generator-frames` | `24` | Number of frames to generate |
| `video-generator-fps` | `8` | Frames per second for output video |
| `video-generator-width` | `320` | Output video width in pixels |
| `video-generator-height` | `576` | Output video height in pixels |
| `video-generator-gpu-layers` | `15` | Number of layers to offload to GPU |
| `video-generator-batch-size` | `1` | Batch size for generation |

### BotModels Service Settings

| Key | Default | Description |
|-----|---------|-------------|
| `botmodels-enabled` | `false` | Enable/disable botmodels integration |
| `botmodels-host` | `0.0.0.0` | Host address for botmodels service |
| `botmodels-port` | `8085` | Port for botmodels service |
| `botmodels-api-key` | - | API key for authentication with botmodels |
| `botmodels-https` | `false` | Use HTTPS for connection to botmodels |

## Example config.csv

```csv
key,value
image-generator-model,../../../../data/diffusion/sd_turbo_f16.gguf
image-generator-steps,4
image-generator-width,512
image-generator-height,512
image-generator-gpu-layers,20
image-generator-batch-size,1
video-generator-model,../../../../data/diffusion/zeroscope_v2_576w
video-generator-frames,24
video-generator-fps,8
video-generator-width,320
video-generator-height,576
video-generator-gpu-layers,15
video-generator-batch-size,1
botmodels-enabled,true
botmodels-host,0.0.0.0
botmodels-port,8085
botmodels-api-key,your-secret-key
botmodels-https,false
```

## BASIC Keywords

Once configured, the following keywords become available in BASIC scripts:

### IMAGE

Generate an image from a text prompt.

```basic
file = IMAGE "a cute cat playing with yarn"
SEND FILE TO user, file
```

### VIDEO

Generate a video from a text prompt.

```basic
file = VIDEO "a rocket launching into space"
SEND FILE TO user, file
```

### AUDIO

Generate speech audio from text.

```basic
file = AUDIO "Hello, welcome to our service!"
SEND FILE TO user, file
```

### SEE

Get a caption/description of an image or video file.

```basic
caption = SEE "/path/to/image.jpg"
TALK caption

// Also works with video files
description = SEE "/path/to/video.mp4"
TALK description
```

## Starting BotModels Service

Before using multimodal features, start the botmodels service:

```bash
cd botmodels
python -m uvicorn src.main:app --host 0.0.0.0 --port 8085
```

Or with HTTPS:

```bash
python -m uvicorn src.main:app --host 0.0.0.0 --port 8085 --ssl-keyfile key.pem --ssl-certfile cert.pem
```

## API Endpoints (BotModels)

The botmodels service exposes these REST endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/image/generate` | POST | Generate image from prompt |
| `/api/video/generate` | POST | Generate video from prompt |
| `/api/speech/generate` | POST | Generate speech from text |
| `/api/speech/totext` | POST | Convert audio to text |
| `/api/vision/describe` | POST | Get description of an image |
| `/api/vision/describe_video` | POST | Get description of a video |
| `/api/vision/vqa` | POST | Visual question answering |
| `/api/health` | GET | Health check |

All endpoints require the `X-API-Key` header for authentication.

## Architecture

```
┌─────────────┐     HTTPS      ┌─────────────┐
│  botserver  │ ────────────▶  │  botmodels  │
│   (Rust)    │                │  (Python)   │
└─────────────┘                └─────────────┘
      │                              │
      │ BASIC Keywords               │ AI Models
      │ - IMAGE                      │ - Stable Diffusion
      │ - VIDEO                      │ - Zeroscope
      │ - AUDIO                      │ - TTS/Whisper
      │ - SEE                        │ - BLIP2
      ▼                              ▼
┌─────────────┐                ┌─────────────┐
│   config    │                │   outputs   │
│   .csv      │                │  (files)    │
└─────────────┘                └─────────────┘
```

## Troubleshooting

### "BotModels is not enabled"

Set `botmodels-enabled=true` in your config.csv.

### Connection refused

1. Ensure botmodels service is running
2. Check host/port configuration
3. Verify firewall settings

### Authentication failed

Ensure `botmodels-api-key` in config.csv matches `API_KEY` environment variable in botmodels.

### Model not found

Verify model paths are correct and models are downloaded to the expected locations.

## Security Notes

1. Always use HTTPS in production (`botmodels-https=true`)
2. Use strong, unique API keys
3. Restrict network access to botmodels service
4. Consider running botmodels on a separate GPU server