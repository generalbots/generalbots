from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from .api.v1.endpoints import image, scoring, speech, video, anomaly
from .core.config import settings
from .core.logging import get_logger
from .services.image_service import get_image_service
from .services.speech_service import get_speech_service
from .services.video_service import get_video_service
from .services.vision_service import get_vision_service

logger = get_logger("main")


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting BotModels API", version=settings.version)
    try:
        get_image_service().initialize()
        get_video_service().initialize()
        get_speech_service().initialize()
        get_vision_service().initialize()
        logger.info("All services initialized")
    except Exception as e:
        logger.error("Failed to initialize services", error=str(e))
    yield
    logger.info("Shutting down BotModels API")


app = FastAPI(
    title=settings.project_name,
    version=settings.version,
    lifespan=lifespan,
    docs_url="/api/docs",
    redoc_url="/api/redoc",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(image.router, prefix=settings.api_v1_prefix)
app.include_router(video.router, prefix=settings.api_v1_prefix)
app.include_router(speech.router, prefix=settings.api_v1_prefix)
app.include_router(scoring.router, prefix=settings.api_v1_prefix)
app.include_router(anomaly.router, prefix=settings.api_v1_prefix)

app.mount("/outputs", StaticFiles(directory="outputs"), name="outputs")


@app.get("/")
async def root():
    return JSONResponse(
        {
            "service": settings.project_name,
            "version": settings.version,
            "commit": settings.commit,
            "status": "running",
            "docs": "/api/docs",
            "endpoints": {
                "image": "/api/image",
                "video": "/api/video",
                "speech": "/api/speech",
                "vision": "/api/vision",
                "scoring": "/api/scoring",
                "anomaly": "/api/anomaly",
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
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("src.main:app", host=settings.host, port=settings.port, reload=True)
