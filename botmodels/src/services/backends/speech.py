"""Speech backends: local-first TTS and STT.

min tier: Piper (ONNX VITS) TTS + faster-whisper int8 STT — both CPU-only.
max tier: Kokoro-82M TTS + Qwen3-ASR STT.

Remote providers are opt-in only (settings.allow_remote_speech). In min mode
botmodels never calls a cloud endpoint; it returns 503 naming the missing local
model instead, so audio never leaves the host by default.
"""

import io
import os
import wave
from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ..errors import BackendNotLoadedError
from .base import Backend

logger = get_logger("backends.speech")


class PiperTtsBackend(Backend):
    """Piper TTS: per-voice VITS exported to ONNX. CPU-first.

    Licence note: the original rhasspy/piper was MIT (archived); the successor
    OHF-Voice/piper1-gpl is GPL-3.0. botmodels runs as a separate process, so
    this is a process boundary rather than linked copyleft.
    """

    name = "piper"
    capability = "tts"

    def __init__(self) -> None:
        super().__init__()
        self.voice = None
        self.sample_rate = 22050

    def is_available(self) -> bool:
        try:
            import piper  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import os as _os

        from piper import PiperVoice

        model_path = _os.environ.get(
            "BOTMODELS_TTS_PATH", "./models/piper/en_US-lessac-medium.onnx"
        )
        logger.info("Loading Piper TTS backend", path=model_path)

        self.voice = PiperVoice.load(model_path)
        self.sample_rate = getattr(self.voice.config, "sample_rate", 22050)

    def synthesize(self, text: str, voice: Optional[str] = None) -> bytes:
        """Return WAV bytes."""
        self.ensure_loaded()
        if self.voice is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        chunks = list(self.voice.synthesize(text))

        buffer = io.BytesIO()
        with wave.open(buffer, "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(self.sample_rate)
            for chunk in chunks:
                audio = getattr(chunk, "audio_float_array", chunk)
                if isinstance(audio, (bytes, bytearray)):
                    handle.writeframes(bytes(audio))
                else:
                    handle.writeframes((float(audio) * 32767).astype("int16").tobytes())
        return buffer.getvalue()


class KokoroTtsBackend(Backend):
    """Kokoro-82M: 82M params, CPU-only, Apache 2.0. max tier default."""

    name = "kokoro"
    capability = "tts"

    def __init__(self) -> None:
        super().__init__()
        self.model = None

    def is_available(self) -> bool:
        try:
            import kokoro  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        from kokoro import KPipeline

        path = settings.tts_model_path
        logger.info("Loading Kokoro TTS backend", path=path)
        self.model = KPipeline(lang_code="a")

    def synthesize(self, text: str, voice: Optional[str] = None) -> bytes:
        self.ensure_loaded()
        if self.model is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import io
        import wave

        speaker = voice or "af_heart"
        audio_chunks = [result.audio for result in self.model(text, voice=speaker)]

        import numpy as np

        merged = np.concatenate(audio_chunks)

        buffer = io.BytesIO()
        with wave.open(buffer, "wb") as handle:
            handle.setnchannels(1)
            handle.setsampwidth(2)
            handle.setframerate(24000)
            handle.writeframes((merged * 32767).astype("int16").tobytes())
        return buffer.getvalue()


class FasterWhisperSttBackend(Backend):
    """faster-whisper (CTranslate2) int8 on CPU. min tier default."""

    name = "faster-whisper"
    capability = "stt"

    def __init__(self) -> None:
        super().__init__()
        self.model = None

    def is_available(self) -> bool:
        try:
            from faster_whisper import WhisperModel  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        from faster_whisper import WhisperModel

        size = os.environ.get("BOTMODELS_STT_MODEL", "small")
        device = settings.device or "cpu"
        compute = "float16" if device == "cuda" else "int8"

        logger.info(
            "Loading faster-whisper STT backend", size=size, device=device, compute=compute
        )
        self.model = WhisperModel(size, device=device, compute_type=compute)

    def transcribe(self, audio_path: str, language: Optional[str] = None) -> dict:
        self.ensure_loaded()
        if self.model is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        segments, info = self.model.transcribe(
            audio_path, language=language, beam_size=5
        )
        text = " ".join(segment.text.strip() for segment in segments).strip()
        return {
            "text": text,
            # faster-whisper's own detection, not a fabricated constant.
            "language": getattr(info, "language", None),
        }


class Qwen3AsrSttBackend(Backend):
    """Qwen3-ASR: 52 languages, best batch WER. max tier default. Apache 2.0."""

    name = "qwen3-asr"
    capability = "stt"

    def __init__(self) -> None:
        super().__init__()
        self.model = None
        self.processor = None

    def is_available(self) -> bool:
        try:
            import torch  # noqa: F401
            from transformers import AutoProcessor  # noqa: F401

            return True
        except ImportError:
            return False

    def _load(self) -> None:
        import torch
        from transformers import AutoModelForSpeechSeq2Seq, AutoProcessor

        path = settings.stt_model_path
        logger.info("Loading Qwen3-ASR backend", path=path)

        device = settings.device or "cpu"
        self.processor = AutoProcessor.from_pretrained(path)
        self.model = AutoModelForSpeechSeq2Seq.from_pretrained(
            path,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32,
            low_cpu_mem_usage=True,
        ).to(device)

    def transcribe(self, audio_path: str, language: Optional[str] = None) -> dict:
        self.ensure_loaded()
        if self.model is None or self.processor is None:  # pragma: no cover
            raise BackendNotLoadedError(self.capability, self.name)

        import numpy as np

        import librosa

        audio, _ = librosa.load(audio_path, sr=16000)
        inputs = self.processor(
            audio, sampling_rate=16000, return_tensors="pt"
        ).to(settings.device or "cpu")

        import torch

        with torch.no_grad():
            generated = self.model.generate(**inputs, max_new_tokens=256)

        text = self.processor.batch_decode(generated, skip_special_tokens=True)[0].strip()
        return {"text": text, "language": language}


TTS_BACKENDS: dict[str, type[Backend]] = {
    "piper": PiperTtsBackend,
    "kokoro": KokoroTtsBackend,
}

STT_BACKENDS: dict[str, type[Backend]] = {
    "faster-whisper": FasterWhisperSttBackend,
    "qwen3-asr": Qwen3AsrSttBackend,
}