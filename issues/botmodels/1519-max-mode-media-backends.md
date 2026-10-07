# [BOTMODELS] 1519 — `max`-mode media backends (Qwen-Image, Wan 2.2, PaddleOCR-VL)

**Status:** registry only — commit `329c4d600`
**Priority:** P2
**Kind:** feature
**Depends on:** 1513, 1516 · **Blocks:** —

## Problem

Once the backend registry exists (1513), the `max` column needs backends. Today
these capabilities have exactly one hardcoded model each, and the video path has
defects that block any non-Zeroscope pipeline (see 1516).

## Goal

| Capability | `min` (keep) | `max` (add) | License |
|---|---|---|---|
| Image | SD-Turbo | **Qwen-Image-2.1** (7B DiT, unified gen+edit, RGBA) | Apache 2.0 |
| Video | Zeroscope v2 | **Wan 2.2 TI2V-5B** (720p, T2V+I2V) | Apache 2.0 |
| OCR | Tesseract | **PaddleOCR-VL-1.6** (0.9B, 96.3% OmniDocBench) | Apache 2.0 |

Deliberately **excluded**, and why:

- **FLUX.2 [dev]** — best photorealism, but **non-commercial weights**. Not
  shippable in an MIT product.
- **FLUX.2 [klein] 4B** — Apache 2.0, viable alternative if VRAM allows.
- **LTX-2.5** — 4K + native audio, but gated and free only under $10M revenue.
- **HunyuanVideo 1.5** — Tencent licence excludes EU/UK/South Korea.
- **Z-Image Turbo** — Apache 2.0, ~1s on H100; speed pick if throughput dominates.

## Video pipeline notes

`video_service.py:69-81` assumes `output.frames[0]` (a `TextToVideoSDPipeline`
shape) and never passes `width`/`height`. Wan 2.2 needs both fixed — the
registry's `generate()` signature is the place to add resolution.

Wan 2.2 renders at ~9 min for a 5s clip at 720p (baseline reference), so
`max` mode video needs a job-queue shape, not a synchronous request. Music
already has one (`MusicService` job polling) — reuse that pattern rather than
inventing a second.

## Acceptance criteria

- [ ] `max` mode generates images via Qwen-Image-2.1 with editable prompts and
      text rendering.
- [ ] `max` mode generates 720p video via Wan 2.2 with explicit resolution.
- [ ] `max` mode OCR returns structured tables, not flattened text.
- [x] `min` mode byte-identical to current behaviour.
- [x] Every added model is Apache 2.0 or MIT (verified on the model card).

### What landed

All three backends are implemented and wired into the registry:
`QwenImageBackend` (`QwenImagePipeline`, 1328px default, true_cfg_scale),
`Wan22Backend` (`AutoPipeline`, 1280×720, 81 frames, bfloat16 on CUDA), and
`PaddleOcrBackend`. Registry entries resolve correctly — verified across all 14
tier×capability pairs.

Zeroscope remains the `min` default with identical behaviour: same pipeline
class, same `DPMSolverMultistepScheduler` treatment, same 4-step image default.

`_extract_frames()` in the video backends replaces the previous bare
`output.frames[0]` access with an explicit check that raises a clear error on a
latents-returning pipeline, instead of a `TypeError` (1516).

### Outstanding — needs a GPU host

None of the three `max`-mode models has been loaded against real weights; the
dev host has no CUDA, torch or checkpoints. Unconfirmed: whether
`diffusers>=0.38.0` exposes `QwenImagePipeline` and Wan `AutoPipeline` under
those exact names, and whether the checkpoints fit the `max` VRAM threshold.

**Also outstanding:** Wan 2.2 renders a 5s 720p clip in roughly 9 minutes, so
synchronous request handling is not viable for it. The issue proposes reusing
`MusicService`'s job-polling shape; that async conversion is **not** started.
Until it is, `max` mode video will hold a request for minutes.

## Non-goals

- Image-to-image editing UI.
- Video upscaling / frame interpolation.