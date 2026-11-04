"""Model source resolution shared by every backend.

A configured value may be either a local directory/file or a Hub id. The
previous backends always passed the configured value to ``from_pretrained``,
so the defaults (``./models/...``) failed on a fresh install even when the Hub
cache already held the weights.

Resolution order:

1. an existing local path (offline bundle, operator-provided weights),
2. the supplied Hub id (cache first, download when online).

``local_files_only`` is not forced here: the caller decides whether network is
allowed by running in offline mode (``HF_HUB_OFFLINE=1``) or by prefetching.
"""

from pathlib import Path
from typing import Optional

# Canonical open-source checkpoints used when no local path is configured.
BLIP_CAPTION_MODEL = "Salesforce/blip-image-captioning-base"
BLIP2_MODEL = "Salesforce/blip2-opt-2.7b"
QWEN_VL_MODEL = "Qwen/Qwen3-VL-4B-Instruct"
SD_TURBO_MODEL = "stabilityai/sd-turbo"
ZEROSCOPE_MODEL = "cerspense/zeroscope_v2_576w"
WAN22_MODEL = "Wan-AI/Wan2.2-TI2V-5B"
PADDLE_OCR_MODEL = "PaddlePaddle/PaddleOCR-VL"


def resolve_model_source(
    configured: Optional[str],
    fallback_id: str,
) -> str:
    """Return a loadable source for ``from_pretrained``.

    Existing local paths win so an offline bundle is authoritative. Otherwise
    the Hub id lets transformers/diffusers use their cache and, when online,
    fetch the weights once.
    """
    candidate = (configured or "").strip()
    if candidate:
        path = Path(candidate).expanduser()
        if path.exists():
            return str(path)
    return fallback_id
