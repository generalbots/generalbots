# [BOTMODELS] 1514 — Fix Rust↔Python route contract mismatches

**Priority:** P0
**Kind:** bug
**Depends on:** — · **Blocks:** —

## Problem

Four `botserver` call sites target `botmodels` routes that do not exist. These
are not theoretical: they 404 or 422 on every call.

| Rust caller | Path sent | Python actually registers |
|---|---|---|
| `botmultimodal/src/multimodal.rs` | `/api/vision/describe_video` | `/api/vision/describe-video` |
| `botbasic_core/.../hearing/processing.rs` | `/api/speech/to-text` | `/api/speech/totext` |
| `botbasic_core/.../hearing/processing.rs` | `/api/vision/qrcode` (raw octet-stream) | `UploadFile = File(...)` → 422 |
| `botbasic_ai/.../ai_tools.rs` | `/ocr` | `/api/vision/ocr` |

Two of these also send a **raw request body** where FastAPI expects `multipart`,
so even with the path corrected they return 422.

Env-var naming is inconsistent across the same Rust tree: `botmultimodal` and
`jukebox` read `BOTMODELS_HOST`, while `ai_tools.rs` and `processing.rs` read
`BOTMODELS_URL`.

## Goal

- Every Rust caller reaches a real `botmodels` route.
- Multipart framing matches what FastAPI declares.
- Python keeps underscore aliases so an already-deployed `botserver` does not
  404 during a rolling upgrade.
- One env var name for the base URL.

## Code anchors

| What | Where |
|---|---|
| Image/video/speech/vision client | `botserver/crates/botmultimodal/src/multimodal.rs` |
| QR + STT callers (raw body bug) | `botserver/crates/botbasic_core/src/.../hearing/processing.rs` |
| OCR caller (wrong path) | `botserver/crates/botbasic_ai/src/.../ai_tools.rs` |
| Canonical Python routes | `src/api/v1/endpoints/vision.py`, `speech.py` |
| Port documented inconsistently (8082) | `botbook/src/10-configuration-deployment/multimodal.md:192` |

## Acceptance criteria

- [ ] `/api/vision/describe_video` and `/api/speech/to-text` work, via corrected
      Rust callers **and** Python aliases.
- [ ] QR and STT calls succeed with real multipart bodies (not 422).
- [ ] `/ocr` reaches `/api/vision/ocr`.
- [ ] `BOTMODELS_HOST` is the single base-URL variable; `BOTMODELS_URL` removed
      or aliased.
- [ ] Port `8082` ambiguity resolved in `multimodal.md` — it currently refers to
      three different services across the tree.

## Non-goals

- Redesigning the multimodal keyword surface (`IMAGE` / `VIDEO` / `AUDIO` / `SEE`).