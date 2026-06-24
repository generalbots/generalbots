from fastapi import APIRouter, Depends, File, UploadFile

from ....schemas.generation import (
    GenerationResponse,
    VideoDescribeResponse,
    VideoGenerateRequest,
)
from ....services.video_service import get_video_service
from ...dependencies import verify_api_key

router = APIRouter(prefix="/video", tags=["Video"])


@router.post("/generate", response_model=GenerationResponse)
async def generate_video(
    request: VideoGenerateRequest,
    api_key: str = Depends(verify_api_key),
    service=Depends(get_video_service),
):
    """
    Generate a video from a text prompt.

    Args:
        request: Video generation parameters including prompt, frames, fps, etc.
        api_key: API key for authentication
        service: Video service instance

    Returns:
        GenerationResponse with file path and generation time
    """
    result = await service.generate(
        prompt=request.prompt,
        num_frames=request.num_frames,
        fps=request.fps,
        steps=request.steps,
        seed=request.seed,
    )
    return GenerationResponse(**result)


@router.post("/describe", response_model=VideoDescribeResponse)
async def describe_video(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_video_service),
):
    """
    Get a description of an uploaded video.

    Note: This endpoint is deprecated. Use /api/vision/describe_video instead
    for full video captioning capabilities.

    Args:
        file: Video file to describe
        api_key: API key for authentication
        service: Video service instance

    Returns:
        VideoDescribeResponse with description and frame count
    """
    video_data = await file.read()
    result = await service.describe(video_data)
    return VideoDescribeResponse(**result)
