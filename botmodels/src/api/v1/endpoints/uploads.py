"""Shared, bounded multipart upload reader (#2802).

Every ``UploadFile`` route used to call ``await file.read()`` with no ceiling:
Starlette spools an arbitrary body to disk and the whole payload is then
materialized in RAM, so one caller could exhaust the process. ``read_upload``
streams in 1 MiB chunks and aborts with 413 as soon as the cap is crossed,
before the accumulated bytes are kept.
"""

from fastapi import HTTPException, UploadFile

from ....utils.error_codes import text as error_text

#: Hard ceiling for a single upload body. Mirrors the audio limit that
#: ``speech.py`` already enforced, so every media route agrees on one number.
MAX_UPLOAD_BYTES = 50 * 1024 * 1024

_CHUNK_BYTES = 1024 * 1024


async def read_upload(
    file: UploadFile,
    *,
    max_bytes: int = MAX_UPLOAD_BYTES,
    empty_detail_key: str | None = None,
    too_large_key: str = "botmodels-upload-too-large",
) -> bytes:
    """Read ``file`` with a hard byte ceiling and a stable error contract."""
    chunks: list[bytes] = []
    total = 0
    while True:
        chunk = await file.read(_CHUNK_BYTES)
        if not chunk:
            break
        total += len(chunk)
        if total > max_bytes:
            raise HTTPException(
                status_code=413,
                detail=error_text(
                    too_large_key, mib=max_bytes // (1024 * 1024)
                ),
            )
        chunks.append(chunk)

    data = b"".join(chunks)
    if empty_detail_key and not data:
        raise HTTPException(status_code=400, detail=error_text(empty_detail_key))
    return data
