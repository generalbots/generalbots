from datetime import datetime
from typing import Any, Dict, List, Literal, Optional

from pydantic import BaseModel, Field, model_validator


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


class MusicGenerateRequest(BaseModel):
    """Validated ACE-Step music generation request."""

    title: str = Field("Untitled song", min_length=1, max_length=160)
    prompt: str = Field("", max_length=4000)
    description: str = Field("", max_length=4000)
    lyrics: str = Field("", max_length=20000)
    simple_mode: bool = True
    instrumental: bool = False
    vocal_language: str = Field("en", min_length=2, max_length=16)
    duration: Optional[float] = Field(60, ge=10, le=600)
    bpm: Optional[int] = Field(None, ge=30, le=300)
    key_scale: str = Field("", max_length=32)
    time_signature: str = Field("", max_length=8)
    inference_steps: int = Field(8, ge=1, le=200)
    guidance_scale: float = Field(7.0, ge=0.0, le=30.0)
    batch_size: int = Field(2, ge=1, le=8)
    seed: Optional[int] = Field(None, ge=0)
    thinking: bool = True
    enhance: bool = True
    audio_format: Literal["mp3", "flac", "wav", "opus", "aac"] = "mp3"
    model: Optional[str] = Field(None, max_length=120)

    @model_validator(mode="after")
    def validate_music_brief(self):
        if self.simple_mode and not self.description.strip():
            raise ValueError("description is required in simple mode")
        if not self.simple_mode and not self.prompt.strip():
            raise ValueError("prompt is required in custom mode")
        if self.instrumental:
            self.lyrics = ""
        return self


class MusicFormatRequest(BaseModel):
    prompt: str = Field(..., min_length=1, max_length=4000)
    lyrics: str = Field("", max_length=20000)
    duration: Optional[float] = Field(None, ge=10, le=600)
    bpm: Optional[int] = Field(None, ge=30, le=300)
    key_scale: str = Field("", max_length=32)
    time_signature: str = Field("", max_length=8)


class MusicJobCreated(BaseModel):
    job_id: str
    status: str = "queued"
    queue_position: Optional[int] = None


class MusicTrack(BaseModel):
    audio_path: str
    prompt: str = ""
    lyrics: str = ""
    duration: Optional[float] = None
    bpm: Optional[int] = None
    key_scale: Optional[str] = None
    time_signature: Optional[str] = None
    seed: Optional[str] = None
    model: Optional[str] = None


class MusicJobStatus(BaseModel):
    job_id: str
    status: Literal["queued", "running", "succeeded", "failed"]
    queue_position: Optional[int] = None
    progress: Optional[float] = None
    tracks: List[MusicTrack] = Field(default_factory=list)
    error: Optional[str] = None


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
