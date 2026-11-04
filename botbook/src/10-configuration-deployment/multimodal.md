# Multimodal Configuration 🟡 BETA

General Bots integrates with botmodels—a Python service for multimodal AI tasks—to enable image generation, video creation, audio synthesis, and vision capabilities directly from BASIC scripts.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Architecture

```
┌─────────────┐   HTTP / WS    ┌──────────────────────────────┐
│  botserver  │ ────────────▶  │  botmodels (local process)   │
│   (Rust)    │                │  faster-whisper · Piper      │
└─────────────┘                │  BLIP · RapidOCR             │
      │                        │  SD-Turbo · Zeroscope        │
      │ BASIC keywords         │  ACE-Step (external process) │
      │ chat attachments       └──────────────────────────────┘
```

When a BASIC script calls a multimodal keyword, or a chat attachment needs
transcription or description, botserver forwards the bytes to botmodels. The
service runs the model selected by its hardware tier and returns text or a
file. Speech, vision and OCR run locally; the opt-in remote providers are the
only paths that leave the host.

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

## Local open-source model bundle

BotModels selects one backend per capability from a hardware tier frozen at
startup (`botmodels/src/core/mode.py`). The tier is `min` unless the host has a
GPU with at least 16 GB VRAM and 32 GB RAM, so a CPU-only host never tries to
load a 7B checkpoint.

| Capability | `min` | `max` |
|------------|-------|-------|
| Vision | BLIP base captioner (`blip`) | Qwen3-VL (`qwen3-vl`) |
| Speech-to-text | faster-whisper (`faster-whisper`) | Qwen3-ASR (`qwen3-asr`) |
| Text-to-speech | Piper (`piper`) | Kokoro (`kokoro`) |
| OCR | Tesseract, falling back to RapidOCR when the binary is absent | PaddleOCR-VL (`paddleocr-vl`) |
| Image | SD-Turbo (`sd-turbo`) | Qwen-Image (`qwen-image`) |
| Video | Zeroscope (`zeroscope`) | Wan 2.2 (`wan22`) |
| Music | ACE-Step 1.5 | ACE-Step 1.5 |

The `min` tier deliberately uses the BLIP **base** captioner instead of
BLIP2 2.7B: the larger checkpoint does not fit the under-8 GB hosts the tier
targets. Both `blip2` and `qwen3-vl` remain registered for larger machines.

### Better CPU alternatives

These are free/open-source upgrades that still run without a GPU. They are
opt-in so the default test path stays light:

| Need | Min default | Better CPU option | Selection |
|------|-------------|-------------------|-----------|
| Speech-to-text | faster-whisper `small` | Whisper `medium` or `large-v3` (int8) | `BOTMODELS_STT_MODEL=medium` |
| Text-to-speech | Piper | Kokoro-82M (Apache-2.0) | `BOTMODELS_TTS_BACKEND=kokoro` |
| Image questions | BLIP caption | Moondream2 (VQA, ~3.7 GB fp32) | `BOTMODELS_VISION_BACKEND=moondream` |

Any capability can be pinned this way: `BOTMODELS_{CAPABILITY}_BACKEND`
(`STT`, `TTS`, `VISION`, `IMAGE`, `VIDEO`, `OCR`). An unknown or unavailable
override falls back to the tier's backend, and the fallback is logged.

### GPU gate

`settings.gpu_available` is true only when the resolved device is CUDA or MPS.
The max-tier backends (`qwen3-vl`, `qwen3-asr`, `qwen-image`, `wan22`,
`paddleocr-vl`) set `requires_gpu`; the base load path refuses them on CPU with
a `503` that names the capability and the reason, before any weights are
touched. `/api/health` exposes `gpu_available` and each loaded backend's
`requires_gpu`, so a deployment can verify the gate without provoking an OOM.

`MODE=auto` selects `max` only with a CUDA GPU of at least 16 GB VRAM and
32 GB RAM (or MPS with 32 GB RAM); otherwise it selects `min`. Forcing
`MODE=max` on a CPU host is a configuration error that the gate reports
per request rather than attempting the load.

### Provisioning

`botmodels/scripts/provision_models.py` verifies the imports and fetches the
weights. It never installs Python packages; the environment manager owns that.

```bash
cd botmodels

# Offline verification: imports, Piper voice file and Hub cache entries.
python scripts/provision_models.py --check

# CI-safe: verify importable dependencies only, no weights.
python scripts/provision_models.py --check --deps-only

# Download the min-tier weights into the Hub cache and the Piper voice.
python scripts/provision_models.py --install
```

`--install` writes `botmodels/models/manifest.json` with each entry's status,
cache path and size. `--tier max` lists the larger checkpoints without
downloading them. The script is idempotent and respects `HF_HUB_OFFLINE=1`.

Required open-source packages for the `min` speech/vision/OCR path:

```text
faster-whisper>=1.0.3
piper-tts>=1.3.0
av>=16,<19            # faster-whisper passes metadata_errors= to av.open
rapidocr-onnxruntime>=1.2.3
```

Piper voices are files, not Hub repositories. The default voice is
`en_US-lessac-medium` at `models/piper/en_US-lessac-medium.onnx`; override it
with `BOTMODELS_TTS_PATH`. Whisper size and device are selected with
`BOTMODELS_STT_MODEL` (default `small`) and `BOTMODELS_DEVICE`.

### Model source resolution

Backends accept either a local path or a Hugging Face id
(`botmodels/src/services/backends/sources.py`). An existing local path wins;
otherwise the canonical Hub id is used, so a prefetched cache works offline and
a fresh install downloads once. The previous defaults (`./models/blip2`,
`./models/zeroscope-v2`) always failed when those directories were absent.

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

Run it from `botmodels/` so the relative model paths resolve. The service's
`API_KEY` must equal the value botserver sends as `BOTMODELS_API_KEY`; otherwise
HTTP calls return `401` and voice sockets close with code `1008`. The sanctioned
`restart.sh` derives it automatically.

```bash
cd botmodels
python -m pip install -r requirements.txt
python scripts/provision_models.py --install
API_KEY="$BOTMODELS_API_KEY" python -m uvicorn src.main:app --host 0.0.0.0 --port 8085
```

For production with HTTPS:

```bash
API_KEY="$BOTMODELS_API_KEY" python -m uvicorn src.main:app \
    --host 0.0.0.0 \
    --port 8085 \
    --ssl-keyfile key.pem \
    --ssl-certfile cert.pem
```

`MODE=min` or `MODE=max` forces a tier; `MODE=auto` probes the hardware once at
startup and freezes the result for the process lifetime. The default remote
speech setting is `ALLOW_REMOTE_SPEECH=false`: audio and images stay on the host
unless an operator explicitly opts in.

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
| `/api/image/generate` | POST | Generate an image from a prompt |
| `/api/video/generate` | POST | Generate a video from a prompt |
| `/api/speech/generate` | POST | Synthesize speech; returns `file_path` under `/outputs/audio/` |
| `/api/speech/totext` | POST | Transcribe an uploaded audio file |
| `/api/speech/detect_language` | POST | Detect the spoken language |
| `/api/speech/realtime` | POST | One-shot speech-to-speech (optional PersonaPlex checkpoint) |
| `/api/vision/describe` | POST | Caption an image |
| `/api/vision/describe-video` | POST | Sample frames and describe a video (`/describe_video` is an alias) |
| `/api/vision/ocr` | POST | Extract text (Tesseract or RapidOCR) |
| `/api/vision/vqa` | POST | Visual question answering |
| `/api/health` | GET | Health, tier and loaded-backend status |

All HTTP endpoints except `/api/health` require the `X-API-Key` header.

### Streaming voice sockets

Two WebSocket endpoints carry raw 16-bit PCM at 16 kHz. They authenticate with
`?api_key=` because an upgrade cannot carry the header dependency.

| Endpoint | Direction | Protocol |
|----------|-----------|----------|
| `/v1/audio/stt/stream` | client → server | binary PCM frames; `{"type":"config","sample_rate":…}`, `{"type":"flush"}`; server answers `partial`/`final` JSON |
| `/v1/audio/tts/stream` | server → client | send `{"text":"…"}`; server answers a `meta` frame, binary PCM frames, then `{"type":"done"}` |
| `/v1/audio/duplex` | both | one socket: binary PCM for STT, `{"type":"say","text":…}` for TTS; incoming audio cancels the utterance and emits `barge_in` |

`/v1/audio/duplex` is the full-duplex state machine: it accepts speech while
synthesising, paces PCM at ~128 ms per frame, and cancels the active utterance
on barge-in. It does **not** perform acoustic echo cancellation; the client
must not route the synthesised output back into its microphone. `/stt/stream`
and `/tts/stream` remain available for half-duplex clients.

```bash
# HTTP round-trip
curl -s -H "X-API-Key: $API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"Local speech works."}' \
  http://127.0.0.1:8085/api/speech/generate

curl -s -H "X-API-Key: $API_KEY" \
  -F "file=@/path/to/audio.wav;type=audio/wav" \
  http://127.0.0.1:8085/api/speech/totext
```

`/api/health` reports `backends` (the tier's selection) and
`backends_loaded` (what has actually loaded, with errors). A backend that is
merely configured does not appear in `backends_loaded` until the first request.

## Chat attachment ingestion

Web chat attachments are staged in the bot's Drive `inbox/` and then handed to
the local model service by `botserver/src/core/bot/pipeline/turn_hooks.rs`:

| Attachment | BotModels call | Prompt content |
|------------|----------------|----------------|
| audio (`.mp3`, `.wav`, `.ogg`, `.m4a`, `.flac`) | `/api/speech/totext` | bounded transcript block |
| image (`.png`, `.jpg`, `.gif`, `.webp`, `.bmp`, `.heic`) | `/api/vision/describe` | caption block |
| video (`.mp4`, `.mov`, `.webm`, `.avi`, `.mkv`) | `/api/vision/describe-video` | description of six sampled frames |

The call uses the bot's `botmodels-*` settings, falling back to
`BOTMODELS_HOST`/`BOTMODELS_API_KEY` in the server environment. When the
service is disabled or unreachable, the model is told plainly that the media
content is unavailable; it is never asked to invent a transcript or caption.

## GB7 test scripts

The published feature suite (`gb7features/test-scripts.html`) exercises chat,
Drive and the local model service. Two scripts depend on BotModels:

| Script | Steps that use BotModels | Local bundle |
|--------|--------------------------|--------------|
| TS-01 | “describe the image” after attaching a PNG | BLIP captioner |
| TS-02 | transcribe a voice note, describe a short video | faster-whisper, BLIP on sampled frames |

TS-03, TS-04, TS-06, TS-07, TS-08, TS-09 and TS-10 do not need a local model
beyond the external chat completion endpoint. TS-05 is a negative test: it
expects voice/live mode to be absent, so no duplex UI is required (or wanted)
for that script.

## Model Paths

Local files live under `botmodels/models/`. Hub checkpoints are resolved from
the Hugging Face cache when the configured path does not exist, so a local
directory is optional once the cache is warm.

```
models/
├── piper/
│   └── en_US-lessac-medium.onnx    # Piper voice (+ .onnx.json)
└── manifest.json                   # written by provision_models.py
```

The path settings are `VISION_MODEL_PATH`, `IMAGE_MODEL_PATH`,
`VIDEO_MODEL_PATH`, `OCR_MODEL_PATH` and `BOTMODELS_TTS_PATH`. `--install`
records the resolved cache paths in `models/manifest.json`.

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

Run `python scripts/provision_models.py --check`. A configured local path that
does not exist falls back to the canonical Hub id; if neither is available the
backend reports the attempted source in the load error. `--install` fetches the
min-tier bundle.

**Out of memory**

Reduce `gpu-layers` or `batch-size`. Video generation is particularly
memory-intensive. On CPU-only hosts the min tier uses the BLIP base captioner
rather than BLIP2 2.7B; forcing `MODE=max` on such a host will fail at load.

**A max-tier backend returns 503 “requires a GPU”**

The host resolved to CPU. This is the gate working as intended: use the min
tier, or select a CPU backend explicitly with
`BOTMODELS_{CAPABILITY}_BACKEND=blip|piper|faster-whisper|rapidocr|zeroscope`.
The max-tier checkpoints need CUDA or MPS.

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