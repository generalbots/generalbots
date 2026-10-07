from pathlib import Path
from typing import Optional

from pydantic_settings import BaseSettings, SettingsConfigDict

from .mode import Mode, ModeSetting, resolve_mode, summary


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
    version: str = "2.1.0"
    api_key: str = "change-me"
    commit: str = "unknown"

    # Model tier: "auto" detects hardware, "min"/"max" force a tier.
    mode: ModeSetting = "auto"

    # Resolved at startup by finalize_mode(); read-only afterwards.
    resolved_mode: Optional[Mode] = None
    mode_source: str = "unresolved"

    # Comma-separated browser origins. Empty means same-origin only.
    allowed_origins: str = ""

    # Remote providers for speech. Opt-in only: min mode never calls these.
    allow_remote_speech: bool = False
    groq_api_key: Optional[str] = None
    openai_api_key: Optional[str] = None

    # Image generation
    image_model_path: str = "./models/stable-diffusion-v1-5"
    image_steps: int = 4
    image_width: int = 512
    image_height: int = 512

    # Video generation
    video_model_path: str = "./models/zeroscope-v2"
    video_frames: int = 24
    video_fps: int = 8
    video_steps: int = 50
    video_width: int = 320
    video_height: int = 576

    # Vision / captioning (BLIP2 in min mode)
    vision_model_path: str = "./models/blip2"

    # Real-time audio model for speech-to-speech
    realtime_audio_model_path: str = "./models/realtime_audio"

    # Speech models (min tier defaults)
    stt_model_path: str = "./models/stt"
    tts_model_path: str = "./models/tts"

    # ACE-Step 1.5 music generation API
    acestep_api_url: str = "http://127.0.0.1:8001"
    acestep_api_key: Optional[str] = None
    acestep_request_timeout: float = 30.0
    acestep_audio_timeout: float = 300.0

    # OCR model (max tier: PaddleOCR-VL)
    ocr_model_path: str = "./models/paddleocr-vl"

    # Device override; empty means "use whatever the tier resolved to".
    device: str = ""

    output_dir: Path = Path("./outputs")

    @property
    def is_production(self) -> bool:
        return self.env == "production"


settings = Settings()
settings.output_dir.mkdir(parents=True, exist_ok=True)
(settings.output_dir / "images").mkdir(exist_ok=True)
(settings.output_dir / "videos").mkdir(exist_ok=True)
(settings.output_dir / "audio").mkdir(exist_ok=True)