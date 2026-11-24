"""App-level request body ceiling (#2802).

The per-endpoint readers cap what they keep in memory; this middleware rejects
an oversized body even earlier when the client declares its length, so a
multi-GB multipart upload is refused before Starlette spools it to disk.
"""

from starlette.responses import JSONResponse

from ..utils.error_codes import text as error_text


class MaxBodySizeMiddleware:
    """Reject declared request bodies larger than ``max_bytes`` with 413."""

    def __init__(self, app, max_bytes: int) -> None:
        self.app = app
        self.max_bytes = max_bytes

    async def __call__(self, scope, receive, send):
        if scope["type"] != "http":
            return await self.app(scope, receive, send)

        headers = dict(scope.get("headers") or [])
        declared = headers.get(b"content-length")
        if declared is not None:
            try:
                length = int(declared)
            except ValueError:
                length = None
            if length is not None and length > self.max_bytes:
                response = JSONResponse(
                    {
                        "detail": error_text(
                            "botmodels-upload-too-large",
                            mib=self.max_bytes // (1024 * 1024),
                        )
                    },
                    status_code=413,
                )
                await response(scope, receive, send)
                return

        return await self.app(scope, receive, send)
