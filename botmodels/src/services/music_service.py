import json
from typing import Any, AsyncIterator, Optional
from urllib.parse import parse_qs, urlparse

import httpx

from ..core.config import settings
from ..core.logging import get_logger
from ..schemas.generation import MusicGenerateRequest, MusicTrack

logger = get_logger("music_service")


class MusicServiceError(RuntimeError):
    """A safe, user-facing failure returned by the ACE-Step service."""


class MusicService:
    """Stateless adapter for the official ACE-Step 1.5 asynchronous API."""

    def __init__(self):
        self.base_url = settings.acestep_api_url.rstrip("/")
        self.api_key = settings.acestep_api_key

    def _headers(self) -> dict[str, str]:
        headers = {"Accept": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers

    @staticmethod
    def _unwrap(response: httpx.Response) -> Any:
        try:
            payload = response.json()
        except ValueError as exc:
            raise MusicServiceError("ACE-Step returned an invalid response") from exc

        if not response.is_success:
            raise MusicServiceError("ACE-Step rejected the request")
        if isinstance(payload, dict) and payload.get("code", 200) != 200:
            raise MusicServiceError("ACE-Step could not process the request")
        if isinstance(payload, dict) and "data" in payload:
            return payload["data"]
        return payload

    async def health(self) -> dict[str, Any]:
        try:
            async with httpx.AsyncClient(
                timeout=settings.acestep_request_timeout
            ) as client:
                response = await client.get(
                    f"{self.base_url}/health", headers=self._headers()
                )
            data = self._unwrap(response)
            return {"healthy": True, "engine": "ACE-Step 1.5", "details": data}
        except (httpx.HTTPError, MusicServiceError) as exc:
            logger.warning("ACE-Step health check failed", error=str(exc))
            return {"healthy": False, "engine": "ACE-Step 1.5"}

    async def models(self) -> list[dict[str, Any]]:
        try:
            async with httpx.AsyncClient(
                timeout=settings.acestep_request_timeout
            ) as client:
                response = await client.get(
                    f"{self.base_url}/v1/models", headers=self._headers()
                )
            data = self._unwrap(response)
        except httpx.HTTPError as exc:
            raise MusicServiceError("ACE-Step is unavailable") from exc

        if isinstance(data, dict) and isinstance(data.get("models"), list):
            return data["models"]
        return []

    async def generate(self, request: MusicGenerateRequest) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "prompt": request.prompt.strip(),
            "lyrics": "" if request.instrumental else request.lyrics,
            "vocal_language": request.vocal_language,
            "audio_duration": request.duration,
            "inference_steps": request.inference_steps,
            "guidance_scale": request.guidance_scale,
            "batch_size": request.batch_size,
            "thinking": request.thinking,
            "use_format": request.enhance,
            "audio_format": request.audio_format,
            "use_random_seed": request.seed is None,
            "seed": request.seed if request.seed is not None else -1,
        }
        if request.simple_mode:
            payload.update(
                {
                    "sample_mode": True,
                    "sample_query": request.description.strip(),
                }
            )
        if request.bpm is not None:
            payload["bpm"] = request.bpm
        if request.key_scale:
            payload["key_scale"] = request.key_scale
        if request.time_signature:
            payload["time_signature"] = request.time_signature
        if request.model:
            payload["model"] = request.model

        try:
            async with httpx.AsyncClient(
                timeout=settings.acestep_request_timeout
            ) as client:
                response = await client.post(
                    f"{self.base_url}/release_task",
                    headers={**self._headers(), "Content-Type": "application/json"},
                    json=payload,
                )
            data = self._unwrap(response)
        except httpx.HTTPError as exc:
            raise MusicServiceError("ACE-Step is unavailable") from exc

        if not isinstance(data, dict) or not data.get("task_id"):
            raise MusicServiceError("ACE-Step did not return a job identifier")

        return {
            "job_id": str(data["task_id"]),
            "status": str(data.get("status", "queued")),
            "queue_position": data.get("queue_position"),
        }

    async def status(self, job_id: str) -> dict[str, Any]:
        try:
            async with httpx.AsyncClient(
                timeout=settings.acestep_request_timeout
            ) as client:
                response = await client.post(
                    f"{self.base_url}/query_result",
                    headers={**self._headers(), "Content-Type": "application/json"},
                    json={"task_id_list": [job_id]},
                )
            data = self._unwrap(response)
        except httpx.HTTPError as exc:
            raise MusicServiceError("ACE-Step is unavailable") from exc

        if not isinstance(data, list) or not data:
            raise MusicServiceError("Music job was not found")

        item = data[0] if isinstance(data[0], dict) else {}
        numeric_status = item.get("status", 0)
        if numeric_status == 2:
            return {
                "job_id": job_id,
                "status": "failed",
                "error": "Music generation failed",
                "tracks": [],
            }
        if numeric_status != 1:
            return {
                "job_id": job_id,
                "status": "running",
                "queue_position": item.get("queue_position"),
                "progress": item.get("progress"),
                "tracks": [],
            }

        tracks = self._parse_tracks(item.get("result"))
        if not tracks:
            logger.error(
                "ACE-Step completed without an audio file",
                job_id=job_id,
                progress_text=str(item.get("progress_text") or ""),
            )
            return {
                "job_id": job_id,
                "status": "failed",
                "error": "Music generation completed without an audio file",
                "tracks": [],
            }
        return {
            "job_id": job_id,
            "status": "succeeded",
            "tracks": [track.model_dump() for track in tracks],
        }

    @classmethod
    def _parse_tracks(cls, raw_result: Any) -> list[MusicTrack]:
        if isinstance(raw_result, str):
            try:
                raw_result = json.loads(raw_result)
            except json.JSONDecodeError as exc:
                raise MusicServiceError("ACE-Step returned invalid track data") from exc
        if isinstance(raw_result, dict):
            raw_result = [raw_result]
        if not isinstance(raw_result, list):
            return []

        tracks: list[MusicTrack] = []
        for item in raw_result:
            if not isinstance(item, dict):
                continue
            audio_path = cls._extract_audio_path(item.get("file"))
            if not audio_path:
                continue
            metas = item.get("metas") if isinstance(item.get("metas"), dict) else {}
            tracks.append(
                MusicTrack(
                    audio_path=audio_path,
                    prompt=str(item.get("prompt") or ""),
                    lyrics=str(item.get("lyrics") or ""),
                    duration=cls._number(metas.get("duration")),
                    bpm=cls._integer(metas.get("bpm")),
                    key_scale=str(metas.get("keyscale") or "") or None,
                    time_signature=str(metas.get("timesignature") or "") or None,
                    seed=str(item.get("seed_value") or "") or None,
                    model=str(item.get("dit_model") or "") or None,
                )
            )
        return tracks

    @staticmethod
    def _extract_audio_path(file_value: Any) -> Optional[str]:
        if not isinstance(file_value, str) or not file_value:
            return None
        parsed = urlparse(file_value)
        if parsed.path.endswith("/v1/audio"):
            values = parse_qs(parsed.query).get("path")
            return values[0] if values else None
        return file_value if MusicService._valid_audio_path(file_value) else None

    @staticmethod
    def _valid_audio_path(audio_path: str) -> bool:
        if not audio_path or len(audio_path) > 4096 or "\x00" in audio_path:
            return False
        normalized = audio_path.replace("\\", "/")
        if ".." in normalized.split("/"):
            return False
        windows_absolute = (
            len(normalized) > 3
            and normalized[0].isalpha()
            and normalized[1:3] == ":/"
        )
        return normalized.startswith("/") or windows_absolute

    @staticmethod
    def _number(value: Any) -> Optional[float]:
        try:
            return float(value) if value is not None else None
        except (TypeError, ValueError):
            return None

    @staticmethod
    def _integer(value: Any) -> Optional[int]:
        try:
            return int(value) if value is not None else None
        except (TypeError, ValueError):
            return None

    async def format_input(self, payload: dict[str, Any]) -> dict[str, Any]:
        try:
            async with httpx.AsyncClient(
                timeout=settings.acestep_audio_timeout
            ) as client:
                response = await client.post(
                    f"{self.base_url}/format_input",
                    headers={**self._headers(), "Content-Type": "application/json"},
                    json=payload,
                )
            data = self._unwrap(response)
        except httpx.HTTPError as exc:
            raise MusicServiceError("ACE-Step is unavailable") from exc
        return data if isinstance(data, dict) else {}

    async def stream_audio(
        self, audio_path: str
    ) -> tuple[str, Optional[int], AsyncIterator[bytes]]:
        if not self._valid_audio_path(audio_path):
            raise MusicServiceError("Invalid audio path")

        client = httpx.AsyncClient(timeout=settings.acestep_audio_timeout)
        request = client.build_request(
            "GET",
            f"{self.base_url}/v1/audio",
            headers=self._headers(),
            params={"path": audio_path},
        )
        try:
            response = await client.send(request, stream=True)
        except httpx.HTTPError as exc:
            await client.aclose()
            raise MusicServiceError("ACE-Step audio is unavailable") from exc
        if not response.is_success:
            await response.aclose()
            await client.aclose()
            raise MusicServiceError("ACE-Step audio is unavailable")

        async def iterator() -> AsyncIterator[bytes]:
            try:
                async for chunk in response.aiter_bytes():
                    yield chunk
            finally:
                await response.aclose()
                await client.aclose()

        content_type = response.headers.get("content-type", "audio/mpeg")
        content_length = self._integer(response.headers.get("content-length"))
        return content_type, content_length, iterator()


_service: Optional[MusicService] = None


def get_music_service() -> MusicService:
    global _service
    if _service is None:
        _service = MusicService()
    return _service
