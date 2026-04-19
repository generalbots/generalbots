import io
import time
from datetime import datetime
from typing import Optional

import torch
from PIL import Image

from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("vision_service")


class VisionService:
    def __init__(self):
        self.model = None
        self.processor = None
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return
        logger.info("Loading vision model (BLIP2)")
        try:
            from transformers import Blip2ForConditionalGeneration, Blip2Processor

            self.processor = Blip2Processor.from_pretrained(settings.vision_model_path)
            self.model = Blip2ForConditionalGeneration.from_pretrained(
                settings.vision_model_path,
                torch_dtype=torch.float16 if self.device == "cuda" else torch.float32,
            )
            self.model = self.model.to(self.device)
            self._initialized = True
            logger.info("Vision model loaded")
        except Exception as e:
            logger.error("Failed to load vision model", error=str(e))
            # Don't raise - allow service to run without vision
            logger.warning("Vision service will return placeholder responses")

    async def describe_image(
        self, image_data: bytes, prompt: Optional[str] = None
    ) -> dict:
        """Generate a caption/description for an image"""
        start = time.time()

        if not self._initialized or self.model is None:
            # Return placeholder if model not loaded
            return {
                "description": "Vision model not initialized. Please check model path configuration.",
                "confidence": 0.0,
                "generation_time": time.time() - start,
            }

        try:
            # Load image from bytes
            image = Image.open(io.BytesIO(image_data)).convert("RGB")

            # Prepare inputs
            if prompt:
                inputs = self.processor(image, text=prompt, return_tensors="pt").to(
                    self.device
                )
            else:
                inputs = self.processor(image, return_tensors="pt").to(self.device)

            # Generate caption
            with torch.no_grad():
                generated_ids = self.model.generate(
                    **inputs, max_new_tokens=100, num_beams=5, early_stopping=True
                )

            # Decode the generated text
            description = self.processor.decode(
                generated_ids[0], skip_special_tokens=True
            )

            return {
                "description": description.strip(),
                "confidence": 0.85,  # BLIP2 doesn't provide confidence scores directly
                "generation_time": time.time() - start,
            }

        except Exception as e:
            logger.error("Image description failed", error=str(e))
            return {
                "description": f"Error describing image: {str(e)}",
                "confidence": 0.0,
                "generation_time": time.time() - start,
            }

    async def describe_video(self, video_data: bytes, num_frames: int = 8) -> dict:
        """Generate a description for a video by sampling frames"""
        start = time.time()

        if not self._initialized or self.model is None:
            return {
                "description": "Vision model not initialized. Please check model path configuration.",
                "frame_count": 0,
                "generation_time": time.time() - start,
            }

        try:
            import tempfile

            import cv2
            import numpy as np

            # Save video to temp file
            with tempfile.NamedTemporaryFile(suffix=".mp4", delete=False) as tmp:
                tmp.write(video_data)
                tmp_path = tmp.name

            # Open video and extract frames
            cap = cv2.VideoCapture(tmp_path)
            total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))

            if total_frames == 0:
                cap.release()
                return {
                    "description": "Could not read video frames",
                    "frame_count": 0,
                    "generation_time": time.time() - start,
                }

            # Sample frames evenly throughout the video
            frame_indices = np.linspace(0, total_frames - 1, num_frames, dtype=int)
            frames = []

            for idx in frame_indices:
                cap.set(cv2.CAP_PROP_POS_FRAMES, idx)
                ret, frame = cap.read()
                if ret:
                    # Convert BGR to RGB
                    frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                    frames.append(Image.fromarray(frame_rgb))

            cap.release()

            # Clean up temp file
            import os

            os.unlink(tmp_path)

            if not frames:
                return {
                    "description": "No frames could be extracted from video",
                    "frame_count": 0,
                    "generation_time": time.time() - start,
                }

            # Generate descriptions for each sampled frame
            descriptions = []
            for frame in frames:
                inputs = self.processor(frame, return_tensors="pt").to(self.device)

                with torch.no_grad():
                    generated_ids = self.model.generate(
                        **inputs, max_new_tokens=50, num_beams=3, early_stopping=True
                    )

                desc = self.processor.decode(generated_ids[0], skip_special_tokens=True)
                descriptions.append(desc.strip())

            # Combine descriptions into a coherent summary
            # Use the most common elements or create a timeline
            unique_descriptions = list(
                dict.fromkeys(descriptions)
            )  # Remove duplicates preserving order

            if len(unique_descriptions) == 1:
                combined = unique_descriptions[0]
            else:
                combined = "Video shows: " + "; ".join(unique_descriptions[:4])

            return {
                "description": combined,
                "frame_count": len(frames),
                "generation_time": time.time() - start,
            }

        except Exception as e:
            logger.error("Video description failed", error=str(e))
            return {
                "description": f"Error describing video: {str(e)}",
                "frame_count": 0,
                "generation_time": time.time() - start,
            }

    async def answer_question(self, image_data: bytes, question: str) -> dict:
        """Visual question answering - ask a question about an image"""
        # Use describe_image with the question as a prompt
        return await self.describe_image(image_data, prompt=question)


_service = None


def get_vision_service():
    global _service
    if _service is None:
        _service = VisionService()
    return _service
