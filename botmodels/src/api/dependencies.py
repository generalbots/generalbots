from fastapi import Header, HTTPException
from ..core.config import settings


async def verify_api_key(x_api_key: str = Header(...)):
    if x_api_key != settings.api_key:
        raise HTTPException(status_code=401, detail="Invalid API key")
    return x_api_key


def get_api_key(x_api_key: str | None) -> str | None:
    if x_api_key is None:
        return None
    if x_api_key == settings.api_key:
        return x_api_key
    return None
