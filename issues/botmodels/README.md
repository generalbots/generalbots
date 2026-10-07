# BotModels Issues — Index

Lightweight model tiering + open-source backend adoption for `botmodels/`.

> **Context.** `botmodels` is the Python FastAPI inference sidecar to `botserver`
> (Rust). It currently hardcodes one model per capability (SD 1.5, Zeroscope v2,
> BLIP2, remote OpenAI/Groq TTS/STT, Tesseract, ACE-Step 1.5). The platform
> advertises "runs offline / air-gapped", but the speech path calls `api.openai.com`
> and falls back to **unauthenticated `translate.google.com`**.

## Model tiers

Two modes, resolved **once at startup** and frozen for the process lifetime.

| | `min` | `max` |
|---|---|---|
| Trigger | no CUDA, or VRAM < 8GB, or RAM < 32GB | VRAM ≥ 16GB **and** RAM ≥ 32GB |
| Image | SD-Turbo (~2–4GB) | Qwen-Image-2.1 7B (~16GB) |
| Vision | BLIP2 (~6GB) | Qwen3-VL-4B (~6GB) |
| Speech | Piper ONNX (CPU) | Kokoro-82M + Qwen3-ASR-1.7B |
| OCR | Tesseract (CPU) | PaddleOCR-VL-1.6 0.9B (~2GB) |
| Video | Zeroscope v2 (~4GB) | Wan 2.2 TI2V-5B (~8GB) |
| Music | ACE-Step 1.5 | ACE-Step 1.5 XL (unchanged) |

`BOTMODELS_MODE=auto` (default) detects; `min` / `max` force. Detection never
switches mid-run — flipping would unload models under in-flight requests.

## Waves

| # | Title | Priority | Kind | Depends on |
|---|-------|----------|------|------------|
| [1513](1513-backend-registry-and-mode-resolution.md) | Backend registry + `min`/`max`/`auto` mode resolution | P0 | feature | — |
| [1514](1514-fix-route-contract-mismatches.md) | Fix Rust↔Python route contract mismatches | P0 | bug | — |
| [1515](1515-close-auth-holes.md) | Close auth holes (`/api/detect`, scoring health, WebSockets) | P0 | security | — |
| [1516](1516-model-load-failure-and-dead-config.md) | Stop swallowing model-load failures; purge dead config | P0 | bug | 1513 |
| [1517](1517-qwen3-vl-vision-backend.md) | Qwen3-VL vision backend (real VQA + OCR) | P1 | feature | 1513, 1516 |
| [1518](1518-local-speech-backends.md) | Kokoro TTS + Qwen3-ASR STT backends | P1 | feature | 1513 |
| [1519](1519-max-mode-media-backends.md) | Qwen-Image-2.1 + Wan 2.2 + PaddleOCR-VL backends | P2 | feature | 1513, 1516 |
| [1520](1520-remove-dead-code.md) | Remove dead code (`app.py`, orphan anomaly app) | P2 | cleanup | — |

**Execution order:** 1514 · 1515 · 1520 are independent and landable at any time.
The tiered sequence is `1513 → 1516 → 1517/1518 → 1519`.

## Invariants

- `min` mode behaviour is **byte-identical** to today's behaviour when
  `BOTMODELS_MODE` is unset — SD-Turbo, BLIP2, Piper, Tesseract, Zeroscope.
- A backend that fails to load returns **503 naming the backend**, never a
  `TypeError` from calling `None`.
- `min` mode **refuses remote speech providers** rather than falling back to a
  cloud API — fail loudly instead of shipping audio off-box.
- Every `max`-mode pick is **Apache 2.0 or MIT**. Excluded on licence grounds:
  FLUX.2 [dev] (non-commercial), HunyuanVideo 1.5 (bans EU/UK/KR), MiniMax H3
  (bans US/EU/UK/KR), F5-TTS (CC-BY-NC weights), YuE2 (NC weights),
  LTX-2.5 (gated, $10M revenue cap).
- Every model file is ≤ 450 lines (AGENTS.md). Split at 350.

## Open decisions

- Piper flipped MIT → **GPL-3.0** (`OHF-Voice/piper1-gpl`; `rhasspy/piper` was
  MIT). `min` TTS accepts this as a process boundary, or moves to Kokoro.
- `transformers>=5.0.0rc3` is a **release-candidate pin**. Qwen3-VL / Qwen3-ASR
  may require the 5.x line, which gates whether `max` mode is buildable.