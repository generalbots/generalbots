"""Regression tests for BotModels resource-exhaustion fixes.

Covers #2802 (bounded uploads), #2803 (bounded num_frames), #2804 (bounded
voice buffers/sample rate) and #2805 (audio path restricted to ACE-Step
outputs with an audio extension).
"""

import asyncio
import io

import pytest
from fastapi import HTTPException
from starlette.datastructures import UploadFile

from src.api.v1.endpoints import voice, vision
from src.api.v1.endpoints.uploads import read_upload
from src.services.music_service import MusicService


def _upload(payload: bytes) -> UploadFile:
    return UploadFile(file=io.BytesIO(payload), filename="payload.bin")


def test_read_upload_accepts_small_body():
    data = asyncio.run(read_upload(_upload(b"abc"), max_bytes=1024))
    assert data == b"abc"


def test_read_upload_rejects_oversized_body():
    upload = _upload(b"x" * (3 * 1024 * 1024))
    with pytest.raises(HTTPException) as exc:
        asyncio.run(read_upload(upload, max_bytes=1024 * 1024))
    assert exc.value.status_code == 413


def test_num_frames_is_bounded():
    assert vision.MAX_VIDEO_FRAMES == 32
    defaults = vision.describe_video.__defaults__ or ()
    bounded = False
    for default in defaults:
        metadata = getattr(default, "metadata", [])
        has_ge = any(getattr(constraint, "ge", None) == 1 for constraint in metadata)
        has_le = any(
            getattr(constraint, "le", None) == vision.MAX_VIDEO_FRAMES
            for constraint in metadata
        )
        if has_ge and has_le:
            bounded = True
    assert bounded, "describe_video must constrain num_frames with ge/le"


def test_voice_sample_rate_is_clamped():
    assert voice._bounded_sample_rate(1, 16000) == voice.MIN_SAMPLE_RATE
    assert voice._bounded_sample_rate(10_000_000, 16000) == voice.MAX_SAMPLE_RATE
    assert voice._bounded_sample_rate("bogus", 16000) == 16000


def test_voice_buffer_stays_bounded():
    buffer = bytearray()
    frame = b"\x01\x00" * 8000
    for _ in range(200):
        voice._append_frame(buffer, frame)
    assert len(buffer) <= voice.MAX_BUFFER_BYTES


def test_music_rejects_paths_outside_outputs():
    assert not MusicService._valid_audio_path("/etc/passwd")
    assert not MusicService._valid_audio_path("/tmp/secret.wav")
    assert not MusicService._valid_audio_path("/tmp/acestep/outputs/notes.txt")
    assert MusicService._valid_audio_path("/tmp/acestep/outputs/song.wav")
