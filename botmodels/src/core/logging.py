import structlog
from .config import settings

def setup_logging():
    if settings.is_production:
        structlog.configure(
            processors=[
                structlog.contextvars.merge_contextvars,
                structlog.stdlib.add_log_level,
                structlog.processors.TimeStamper(fmt="iso"),
                structlog.processors.JSONRenderer()
            ],
            wrapper_class=structlog.make_filtering_bound_logger(
                getattr(structlog.stdlib.logging, settings.log_level.upper())
            ),
        )
    else:
        structlog.configure(
            processors=[
                structlog.contextvars.merge_contextvars,
                structlog.stdlib.add_log_level,
                structlog.processors.TimeStamper(fmt="iso"),
                structlog.dev.ConsoleRenderer(colors=True)
            ],
        )

def get_logger(name: str = None):
    logger = structlog.get_logger()
    if name:
        logger = logger.bind(service=name)
    return logger

setup_logging()
