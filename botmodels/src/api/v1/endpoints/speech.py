import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from uuid import uuid4

from fastapi import APIRouter, Depends, File, HTTPException, UploadFile

from ....core.config import settings
from ....schemas.generation import (
    GenerationResponse,
    SpeechGenerateRequest,
    SpeechToTextResponse,
)
from ....services.errors import BotModelsError
from ....services.realtime_audio_service import get_realtime_audio_service
from ....services.speech_service import get_speech_service
from ...dependencies import verify_api_key

router = APIRouter(prefix="/speech", tags=["Speech"])

# Reject oversized uploads before they hit disk or a model.
MAX_AUDIO_BYTES = 50 * 1024 * 1024


def _suffix_from_magic(data: bytes) -> str:
    """Pick a file extension from the audio container magic bytes.

    The local backend returns WAV; the opt-in remote backend returns MP3.
    Writing an MP3 payload to a ``.wav`` file breaks players and downstream
    probes, so the extension follows the payload, not the backend name.
    """
    if data.startswith(b"RIFF"):
        return ".wav"
    if data.startswith(b"ID3"):
        return ".mp3"
    if data.startswith(b"OggS"):
        return ".ogg"
    if data.startswith(b"fLaC"):
        return ".flac"
    if len(data) > 1 and data[0] == 0xFF and (data[1] & 0xE0) == 0xE0:
        return ".mp3"
    return ".wav"


def _suffix_from_filename(name: str | None) -> str:
    suffix = Path(name or "").suffix.lower()
    if suffix in {".wav", ".mp3", ".m4a", ".flac", ".ogg", ".webm"}:
        return suffix
    return ".wav"


def _write_temp(data: bytes, suffix: str) -> Path:
    handle = tempfile.NamedTemporaryFile(delete=False, suffix=suffix)
    try:
        handle.write(data)
        handle.flush()
    finally:
        handle.close()
    return Path(handle.name)


async def _read_audio(file: UploadFile) -> bytes:
    data = await file.read()
    if not data:
        raise HTTPException(status_code=400, detail="Empty audio upload")
    if len(data) > MAX_AUDIO_BYTES:
        raise HTTPException(
            status_code=413,
            detail=f"Audio upload exceeds {MAX_AUDIO_BYTES // (1024 * 1024)} MiB",
        )
    return data


@router.post("/generate", response_model=GenerationResponse)
async def generate_speech(
    request: SpeechGenerateRequest,
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """Generate speech audio from text and persist it under ``outputs/audio``.

    The service returns raw bytes; the HTTP contract has always been a
    ``file_path`` (mirroring the image and video routes), so the route owns the
    persistence step.
    """
    try:
        result = await service.generate_speech(
            prompt=request.prompt,
            voice=request.voice,
            language=request.language,
        )
    except BotModelsError as exc:
        raise HTTPException(status_code=503, detail=str(exc)) from exc

    audio = result.get("audio")
    if not audio:
        raise HTTPException(status_code=502, detail="Speech backend returned no audio")

    suffix = _suffix_from_magic(audio)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    filename = f"{stamp}_{uuid4().hex[:8]}{suffix}"
    output_path = settings.output_dir / "audio" / filename
    output_path.write_bytes(audio)

    return GenerationResponse(
        status=result.get("status", "completed"),
        file_path=f"/outputs/audio/{filename}",
        generation_time=result.get("generation_time", time.time()),
    )


@router.post("/totext", response_model=SpeechToTextResponse)
async def speech_to_text(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """Transcribe an uploaded audio file.

    ``SpeechService.transcribe_audio`` takes a filesystem path; the previous
    route passed bytes to a method that never existed, so every call raised
    ``AttributeError``. The upload is materialized to a temporary file and
    removed in ``finally``.
    """
    data = await _read_audio(file)
    path = _write_temp(data, _suffix_from_filename(file.filename))
    try:
        result = await service.transcribe_audio(str(path))
    except BotModelsError as exc:
        raise HTTPException(status_code=503, detail=str(exc)) from exc
    finally:
        path.unlink(missing_ok=True)

    return SpeechToTextResponse(
        text=result.get("text", ""),
        language=result.get("language"),
    )


@router.post("/detect_language")
async def detect_language(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """Detect the spoken language of an uploaded audio file."""
    data = await _read_audio(file)
    path = _write_temp(data, _suffix_from_filename(file.filename))
    try:
        result = await service.detect_language(str(path))
    except BotModelsError as exc:
        raise HTTPException(status_code=503, detail=str(exc)) from exc
    finally:
        path.unlink(missing_ok=True)
    return result


@router.post("/realtime")
async def realtime_audio(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_realtime_audio_service),
):
    """Process audio with the optional speech-to-speech model.

    Kept as a one-shot endpoint: it is not a duplex stream. The streaming
    contracts live in ``/v1/audio/stt/stream`` and ``/v1/audio/tts/stream``.
    """
    audio_data = await _read_audio(file)
    result = await service.process_audio(audio_data)
    if result.get("status") == "error":
        raise HTTPException(status_code=503, detail=result.get("error", "realtime audio unavailable"))
    return result
