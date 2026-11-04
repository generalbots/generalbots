"""OCR backends.

min tier: Tesseract via pytesseract — CPU-only, no GPU needed. Kept as the
fallback for hosts with no accelerator, even though it is the weakest option:
it flattens tables and is out of scope for handwriting.
max tier: PaddleOCR-VL-1.6 — 0.9B VLM, 96.3% on OmniDocBench v1.6, Apache 2.0.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend
from .sources import PADDLE_OCR_MODEL, resolve_model_source

logger = get_logger("backends.ocr")


class TesseractBackend(Backend):
    """Tesseract via pytesseract. min tier default, CPU-only."""

    name = "tesseract"
    capability = "ocr"

    def is_available(self) -> bool:
        try:
            import pytesseract  # noqa: F401

            # The pip package does not install the system binary; without it
            # every call raises, so probe the binary too.
            pytesseract.get_tesseract_version()
            return True
        except Exception:  # noqa: BLE001 - any failure means unavailable
            return False

    def extract(self, image, languages: str = "eng") -> dict:
        self.ensure_loaded()

        import pytesseract
        from pytesseract import Output

        data = pytesseract.image_to_data(
            image, lang=languages, output_type=Output.DICT
        )
        words: list[str] = []
        confidences: list[float] = []

        for text, conf in zip(data["text"], data["conf"]):
            stripped = text.strip()
            if not stripped:
                continue
            try:
                confidence = float(conf)
            except (TypeError, ValueError):
                continue
            if confidence < 0:
                continue
            words.append(stripped)
            confidences.append(confidence)

        return {
            "text": " ".join(words),
            "confidence": round(sum(confidences) / len(confidences) / 100, 3)
            if confidences
            else 0.0,
        }


class PaddleOcrBackend(Backend):
    """PaddleOCR-VL-1.6. max tier: structured markdown tables. Apache 2.0."""

    name = "paddleocr-vl"
    capability = "ocr"

    def __init__(self) -> None:
        super().__init__()
        self.pipeline = None

    def is_available(self) -> bool:
        try:
            import paddleocr  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        from paddleocr import PaddleOCRVL

        source = resolve_model_source(settings.ocr_model_path, PADDLE_OCR_MODEL)
        logger.info("Loading PaddleOCR-VL backend", source=source)
        self.pipeline = PaddleOCRVL(model_name=source)

    def extract(self, image, languages: str = "eng") -> dict:
        self.ensure_loaded()
        if self.pipeline is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        results = self.pipeline.predict(image)
        text_parts: list[str] = []
        for result in results:
            markdown = getattr(result, "markdown", None)
            if callable(markdown):
                parsed = markdown()
                text = getattr(parsed, "text", None)
                if isinstance(text, str):
                    text_parts.append(text)
                    continue
            if isinstance(markdown, str):
                text_parts.append(markdown)

        return {
            # Markdown tables, not flattened text: this is the whole point of
            # the max-tier backend.
            "text": "\n\n".join(text_parts),
            "structured": True,
        }


class RapidOcrBackend(Backend):
    """RapidOCR (PP-OCR ONNX). Pip-installable fallback when tesseract is absent.

    Tesseract needs a system binary installed by root; constrained/dev hosts
    often lack it. RapidOCR ships ONNX models through pip and runs on the same
    onnxruntime dependency already used by the speech stack.
    """

    name = "rapidocr"
    capability = "ocr"

    def __init__(self) -> None:
        super().__init__()
        self.engine = None

    def is_available(self) -> bool:
        try:
            import rapidocr_onnxruntime  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        from rapidocr_onnxruntime import RapidOCR

        logger.info("Loading RapidOCR backend")
        self.engine = RapidOCR()

    def extract(self, image, languages: str = "eng") -> dict:
        self.ensure_loaded()
        if self.engine is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import numpy as np

        result, _elapsed = self.engine(np.array(image.convert("RGB")))
        items = result or []
        texts = [str(item[1]) for item in items if len(item) > 1 and item[1]]
        scores = [
            float(item[2])
            for item in items
            if len(item) > 2 and item[2] is not None
        ]
        return {
            "text": " ".join(texts),
            "confidence": round(sum(scores) / len(scores), 3) if scores else 0.0,
        }


OCR_BACKENDS: dict[str, type[Backend]] = {
    "tesseract": TesseractBackend,
    "rapidocr": RapidOcrBackend,
    "paddleocr-vl": PaddleOcrBackend,
}