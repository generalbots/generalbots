"""Image generation backends.

min tier: SD-Turbo (SD family) — small, seconds on CPU, mature LoRA/ControlNet.
max tier: Qwen-Image-2.1 — 7B DiT, unified generation + editing, native RGBA.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend

logger = get_logger("backends.image")


class StableDiffusionBackend(Backend):
    """SD family (SD-Turbo / SD 1.5). Default image backend in both tiers."""

    name = "sd-turbo"
    capability = "image"

    def __init__(self) -> None:
        super().__init__()
        self.pipeline = None

    def is_available(self) -> bool:
        try:
            import diffusers  # noqa: F401
            import torch  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from diffusers import DPMSolverMultistepScheduler, StableDiffusionPipeline

        path = settings.image_model_path
        logger.info("Loading SD image backend", path=path)

        device = settings.device or "cpu"
        pipeline = StableDiffusionPipeline.from_pretrained(
            path,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
            safety_checker=None,
        )
        pipeline.scheduler = DPMSolverMultistepScheduler.from_config(
            pipeline.scheduler.config
        )
        pipeline = pipeline.to(device)
        if device == "cuda":
            pipeline.enable_attention_slicing()

        self.pipeline = pipeline

    def generate(
        self,
        prompt: str,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        guidance_scale: Optional[float] = None,
        seed: Optional[int] = None,
    ):
        """Return a PIL image. Raises BackendNotLoadedError if not loaded."""
        self.ensure_loaded()
        if self.pipeline is None:  # pragma: no cover - defensive
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        actual_steps = steps if steps is not None else settings.image_steps
        actual_width = width if width is not None else settings.image_width
        actual_height = height if height is not None else settings.image_height
        actual_guidance = guidance_scale if guidance_scale is not None else 7.5

        # `is not None` matters: seed=0 is a valid seed.
        generator = (
            torch.Generator(device=settings.device or "cpu").manual_seed(seed)
            if seed is not None
            else None
        )

        output = self.pipeline(
            prompt=prompt,
            num_inference_steps=actual_steps,
            guidance_scale=actual_guidance,
            width=actual_width,
            height=actual_height,
            generator=generator,
        )
        return output.images[0]


class QwenImageBackend(Backend):
    """Qwen-Image-2.1. max tier only.

    7B single-stream DiT with prefix KV-cache reuse, unified text-to-image and
    editing, native RGBA output. Apache 2.0.
    """

    name = "qwen-image"
    capability = "image"

    def __init__(self) -> None:
        super().__init__()
        self.pipeline = None

    def is_available(self) -> bool:
        try:
            import diffusers  # noqa: F401
            import torch  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch

        from diffusers import QwenImagePipeline

        path = settings.image_model_path
        logger.info("Loading Qwen-Image backend", path=path)

        device = settings.device or "cpu"
        pipeline = QwenImagePipeline.from_pretrained(
            path,
            torch_dtype=torch.bfloat16 if device == "cuda" else torch.float32,
        )
        self.pipeline = pipeline.to(device)

    def generate(
        self,
        prompt: str,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        guidance_scale: Optional[float] = None,
        seed: Optional[int] = None,
    ):
        self.ensure_loaded()
        if self.pipeline is None:  # pragma: no cover - defensive
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        generator = (
            torch.Generator(device=settings.device or "cpu").manual_seed(seed)
            if seed is not None
            else None
        )

        output = self.pipeline(
            prompt=prompt,
            num_inference_steps=steps if steps is not None else 30,
            width=width if width is not None else 1328,
            height=height if height is not None else 1328,
            true_cfg_scale=guidance_scale if guidance_scale is not None else 4.0,
            generator=generator,
        )
        return output.images[0]


IMAGE_BACKENDS: dict[str, type[Backend]] = {
    "sd-turbo": StableDiffusionBackend,
    "sd": StableDiffusionBackend,
    "qwen-image": QwenImageBackend,
}