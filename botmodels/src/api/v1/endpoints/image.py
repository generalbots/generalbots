from fastapi import APIRouter, Depends, File, UploadFile

from ....schemas.generation import (
    GenerationResponse,
    ImageDescribeResponse,
    ImageGenerateRequest,
)
from ....services.image_service import get_image_service
from ...dependencies import verify_api_key

router = APIRouter(prefix="/image", tags=["Image"])


@router.post("/generate", response_model=GenerationResponse)
async def generate_image(
    request: ImageGenerateRequest,
    api_key: str = Depends(verify_api_key),
    service=Depends(get_image_service),
):
    """
    Generate an image from a text prompt.

    Args:
        request: Image generation parameters including prompt, steps, dimensions, etc.
        api_key: API key for authentication
        service: Image service instance

    Returns:
        GenerationResponse with file path and generation time
    """
    result = await service.generate(
        prompt=request.prompt,
        steps=request.steps,
        width=request.width,
        height=request.height,
        guidance_scale=request.guidance_scale,
        seed=request.seed,
    )
    return GenerationResponse(**result)


@router.post("/describe", response_model=ImageDescribeResponse)
async def describe_image(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_image_service),
):
    """
    Get a description of an uploaded image.

    Note: This endpoint is deprecated. Use /api/vision/describe instead
    for full captioning capabilities.

    Args:
        file: Image file to describe
        api_key: API key for authentication
        service: Image service instance

    Returns:
        ImageDescribeResponse with description
    """
    image_data = await file.read()
    result = await service.describe(image_data)
    return ImageDescribeResponse(**result)
