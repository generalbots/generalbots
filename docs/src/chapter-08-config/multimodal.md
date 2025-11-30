# Multimodal Configuration

General Bots integrates with botmodels—a Python service for multimodal AI tasks—to enable image generation, video creation, audio synthesis, and vision capabilities directly from BASIC scripts.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

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
```

When a BASIC script calls a multimodal keyword, botserver forwards the request to botmodels, which runs the appropriate AI model and returns the generated content.

## Configuration

Add these settings to your bot's `config.csv` file to enable multimodal capabilities.

### BotModels Service

| Key | Default | Description |
|-----|---------|-------------|
| `botmodels-enabled` | `false` | Enable botmodels integration |
| `botmodels-host` | `0.0.0.0` | Host address for botmodels service |
| `botmodels-port` | `8085` | Port for botmodels service |
| `botmodels-api-key` | — | API key for authentication |
| `botmodels-https` | `false` | Use HTTPS for connection |

### Image Generation

| Key | Default | Description |
|-----|---------|-------------|
| `image-generator-model` | — | Path to image generation model |
| `image-generator-steps` | `4` | Inference steps (more = higher quality, slower) |
| `image-generator-width` | `512` | Output image width in pixels |
| `image-generator-height` | `512` | Output image height in pixels |
| `image-generator-gpu-layers` | `20` | Layers to offload to GPU |
| `image-generator-batch-size` | `1` | Batch size for generation |

### Video Generation

| Key | Default | Description |
|-----|---------|-------------|
| `video-generator-model` | — | Path to video generation model |
| `video-generator-frames` | `24` | Number of frames to generate |
| `video-generator-fps` | `8` | Output frames per second |
| `video-generator-width` | `320` | Output video width in pixels |
| `video-generator-height` | `576` | Output video height in pixels |
| `video-generator-gpu-layers` | `15` | Layers to offload to GPU |
| `video-generator-batch-size` | `1` | Batch size for generation |

## Example Configuration

```csv
key,value
botmodels-enabled,true
botmodels-host,0.0.0.0
botmodels-port,8085
botmodels-api-key,your-secret-key
botmodels-https,false
image-generator-model,../../../../data/diffusion/sd_turbo_f16.gguf
image-generator-steps,4
image-generator-width,512
image-generator-height,512
image-generator-gpu-layers,20
video-generator-model,../../../../data/diffusion/zeroscope_v2_576w
video-generator-frames,24
video-generator-fps,8
```

## BASIC Keywords

Once configured, these keywords become available in your scripts.

### IMAGE

Generate an image from a text prompt:

```basic
file = IMAGE "a sunset over mountains with purple clouds"
SEND FILE TO user, file
```

The keyword returns a path to the generated image file.

### VIDEO

Generate a video from a text prompt:

```basic
file = VIDEO "a rocket launching into space"
SEND FILE TO user, file
```

Video generation is more resource-intensive than image generation. Expect longer processing times.

### AUDIO

Generate speech audio from text:

```basic
file = AUDIO "Hello, welcome to our service!"
SEND FILE TO user, file
```

### SEE

Analyze an image or video and get a description:

```basic
' Describe an image
caption = SEE "/path/to/image.jpg"
TALK caption

' Describe a video
description = SEE "/path/to/video.mp4"
TALK description
```

The SEE keyword uses vision models to understand visual content and return natural language descriptions.

## Starting BotModels

Before using multimodal features, start the botmodels service:

```bash
cd botmodels
python -m uvicorn src.main:app --host 0.0.0.0 --port 8085
```

For production with HTTPS:

```bash
python -m uvicorn src.main:app \
    --host 0.0.0.0 \
    --port 8085 \
    --ssl-keyfile key.pem \
    --ssl-certfile cert.pem
```

## BotModels API Endpoints

The botmodels service exposes these REST endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/image/generate` | POST | Generate image from prompt |
| `/api/video/generate` | POST | Generate video from prompt |
| `/api/speech/generate` | POST | Generate speech from text |
| `/api/speech/totext` | POST | Transcribe audio to text |
| `/api/vision/describe` | POST | Describe an image |
| `/api/vision/describe_video` | POST | Describe a video |
| `/api/vision/vqa` | POST | Visual question answering |
| `/api/health` | GET | Health check |

All endpoints except `/api/health` require the `X-API-Key` header for authentication.

## Model Paths

Configure model paths relative to the botmodels service directory. Typical layout:

```
data/
├── diffusion/
│   ├── sd_turbo_f16.gguf          # Stable Diffusion
│   └── zeroscope_v2_576w/         # Zeroscope video
├── tts/
│   └── model.onnx                 # Text-to-speech
├── whisper/
│   └── model.bin                  # Speech-to-text
└── vision/
    └── blip2/                     # Vision model
```

## GPU Acceleration

Both image and video generation benefit significantly from GPU acceleration. Configure GPU layers based on your hardware:

| GPU VRAM | Recommended GPU Layers |
|----------|----------------------|
| 4GB | 8-12 |
| 8GB | 15-20 |
| 12GB+ | 25-35 |

Lower GPU layers if you experience out-of-memory errors.

## Troubleshooting

**"BotModels is not enabled"**

Set `botmodels-enabled=true` in your config.csv.

**Connection refused**

Verify botmodels service is running and check host/port configuration. Test connectivity:

```bash
curl http://localhost:8085/api/health
```

**Authentication failed**

Ensure `botmodels-api-key` in config.csv matches the `API_KEY` environment variable in botmodels.

**Model not found**

Verify model paths are correct and models are downloaded to the expected locations.

**Out of memory**

Reduce `gpu-layers` or `batch-size`. Video generation is particularly memory-intensive.

## Security Considerations

**Use HTTPS in production.** Set `botmodels-https=true` and configure SSL certificates on the botmodels service.

**Use strong API keys.** Generate cryptographically random keys for the `botmodels-api-key` setting.

**Restrict network access.** Limit botmodels service access to trusted hosts only.

**Consider GPU isolation.** Run botmodels on a dedicated GPU server if sharing resources with other services.

## Performance Tips

**Image generation** runs fastest with SD Turbo models and 4-8 inference steps. More steps improve quality but increase generation time linearly.

**Video generation** is the most resource-intensive operation. Keep frame counts low (24-48) for reasonable response times.

**Batch processing** improves throughput when generating multiple items. Increase `batch-size` if you have sufficient GPU memory.

**Caching** generated content when appropriate. If multiple users request similar content, consider storing results.

## See Also

- [LLM Configuration](./llm-config.md) - Language model settings
- [Bot Parameters](./parameters.md) - All configuration options
- [IMAGE Keyword](../chapter-06-gbdialog/keywords.md) - Image generation reference
- [SEE Keyword](../chapter-06-gbdialog/keywords.md) - Vision capabilities