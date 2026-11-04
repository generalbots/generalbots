"""Vision backends: captioning, VQA and OCR.

min tier: BLIP base captioner — fits under 8 GB RAM and is good at captions.
BLIP2 and Qwen3-VL remain available for larger hosts.
max tier: Qwen3-VL — real VQA, 32-language OCR, charts/tables, grounding.

The min tier deliberately does not load BLIP2: its 2.7B weights need more
memory than a constrained CPU host has, so it failed on exactly the machines
the min tier is meant to serve.
"""

import os
from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend
from .sources import (
    BLIP2_MODEL,
    BLIP_CAPTION_MODEL,
    MOONDREAM_MODEL,
    QWEN_VL_MODEL,
    resolve_model_source,
)

logger = get_logger("backends.vision")


class BlipCaptionBackend(Backend):
    """BLIP image captioner. Default vision backend in min mode.

    BLIP base is ~1 GB in float32 and captions a still image in seconds on CPU.
    It is a captioner, not a VQA model: questions are used as a caption prompt
    and callers must not present the result as reasoning.
    """

    name = "blip"
    capability = "vision"

    def __init__(self) -> None:
        super().__init__()
        self.model = None
        self.processor = None

    def is_available(self) -> bool:
        try:
            import torch  # noqa: F401
            from transformers import BlipForConditionalGeneration  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from transformers import BlipForConditionalGeneration, BlipProcessor

        source = resolve_model_source(
            os.environ.get("BOTMODELS_BLIP_MODEL"),
            BLIP_CAPTION_MODEL,
        )
        logger.info("Loading BLIP caption backend", source=source)

        device = settings.device or "cpu"
        self.processor = BlipProcessor.from_pretrained(source)
        self.model = BlipForConditionalGeneration.from_pretrained(
            source,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        ).to(device)

    def describe(
        self,
        image,
        prompt: Optional[str] = None,
        max_new_tokens: int = 60,
    ) -> str:
        """Caption an image, optionally conditioned on ``prompt``."""
        self.ensure_loaded()
        if self.model is None or self.processor is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        device = settings.device or "cpu"
        if prompt:
            inputs = self.processor(images=image, text=prompt, return_tensors="pt")
        else:
            inputs = self.processor(images=image, return_tensors="pt")
        inputs = {key: value.to(device) for key, value in inputs.items()}

        with torch.no_grad():
            generated = self.model.generate(**inputs, max_new_tokens=max_new_tokens)

        return self.processor.decode(generated[0], skip_special_tokens=True).strip()


class MoondreamBackend(Backend):
    """Moondream2: a compact VLM that works on CPU and answers questions.

    It is the better-quality CPU alternative to BLIP: BLIP only captions, while
    Moondream can follow a question. It is heavier (~3.7 GB in float32), so it
    is opt-in via ``BOTMODELS_VISION_BACKEND=moondream`` rather than the min
    default. No GPU is required, but a GPU host is much faster.
    """

    name = "moondream"
    capability = "vision"

    def __init__(self) -> None:
        super().__init__()
        self.model = None

    def is_available(self) -> bool:
        try:
            import torch  # noqa: F401
            from transformers import AutoModelForCausalLM  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from transformers import AutoModelForCausalLM

        source = resolve_model_source(
            os.environ.get("BOTMODELS_MOONDREAM_MODEL"),
            MOONDREAM_MODEL,
        )
        logger.info("Loading Moondream vision backend", source=source)

        device = settings.device or "cpu"
        self.model = AutoModelForCausalLM.from_pretrained(
            source,
            trust_remote_code=True,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        ).to(device)

    def describe(
        self,
        image,
        prompt: Optional[str] = None,
        max_new_tokens: int = 128,
    ) -> str:
        self.ensure_loaded()
        if self.model is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        if prompt:
            result = self.model.query(image, prompt)
        else:
            result = self.model.caption(image, length="normal")
        if isinstance(result, dict):
            result = result.get("caption") or result.get("answer") or ""
        return str(result).strip()


class Blip2Backend(Backend):
    """BLIP2 captioner. Available on larger hosts; not the min default."""

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

        source = resolve_model_source(settings.vision_model_path, BLIP2_MODEL)
        logger.info("Loading BLIP2 vision backend", source=source)

        device = settings.device or "cpu"
        self.processor = Blip2Processor.from_pretrained(source)
        self.model = Blip2ForConditionalGeneration.from_pretrained(
            source,
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
    requires_gpu = True

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

        source = resolve_model_source(settings.vision_model_path, QWEN_VL_MODEL)
        logger.info("Loading Qwen3-VL vision backend", source=source)

        device = settings.device or "cpu"
        self.processor = AutoProcessor.from_pretrained(source)
        self.model = AutoModelForImageTextToText.from_pretrained(
            source,
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
    "blip": BlipCaptionBackend,
    "moondream": MoondreamBackend,
    "blip2": Blip2Backend,
    "qwen3-vl": Qwen3VLBackend,
}