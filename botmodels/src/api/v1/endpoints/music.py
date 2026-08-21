from fastapi import APIRouter, Depends, HTTPException, Query
from fastapi.responses import StreamingResponse

from ....schemas.generation import (
    MusicFormatRequest,
    MusicGenerateRequest,
    MusicJobCreated,
    MusicJobStatus,
)
from ....services.music_service import (
    MusicService,
    MusicServiceError,
    get_music_service,
)
from ...dependencies import verify_api_key

router = APIRouter(prefix="/music", tags=["Music"])


def service_error(exc: MusicServiceError) -> HTTPException:
    return HTTPException(status_code=502, detail=str(exc))


@router.get("/health")
async def music_health(
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    return await service.health()


@router.get("/models")
async def music_models(
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    try:
        return {"models": await service.models()}
    except MusicServiceError as exc:
        raise service_error(exc) from exc


@router.post("/generate", response_model=MusicJobCreated)
async def generate_music(
    request: MusicGenerateRequest,
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    try:
        return MusicJobCreated(**(await service.generate(request)))
    except MusicServiceError as exc:
        raise service_error(exc) from exc


@router.get("/jobs/{job_id}", response_model=MusicJobStatus)
async def music_job_status(
    job_id: str,
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    if not job_id or len(job_id) > 160:
        raise HTTPException(status_code=400, detail="Invalid job identifier")
    try:
        return MusicJobStatus(**(await service.status(job_id)))
    except MusicServiceError as exc:
        raise service_error(exc) from exc


@router.post("/format")
async def format_music_input(
    request: MusicFormatRequest,
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    payload = request.model_dump(exclude_none=True)
    payload["param_obj"] = {
        key: payload.pop(key)
        for key in ("duration", "bpm", "key_scale", "time_signature")
        if key in payload
    }
    try:
        return await service.format_input(payload)
    except MusicServiceError as exc:
        raise service_error(exc) from exc


@router.get("/audio")
async def music_audio(
    path: str = Query(..., min_length=1, max_length=4096),
    api_key: str = Depends(verify_api_key),
    service: MusicService = Depends(get_music_service),
):
    try:
        content_type, content_length, body = await service.stream_audio(path)
    except MusicServiceError as exc:
        raise service_error(exc) from exc

    headers = {}
    if content_length is not None:
        headers["Content-Length"] = str(content_length)
    return StreamingResponse(body, media_type=content_type, headers=headers)
