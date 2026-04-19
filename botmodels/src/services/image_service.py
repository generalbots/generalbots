import time
from datetime import datetime
from typing import Optional

import torch
from diffusers import DPMSolverMultistepScheduler, StableDiffusionPipeline
from PIL import Image

from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("image_service")


class ImageService:
    def __init__(self):
        self.pipeline: Optional[StableDiffusionPipeline] = None
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return
        logger.info("Loading Stable Diffusion model", path=settings.image_model_path)
        try:
            self.pipeline = StableDiffusionPipeline.from_pretrained(
                settings.image_model_path,
                torch_dtype=torch.float16 if self.device == "cuda" else torch.float32,
                safety_checker=None,
            )
            self.pipeline.scheduler = DPMSolverMultistepScheduler.from_config(
                self.pipeline.scheduler.config
            )
            self.pipeline = self.pipeline.to(self.device)
            if self.device == "cuda":
                self.pipeline.enable_attention_slicing()
            self._initialized = True
            logger.info("Stable Diffusion loaded successfully")
        except Exception as e:
            logger.warning(
                "Stable Diffusion model not available, image generation disabled",
                error=str(e),
            )
            self._initialized = True

    async def generate(
        self,
        prompt: str,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        guidance_scale: Optional[float] = None,
        seed: Optional[int] = None,
    ) -> dict:
        if not self._initialized:
            self.initialize()

        # Use config defaults if not specified
        actual_steps = steps if steps is not None else settings.image_steps
        actual_width = width if width is not None else settings.image_width
        actual_height = height if height is not None else settings.image_height
        actual_guidance = guidance_scale if guidance_scale is not None else 7.5

        start = time.time()
        generator = (
            torch.Generator(device=self.device).manual_seed(seed) if seed else None
        )

        logger.info(
            "Generating image",
            prompt=prompt[:50],
            steps=actual_steps,
            width=actual_width,
            height=actual_height,
        )

        output = self.pipeline(
            prompt=prompt,
            num_inference_steps=actual_steps,
            guidance_scale=actual_guidance,
            width=actual_width,
            height=actual_height,
            generator=generator,
        )

        image: Image.Image = output.images[0]
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{hash(prompt) & 0xFFFFFF:06x}.png"
        output_path = settings.output_dir / "images" / filename
        image.save(output_path)

        generation_time = time.time() - start
        logger.info("Image generated", file=filename, time=generation_time)

        return {
            "status": "completed",
            "file_path": f"/outputs/images/{filename}",
            "generation_time": generation_time,
        }

    async def describe(self, image_data: bytes) -> dict:
        # Placeholder for backward compatibility
        # Use vision_service for actual image description
        return {"description": "Use /api/vision/describe endpoint", "confidence": 0.0}


_service = None


def get_image_service():
    global _service
    if _service is None:
        _service = ImageService()
    return _service
