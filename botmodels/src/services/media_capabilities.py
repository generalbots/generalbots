"""Availability of media capabilities backed by optional packages/libraries.

`pyzbar` loads the libzbar shared library when imported, so a host without the
package used to fail at module import and take STT/TTS/vision down with it
(issue 1746). Import it on demand from the barcode path instead, keep pyzbar in
the optional requirements-barcode.txt extra, and report the state on /api/health
so the installer can surface the remediation up front.
"""

import shutil
import threading
from ctypes.util import find_library
from importlib import import_module
from importlib.util import find_spec
from typing import Any, Optional

from ..core.logging import get_logger
from .errors import BarcodeUnavailableError

logger = get_logger("media_capabilities")

LIBZBAR_PACKAGE = "libzbar0"
LIBZBAR_REMEDIATION = "apt-get install -y libzbar0"
BARCODE_PIP_REMEDIATION = "pip install -r requirements-barcode.txt"
OCR_REMEDIATION = (
    "apt-get install -y tesseract-ocr (or pip install rapidocr-onnxruntime)"
)

_lock = threading.Lock()
_checked = False
_unavailable_reason: Optional[str] = None
_remediation: Optional[str] = None


def _load_pyzbar() -> Any:
    """Import the decoder module, which loads libzbar as a side effect."""
    return import_module("pyzbar.pyzbar")


def require_pyzbar() -> Any:
    """Return the pyzbar decoder module or raise with the install command."""
    try:
        return _load_pyzbar()
    except (ImportError, OSError) as exc:
        raise BarcodeUnavailableError(str(exc) or type(exc).__name__) from exc


def _libzbar_present() -> bool:
    """Whether the system linker can find libzbar (without importing pyzbar)."""
    try:
        return find_library("zbar") is not None
    except (OSError, ValueError):
        return False


def _barcode_probe() -> tuple[Optional[str], Optional[str]]:
    """Return ``(detail, remediation)`` for barcode decoding.

    Distinguishes a package that was deliberately not installed (the desktop
    pythonbox leaves pyzbar out when libzbar is absent) from a missing system
    library. Probed once per process: a missing system library cannot appear
    later and retrying on every health call would only repeat the failure.
    """
    global _checked, _unavailable_reason, _remediation
    if _checked:
        return _unavailable_reason, _remediation

    with _lock:
        if _checked:
            return _unavailable_reason, _remediation
        if find_spec("pyzbar") is None:
            if _libzbar_present():
                _unavailable_reason = "pyzbar is not installed"
                _remediation = BARCODE_PIP_REMEDIATION
            else:
                _unavailable_reason = "pyzbar is not installed and libzbar0 is missing"
                _remediation = (
                    f"{LIBZBAR_REMEDIATION} && {BARCODE_PIP_REMEDIATION}"
                )
        else:
            try:
                _load_pyzbar()
                _unavailable_reason = None
                _remediation = None
            except (ImportError, OSError) as exc:
                _unavailable_reason = str(exc) or type(exc).__name__
                _remediation = LIBZBAR_REMEDIATION
        if _unavailable_reason is not None:
            logger.warning(
                "Barcode decoding unavailable",
                library=LIBZBAR_PACKAGE,
                error=_unavailable_reason,
                remediation=_remediation,
            )
        _checked = True
        return _unavailable_reason, _remediation


def barcode_unavailable_reason() -> Optional[str]:
    """Why barcode decoding is unavailable, or None when pyzbar imports."""
    return _barcode_probe()[0]


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
    reason, remediation = _barcode_probe()
    ocr = ocr_available()
    return {
        "barcode": {
            "available": reason is None,
            "library": LIBZBAR_PACKAGE,
            "detail": reason,
            "remediation": remediation,
        },
        "ocr": {
            "available": ocr,
            "detail": None if ocr else "no OCR backend (tesseract or rapidocr) found",
            "remediation": None if ocr else OCR_REMEDIATION,
        },
    }


__all__ = [
    "BARCODE_PIP_REMEDIATION",
    "LIBZBAR_PACKAGE",
    "LIBZBAR_REMEDIATION",
    "OCR_REMEDIATION",
    "barcode_unavailable_reason",
    "media_capabilities",
    "ocr_available",
    "require_pyzbar",
]
