"""Availability of media capabilities backed by system libraries.

`pyzbar` loads the libzbar shared library when imported, so a host without the
package used to fail at module import and take STT/TTS/vision down with it
(issue 1746). Import it on demand from the barcode path instead, and report the
state on /api/health so the installer can surface the remediation up front.
"""

import shutil
import threading
from importlib import import_module
from importlib.util import find_spec
from typing import Any, Optional

from ..core.logging import get_logger
from .errors import BarcodeUnavailableError

logger = get_logger("media_capabilities")

LIBZBAR_PACKAGE = "libzbar0"
LIBZBAR_REMEDIATION = "apt-get install -y libzbar0"
OCR_REMEDIATION = (
    "apt-get install -y tesseract-ocr (or pip install rapidocr-onnxruntime)"
)

_lock = threading.Lock()
_checked = False
_unavailable_reason: Optional[str] = None


def _load_pyzbar() -> Any:
    """Import the decoder module, which loads libzbar as a side effect."""
    return import_module("pyzbar.pyzbar")


def require_pyzbar() -> Any:
    """Return the pyzbar decoder module or raise with the install command."""
    try:
        return _load_pyzbar()
    except (ImportError, OSError) as exc:
        raise BarcodeUnavailableError(str(exc) or type(exc).__name__) from exc


def barcode_unavailable_reason() -> Optional[str]:
    """Why barcode decoding is unavailable, or None when pyzbar imports.

    Probed once per process: a missing system library cannot appear later and
    retrying on every health call would only repeat the import failure.
    """
    global _checked, _unavailable_reason
    if _checked:
        return _unavailable_reason

    with _lock:
        if _checked:
            return _unavailable_reason
        try:
            _load_pyzbar()
            _unavailable_reason = None
        except (ImportError, OSError) as exc:
            _unavailable_reason = str(exc) or type(exc).__name__
            logger.warning(
                "Barcode decoding unavailable",
                library=LIBZBAR_PACKAGE,
                error=_unavailable_reason,
                remediation=LIBZBAR_REMEDIATION,
            )
        _checked = True
        return _unavailable_reason


def _installed(module: str) -> bool:
    """Whether a module can be resolved without importing it."""
    try:
        return find_spec(module) is not None
    except (ImportError, ValueError):
        return False


def ocr_available() -> bool:
    """Mirror the OCR backends' probes without importing them.

    Tesseract needs both the pip package and the system binary; RapidOCR ships
    its ONNX models through pip and is the fallback when the binary is absent.
    """
    tesseract = _installed("pytesseract") and shutil.which("tesseract") is not None
    return tesseract or _installed("rapidocr_onnxruntime")


def media_capabilities() -> dict:
    """Per-capability availability for /api/health. Never raises."""
    reason = barcode_unavailable_reason()
    ocr = ocr_available()
    return {
        "barcode": {
            "available": reason is None,
            "library": LIBZBAR_PACKAGE,
            "detail": reason,
            "remediation": None if reason is None else LIBZBAR_REMEDIATION,
        },
        "ocr": {
            "available": ocr,
            "detail": None if ocr else "no OCR backend (tesseract or rapidocr) found",
            "remediation": None if ocr else OCR_REMEDIATION,
        },
    }


__all__ = [
    "LIBZBAR_PACKAGE",
    "LIBZBAR_REMEDIATION",
    "OCR_REMEDIATION",
    "barcode_unavailable_reason",
    "media_capabilities",
    "ocr_available",
    "require_pyzbar",
]
