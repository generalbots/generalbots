import time
from datetime import datetime
from typing import Optional

import imageio
import torch

from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("video_service")


class VideoService:
    def __init__(self):
        self.pipeline = None
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return
        logger.info("Loading video model", path=settings.video_model_path)
        try:
            from diffusers import DiffusionPipeline

            self.pipeline = DiffusionPipeline.from_pretrained(
                settings.video_model_path,
                torch_dtype=torch.float16 if self.device == "cuda" else torch.float32,
            )
            self.pipeline = self.pipeline.to(self.device)
            self._initialized = True
            logger.info("Video model loaded successfully")
        except Exception as e:
            logger.warning(
                "Video model not available, video generation disabled", error=str(e)
            )
            self._initialized = True

    async def generate(
        self,
        prompt: str,
        num_frames: Optional[int] = None,
        fps: Optional[int] = None,
        steps: Optional[int] = None,
        seed: Optional[int] = None,
    ) -> dict:
        if not self._initialized:
            self.initialize()

        # Use config defaults if not specified
        actual_frames = num_frames if num_frames is not None else settings.video_frames
        actual_fps = fps if fps is not None else settings.video_fps
        actual_steps = steps if steps is not None else 50

        start = time.time()
        generator = (
            torch.Generator(device=self.device).manual_seed(seed) if seed else None
        )

        logger.info(
            "Generating video",
            prompt=prompt[:50],
            frames=actual_frames,
            fps=actual_fps,
            steps=actual_steps,
        )

        output = self.pipeline(
            prompt=prompt,
            num_frames=actual_frames,
            num_inference_steps=actual_steps,
            generator=generator,
        )

        frames = output.frames[0]
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{hash(prompt) & 0xFFFFFF:06x}.mp4"
        output_path = settings.output_dir / "videos" / filename

        imageio.mimsave(output_path, frames, fps=actual_fps, codec="libx264")

        generation_time = time.time() - start
        logger.info("Video generated", file=filename, time=generation_time)

        return {
            "status": "completed",
            "file_path": f"/outputs/videos/{filename}",
            "generation_time": generation_time,
        }

    async def describe(self, video_data: bytes) -> dict:
        # Placeholder for backward compatibility
        # Use vision_service for actual video description
        return {
            "description": "Use /api/vision/describe_video endpoint",
            "frame_count": 0,
        }


_service = None


def get_video_service():
    global _service
    if _service is None:
        _service = VideoService()
    return _service
