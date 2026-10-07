import hashlib
import time
from datetime import datetime, timezone
from typing import Optional

from ..core.config import settings
from ..core.logging import get_logger
from .backends import registry

logger = get_logger("video_service")


class VideoService:
    """Video generation over the selected backend.

    Resolution is forwarded to the pipeline (the previous service read
    VIDEO_WIDTH/VIDEO_HEIGHT from config but never passed them through).
    """

    def __init__(self) -> None:
        self.backend = registry.get("video")

    async def generate(
        self,
        prompt: str,
        num_frames: Optional[int] = None,
        fps: Optional[int] = None,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        seed: Optional[int] = None,
    ) -> dict:
        start = time.time()

        logger.info(
            "Generating video",
            backend=self.backend.name,
            prompt=prompt[:50],
            frames=num_frames,
        )

        result = self.backend.generate(
            prompt=prompt,
            num_frames=num_frames,
            fps=fps,
            steps=steps,
            width=width,
            height=height,
            seed=seed,
        )

        import imageio

        digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()[:8]
        timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{digest}.mp4"
        output_path = settings.output_dir / "videos" / filename

        imageio.mimsave(
            output_path, result["frames"], fps=result["fps"], codec="libx264"
        )

        generation_time = time.time() - start
        logger.info("Video generated", backend=self.backend.name, file=filename)

        return {
            "status": "completed",
            "file_path": f"/outputs/videos/{filename}",
            "generation_time": generation_time,
            "backend": self.backend.name,
        }


_service = None


def get_video_service():
    global _service
    if _service is None:
        _service = VideoService()
    return _service