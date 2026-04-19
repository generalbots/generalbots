from pathlib import Path
from typing import Optional

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    env: str = "development"
    host: str = "0.0.0.0"
    port: int = 8085
    log_level: str = "INFO"
    api_v1_prefix: str = "/api"
    project_name: str = "BotModels API"
    version: str = "2.0.0"
    api_key: str = "change-me"

    # External Providers for Speech (Optional)
    groq_api_key: Optional[str] = None
    openai_api_key: Optional[str] = None

    # Image generation model
    image_model_path: str = "./models/stable-diffusion-v1-5"
    image_steps: int = 4
    image_width: int = 512
    image_height: int = 512
    image_gpu_layers: int = 20
    image_batch_size: int = 1

    # Video generation model
    video_model_path: str = "./models/zeroscope-v2"
    video_frames: int = 24
    video_fps: int = 8
    video_width: int = 320
    video_height: int = 576
    video_gpu_layers: int = 15
    video_batch_size: int = 1

    # Speech/TTS model
    speech_model_path: str = "./models/tts"

    # Vision model (BLIP2 for captioning)
    vision_model_path: str = "./models/blip2"

    # Whisper model for speech-to-text
    whisper_model_path: str = "./models/whisper"

    # Real-time Audio model for speech-to-speech
    realtime_audio_model_path: str = "./models/realtime_audio"

    # Device configuration
    device: str = "cuda"

    # Output directory for generated files
    output_dir: Path = Path("./outputs")

    @property
    def is_production(self) -> bool:
        return self.env == "production"


settings = Settings()
settings.output_dir.mkdir(parents=True, exist_ok=True)
(settings.output_dir / "images").mkdir(exist_ok=True)
(settings.output_dir / "videos").mkdir(exist_ok=True)
(settings.output_dir / "audio").mkdir(exist_ok=True)
