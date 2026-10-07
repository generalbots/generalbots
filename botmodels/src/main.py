from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from .api.v1.endpoints import (
    anomaly,
    image,
    music,
    scoring,
    speech,
    video,
    vision,
    voice,
)
from .core.config import settings
from .core.logging import get_logger
from .core.mode import InvalidModeError
from .core.startup import finalize_mode, health_summary
from .services.backends import registry

logger = get_logger("main")


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting BotModels API", version=settings.version)

    try:
        finalize_mode()
    except InvalidModeError as exc:
        # Fail fast on a bad BOTMODELS_MODE rather than silently degrading.
        logger.error("Invalid BOTMODELS_MODE", error=str(exc))
        raise

    logger.info("Mode summary", **health_summary())

    # Services are constructed lazily through the backend registry, so nothing
    # is loaded here. This is deliberate: previously every model was loaded
    # eagerly at startup and held forever, and --workers 4 quadruplicated VRAM.
    yield

    logger.info("Shutting down BotModels API")


app = FastAPI(
    title=settings.project_name,
    version=settings.version,
    lifespan=lifespan,
    docs_url="/api/docs",
    redoc_url="/api/redoc",
)

# Credentials require a specific origin list; browsers reject "*" together with
# allow_credentials, which silently broke credentialed cross-origin requests.
ALLOWED_ORIGINS = [
    origin.strip()
    for origin in settings.allowed_origins.split(",")
    if origin.strip()
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=ALLOWED_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(image.router, prefix=settings.api_v1_prefix)
app.include_router(music.router, prefix=settings.api_v1_prefix)
app.include_router(video.router, prefix=settings.api_v1_prefix)
app.include_router(speech.router, prefix=settings.api_v1_prefix)
app.include_router(vision.router, prefix=settings.api_v1_prefix)
app.include_router(scoring.router, prefix=settings.api_v1_prefix)
app.include_router(anomaly.router, prefix=settings.api_v1_prefix)
app.include_router(voice.router)

app.mount("/outputs", StaticFiles(directory=str(settings.output_dir)), name="outputs")


@app.get("/")
async def root():
    return JSONResponse(
        {
            "service": settings.project_name,
            "version": settings.version,
            "commit": settings.commit,
            "status": "running",
            "mode": settings.resolved_mode,
            "docs": "/api/docs",
            "endpoints": {
                "image": "/api/image",
                "music": "/api/music",
                "video": "/api/video",
                "speech": "/api/speech",
                "vision": "/api/vision",
                "scoring": "/api/scoring",
                "anomaly": "/api/detect",
                "voice": "/v1/audio",
            },
        }
    )


@app.get("/api/health")
async def health():
    return {
        "status": "healthy",
        "version": settings.version,
        "commit": settings.commit,
        "device": settings.device,
        **health_summary(),
        "backends_loaded": registry.status(),
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("src.main:app", host=settings.host, port=settings.port, reload=True)