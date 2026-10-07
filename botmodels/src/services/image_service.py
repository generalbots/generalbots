import hashlib
import time
from datetime import datetime, timezone
from typing import Optional

from ..core.config import settings
from ..core.logging import get_logger
from .backends import registry
from .errors import BackendNotLoadedError

logger = get_logger("image_service")


class ImageService:
    """Image generation over the selected backend.

    Delegates model concerns to the backend registry, so SD-Turbo and Qwen-Image
    are interchangeable without changing callers. A failed load raises
    BackendNotLoadedError (surfaced as a 503) instead of calling None.
    """

    def __init__(self) -> None:
        self.backend = registry.get("image")

    async def generate(
        self,
        prompt: str,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        guidance_scale: Optional[float] = None,
        seed: Optional[int] = None,
    ) -> dict:
        start = time.time()

        logger.info(
            "Generating image",
            backend=self.backend.name,
            prompt=prompt[:50],
            steps=steps,
            width=width,
            height=height,
        )

        image = self.backend.generate(
            prompt=prompt,
            steps=steps,
            width=width,
            height=height,
            guidance_scale=guidance_scale,
            seed=seed,
        )

        # sha256 rather than hash(): PYTHONHASHSEED randomises str hashing per
        # process, so filenames were not reproducible across restarts.
        digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()[:8]
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{digest}.png"

        output_path = settings.output_dir / "images" / filename
        image.save(output_path)

        generation_time = time.time() - start
        logger.info("Image generated", backend=self.backend.name, file=filename)

        return {
            "status": "completed",
            "file_path": f"/outputs/images/{filename}",
            "generation_time": generation_time,
            "backend": self.backend.name,
        }


_service = None


def get_image_service():
    global _service
    if _service is None:
        _service = ImageService()
    return _service