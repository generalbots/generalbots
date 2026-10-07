import os

import structlog


def _log_level() -> str:
    return os.getenv("LOG_LEVEL", "INFO").upper()


def _is_production() -> bool:
    return os.getenv("ENV", "development") == "production"


def setup_logging():
    """Configure structlog. Reads env directly to avoid a circular import:
    config imports mode -> hardware -> logging, so logging must not import
    config."""
    if _is_production():
        structlog.configure(
            processors=[
                structlog.contextvars.merge_contextvars,
                structlog.stdlib.add_log_level,
                structlog.processors.TimeStamper(fmt="iso"),
                structlog.processors.JSONRenderer(),
            ],
            wrapper_class=structlog.make_filtering_bound_logger(
                getattr(structlog.stdlib.logging, _log_level())
            ),
        )
    else:
        structlog.configure(
            processors=[
                structlog.contextvars.merge_contextvars,
                structlog.stdlib.add_log_level,
                structlog.processors.TimeStamper(fmt="iso"),
                structlog.dev.ConsoleRenderer(colors=True),
            ],
        )


def get_logger(name: str = None):
    logger = structlog.get_logger()
    if name:
        logger = logger.bind(service=name)
    return logger


setup_logging()