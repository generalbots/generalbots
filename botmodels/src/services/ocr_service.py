"""OCR over the selected backend.

Kept separate from vision.py so the endpoint module stays under the 450-line
rule while the OCR-specific logic lives with the backends.
"""

import io

from PIL import Image

from .backends import registry
from .errors import BotModelsError


def run_ocr(image_data: bytes, language: str = "eng") -> dict:
    """Extract text from an image using the tier's OCR backend."""
    backend = registry.get("ocr")
    image = Image.open(io.BytesIO(image_data)).convert("RGB")

    try:
        result = backend.extract(image, languages=language)
    except BotModelsError as exc:
        return {
            "success": False,
            "text": "",
            "confidence": 0.0,
            "language": language,
            "word_count": 0,
            "backend": backend.name,
            "error": str(exc),
        }

    text = result.get("text", "")
    return {
        "success": True,
        "text": text.strip(),
        # Tesseract reports real per-word confidence; PaddleOCR-VL does not, so
        # the field is absent rather than a fabricated constant.
        **({"confidence": result["confidence"]} if "confidence" in result else {}),
        **({"structured": True} if result.get("structured") else {}),
        "language": language,
        "word_count": len(text.split()),
        "backend": backend.name,
        "error": None,
    }


def describe_codes(image_data: bytes) -> list[dict]:
    """QR and barcode payloads found in an image."""
    from pyzbar import pyzbar

    image = Image.open(io.BytesIO(image_data))
    if image.mode != "RGB":
        image = image.convert("RGB")

    return [
        {
            "data": obj.data.decode("utf-8", errors="replace"),
            "type": obj.type,
        }
        for obj in pyzbar.decode(image)
    ]


def image_metadata(image_data: bytes) -> dict:
    image = Image.open(io.BytesIO(image_data))
    return {
        "width": image.width,
        "height": image.height,
        "format": image.format,
        "mode": image.mode,
    }