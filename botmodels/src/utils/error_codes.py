"""User-visible error keys for the BotModels HTTP API (#2095).

Every message this service authors for a person (not a stack trace) is defined
once, here, under a stable kebab-case key. The English text is the source of
truth; `botmodels` does not ship per-locale files yet, so the key is what a
client (or a future catalogue) translates.

Where these strings are rendered
--------------------------------
`text()` returns the English message, and the routes keep putting it in the
FastAPI `detail` field, so the wire format is unchanged:

    HTTP 400/401/413/5xx  ->  {"detail": "Empty audio upload"}

`botserver` reads `detail` from the BotModels client and surfaces it as the
`error` field of its own JSON response (or the chat tool output), which the
web suite displays through `window.gbT` where a translated string exists.
A client that wants its own translation should switch on the key rather than on
the English sentence: `ERROR_MESSAGES` is the contract, `text()` the default.

Keep the keys kebab-case and prefixed with `botmodels-`; the matching Fluent
namespace, when it lands, is `botlib/locales/<locale>/errors.ftl`.
"""

from typing import Any, Dict

# key -> English text. `{name}` placeholders are filled by `text()`.
ERROR_MESSAGES: Dict[str, str] = {
    # --- auth (api/dependencies.py) ---
    "botmodels-auth-missing-key": "Missing API key",
    "botmodels-auth-invalid-key": "Invalid API key",
    # --- speech (api/v1/endpoints/speech.py) ---
    "botmodels-speech-empty-upload": "Empty audio upload",
    "botmodels-speech-audio-too-large": "Audio upload exceeds {mib} MiB",
    "botmodels-speech-no-audio": "Speech backend returned no audio",
    "botmodels-speech-realtime-unavailable": "Realtime audio unavailable",
    "botmodels-speech-backend-failed": "Speech backend failed: {error}",
    # --- scoring (api/v1/endpoints/scoring.py) ---
    "botmodels-scoring-failed": "Scoring failed: {error}",
    "botmodels-scoring-batch-failed": "Batch scoring failed: {error}",
    # --- music (api/v1/endpoints/music.py) ---
    "botmodels-music-invalid-job": "Invalid job identifier",
    "botmodels-music-backend-failed": "Music backend failed: {error}",
}


def text(key: str, **params: Any) -> str:
    """English message for ``key`` with ``{name}`` placeholders filled.

    An unknown key returns the key itself: a visible identifier beats an empty
    error body, and it names the missing entry in a bug report.
    """
    template = ERROR_MESSAGES.get(key)
    if template is None:
        return key
    if not params:
        return template
    try:
        return template.format(**params)
    except (KeyError, IndexError, ValueError):
        return template


def detail(key: str, **params: Any) -> Dict[str, str]:
    """Structured ``detail`` payload: the stable key plus the English text.

    Routes that want clients to translate should raise
    ``HTTPException(status_code=..., detail=detail("botmodels-..."))``; the
    plain `text()` form stays for the existing string contract.
    """
    return {"code": key, "message": text(key, **params)}
