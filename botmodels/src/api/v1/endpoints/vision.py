import io
from typing import Optional

from fastapi import APIRouter, Depends, File, Form, UploadFile
from PIL import Image
from pyzbar import pyzbar

from ....schemas.generation import (
    ImageDescribeResponse,
    QRCodeResponse,
    VideoDescribeResponse,
)
from ....services.ocr_service import describe_codes, image_metadata, run_ocr
from ....services.vision_service import get_vision_service
from ...dependencies import verify_api_key

router = APIRouter(prefix="/vision", tags=["Vision"])


@router.post("/describe", response_model=ImageDescribeResponse)
async def describe_image(
    file: UploadFile = File(...),
    prompt: Optional[str] = Form(None),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_vision_service),
):
    """
    Get a caption/description for an image.
    Optionally provide a prompt to guide the description.
    """
    image_data = await file.read()
    result = await service.describe_image(image_data, prompt)
    return ImageDescribeResponse(**result)


@router.post("/describe-video", response_model=VideoDescribeResponse)
async def describe_video(
    file: UploadFile = File(...),
    num_frames: int = Form(8),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_vision_service),
):
    """
    Get a description for a video by sampling and analyzing frames.

    Args:
        file: Video file (mp4, avi, mov, webm, mkv)
        num_frames: Number of frames to sample for analysis (default: 8)
    """
    video_data = await file.read()
    result = await service.describe_video(video_data, num_frames)
    return VideoDescribeResponse(**result)


# Alias for botserver callers that send the underscore spelling. FastAPI would
# otherwise 404 on /api/vision/describe_video (see issue 1514).
router.add_api_route(
    "/describe_video",
    describe_video,
    methods=["POST"],
    response_model=VideoDescribeResponse,
    include_in_schema=False,
)


@router.post("/vqa")
async def visual_question_answering(
    file: UploadFile = File(...),
    question: str = Form(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_vision_service),
):
    """
    Visual Question Answering - ask a question about an image.

    Args:
        file: Image file
        question: Question to ask about the image
    """
    image_data = await file.read()
    result = await service.answer_question(image_data, question)
    return ImageDescribeResponse(**result)


@router.post("/qrcode", response_model=QRCodeResponse)
async def read_qrcode(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
):
    """
    Read QR code(s) from an image.

    Returns all QR codes found in the image with their data and positions.

    Args:
        file: Image file containing QR code(s)

    Returns:
        QRCodeResponse with data from all found QR codes
    """
    image_data = await file.read()

    try:
        # Load image
        image = Image.open(io.BytesIO(image_data))

        # Convert to RGB if necessary (pyzbar works best with RGB)
        if image.mode != "RGB":
            image = image.convert("RGB")

        # Decode QR codes
        decoded_objects = pyzbar.decode(image)

        if not decoded_objects:
            return QRCodeResponse(
                success=False,
                data=None,
                codes=[],
                count=0,
                error="No QR code found in image",
            )

        codes = []
        for obj in decoded_objects:
            code_info = {
                "data": obj.data.decode("utf-8", errors="replace"),
                "type": obj.type,
                "rect": {
                    "left": obj.rect.left,
                    "top": obj.rect.top,
                    "width": obj.rect.width,
                    "height": obj.rect.height,
                },
                "polygon": [{"x": p.x, "y": p.y} for p in obj.polygon]
                if obj.polygon
                else None,
            }
            codes.append(code_info)

        # Return the first QR code data as the main data field for convenience
        primary_data = codes[0]["data"] if codes else None

        return QRCodeResponse(
            success=True, data=primary_data, codes=codes, count=len(codes), error=None
        )

    except Exception as e:
        return QRCodeResponse(
            success=False,
            data=None,
            codes=[],
            count=0,
            error=f"Failed to process image: {str(e)}",
        )


@router.post("/barcode")
async def read_barcode(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
):
    """
    Read barcode(s) from an image (supports multiple barcode formats).

    Supports: QR Code, Code128, Code39, EAN-13, EAN-8, UPC-A, UPC-E,
    Interleaved 2 of 5, Codabar, PDF417, DataMatrix

    Args:
        file: Image file containing barcode(s)

    Returns:
        List of all barcodes found with their data and type
    """
    image_data = await file.read()

    try:
        image = Image.open(io.BytesIO(image_data))

        if image.mode != "RGB":
            image = image.convert("RGB")

        decoded_objects = pyzbar.decode(image)

        if not decoded_objects:
            return {
                "success": False,
                "barcodes": [],
                "count": 0,
                "error": "No barcode found in image",
            }

        barcodes = []
        for obj in decoded_objects:
            barcode_info = {
                "data": obj.data.decode("utf-8", errors="replace"),
                "type": obj.type,
                "rect": {
                    "left": obj.rect.left,
                    "top": obj.rect.top,
                    "width": obj.rect.width,
                    "height": obj.rect.height,
                },
            }
            barcodes.append(barcode_info)

        return {
            "success": True,
            "barcodes": barcodes,
            "count": len(barcodes),
            "error": None,
        }

    except Exception as e:
        return {
            "success": False,
            "barcodes": [],
            "count": 0,
            "error": f"Failed to process image: {str(e)}",
        }


@router.post("/ocr")
async def extract_text(
    file: UploadFile = File(...),
    language: str = Form("eng"),
    api_key: str = Depends(verify_api_key),
):
    """
    Extract text from an image using the tier's OCR backend.

    min mode uses Tesseract (CPU-only); max mode uses PaddleOCR-VL, which returns
    markdown tables rather than flattened text.

    Args:
        file: Image file
        language: Language code for OCR (default: eng).
                  Use 'por' for Portuguese, 'spa' for Spanish, etc.

    Returns:
        Extracted text from the image
    """
    image_data = await file.read()
    return run_ocr(image_data, language)


@router.post("/analyze")
async def analyze_image(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_vision_service),
):
    """
    Comprehensive image analysis - combines description, OCR, and barcode detection.

    Returns a complete analysis of the image including:
    - AI-generated description
    - Any text found (OCR)
    - Any QR codes or barcodes found

    Args:
        file: Image file to analyze
    """
    image_data = await file.read()

    result: dict = {"description": None, "text": None, "codes": [], "metadata": {}}

    # Each stage is independent: a failure in one degrades that field only.
    # (The previous implementation used three bare `except:` clauses, which the
    # botmodels README forbids.)
    try:
        result["metadata"] = image_metadata(image_data)
    except (OSError, ValueError) as exc:
        result["metadata"] = {"error": str(exc)}

    try:
        desc_result = await service.describe_image(image_data, None)
        result["description"] = desc_result.get("description")
    except Exception as exc:  # noqa: BLE001 - optional stage
        result["description_error"] = str(exc)

    try:
        ocr_result = run_ocr(image_data)
        if ocr_result.get("text"):
            result["text"] = ocr_result["text"]
            result["ocr_backend"] = ocr_result.get("backend")
    except Exception as exc:  # noqa: BLE001 - optional stage
        result["ocr_error"] = str(exc)

    try:
        result["codes"] = describe_codes(image_data)
    except Exception as exc:  # noqa: BLE001 - optional stage
        result["codes_error"] = str(exc)

    return {"success": True, **result}
