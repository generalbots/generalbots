# [BOTMODELS] 1518 — Local speech backends (Kokoro TTS + Qwen3-ASR STT)

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

- [ ] With no API keys set, `min` mode returns 503 — **never** calls
      `translate.google.com` or any cloud endpoint.
- [ ] Google Translate fallback deleted from the codebase.
- [ ] `totext` returns a non-empty transcript locally, or a clear error.
- [ ] `detect_language` never returns `"auto"`.
- [ ] No fabricated `confidence` values.
- [ ] Temp files cleaned up on every failure path.
- [ ] `SPEECH_MODEL_PATH` / `WHISPER_MODEL_PATH` either honoured or removed from
      `.env.example`.
- [ ] `faster-whisper`, `piper-tts` and `onnxruntime` added to `requirements.txt`
      (the realtime voice path currently cannot install).

## Non-goals

- Realtime speech-to-speech (`/api/speech/realtime` is currently a mislabelled
  Whisper transcription path returning **text, not audio** — a separate issue).
- Voice cloning.