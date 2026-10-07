"""Video generation backends.

min tier: Zeroscope v2 — small, fast, 320x576 thumbnails.
max tier: Wan 2.2 TI2V-5B — 720p text-to-video and image-to-video, Apache 2.0.

Resolution is passed explicitly for both: the previous service read
VIDEO_WIDTH/VIDEO_HEIGHT from config but never forwarded them to the pipeline.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend

logger = get_logger("backends.video")


def _extract_frames(output):
    """Normalise diffusers output to a list of numpy frames.

    Text-to-video pipelines return `frames`; latent pipelines return `latents`,
    which would previously break with a TypeError. Raise a clear error instead.
    """
    frames = getattr(output, "frames", None)
    if frames is not None:
        return frames[0]

    raise BackendNotLoadedError(
        "video",
        "pipeline",
        "pipeline returned latents, not decoded frames; decode before saving",
    )


class ZeroscopeBackend(Backend):
    """Zeroscope v2. min tier default."""

    name = "zeroscope"
    capability = "video"

    def __init__(self) -> None:
        super().__init__()
        self.pipeline = None

    def is_available(self) -> bool:
        try:
            import diffusers  # noqa: F401
            import imageio  # noqa: F401
            import torch  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from diffusers import DiffusionPipeline

        path = settings.video_model_path
        logger.info("Loading Zeroscope backend", path=path)

        device = settings.device or "cpu"
        self.pipeline = DiffusionPipeline.from_pretrained(
            path,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        ).to(device)

    def generate(
        self,
        prompt: str,
        num_frames: Optional[int] = None,
        fps: Optional[int] = None,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        seed: Optional[int] = None,
    ) -> dict:
        self.ensure_loaded()
        if self.pipeline is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        generator = (
            torch.Generator(device=settings.device or "cpu").manual_seed(seed)
            if seed is not None
            else None
        )

        output = self.pipeline(
            prompt=prompt,
            num_frames=num_frames if num_frames is not None else settings.video_frames,
            num_inference_steps=steps if steps is not None else settings.video_steps,
            width=width if width is not None else settings.video_width,
            height=height if height is not None else settings.video_height,
            generator=generator,
        )

        return {
            "frames": _extract_frames(output),
            "fps": fps if fps is not None else settings.video_fps,
        }


class Wan22Backend(Backend):
    """Wan 2.2 TI2V-5B. max tier: 720p T2V + I2V, Apache 2.0."""

    name = "wan22"
    capability = "video"

    def __init__(self) -> None:
        super().__init__()
        self.pipeline = None

    def is_available(self) -> bool:
        try:
            import diffusers  # noqa: F401
            import imageio  # noqa: F401
            import torch  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from diffusers import AutoPipeline

        path = settings.video_model_path
        logger.info("Loading Wan 2.2 backend", path=path)

        device = settings.device or "cpu"
        self.pipeline = AutoPipeline.from_pretrained(
            path,
            torch_dtype=torch.bfloat16 if device == "cuda" else torch.float32,
        ).to(device)

    def generate(
        self,
        prompt: str,
        num_frames: Optional[int] = None,
        fps: Optional[int] = None,
        steps: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
        seed: Optional[int] = None,
    ) -> dict:
        self.ensure_loaded()
        if self.pipeline is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import torch

        generator = (
            torch.Generator(device=settings.device or "cpu").manual_seed(seed)
            if seed is not None
            else None
        )

        output = self.pipeline(
            prompt=prompt,
            num_frames=num_frames if num_frames is not None else 81,
            num_inference_steps=steps if steps is not None else 30,
            width=width if width is not None else 1280,
            height=height if height is not None else 720,
            generator=generator,
        )

        return {
            "frames": _extract_frames(output),
            "fps": fps if fps is not None else 24,
        }


VIDEO_BACKENDS: dict[str, type[Backend]] = {
    "zeroscope": ZeroscopeBackend,
    "wan22": Wan22Backend,
}