import os
import time
from typing import Optional

from ..core.config import settings
from ..core.logging import get_logger
from .backends import registry
from .errors import BackendNotLoadedError, BotModelsError

logger = get_logger("speech_service")


class RemoteProvidersDisabled(BotModelsError):
    """A cloud speech provider was requested while remote calls are disabled.

    min mode never calls out: botmodels is the offline path, so failing loudly
    here is the whole point. The previous implementation silently fell back to
    the unauthenticated Google Translate TTS endpoint.
    """


class SpeechService:
    """Local-first TTS and STT.

    Resolution order is local -> optional remote. Remote providers are used
    only when settings.allow_remote_speech is explicitly enabled.
    """

    def __init__(self) -> None:
        self.tts = registry.get("tts")
        self.stt = registry.get("stt")

    # ---- TTS -------------------------------------------------------------

    async def generate_speech(
        self,
        prompt: str,
        voice: Optional[str] = None,
        language: Optional[str] = None,
    ) -> dict:
        start = time.time()

        try:
            audio = self.tts.synthesize(prompt, voice=voice)
            return {
                "status": "completed",
                "audio": audio,
                "backend": self.tts.name,
                "generation_time": time.time() - start,
            }
        except BotModelsError as exc:
            logger.warning("Local TTS unavailable", backend=self.tts.name, error=str(exc))

        if settings.allow_remote_speech:
            return await self._remote_tts(prompt, voice)

        raise BackendNotLoadedError(
            "tts",
            self.tts.name,
            "local TTS not loaded and remote providers are disabled "
            "(set ALLOW_REMOTE_SPEECH=true to permit cloud calls)",
        )

    async def _remote_tts(self, prompt: str, voice: Optional[str]) -> dict:
        """OpenAI TTS. Opt-in only.

        The Google Translate fallback is gone: it was unauthenticated and sent
        user text to a third party with no operator consent.
        """
        if not settings.openai_api_key:
            raise RemoteProvidersDisabled("OPENAI_API_KEY not set")

        import httpx

        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                "https://api.openai.com/v1/audio/speech",
                headers={
                    "Authorization": f"Bearer {settings.openai_api_key}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": os.environ.get("OPENAI_TTS_MODEL", "tts-1"),
                    "voice": voice or "alloy",
                    "input": prompt,
                },
            )
            response.raise_for_status()
            return {
                "status": "completed",
                "audio": response.content,
                "backend": "openai",
            }

    # ---- STT -------------------------------------------------------------

    async def transcribe_audio(
        self,
        audio_path: str,
        language: Optional[str] = None,
    ) -> dict:
        start = time.time()

        try:
            result = self.stt.transcribe(audio_path, language=language)
            return {
                "text": result["text"],
                "language": result.get("language"),
                "backend": self.stt.name,
                "generation_time": time.time() - start,
            }
        except BotModelsError as exc:
            logger.warning("Local STT unavailable", backend=self.stt.name, error=str(exc))

        if settings.allow_remote_speech:
            return await self._remote_stt(audio_path)

        raise BackendNotLoadedError(
            "stt",
            self.stt.name,
            "local STT not loaded and remote providers are disabled "
            "(set ALLOW_REMOTE_SPEECH=true to permit cloud calls)",
        )

    async def _remote_stt(self, audio_path: str) -> dict:
        """Groq -> OpenAI. Opt-in only. Returns a clear error when both fail."""
        last_error: Optional[Exception] = None

        if settings.groq_api_key:
            try:
                return await self._groq_stt(audio_path)
            except Exception as exc:  # noqa: BLE001 - fall through to OpenAI
                last_error = exc
                logger.warning("Groq STT failed, trying OpenAI", error=str(exc))

        if settings.openai_api_key:
            try:
                return await self._openai_stt(audio_path)
            except Exception as exc:  # noqa: BLE001
                last_error = exc

        raise RemoteProvidersDisabled(
            f"no remote STT provider available: {last_error}"
        )

    async def _groq_stt(self, audio_path: str) -> dict:
        import httpx

        with open(audio_path, "rb") as handle:
            async with httpx.AsyncClient(timeout=60.0) as client:
                response = await client.post(
                    "https://api.groq.com/openai/v1/audio/transcriptions",
                    headers={"Authorization": f"Bearer {settings.groq_api_key}"},
                    files={"file": handle},
                    data={"model": "whisper-large-v3-turbo"},
                )
                response.raise_for_status()
                payload = response.json()
                return {
                    "text": payload.get("text", ""),
                    "language": payload.get("language"),
                    "backend": "groq",
                }

    async def _openai_stt(self, audio_path: str) -> dict:
        import httpx

        with open(audio_path, "rb") as handle:
            async with httpx.AsyncClient(timeout=60.0) as client:
                response = await client.post(
                    "https://api.openai.com/v1/audio/transcriptions",
                    headers={"Authorization": f"Bearer {settings.openai_api_key}"},
                    files={"file": handle},
                    data={"model": "whisper-1"},
                )
                response.raise_for_status()
                payload = response.json()
                return {
                    "text": payload.get("text", ""),
                    "language": payload.get("language"),
                    "backend": "openai",
                }

    async def detect_language(self, audio_path: str) -> dict:
        """Language detection from the model's own output.

        Never returns the literal "auto": an unresolved language is reported as
        None so callers can tell it apart from a real detection.
        """
        result = await self.transcribe_audio(audio_path)
        language = result.get("language")
        if language == "auto":
            language = None
        return {
            "language": language,
            "backend": result.get("backend"),
        }


_service = None


def get_speech_service():
    global _service
    if _service is None:
        _service = SpeechService()
    return _service