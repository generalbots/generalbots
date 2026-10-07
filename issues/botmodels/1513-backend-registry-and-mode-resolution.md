# [BOTMODELS] 1513 — Backend registry + `min`/`max`/`auto` mode resolution

**Status:** implemented — commit `329c4d600`
**Priority:** P0
**Kind:** feature
**Depends on:** — · **Blocks:** 1516, 1517, 1518, 1519

## Problem

Each capability hardcodes exactly one model, so there is no way to add a second
option without editing the service body. The `*_model_path` settings look like
they configure the model, but they only configure *where* a hardcoded class
loads from:

| Hardcoded class | Location |
|---|---|
| `StableDiffusionPipeline` | `src/services/image_service.py:6` |
| `Blip2ForConditionalGeneration` | `src/services/vision_service.py:27` |
| `DiffusionPipeline` (generic) | `src/services/video_service.py:25` |

Pointing `VISION_MODEL_PATH` at a Qwen3-VL checkpoint cannot work — it will load
into `Blip2ForConditionalGeneration` and fail. The same goes for
`IMAGE_MODEL_PATH` → Qwen-Image.

Secondarily: models load **eagerly** at startup and are held forever, with no
way to express "only load this if the host can afford it". `--workers 4`
quadruples VRAM.

## Goal

- A backend registry: one module per capability, each backend exposing
  `load()` / `available()` / the capability's operation.
- `BOTMODELS_MODE=auto|min|max` resolved **once** in `lifespan` and frozen.
- `auto` detects CUDA + VRAM + RAM; `min`/`max` force.
- Lazy load on first use behind an `asyncio.Lock`.
- `/api/health` reports mode, detection source, and the resolved backend per
  capability.

## Mode resolution order (in `lifespan`, before any service init)

```
1. mode = env BOTMODELS_MODE (default "auto"); validate ∈ {auto,min,max}
2. if mode == auto:
     cuda    = torch.cuda.is_available() and torch.cuda.device_count() > 0
     vram_gb = round(torch.cuda.get_device_properties(0).total_memory / 2**30)
     ram_gb  = host RAM via psutil.virtual_memory(); fallback /proc/meminfo
               MemTotal (Linux) or sysctl (macOS)
     mps     = torch.backends.mps.is_available()
     max if (cuda and vram_gb >= 16 and ram_gb >= 32)
         or (mps and ram_gb >= 32)          # unified memory: key off RAM
     else min
3. bind *_backend per capability from the mode table
4. init services
```

### Constraints

- **MPS / unified memory** — `total_memory` from the GPU is unreliable under
  unified memory, so Apple Silicon keys off `ram_gb` alone.
- **`CUDA_VISIBLE_DEVICES=""`** is the documented way to force CPU under Docker.
  Detection must honour it and land in `min`. Needs an explicit test.
- **No mid-run switching.** Mode is immutable after startup; flipping it would
  unload models under in-flight requests.
- **`psutil` is not currently pinned.** Add it, and keep the `/proc/meminfo` +
  `sysctl` fallback so detection works if the dep is unavailable.
- **First-request race** — the current module-global singletons
  (`image_service.py:107`) have no lock, so two concurrent first requests can
  both call `initialize()`. The registry's `load()` fixes this for every backend.

## Code anchors

| What | Where |
|------|-------|
| Startup hook (mode resolution goes here) | `src/main.py:19-32` (`lifespan`) |
| Settings to extend (`mode`, `*_backend`) | `src/core/config.py` |
| Health endpoint (must report mode) | `src/main.py:83-90` |
| Unlocked singleton pattern to replace | `src/services/image_service.py:107-114` |
| Eager init list (music/anomaly/scoring/voice omitted today) | `src/main.py:22-26` |

## Acceptance criteria

- [x] With `BOTMODELS_MODE` unset, every capability resolves to its current
      model and `/api/health` reports `mode: "min"` or `"max"` with the
      detection evidence.
- [x] `BOTMODELS_MODE=max` on a host without 16GB VRAM forces `max` (explicit
      override, no re-detection).
- [x] `BOTMODELS_MODE=invalid` fails fast at startup with a clear message.
- [x] Concurrent first requests to one backend trigger exactly one load.
- [x] `/api/health` exposes `mode`, `mode_source` (`env`|`auto`), `detected`
      (`{cuda, vram_gb, ram_gb, device}`) and the resolved backend per capability.
- [x] `CUDA_VISIBLE_DEVICES=""` resolves to `min`.
- [x] Detection works without `psutil` installed (fallback path covered).

### Verification

`tests/test_mode.py` — 19 cases, all passing. Covers the full tier matrix
(no GPU / low VRAM / low RAM / large GPU / MPS-high-RAM / MPS-low-RAM /
undetectable RAM), env override, case and whitespace normalisation, invalid
input, and the backend table. Run with:

```bash
python3 -m unittest tests.test_mode
```

`CUDA_VISIBLE_DEVICES=""` and the no-`psutil` fallback were both exercised on
the dev host (4 CPU / 7.8GB / no GPU), where `auto` correctly resolves `min`.
Tier resolution for a real CUDA or MPS host is covered by mocking
`HardwareProfile`, not by live hardware.

## Non-goals

- Adding the `max`-mode backends themselves (1517, 1518, 1519).
- Per-request backend override in the HTTP API.
- Runtime mode flipping (explicitly rejected above).