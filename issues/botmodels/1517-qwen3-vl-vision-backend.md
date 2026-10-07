# [BOTMODELS] 1517 — Qwen3-VL vision backend (real VQA + OCR)

**Priority:** P1
**Kind:** feature
**Depends on:** 1513, 1516 · **Blocks:** —

## Problem

`/api/vision/vqa` is not a VQA model. `vision_service.py:191-194` forwards the
question as the caption prompt to BLIP2, which is a captioner. It cannot reason
about an image, and it cannot read text in one.

`confidence: 0.85` at `vision_service.py:81` is **fabricated** — the code comment
concedes "BLIP2 doesn't provide confidence scores directly". The value is
indistinguishable from a real score.

OCR is `pytesseract` (Tesseract, 2005). It flattens tables into linear text,
degrades sharply on poor scans, and is out of scope for handwriting. The pip
package is pinned but **nothing installs the system binary**.

`describe_video` samples frames and concatenates captions
(`"Video shows: " + "; ".join(unique[:4])`) — no temporal model, no audio track.

## Goal

- `max` mode gains **Qwen3-VL** as the vision backend: real VQA, OCR across 32
  languages, chart and table reading, native grounding (2D boxes + points).
- **BLIP2 stays the `min`-mode captioner** — it is small, fast, and good at
  captions. Qwen3-VL fills the reasoning/OCR gap; it does not replace captioning.
- Fabricated confidence replaced with a labelled heuristic or removed.

### Model choice

| Model | Params | VRAM (Q4) | License |
|---|---|---|---|
| Qwen3-VL-4B | 4B | ~6GB | Apache 2.0 |
| Qwen3-VL-8B | 8B | ~8GB | Apache 2.0 |

Default to 4B; 8B behind a config path. DeepSeek-OCR 2 (MIT, 3B) is the
document-parsing specialist if OCR volume alone justifies a second model.

## Code anchors

| What | Where |
|---|---|
| Fake VQA (re-prompts the captioner) | `src/services/vision_service.py:191-194` |
| Fabricated confidence | `src/services/vision_service.py:81` |
| Frame-sampling video description | `src/services/vision_service.py:93-189` |
| Tesseract OCR | `src/api/v1/endpoints/vision.py` (`/vision/ocr`) |
| Three bare `except:` clauses (forbidden by the README) | `vision_service.py:303,313,329` |

## Acceptance criteria

- [ ] `max` mode: `/api/vision/vqa` answers reasoned questions about an image.
- [ ] `max` mode: OCR handles tables and non-English text.
- [ ] `min` mode behaviour unchanged (BLIP2 captions).
- [ ] No response returns a fabricated confidence; heuristics are labelled.
- [ ] Bare `except:` removed; errors return structured JSON per the README.
- [ ] `/vision/analyze` reuses the OCR/caption handlers instead of duplicating
      them inline.

## Non-goals

- Replacing BLIP2 as the captioner.
- Audio-track understanding in `describe_video`.