import io
import tempfile
import time
import os
import urllib.parse
from datetime import datetime
from typing import Optional

import httpx
from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("speech_service")


class SpeechService:
    def __init__(self):
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return

        logger.info("Speech service ready (external providers: OpenAI/Google)")
        self._initialized = True

    async def generate(
        self,
        prompt: str,
        voice: Optional[str] = None,
        language: Optional[str] = None,
    ) -> dict:
        """Generate speech audio from text"""
        if not self._initialized:
            self.initialize()

        start = time.time()
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        filename = f"{timestamp}_{hash(prompt) & 0xFFFFFF:06x}.wav"
        output_path = settings.output_dir / "audio" / filename

        # Prefer OpenAI/Groq for high quality/speed if configured
        if settings.openai_api_key:
            logger.info("Generating speech via OpenAI API")
            try:
                async with httpx.AsyncClient() as client:
                    response = await client.post(
                        "https://api.openai.com/v1/audio/speech",
                        headers={"Authorization": f"Bearer {settings.openai_api_key}"},
                        json={
                            "model": "tts-1",
                            "input": prompt,
                            "voice": voice or "alloy",
                        },
                        timeout=30.0,
                    )
                    response.raise_for_status()
                    with open(output_path, "wb") as f:
                        f.write(response.content)

                generation_time = time.time() - start
                return {
                    "status": "completed",
                    "file_path": f"/outputs/audio/{filename}",
                    "generation_time": generation_time,
                    "provider": "openai",
                }
            except Exception as e:
                logger.error(
                    "OpenAI speech generation failed, falling back", error=str(e)
                )

        # Fallback: Google Translate TTS (free, no API key needed)
        try:
            logger.info("Generating speech via Google Translate TTS")
            lang = language or "pt-BR"
            google_url = f"https://translate.google.com/translate_tts?ie=UTF-8&q={urllib.parse.quote(prompt)}&tl={lang}&client=tw-ob"
            async with httpx.AsyncClient() as client:
                response = await client.get(google_url, timeout=30.0)
                response.raise_for_status()
                with open(output_path, "wb") as f:
                    f.write(response.content)

            generation_time = time.time() - start
            return {
                "status": "completed",
                "file_path": f"/outputs/audio/{filename}",
                "generation_time": generation_time,
                "provider": "google-translate",
            }
        except Exception as e:
            logger.warning("Google Translate TTS failed", error=str(e))

        logger.error("No TTS provider available")
        return {
            "status": "error",
            "error": "No TTS provider initialized",
            "file_path": None,
            "generation_time": time.time() - start,
        }

    async def to_text(self, audio_data: bytes) -> dict:
        """Convert speech audio to text using Groq or OpenAI transcription"""
        if not self._initialized:
            self.initialize()

        start = time.time()

        # 1. Try Groq (Ultra-fast Whisper)
        if settings.groq_api_key:
            logger.info("Transcribing via Groq Cloud")
            try:
                # Save to temp file for Groq API
                with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
                    tmp.write(audio_data)
                    tmp_path = tmp.name

                async with httpx.AsyncClient() as client:
                    with open(tmp_path, "rb") as audio_file:
                        files = {
                            "file": (
                                os.path.basename(tmp_path),
                                audio_file,
                                "audio/wav",
                            )
                        }
                        data = {"model": "whisper-large-v3-turbo"}
                        response = await client.post(
                            "https://api.groq.com/openai/v1/audio/transcriptions",
                            headers={
                                "Authorization": f"Bearer {settings.groq_api_key}"
                            },
                            files=files,
                            data=data,
                            timeout=30.0,
                        )
                    response.raise_for_status()
                    result = response.json()

                os.unlink(tmp_path)
                return {
                    "text": result["text"].strip(),
                    "language": result.get("language", "auto"),
                    "confidence": 0.99,
                    "provider": "groq",
                }
            except Exception as e:
                logger.error("Groq transcription failed, falling back", error=str(e))

        # 2. Try OpenAI
        if settings.openai_api_key:
            logger.info("Transcribing via OpenAI API")
            try:
                with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
                    tmp.write(audio_data)
                    tmp_path = tmp.name

                async with httpx.AsyncClient() as client:
                    with open(tmp_path, "rb") as audio_file:
                        files = {
                            "file": (
                                os.path.basename(tmp_path),
                                audio_file,
                                "audio/wav",
                            )
                        }
                        data = {"model": "whisper-1"}
                        response = await client.post(
                            "https://api.openai.com/v1/audio/transcriptions",
                            headers={
                                "Authorization": f"Bearer {settings.openai_api_key}"
                            },
                            files=files,
                            data=data,
                            timeout=30.0,
                        )
                    response.raise_for_status()
                    result = response.json()

                os.unlink(tmp_path)
                return {
                    "text": result["text"].strip(),
                    "language": result.get("language", "auto"),
                    "confidence": 0.99,
                    "provider": "openai",
                }
            except Exception as e:
                logger.error("OpenAI transcription failed, falling back", error=str(e))

        # 3. No local fallback available
        return {"text": "", "error": "No STT provider available"}

    async def detect_language(self, audio_data: bytes) -> dict:
        """Detect language (simplified to reuse to_text if needed)"""
        # Just use to_text and return the language field
        result = await self.to_text(audio_data)
        return {
            "language": result.get("language"),
            "confidence": result.get("confidence", 0.0),
        }


_service = None


def get_speech_service():
    global _service
    if _service is None:
        _service = SpeechService()
    return _service
