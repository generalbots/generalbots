import io
import time
from typing import Optional

from ..core.config import settings
from ..core.logging import get_logger
from .backends import registry

logger = get_logger("vision_service")


class VisionService:
    """Captioning, VQA and video frame description over the selected backend.

    In min mode this is BLIP2: good at captions, no reasoning or OCR. In max
    mode Qwen3-VL answers questions and reads text genuinely, which BLIP2 cannot.
    """

    def __init__(self) -> None:
        self.backend = registry.get("vision")

    async def describe_image(
        self, image_data: bytes, prompt: Optional[str] = None
    ) -> dict:
        """Caption (or answer, when `prompt` is a question) about an image."""
        start = time.time()

        from PIL import Image

        image = Image.open(io.BytesIO(image_data)).convert("RGB")
        description = self.backend.describe(image, prompt=prompt)

        return {
            "description": description,
            "backend": self.backend.name,
            "generation_time": time.time() - start,
        }

    async def describe_video(self, video_data: bytes, num_frames: int = 8) -> dict:
        """Sample frames evenly and describe each. No temporal model."""
        start = time.time()

        import cv2
        import numpy as np
        import tempfile
        from PIL import Image

        with tempfile.NamedTemporaryFile(suffix=".mp4", delete=False) as tmp:
            tmp.write(video_data)
            tmp_path = tmp.name

        try:
            capture = cv2.VideoCapture(tmp_path)
            total_frames = int(capture.get(cv2.CAP_PROP_FRAME_COUNT))

            if total_frames == 0:
                capture.release()
                return {
                    "description": "Could not read video frames",
                    "frame_count": 0,
                    "generation_time": time.time() - start,
                }

            indices = np.linspace(0, total_frames - 1, num_frames, dtype=int)
            descriptions: list[str] = []

            for index in indices:
                capture.set(cv2.CAP_PROP_POS_FRAMES, index)
                ok, frame = capture.read()
                if not ok:
                    continue
                rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                descriptions.append(
                    self.backend.describe(Image.fromarray(rgb), max_new_tokens=50)
                )

            capture.release()

            if not descriptions:
                return {
                    "description": "No frames could be extracted from video",
                    "frame_count": 0,
                    "generation_time": time.time() - start,
                }

            unique = list(dict.fromkeys(descriptions))
            combined = (
                unique[0]
                if len(unique) == 1
                else "Video shows: " + "; ".join(unique[:4])
            )

            return {
                "description": combined,
                "frame_count": len(descriptions),
                "backend": self.backend.name,
                "generation_time": time.time() - start,
            }
        finally:
            import os

            if os.path.exists(tmp_path):
                os.unlink(tmp_path)

    async def answer_question(self, image_data: bytes, question: str) -> dict:
        """Visual question answering.

        With Qwen3-VL this genuinely reasons about the image. With BLIP2 the
        question is used as a caption prompt — BLIP2 is not a VQA model, and the
        response records which backend answered.
        """
        result = await self.describe_image(image_data, prompt=question)
        result["answer"] = result["description"]
        result["reasoning"] = self.backend.name != "blip2"
        return result


_service = None


def get_vision_service():
    global _service
    if _service is None:
        _service = VisionService()
    return _service