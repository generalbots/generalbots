import io
import tempfile
import time
import os
import urllib.parse
from datetime import datetime
from pathlib import Path
from typing import Optional

import httpx
from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("speech_service")


class SpeechService:
    def __init__(self):
        self.tts_model = None
        self.whisper_model = None
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return

        # We only need local models if external providers are NOT configured
        if not settings.groq_api_key and not settings.openai_api_key:
            logger.info(
                "External providers not configured, loading local speech models"
            )
            try:
                # Load TTS model (Coqui TTS)
                self._load_tts_model()

                # Load Whisper model for speech-to-text
                self._load_whisper_model()
            except Exception as e:
                logger.error("Failed to load local speech models", error=str(e))
        else:
            logger.info("External speech providers detected (Groq/OpenAI)")

        self._initialized = True

    def _load_tts_model(self):
        """Load TTS model for text-to-speech generation"""
        try:
            from TTS.api import TTS

            # Use a lightweight model for low-RAM systems (4GB)
            self.tts_model = TTS(
                model_name="tts_models/multilingual/multi-dataset/xtts_v2",
                progress_bar=False,
                gpu=(self.device == "cuda"),
            )
            logger.info("Local TTS model loaded (xtts_v2)")
        except Exception as e:
            logger.warning("XTTS failed, trying lighter model", error=str(e))
            try:
                self.tts_model = TTS(
                    model_name="tts_models/en/ljspeech/tacotron2-DDC",
                    progress_bar=False,
                    gpu=False,
                )
                logger.info("Local TTS model loaded (tacotron2)")
            except Exception as e2:
                logger.warning("Local TTS model not available", error=str(e2))
                self.tts_model = None

    def _load_whisper_model(self):
        """Load Whisper model for speech-to-text"""
        try:
            import whisper

            # Use base model for balance of speed and accuracy
            model_size = "base"
            if Path(settings.whisper_model_path).exists():
                self.whisper_model = whisper.load_model(
                    model_size, download_root=settings.whisper_model_path
                )
            else:
                self.whisper_model = whisper.load_model(model_size)
            logger.info("Local Whisper model loaded", model=model_size)
        except Exception as e:
            logger.warning("Local Whisper model not available", error=str(e))
            self.whisper_model = None

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

        if self.tts_model is None:
            logger.error("No TTS provider available")
            return {
                "status": "error",
                "error": "No TTS provider initialized",
                "file_path": None,
                "generation_time": time.time() - start,
            }

        try:
            logger.info("Generating speech via local model")
            self.tts_model.tts_to_file(
                text=prompt,
                file_path=str(output_path),
            )

            generation_time = time.time() - start
            return {
                "status": "completed",
                "file_path": f"/outputs/audio/{filename}",
                "generation_time": generation_time,
                "provider": "local",
            }

        except Exception as e:
            logger.error("Local speech generation failed", error=str(e))
            return {
                "status": "error",
                "error": str(e),
                "file_path": None,
                "generation_time": time.time() - start,
            }

    async def to_text(self, audio_data: bytes) -> dict:
        """Convert speech audio to text using Whisper (External or Local)"""
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

        # 3. Fallback to Local Whisper
        if self.whisper_model is None:
            return {"text": "", "error": "No STT provider available"}

        try:
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
                tmp.write(audio_data)
                tmp_path = tmp.name

            logger.info("Transcribing via local model")
            result = self.whisper_model.transcribe(tmp_path)
            os.unlink(tmp_path)

            return {
                "text": result["text"].strip(),
                "language": result.get("language", "auto"),
                "confidence": 0.95,
                "provider": "local",
            }
        except Exception as e:
            return {"text": "", "error": str(e)}

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
