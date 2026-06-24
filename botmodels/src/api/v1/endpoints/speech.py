from fastapi import APIRouter, Depends, File, UploadFile

from ....schemas.generation import (
    GenerationResponse,
    SpeechGenerateRequest,
    SpeechToTextResponse,
)
from ....services.speech_service import get_speech_service
from ....services.realtime_audio_service import get_realtime_audio_service
from ...dependencies import verify_api_key

router = APIRouter(prefix="/speech", tags=["Speech"])


@router.post("/generate", response_model=GenerationResponse)
async def generate_speech(
    request: SpeechGenerateRequest,
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """
    Generate speech audio from text (Text-to-Speech).

    Args:
        request: Speech generation parameters including:
            - prompt: Text to convert to speech
            - voice: Voice model to use (optional, default: "default")
            - language: Language code (optional, default: "en")
        api_key: API key for authentication
        service: Speech service instance

    Returns:
        GenerationResponse with file path to generated audio and generation time
    """
    result = await service.generate(
        prompt=request.prompt,
        voice=request.voice,
        language=request.language,
    )
    return GenerationResponse(**result)


@router.post("/totext", response_model=SpeechToTextResponse)
async def speech_to_text(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """
    Convert speech audio to text (Speech-to-Text) using Whisper.

    Supported audio formats: wav, mp3, m4a, flac, ogg

    Args:
        file: Audio file to transcribe
        api_key: API key for authentication
        service: Speech service instance

    Returns:
        SpeechToTextResponse with transcribed text, detected language, and confidence
    """
    audio_data = await file.read()
    result = await service.to_text(audio_data)
    return SpeechToTextResponse(**result)


@router.post("/detect_language")
async def detect_language(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_speech_service),
):
    """
    Detect the language of spoken audio using Whisper.

    Args:
        file: Audio file to analyze
        api_key: API key for authentication
        service: Speech service instance

    Returns:
        dict with detected language code and confidence score
    """
    audio_data = await file.read()
    result = await service.detect_language(audio_data)
    return result


@router.post("/realtime")
async def realtime_audio(
    file: UploadFile = File(...),
    api_key: str = Depends(verify_api_key),
    service=Depends(get_realtime_audio_service),
):
    """
    Process audio using real-time speech-to-speech models.

    Args:
        file: Audio file to process
        api_key: API key for authentication
        service: Real-time Audio service instance

    Returns:
        dict with transcribed text and execution info
    """
    audio_data = await file.read()
    result = await service.process_audio(audio_data)
    return result
