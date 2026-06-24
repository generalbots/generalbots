from datetime import datetime
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field


class GenerationRequest(BaseModel):
    prompt: str = Field(..., min_length=1, max_length=2000)
    seed: Optional[int] = None


class ImageGenerateRequest(GenerationRequest):
    steps: Optional[int] = Field(30, ge=1, le=150)
    width: Optional[int] = Field(512, ge=64, le=2048)
    height: Optional[int] = Field(512, ge=64, le=2048)
    guidance_scale: Optional[float] = Field(7.5, ge=1.0, le=20.0)


class VideoGenerateRequest(GenerationRequest):
    num_frames: Optional[int] = Field(24, ge=8, le=128)
    fps: Optional[int] = Field(8, ge=1, le=60)
    steps: Optional[int] = Field(50, ge=10, le=100)


class SpeechGenerateRequest(GenerationRequest):
    voice: Optional[str] = Field("default", description="Voice model")
    language: Optional[str] = Field("en", description="Language code")


class GenerationResponse(BaseModel):
    status: str
    file_path: Optional[str] = None
    generation_time: Optional[float] = None
    error: Optional[str] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class DescribeRequest(BaseModel):
    file_data: bytes


class ImageDescribeResponse(BaseModel):
    description: str
    confidence: Optional[float] = None
    generation_time: Optional[float] = None


class VideoDescribeResponse(BaseModel):
    description: str
    frame_count: int
    generation_time: Optional[float] = None


class SpeechToTextResponse(BaseModel):
    text: str
    language: Optional[str] = None
    confidence: Optional[float] = None


class QRCodeInfo(BaseModel):
    """Information about a single QR code found in an image"""

    data: str = Field(..., description="The decoded data from the QR code")
    type: str = Field(..., description="The type of code (QRCODE, BARCODE, etc.)")
    rect: Optional[Dict[str, int]] = Field(
        None, description="Bounding rectangle {left, top, width, height}"
    )
    polygon: Optional[List[Dict[str, int]]] = Field(
        None, description="Polygon points [{x, y}, ...]"
    )


class QRCodeResponse(BaseModel):
    """Response from QR code reading endpoint"""

    success: bool = Field(..., description="Whether the operation was successful")
    data: Optional[str] = Field(
        None, description="The primary QR code data (first found)"
    )
    codes: List[Dict[str, Any]] = Field(
        default_factory=list, description="All QR codes found in the image"
    )
    count: int = Field(0, description="Number of QR codes found")
    error: Optional[str] = Field(None, description="Error message if any")


class BarcodeResponse(BaseModel):
    """Response from barcode reading endpoint"""

    success: bool
    barcodes: List[Dict[str, Any]] = Field(default_factory=list)
    count: int = 0
    error: Optional[str] = None


class OCRResponse(BaseModel):
    """Response from OCR text extraction endpoint"""

    success: bool
    text: str = ""
    confidence: float = 0.0
    language: str = "eng"
    word_count: int = 0
    error: Optional[str] = None


class ImageAnalysisResponse(BaseModel):
    """Comprehensive image analysis response"""

    success: bool
    description: Optional[str] = None
    text: Optional[str] = None
    codes: List[Dict[str, Any]] = Field(default_factory=list)
    metadata: Dict[str, Any] = Field(default_factory=dict)
    error: Optional[str] = None
