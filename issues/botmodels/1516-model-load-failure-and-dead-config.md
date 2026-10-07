# [BOTMODELS] 1516 — Stop swallowing model-load failures; purge dead config

**Priority:** P0
**Kind:** bug
**Depends on:** 1513 · **Blocks:** 1517, 1519

## Problem

Model-load failures are swallowed in three different broken ways, so a bad
config surfaces as a `TypeError` deep in a request instead of a clear 503.

| Bug | Location | Effect |
|---|---|---|
| `_initialized = True` **inside** `except` | `src/services/image_service.py:39-44` | `generate()` then calls `None` → `TypeError: 'NoneType' object is not callable` |
| same pattern | `src/services/video_service.py:34-38` | same |
| `_initialized` **never set** on failure | `src/services/vision_service.py:37-40` | every request re-runs `from_pretrained` — a retry storm re-downloading weights |

Other defects in the same paths:

- `image_service.py:66` — `if seed else None`: **`seed=0` is silently treated as
  no generator.**
- `image_service.py:88` — `hash(prompt)` is randomised per process (PYTHONHASHSEED),
  so output filenames are not reproducible across restarts.
- `video_service.py:69-81` — `width`/`height` are **never passed** to the
  pipeline, so `VIDEO_WIDTH`/`VIDEO_HEIGHT` config is dead. Output assumes
  `output.frames[0]` (a `TextToVideoSDPipeline` shape); a latent-output pipeline
  breaks here.
- `steps` defaults to a hardcoded `50` in `video_service.py:54`, not config.

### Dead configuration

Never read by any service: `image_gpu_layers`, `image_batch_size`,
`video_gpu_layers`, `video_batch_size`, `video_width`, `video_height`.

Declared in `.env.example` but absent from `Settings`, so `extra="ignore"`
drops them without complaint: `SPEECH_MODEL_PATH`, `WHISPER_MODEL_PATH`.

The botbook's `image-generator-model,../../../../data/diffusion/sd_turbo_f16.gguf`
is **stale**: the Python tree contains zero GGUF / llama.cpp / ONNX-runtime
support, so `StableDiffusionPipeline.from_pretrained("*.gguf")` cannot work.

## Goal

- A failed load raises a typed error → HTTP 503 naming the model and reason.
- `seed=0` honoured; filenames deterministic.
- Every config key is either honoured or deleted.
- Stale GGUF guidance corrected.

## Acceptance criteria

- [ ] Bad `IMAGE_MODEL_PATH` → 503 with the model id and cause, not a `TypeError`.
- [ ] Failed vision load does **not** retry `from_pretrained` on every request.
- [ ] `seed=0` produces a reproducible image.
- [ ] Filenames deterministic across process restarts (sha256 of prompt).
- [ ] `VIDEO_WIDTH`/`VIDEO_HEIGHT` are passed to the pipeline, or removed.
- [ ] Dead keys removed from `Settings`, `.env.example` and the botbook.
- [ ] Startup logs report which backends loaded and which failed, per mode.

## Non-goals

- Adding GGUF / llama.cpp image support to Python (correct path is to drop the
  stale doc entry).
- `--workers` guidance (a separate operational concern).