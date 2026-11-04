"""Realtime voice streaming endpoints (offline sovereignty).

Binary PCM travels over WebSockets and is processed exclusively by the
local engines declared in :mod:`src.services.realtime`; no audio or
transcript leaves the host. When an engine is unavailable the socket
receives a JSON error frame and closes gracefully.
"""

import asyncio
import hmac
import json

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from ....core.config import settings
from ....services.realtime import (
    ENERGY_GATE,
    SILENCE_THRESHOLD_MS,
    TARGET_SAMPLE_RATE,
    WINDOW_SECONDS,
    EngineFactory,
    SttEngine,
    frame_duration_ms,
    rms_energy,
)
from ....core.logging import get_logger

logger = get_logger("voice")

router = APIRouter(prefix="/v1/audio", tags=["Voice"])


async def _close_with_error(socket: WebSocket, message: str) -> None:
    """Send one JSON error frame and close the socket politely."""
    await socket.send_json({"type": "error", "message": message})
    await socket.close(code=1011)


def _key_from_query(socket: WebSocket) -> str | None:
    """Read the API key from the upgrade query string.

    A Header dependency cannot run on a WebSocket upgrade here, so both voice
    sockets took the key as ``?api_key=``. They were previously unauthenticated
    despite the README stating every endpoint requires the header.
    """
    return socket.query_params.get("api_key") or socket.query_params.get("token")


async def _authorize(socket: WebSocket) -> bool:
    """Accept the socket only when the key is valid, else close with 1008.

    Policy violation, not an internal error, hence 1008 rather than 1011.
    """
    candidate = _key_from_query(socket)
    expected = settings.api_key

    if candidate is None:
        await socket.close(code=1008, reason="Missing API key")
        return False

    if not hmac.compare_digest(candidate.encode("utf-8"), expected.encode("utf-8")):
        await socket.close(code=1008, reason="Invalid API key")
        return False

    await socket.accept()
    return True


async def _transcribe(engine: SttEngine, audio: bytes, sample_rate: int) -> str:
    """Run the CPU-bound transcription off the event loop."""
    return await asyncio.to_thread(engine.transcribe_chunk, audio, sample_rate)


@router.websocket("/stt/stream")
async def stt_stream(socket: WebSocket) -> None:
    """Stream partial and final transcripts from binary PCM frames.

    Binary frames accumulate into 0.5 s windows answered with ``partial``
    events; a ``final`` event follows once silence exceeds 700 ms or the
    client sends ``{"type": "flush"}``. ``{"type": "config"}`` may adjust
    the inbound ``sample_rate`` (default 16000).
    """
    if not await _authorize(socket):
        return
    engine = EngineFactory.stt()
    if engine is None:
        await _close_with_error(socket, "STT engine unavailable; set BOTMODELS_STT_ENGINE=local with faster_whisper installed.")
        return
    sample_rate = TARGET_SAMPLE_RATE
    buffer = bytearray()
    buffered_seconds = 0.0
    silence_ms = 0.0
    try:
        while True:
            message = await socket.receive()
            if message["type"] == "websocket.disconnect":
                break
            text = message.get("text")
            if text:
                event = json.loads(text)
                kind = event.get("type")
                if kind == "config":
                    sample_rate = int(event.get("sample_rate") or sample_rate)
                elif kind == "flush":
                    transcript = await _transcribe(engine, bytes(buffer), sample_rate)
                    buffer.clear()
                    buffered_seconds = 0.0
                    silence_ms = 0.0
                    await socket.send_json({"type": "final", "text": transcript})
                continue
            frame = message.get("bytes") or b""
            if not frame:
                continue
            buffer.extend(frame)
            buffered_seconds += frame_duration_ms(frame, sample_rate) / 1000.0
            if rms_energy(frame) < ENERGY_GATE:
                silence_ms += frame_duration_ms(frame, sample_rate)
            else:
                silence_ms = 0.0
            if buffered_seconds >= WINDOW_SECONDS:
                buffered_seconds = 0.0
                transcript = await _transcribe(engine, bytes(buffer), sample_rate)
                if transcript:
                    await socket.send_json({"type": "partial", "text": transcript})
            if silence_ms >= SILENCE_THRESHOLD_MS and buffer:
                transcript = await _transcribe(engine, bytes(buffer), sample_rate)
                buffer.clear()
                buffered_seconds = 0.0
                silence_ms = 0.0
                await socket.send_json({"type": "final", "text": transcript})
    except WebSocketDisconnect:
        logger.info("STT stream disconnected")
    except Exception as exc:
        logger.warning("STT stream terminated", error=str(exc))
        try:
            await _close_with_error(socket, f"STT stream failed: {exc}")
        except Exception:
            logger.debug("STT error frame could not be delivered")


@router.websocket("/tts/stream")
async def tts_stream(socket: WebSocket) -> None:
    """Accept text messages and answer with synthesized 16 kHz PCM frames.

    The first outbound message is a ``meta`` JSON frame describing the
    engine; subsequent binary messages carry raw PCM. Synthesis errors are
    reported as ``error`` JSON frames without dropping the connection.
    """
    if not await _authorize(socket):
        return
    engine = EngineFactory.tts()
    if engine is None:
        await _close_with_error(socket, "TTS engine unavailable; set BOTMODELS_TTS_ENGINE=local with piper installed.")
        return
    await socket.send_json({"type": "meta", "engine": engine.name, "sample_rate": TARGET_SAMPLE_RATE})
    try:
        while True:
            message = await socket.receive()
            if message["type"] == "websocket.disconnect":
                break
            text = message.get("text")
            if not text:
                continue
            try:
                payload = json.loads(text)
                utterance = payload.get("text") if isinstance(payload, dict) else str(payload)
            except json.JSONDecodeError:
                utterance = text
            utterance = (utterance or "").strip()
            if not utterance:
                await socket.send_json({"type": "error", "message": "Empty text frame ignored."})
                continue

            def render(utterance: str = utterance) -> list:
                return list(engine.synthesize_stream(utterance))

            try:
                frames = await asyncio.to_thread(render)
            except RuntimeError as exc:
                await socket.send_json({"type": "error", "message": str(exc)})
                continue
            for pcm_frame in frames:
                await socket.send_bytes(pcm_frame)
            await socket.send_json({"type": "done"})
    except WebSocketDisconnect:
        logger.info("TTS stream disconnected")
    except Exception as exc:
        logger.warning("TTS stream terminated", error=str(exc))


@router.websocket("/duplex")
async def duplex_stream(socket: WebSocket) -> None:
    """Bidirectional speech on one socket with barge-in.

    Client -> server:
      * binary PCM frames (STT), same framing as ``/stt/stream``;
      * ``{"type":"say","text":"…"}`` to synthesize;
      * ``{"type":"stop"}``, ``{"type":"flush"}``, ``{"type":"config"}``.

    Server -> client: ``meta``, ``partial``/``final`` transcripts, and for TTS
    ``speech_start`` + binary PCM + ``speech_end``. Incoming audio while the
    server is speaking cancels that utterance and emits ``barge_in``.

    The socket is not echo-cancelled: the client must not feed the synthesised
    output back into the microphone. This is the duplex state machine; acoustic
    echo cancellation and voice-activity tuning remain client-side.
    """
    if not await _authorize(socket):
        return

    stt = EngineFactory.stt()
    tts = EngineFactory.tts()
    if stt is None or tts is None:
        await _close_with_error(socket, "Duplex requires local STT and TTS engines.")
        return

    sample_rate = TARGET_SAMPLE_RATE
    buffer = bytearray()
    buffered_seconds = 0.0
    silence_ms = 0.0
    send_lock = asyncio.Lock()
    speech_task: asyncio.Task | None = None

    async def send_json(payload: dict) -> None:
        async with send_lock:
            await socket.send_json(payload)

    async def speak(text: str) -> None:
        try:
            frames = await asyncio.to_thread(lambda: list(tts.synthesize_stream(text)))
            await send_json({"type": "speech_start", "text": text})
            for frame in frames:
                async with send_lock:
                    await socket.send_bytes(frame)
                # 4096 bytes of 16-bit mono at 16 kHz is ~128 ms of audio.
                # Pacing keeps the duplex stream real-time and gives inbound
                # speech a window to barge in; blasting the buffer made the
                # utterance finish before the client could interrupt.
                await asyncio.sleep(0.12)
            await send_json({"type": "speech_end"})
        except asyncio.CancelledError:
            raise
        except Exception as exc:  # noqa: BLE001 - reported as a socket frame
            await send_json({"type": "error", "message": f"TTS failed: {exc}"})

    def cancel_speech() -> None:
        nonlocal speech_task
        if speech_task is not None and not speech_task.done():
            speech_task.cancel()
        speech_task = None

    await send_json(
        {
            "type": "meta",
            "stt": stt.name,
            "tts": tts.name,
            "sample_rate": TARGET_SAMPLE_RATE,
            "duplex": True,
        }
    )

    try:
        while True:
            message = await socket.receive()
            if message["type"] == "websocket.disconnect":
                break

            text = message.get("text")
            if text:
                try:
                    event = json.loads(text)
                except json.JSONDecodeError:
                    continue
                kind = event.get("type")
                if kind == "config":
                    sample_rate = int(event.get("sample_rate") or sample_rate)
                elif kind == "say":
                    cancel_speech()
                    speech_task = asyncio.create_task(
                        speak(str(event.get("text") or ""))
                    )
                elif kind == "stop":
                    cancel_speech()
                    await send_json({"type": "speech_stopped"})
                elif kind == "flush":
                    transcript = await _transcribe(stt, bytes(buffer), sample_rate)
                    buffer.clear()
                    buffered_seconds = 0.0
                    silence_ms = 0.0
                    await send_json({"type": "final", "text": transcript})
                continue

            frame = message.get("bytes") or b""
            if not frame:
                continue

            # New inbound audio is a barge-in: stop the current utterance
            # before processing the frame, so the reply never overlaps it.
            if speech_task is not None and not speech_task.done():
                cancel_speech()
                await send_json({"type": "barge_in"})

            buffer.extend(frame)
            buffered_seconds += frame_duration_ms(frame, sample_rate) / 1000.0
            if rms_energy(frame) < ENERGY_GATE:
                silence_ms += frame_duration_ms(frame, sample_rate)
            else:
                silence_ms = 0.0

            if buffered_seconds >= WINDOW_SECONDS:
                buffered_seconds = 0.0
                transcript = await _transcribe(stt, bytes(buffer), sample_rate)
                if transcript:
                    await send_json({"type": "partial", "text": transcript})
            if silence_ms >= SILENCE_THRESHOLD_MS and buffer:
                transcript = await _transcribe(stt, bytes(buffer), sample_rate)
                buffer.clear()
                buffered_seconds = 0.0
                silence_ms = 0.0
                await send_json({"type": "final", "text": transcript})
    except WebSocketDisconnect:
        logger.info("Duplex stream disconnected")
    except Exception as exc:  # noqa: BLE001 - close politely
        logger.warning("Duplex stream terminated", error=str(exc))
        try:
            await _close_with_error(socket, f"Duplex stream failed: {exc}")
        except Exception:  # noqa: BLE001 - socket may already be closed
            logger.debug("Duplex error frame could not be delivered")
    finally:
        cancel_speech()
