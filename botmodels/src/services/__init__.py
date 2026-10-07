"""Service layer.

Submodules are imported lazily so that importing one service (or its backends)
does not pull in every dependency of every other. This keeps `src.services.backends`
importable without the ML and HTTP stack, which the mode/backend unit tests rely on.
"""

from typing import Any

__all__ = [
    "ImageService",
    "get_image_service",
    "MusicService",
    "get_music_service",
    "VideoService",
    "get_video_service",
    "SpeechService",
    "get_speech_service",
    "VisionService",
    "get_vision_service",
]

_EXPORTS = {
    "ImageService": "image_service",
    "get_image_service": "image_service",
    "MusicService": "music_service",
    "get_music_service": "music_service",
    "VideoService": "video_service",
    "get_video_service": "video_service",
    "SpeechService": "speech_service",
    "get_speech_service": "speech_service",
    "VisionService": "vision_service",
    "get_vision_service": "vision_service",
}


def __getattr__(name: str) -> Any:
    module_name = _EXPORTS.get(name)
    if module_name is None:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

    from importlib import import_module

    return getattr(import_module(f".{module_name}", __name__), name)


def __dir__() -> list[str]:
    return sorted(__all__)