import time
import torch
from pathlib import Path
from typing import Optional, Dict, Any

from ..core.config import settings
from ..core.logging import get_logger

logger = get_logger("realtime_audio_service")

class RealtimeAudioService:
    def __init__(self):
        self.model = None
        self.processor = None
        self.device = settings.device
        self._initialized = False

    def initialize(self):
        if self._initialized:
            return
        
        logger.info("Loading Real-time Audio model")
        try:
            from transformers import AutoModelForSpeechSeq2Seq, AutoProcessor

            # Default to PersonaPlex but naming is generic
            model_id = "nvidia/personaplex-7b-v1"
            model_path = Path(settings.realtime_audio_model_path)

            if model_path.exists():
                load_path = str(model_path)
            else:
                load_path = model_id

            logger.info(f"Loading model from {load_path}")
            
            torch_dtype = torch.float16 if self.device == "cuda" else torch.float32

            self.processor = AutoProcessor.from_pretrained(load_path)
            self.model = AutoModelForSpeechSeq2Seq.from_pretrained(
                load_path,
                torch_dtype=torch_dtype,
                low_cpu_mem_usage=True,
                use_safetensors=True
            ).to(self.device)

            self._initialized = True
            logger.info("Real-time Audio model loaded successfully")
        except Exception as e:
            logger.error("Failed to load Real-time Audio model", error=str(e))
            self.model = None
            self.processor = None

    async def process_audio(self, audio_data: bytes, conversation_context: Optional[str] = None) -> Dict[str, Any]:
        """
        Process audio input using real-time S2S model.
        """
        if not self._initialized:
            self.initialize()

        if self.model is None or self.processor is None:
            return {
                "status": "error",
                "error": "Real-time Audio model not initialized"
            }

        start_time = time.time()
        
        try:
            import numpy as np
            import librosa
            import io

            audio_buf = io.BytesIO(audio_data)
            y, sr = librosa.load(audio_buf, sr=16000)

            inputs = self.processor(y, sampling_rate=16000, return_tensors="pt").to(self.device)
            inputs["input_features"] = inputs["input_features"].to(dtype=self.model.dtype)

            with torch.no_grad():
                generated_ids = self.model.generate(
                    inputs["input_features"],
                    max_new_tokens=256,
                    do_sample=True,
                    temperature=0.7
                )

            transcription = self.processor.batch_decode(generated_ids, skip_special_tokens=True)[0]

            execution_time = time.time() - start_time
            logger.info("Processed real-time audio", time=execution_time)

            return {
                "status": "completed",
                "text": transcription.strip(),
                "execution_time": execution_time,
                "model": "realtime-audio-v1"
            }

        except Exception as e:
            logger.error("Real-time audio processing failed", error=str(e))
            return {
                "status": "error",
                "error": str(e)
            }

_service = None

def get_realtime_audio_service():
    global _service
    if _service is None:
        _service = RealtimeAudioService()
    return _service
