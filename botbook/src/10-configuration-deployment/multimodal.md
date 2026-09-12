# Multimodal Configuration 🟡 BETA

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
      │ - AUDIO                      │ - TTS (OpenAI/Google) / STT (Groq/OpenAI API)
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

## Media Auto-Task (inbound media → classification → filing)

A photo or document sent to the Telegram channel becomes a filed item without
any user instruction. The chain is:

1. The channel stores the attachment in the bot's Drive (`inbox/…`) and the
   conversation carries the `[image] inbox/9f3c.jpg` marker (see
   [Telegram Channel](../06-channels/telegram-channel.md)).
2. The agent passes that path to the `classify_media` tool shipped in the
   `media-filing` template, which is the only supported way to file inbound
   media.
3. The tool perceives the content according to the file extension:
   `DESCRIBE IMAGE` for pictures, `SPEECH TO TEXT` for `[voice]`/`[audio]`
   notes, `DESCRIBE VIDEO` for `[video]` and `GET` for documents. Binary audio
   and video never reach the document text extractor, which cannot read them.
4. It asks the model for one word of a **closed taxonomy**
   (`invoice`, `receipt`, `contract`, `identity`, `report`, `audio`, `video`,
   `unsorted`) and moves the file to `media/{year}/{month}/{category}/`,
   writing a `.meta.txt` audit trail next to it.

The taxonomy being closed is the safety property: a model answer that is not
one of those words degrades to `unsorted`, so a misclassification can never
create an arbitrary folder. Every keyword involved already exists
(`DESCRIBE IMAGE`, `DESCRIBE VIDEO`, `SPEECH TO TEXT`, `GET`, `LLM`, `MOVE`,
`CREATE FILE`, `SPLIT`, `LAST`, `FIRST`, `STR`, `LEN`, `LCASE`, `INSTR`,
`TRIM`, `REPLACE`, `LEFT`, `TODAY`).

### When the vision service is unavailable

The flow degrades instead of failing. If `DESCRIBE IMAGE` (or the document
read) raises an error — BotModels not running, model missing, unsupported
format — the tool falls back to the caption already carried by the marker,
still applies the closed taxonomy, and files the item; the reply notes that the
content analysis was unavailable. With no caption either, the item is filed as
`unsorted`. Installing/preparing BotModels therefore upgrades classification
quality with no script change.

`DESCRIBE IMAGE` requires two conditions, and both must hold:

- the bot configuration has `botmodels-enabled,true` (see
  [Configuration](#configuration)), and
- the service is actually reachable at the address the server uses, e.g.
  `BOTMODELS_HOST=http://<bot-host>:8082` in the botserver unit.

Check reachability — a connection-refused (`000`) result is the usual cause of
a degraded classification:

```bash
curl -s -o /dev/null -w '%{http_code}\n' http://<botmodels-host>:<port>/api/health
```

### Deploying the template

The template ships as `bottemplates/bots/media-filing/media-filing.gbai/` and is
staged by `POST /api/templates/deploy/{id}` (the id returned by
`GET /api/templates/list`) into the **org layout** every runtime resolver keys
on:

```
{org}.gborg/{bot}.gbai/{bot}.gbdialog/    # scripts, incl. classify_media
{org}.gborg/{bot}.gbai/{bot}.gbot/        # PROMPT-TELEGRAM.md
```

`{bot}` is the name chosen in the Templates app (it defaults to the template
name). The template's inner `.gbdialog`/`.gbot` directories are renamed to the
bot name while staging, because the tool-execution, prompt and MCP resolvers
look those directories up by bot name — a bot whose dialog directory kept the
template name would load no scripts. The drive monitor then uploads the staged
tree to MinIO and registers the bot, the same mechanism bootstrap uses for the
shipped catalog.

Telegram also needs a bot token before the channel can be reached. Provide it at
deploy time through the `telegram_token` field of the deploy request, or later by
writing the `telegram-bot-token` configuration key for the bot. The key is
recognised as sensitive, so it is stored in Vault at
`secret/gbo/{org_id}/{branch_id}/{bot_id}` and never in the database — the value
comes from BotFather. This step is best-effort: if the bot row does not exist yet
(the drive monitor creates it after the upload), the deploy response reports
`telegram.status = "pending"` and the token can be written again afterwards.

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
- [IMAGE Keyword](../04-basic-scripting/keywords.md) - Image generation reference
- [SEE Keyword](../04-basic-scripting/keywords.md) - Vision capabilities