"""Vision backends: captioning, VQA and OCR.

min tier: BLIP2 — small and fast, good at captions, no reasoning or OCR.
max tier: Qwen3-VL — real VQA, 32-language OCR, charts/tables, grounding.

BLIP2 stays the caption default in min mode; Qwen3-VL fills the reasoning gap
rather than replacing captioning.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend

logger = get_logger("backends.vision")


class Blip2Backend(Backend):
    """BLIP2 captioner. Default vision backend in min mode."""

    name = "blip2"
    capability = "vision"

    def __init__(self) -> None:
        super().__init__()
        self.model = None
        self.processor = None

    def is_available(self) -> bool:
        try:
            import torch  # noqa: F401
            from transformers import Blip2ForConditionalGeneration  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from transformers import Blip2ForConditionalGeneration, Blip2Processor

        path = settings.vision_model_path
        logger.info("Loading BLIP2 vision backend", path=path)

        device = settings.device or "cpu"
        self.processor = Blip2Processor.from_pretrained(path)
        self.model = Blip2ForConditionalGeneration.from_pretrained(
            path,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        ).to(device)

    def describe(
        self,
        image,
        prompt: Optional[str] = None,
        max_new_tokens: int = 100,
    ) -> str:
        """Caption an image, optionally conditioned on a prompt."""
        self.ensure_loaded()
        if self.model is None or self.processor is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        if prompt:
            inputs = self.processor(image, text=prompt, return_tensors="pt").to(
                settings.device or "cpu"
            )
        else:
            inputs = self.processor(image, return_tensors="pt").to(
                settings.device or "cpu"
            )

        with torch.no_grad():
            generated = self.model.generate(
                **inputs, max_new_tokens=max_new_tokens, num_beams=5, early_stopping=True
            )

        return self.processor.decode(generated[0], skip_special_tokens=True).strip()


class Qwen3VLBackend(Backend):
    """Qwen3-VL. max tier: real VQA, OCR and chart/table reading."""

    name = "qwen3-vl"
    capability = "vision"

    def __init__(self) -> None:
        super().__init__()
        self.model = None
        self.processor = None

    def is_available(self) -> bool:
        try:
            import torch  # noqa: F401
            from transformers import AutoProcessor  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from transformers import AutoModelForImageTextToText, AutoProcessor

        path = settings.vision_model_path
        logger.info("Loading Qwen3-VL vision backend", path=path)

        device = settings.device or "cpu"
        self.processor = AutoProcessor.from_pretrained(path)
        self.model = AutoModelForImageTextToText.from_pretrained(
            path,
            torch_dtype=torch.bfloat16 if device == "cuda" else torch.float32,
            device_map="auto" if device == "cuda" else None,
        )
        if device != "cuda":
            self.model.to(device)

    def describe(
        self,
        image,
        prompt: Optional[str] = None,
        max_new_tokens: int = 256,
    ) -> str:
        self.ensure_loaded()
        if self.model is None or self.processor is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        # Default to a captioning instruction; explicit prompts (a VQA question)
        # pass through unchanged so this backend genuinely answers them.
        text = prompt or "Describe this image in detail."
        messages = [
            {
                "role": "user",
                "content": [
                    {"type": "image"},
                    {"type": "text", "text": text},
                ],
            }
        ]
        rendered = self.processor.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True
        )
        inputs = self.processor(
            text=[rendered], images=[image], return_tensors="pt"
        ).to(self.model.device)

        with torch.no_grad():
            generated = self.model.generate(
                **inputs, max_new_tokens=max_new_tokens, do_sample=False
            )

        trimmed = generated[:, inputs["input_ids"].shape[1] :]
        return self.processor.batch_decode(trimmed, skip_special_tokens=True)[0].strip()

    def ocr(self, image, max_new_tokens: int = 1024) -> str:
        """Structured transcription: markdown tables, not flattened text."""
        self.ensure_loaded()
        if self.model is None or self.processor is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        messages = [
            {
                "role": "user",
                "content": [
                    {"type": "image"},
                    {
                        "type": "text",
                        "text": (
                            "Transcribe all text from this image. Preserve table "
                            "structure as markdown tables and keep column order."
                        ),
                    },
                ],
            }
        ]
        rendered = self.processor.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True
        )
        inputs = self.processor(
            text=[rendered], images=[image], return_tensors="pt"
        ).to(self.model.device)

        with torch.no_grad():
            generated = self.model.generate(
                **inputs, max_new_tokens=max_new_tokens, do_sample=False
            )

        trimmed = generated[:, inputs["input_ids"].shape[1] :]
        return self.processor.batch_decode(trimmed, skip_special_tokens=True)[0].strip()


VISION_BACKENDS: dict[str, type[Backend]] = {
    "blip2": Blip2Backend,
    "qwen3-vl": Qwen3VLBackend,
}