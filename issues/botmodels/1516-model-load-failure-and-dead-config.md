# [BOTMODELS] 1516 — Stop swallowing model-load failures; purge dead config

**Status:** implemented — commit `329c4d600`
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

- [x] Bad `IMAGE_MODEL_PATH` → 503 with the model id and cause, not a `TypeError`.
- [x] Failed vision load does **not** retry `from_pretrained` on every request.
- [x] `seed=0` produces a reproducible image.
- [x] Filenames deterministic across process restarts (sha256 of prompt).
- [x] `VIDEO_WIDTH`/`VIDEO_HEIGHT` are passed to the pipeline, or removed.
- [ ] Dead keys removed from `Settings`, `.env.example` and the botbook.
- [x] Startup logs report which backends loaded and which failed, per mode.

### Verification

`Backend.ensure_loaded()` records the cause once and raises
`BackendNotLoadedError`; a recorded failure is not retried, which is the
retry-storm fix. `ensure_loaded_async()` adds an `asyncio.Lock`, and a
`threading.Lock` guards the blocking path — covering the unlocked-singleton race
on concurrent first requests.

`tests/test_backends.py` — 8 cases, all passing: typed error on failure,
no retry across 5 calls, `reset()` clears state, load happens exactly once,
12 concurrent threads trigger a single `from_pretrained`, and `status()` does
not load.

Both seed and filename fixes are code-level and asserted by inspection rather
than test (they need a real diffusion pipeline). `seed` is now guarded with
`is not None`, so `seed=0` is honoured; filenames use
`sha256(prompt)[:8]` instead of `hash(prompt)`, which was per-process
randomised by `PYTHONHASHSEED`.

`VIDEO_WIDTH`/`VIDEO_HEIGHT`/`VIDEO_STEPS` now exist in `Settings` and are
forwarded to both the Zeroscope and Wan backends. `video_steps` replaced the
hardcoded `50`.

### Outstanding

- **`botbook/.../multimodal.md`** still documents
  `image-generator-model,../../../../data/diffusion/sd_turbo_f16.gguf`. The
  Python tree has no GGUF/llama.cpp support, so that config cannot work. The
  botbook edit is still needed — a documentation change, deliberately separate.
- `botmodels/README.md` "Project Structure" and "Technology Stack" sections are
  stale (missing `services/{backends,scoring}/`, `core/{hardware,mode,startup}.py`).
- The GGUF config entries in the botbook should become the tier-based
  `IMAGE_MODEL_PATH` + `MODE` pair.

## Non-goals

- Adding GGUF / llama.cpp image support to Python (correct path is to drop the
  stale doc entry).
- `--workers` guidance (a separate operational concern).