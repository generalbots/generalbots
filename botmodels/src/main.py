from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from .api.dependencies import is_valid_api_key
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
from .services.errors import BotModelsError
from .services.media_capabilities import media_capabilities
from .utils.error_codes import text as error_text

logger = get_logger("main")


def docs_urls_for(production: bool) -> dict:
    """FastAPI docs/OpenAPI kwargs: disabled entirely in production.

    Pure so the production posture can be unit-tested without booting a server
    or mutating the module-level settings singleton.
    """
    if production:
        return {"docs_url": None, "redoc_url": None, "openapi_url": None}
    return {
        "docs_url": "/api/docs",
        "redoc_url": "/api/redoc",
        "openapi_url": "/openapi.json",
    }


class AuthenticatedStaticFiles(StaticFiles):
    """StaticFiles that requires the BotModels API key on every artifact.

    ``app.mount()`` has no ``dependencies=`` parameter, so a normal FastAPI
    dependency cannot guard a static mount. The check runs at the ASGI layer
    instead, using the same constant-time predicate as the HTTP routes. A 401
    is returned before the file is looked up, so a missing file and a missing
    key are indistinguishable to an unauthenticated caller.
    """

    async def __call__(self, scope, receive, send):
        if scope["type"] == "http":
            headers = {
                name.lower(): value for name, value in scope.get("headers", [])
            }
            candidate = headers.get(b"x-api-key")
            if candidate is None:
                detail = error_text("botmodels-auth-missing-key")
            elif not is_valid_api_key(candidate.decode("latin-1")):
                detail = error_text("botmodels-auth-invalid-key")
            else:
                detail = None
            if detail is not None:
                response = JSONResponse({"detail": detail}, status_code=401)
                await response(scope, receive, send)
                return
        await super().__call__(scope, receive, send)


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
    **docs_urls_for(settings.is_production),
)


@app.exception_handler(BotModelsError)
async def botmodels_error_handler(request, exc: BotModelsError):
    """Map backend/load failures to 503 with the capability and cause.

    The GPU gate raises before any weights load; returning 503 keeps that
    distinguishable from a malformed request (4xx) and from an internal bug.
    """
    logger.warning(
        "Backend unavailable",
        path=str(request.url.path),
        error=str(exc),
        kind=type(exc).__name__,
    )
    return JSONResponse(
        status_code=503,
        content={"detail": str(exc), "error_type": type(exc).__name__},
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

app.mount(
    "/outputs",
    AuthenticatedStaticFiles(directory=str(settings.output_dir)),
    name="outputs",
)


@app.get("/")
async def root():
    return JSONResponse(
        {
            "service": settings.project_name,
            "version": settings.version,
            "commit": settings.commit,
            "status": "running",
            "mode": settings.resolved_mode,
            "docs": None if settings.is_production else "/api/docs",
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
        # System libraries pip cannot install: reported here so a missing
        # libzbar degrades barcode decoding instead of failing at import.
        "media_capabilities": media_capabilities(),
    }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("src.main:app", host=settings.host, port=settings.port, reload=True)