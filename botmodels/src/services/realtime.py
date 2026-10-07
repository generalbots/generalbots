"""Local realtime speech engines (offline sovereignty).

Transcription and synthesis execute entirely on-premises; no audio ever
leaves the host machine. Engine libraries are optional: when a library is
absent the factory returns ``None`` and the API layer answers with a JSON
error frame instead of failing the socket.
"""

import os
import re
from typing import Iterator, List, Optional

import numpy as np

from ..core.logging import get_logger

logger = get_logger("realtime")

TARGET_SAMPLE_RATE = 16000
WINDOW_SECONDS = 0.5
SILENCE_THRESHOLD_MS = 700.0
ENERGY_GATE = 0.01
PCM_FRAME_BYTES = 4096

try:  # pragma: no cover - optional dependency
    from faster_whisper import WhisperModel

    HAS_FASTER_WHISPER = True
except ImportError:
    WhisperModel = None
    HAS_FASTER_WHISPER = False

try:  # pragma: no cover - optional dependency
    from piper import PiperVoice

    HAS_PIPER = True
except ImportError:
    PiperVoice = None
    HAS_PIPER = False


def pcm_to_float32(audio: bytes) -> np.ndarray:
    """Decode signed 16-bit little-endian PCM into normalized float samples."""
    return np.frombuffer(audio, dtype=np.int16).astype(np.float32) / 32768.0


def rms_energy(audio: bytes) -> float:
    """Return the root mean square of a PCM frame (normalized range)."""
    if not audio:
        return 0.0
    samples = pcm_to_float32(audio)
    return float(np.sqrt(np.mean(np.square(samples))))


def frame_duration_ms(frame: bytes, sample_rate: int) -> float:
    """Duration of a 16-bit mono PCM frame in milliseconds."""
    if sample_rate <= 0:
        return 0.0
    return (len(frame) / 2.0) / sample_rate * 1000.0


def split_sentences(text: str) -> List[str]:
    """Split text into sentences on terminal punctuation and line breaks."""
    parts = re.split(r"(?<=[.!?;:\n])\s+", text.strip())
    return [part.strip() for part in parts if part.strip()]


def resample_linear(samples: np.ndarray, origin_rate: int, target_rate: int) -> np.ndarray:
    """Naive linear-interpolation resampler between arbitrary sample rates."""
    if origin_rate == target_rate or samples.size == 0:
        return samples.astype(np.float32)
    duration = samples.shape[0] / float(origin_rate)
    target_length = max(int(duration * target_rate), 1)
    source = np.linspace(0.0, duration, samples.shape[0], endpoint=False)
    grid = np.linspace(0.0, duration, target_length, endpoint=False)
    return np.interp(grid, source, samples).astype(np.float32)


class SttEngine:
    """Base contract for streaming speech-to-text engines."""

    name: str = "abstract"

    def transcribe_chunk(self, audio: bytes, sample_rate: int) -> str:
        """Return a partial transcript for one PCM window."""
        raise NotImplementedError


class TtsEngine:
    """Base contract for streaming text-to-speech engines."""

    name: str = "abstract"

    def synthesize_stream(self, text: str) -> Iterator[bytes]:
        """Yield PCM byte frames synthesizing the supplied text."""
        raise NotImplementedError


class LocalWhisperStt(SttEngine):
    """Faster-whisper transcription guarded by a VAD-lite energy gate."""

    name = "local-whisper"

    def __init__(self) -> None:
        self.model_size: str = os.getenv("BOTMODELS_STT_MODEL", "small")
        self._model: Optional["WhisperModel"] = None

    def _ensure_model(self) -> bool:
        if self._model is not None:
            return True
        if not HAS_FASTER_WHISPER:
            return False
        device = "cuda" if os.getenv("BOTMODELS_DEVICE", "cpu") == "cuda" else "cpu"
        compute = "float16" if device == "cuda" else "int8"
        logger.info("Loading faster-whisper model", model=self.model_size, device=device)
        self._model = WhisperModel(self.model_size, device=device, compute_type=compute)
        return True

    def transcribe_chunk(self, audio: bytes, sample_rate: int) -> str:
        if rms_energy(audio) < ENERGY_GATE:
            return ""
        if not self._ensure_model():
            raise RuntimeError("faster_whisper library is not installed")
        samples = pcm_to_float32(audio)
        if sample_rate != TARGET_SAMPLE_RATE:
            samples = resample_linear(samples, sample_rate, TARGET_SAMPLE_RATE)
        segments, _info = self._model.transcribe(samples, language=None, beam_size=1)
        return " ".join(segment.text.strip() for segment in segments).strip()


class LocalTts(TtsEngine):
    """Piper synthesis emitting framed 16 kHz PCM split by sentence."""

    name = "local-piper"

    def __init__(self) -> None:
        self.voice_name: str = os.getenv("BOTMODELS_TTS_MODEL", "en_US-lessac-medium")
        self._voice: Optional["PiperVoice"] = None
        self._voice_rate: int = TARGET_SAMPLE_RATE

    def _ensure_voice(self) -> bool:
        if self._voice is not None:
            return True
        if not HAS_PIPER:
            return False
        default_path = f"./models/piper/{self.voice_name}.onnx"
        model_path = os.getenv("BOTMODELS_TTS_PATH", default_path)
        logger.info("Loading piper voice", voice=model_path)
        self._voice = PiperVoice.load(model_path)
        rate = getattr(getattr(self._voice, "config", None), "sample_rate", TARGET_SAMPLE_RATE)
        self._voice_rate = int(rate or TARGET_SAMPLE_RATE)
        return True

    def synthesize_stream(self, text: str) -> Iterator[bytes]:
        if not self._ensure_voice():
            raise RuntimeError("piper library is not installed")
        for sentence in split_sentences(text):
            yield from self._synthesize_sentence(sentence)

    def _synthesize_sentence(self, sentence: str) -> Iterator[bytes]:
        pending = bytearray()
        try:
            for chunk in self._voice.synthesize(sentence):
                if isinstance(chunk, (bytes, bytearray)):
                    data = bytes(chunk)
                else:
                    data = getattr(chunk, "audio_int16_bytes", None)
                if not data:
                    continue
                samples = np.frombuffer(data, dtype=np.int16)
                resampled = resample_linear(samples, self._voice_rate, TARGET_SAMPLE_RATE)
                pending.extend(resampled.astype(np.int16).tobytes())
                while len(pending) >= PCM_FRAME_BYTES:
                    yield bytes(pending[:PCM_FRAME_BYTES])
                    del pending[:PCM_FRAME_BYTES]
        except Exception as exc:
            logger.error("Piper synthesis failed", error=str(exc))
            raise RuntimeError(f"Piper synthesis failed: {exc}") from exc
        if pending:
            yield bytes(pending)


class EngineFactory:
    """Select engines through BOTMODELS_STT_ENGINE / BOTMODELS_TTS_ENGINE."""

    @staticmethod
    def stt() -> Optional[SttEngine]:
        choice = os.getenv("BOTMODELS_STT_ENGINE", "local").strip().lower()
        if choice == "none":
            return None
        if not HAS_FASTER_WHISPER:
            logger.warning("STT engine 'local' requested but faster_whisper is unavailable")
            return None
        return LocalWhisperStt()

    @staticmethod
    def tts() -> Optional[TtsEngine]:
        choice = os.getenv("BOTMODELS_TTS_ENGINE", "local").strip().lower()
        if choice == "none":
            return None
        if not HAS_PIPER:
            logger.warning("TTS engine 'local' requested but piper is unavailable")
            return None
        return LocalTts()
