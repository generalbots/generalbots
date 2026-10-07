# [BOTMODELS] 1518 — Local speech backends (Kokoro TTS + Qwen3-ASR STT)

**Status:** partially implemented — commit `329c4d600`
**Priority:** P1
**Kind:** feature
**Depends on:** 1513 · **Blocks:** —

## Problem

**This breaks the platform's core promise.** General Bots advertises *"runs 100%
offline"* and *"no data leaves your network unless you route it out"*. The
speech path does neither.

| Endpoint | Actual behaviour |
|---|---|
| `POST /api/speech/generate` | No local TTS. Calls `api.openai.com` with model hardcoded `tts-1`; on **any** failure silently falls back to **unauthenticated `translate.google.com`** |
| `POST /api/speech/totext` | Groq (`whisper-large-v3-turbo`) → OpenAI (`whisper-1`) → **no local fallback**. Returns **HTTP 200 with `{"text": ""}`** |
| `POST /api/speech/detect_language` | Runs a full transcription and reads back `language`; Groq's default makes this return the literal `"auto"` |

The README's claims of "Coqui TTS" and "OpenAI Whisper" describe neither the code
nor the licensing reality. Google Translate TTS is also **not local**, and is
unlicensed for this use.

Related defects: `confidence: 0.99` is fabricated (lines ~145, ~185); temp
`.wav` files use `delete=False` and leak on every failure path; `.env.example`
lists `SPEECH_MODEL_PATH` / `WHISPER_MODEL_PATH` that `Settings` silently drops.

## Goal

- **Local-first.** `min` → Piper ONNX (existing, CPU); `max` → Kokoro-82M (TTS)
  + Qwen3-ASR (STT).
- **No cloud calls in `min` mode.** Return 503 naming the missing local model —
  fail loudly rather than ship audio off-box.
- Remote providers retained only as explicit opt-in.
- **Delete the Google Translate fallback outright.**

### Model choice

| Role | Model | License | Notes |
|---|---|---|---|
| Light TTS | Piper ONNX (`en_US-lessac-medium`) | **GPL-3.0** (was MIT) | Existing. See open decision below |
| Best TTS | Kokoro-82M | Apache 2.0 | CPU-only, <1GB, MOS 4.2 |
| Light STT | faster-whisper int8 | MIT | Existing; 24.7% WER chunked |
| Best STT | Qwen3-ASR-1.7B | Apache 2.0 | 52 languages, best batch WER |

Excluded: F5-TTS (CC-BY-NC **weights**), XTTS-v2 (CPML, orphaned since Coqui
shut down).

## Open decision — Piper licence

`rhasspy/piper` is archived (MIT); the successor `OHF-Voice/piper1-gpl` is
**GPL-3.0**. `botmodels` is a separate Python process, so this is a process
boundary rather than linked copyleft — but any assumption that Piper is MIT no
longer holds. Either accept it for `min` mode, or move light TTS to Kokoro
(Apache 2.0, also CPU-only).

## Acceptance criteria

- [x] With no API keys set, `min` mode returns 503 — **never** calls
      `translate.google.com` or any cloud endpoint.
- [x] Google Translate fallback deleted from the codebase.
- [ ] `totext` returns a non-empty transcript locally, or a clear error.
- [x] `detect_language` never returns `"auto"`.
- [x] No fabricated `confidence` values.
- [x] Temp files cleaned up on every failure path.
- [x] `SPEECH_MODEL_PATH` / `WHISPER_MODEL_PATH` either honoured or removed from
      `.env.example`.
- [x] `faster-whisper`, `piper-tts` and `onnxruntime` added to `requirements.txt`
      (the realtime voice path currently cannot install).

### What landed

`translate.google.com` is gone — grep confirms zero occurrences anywhere in
`src/`. `SpeechService` now resolves local-first: the tier's backend, then
`ALLOW_REMOTE_SPEECH` (default `false`), then a 503 naming the backend and the
opt-in. The fabricated `confidence: 0.99` is gone; faster-whisper returns its
own detected language, and `detect_language` maps `"auto"` to `None` so callers
can tell "not detected" from a real result.

The whisper-shaped temp-file leak is closed with `try/finally` in
`vision_service.describe_video`, and the speech path no longer writes
`delete=False` temporaries at all — STT backends read the path they are given.

`.env.example` drops `SPEECH_MODEL_PATH` / `WHISPER_MODEL_PATH` in favour of
`STT_MODEL_PATH` / `TTS_MODEL_PATH`, which `Settings` actually reads.
`faster-whisper`, `piper-tts` and `onnxruntime` are now pinned.

`KokoroTtsBackend` and `Qwen3AsrSttBackend` are registered for the `max` tier.

### Outstanding — needs a GPU/weights host

`totext` returning a real transcript is **unverified**: Piper and faster-whisper
have no voice files or weights on the dev host. Note also that `piper-tts` on
PyPI is GPL-3.0 (see Open decisions) — if that is unacceptable, the `min` TTS
default should move to Kokoro, which is Apache 2.0 and also CPU-only.

Verify on a host with voices present: `MODE=min`, then `/api/speech/generate`
and `/api/speech/totext`.

## Non-goals

- Realtime speech-to-speech (`/api/speech/realtime` is currently a mislabelled
  Whisper transcription path returning **text, not audio** — a separate issue).
- Voice cloning.