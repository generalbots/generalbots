"""Route wiring tests for the speech API.

These run without any model installed: the SpeechService dependency is
replaced with a fake, so the tests fail if the route and the service contract
drift apart again (the 500s in issue 1683).
"""

import unittest
from pathlib import Path

from fastapi.testclient import TestClient

from src.api.v1.endpoints.speech import router as speech_router  # noqa: F401
from src.core.config import settings
from src.main import app
from src.services.errors import BackendNotLoadedError
from src.services.speech_service import get_speech_service

AUTH = {"X-API-Key": settings.api_key}
WAV = b"RIFF\x24\x00\x00\x00WAVEfmt "


class FakeSpeechService:
    def __init__(self):
        self.last_path = None

    async def generate_speech(self, prompt, voice=None, language=None):
        return {
            "status": "completed",
            "audio": WAV,
            "generation_time": 0.25,
            "backend": "fake",
        }

    async def transcribe_audio(self, audio_path, language=None):
        self.last_path = Path(audio_path)
        self.last_bytes = self.last_path.read_bytes()
        return {
            "text": "hello world",
            "language": "en",
            "backend": "fake",
            "generation_time": 0.1,
        }

    async def detect_language(self, audio_path):
        self.last_path = Path(audio_path)
        self.last_bytes = self.last_path.read_bytes()
        return {"language": "en", "backend": "fake"}


class FailingSpeechService:
    async def generate_speech(self, prompt, voice=None, language=None):
        raise BackendNotLoadedError("tts", "piper", "piper is not installed")

    async def transcribe_audio(self, audio_path, language=None):
        raise BackendNotLoadedError("stt", "faster-whisper", "faster_whisper is not installed")

    async def detect_language(self, audio_path):
        raise BackendNotLoadedError("stt", "faster-whisper", "faster_whisper is not installed")


class SpeechRouteTests(unittest.TestCase):
    def setUp(self):
        self.fake = FakeSpeechService()
        app.dependency_overrides[get_speech_service] = lambda: self.fake
        self.client = TestClient(app)
        self.created = []

    def tearDown(self):
        app.dependency_overrides.pop(get_speech_service, None)
        for path in self.created:
            Path(path).unlink(missing_ok=True)

    def test_generate_persists_wav_and_returns_file_path(self):
        response = self.client.post(
            "/api/speech/generate",
            headers=AUTH,
            json={"prompt": "hello"},
        )
        self.assertEqual(response.status_code, 200, response.text)
        body = response.json()
        self.assertEqual(body["status"], "completed")
        self.assertTrue(body["file_path"].startswith("/outputs/audio/"))
        self.assertTrue(body["file_path"].endswith(".wav"))
        saved = settings.output_dir / body["file_path"].removeprefix("/outputs/")
        self.addCleanup(saved.unlink, missing_ok=True)
        self.assertTrue(saved.exists())
        self.assertEqual(saved.read_bytes(), WAV)

    def test_totext_passes_a_real_file_to_the_service(self):
        response = self.client.post(
            "/api/speech/totext",
            headers=AUTH,
            files={"file": ("note.wav", WAV, "audio/wav")},
        )
        self.assertEqual(response.status_code, 200, response.text)
        self.assertEqual(response.json()["text"], "hello world")
        self.assertEqual(response.json()["language"], "en")
        self.assertEqual(self.fake.last_bytes, WAV)
        self.assertFalse(self.fake.last_path.exists(), "temporary upload was not removed")

    def test_detect_language_accepts_upload(self):
        response = self.client.post(
            "/api/speech/detect_language",
            headers=AUTH,
            files={"file": ("note.ogg", b"OggSfake", "audio/ogg")},
        )
        self.assertEqual(response.status_code, 200, response.text)
        self.assertEqual(response.json()["language"], "en")
        self.assertEqual(self.fake.last_bytes, b"OggSfake")

    def test_missing_backend_maps_to_503(self):
        app.dependency_overrides[get_speech_service] = lambda: FailingSpeechService()
        response = self.client.post(
            "/api/speech/totext",
            headers=AUTH,
            files={"file": ("note.wav", WAV, "audio/wav")},
        )
        self.assertEqual(response.status_code, 503)
        self.assertIn("faster-whisper", response.json()["detail"])

    def test_requires_api_key(self):
        response = self.client.post(
            "/api/speech/generate",
            json={"prompt": "hello"},
        )
        self.assertEqual(response.status_code, 422)  # missing required header


if __name__ == "__main__":
    unittest.main()
