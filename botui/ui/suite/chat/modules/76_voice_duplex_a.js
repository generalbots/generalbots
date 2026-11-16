/* chat/modules/76_voice_duplex.js — continuous full-duplex voice transport.
 *
 * Opens an authenticated WebSocket to botserver `/api/speech/duplex`, which
 * bridges to BotModels `/v1/audio/duplex`. Microphone audio is captured by
 * 76_voice_duplex_audio.js, resampled to 16-bit mono 16 kHz and streamed as
 * binary frames; assistant PCM frames are played by the same module. While the
 * assistant speaks, microphone frames are gated (echo control); sustained
 * local speech stops playback immediately and tells the service to cancel the
 * utterance (barge-in). Any transport failure hands the session back to
 * 76_voice.js, which keeps the record -> transcribe loop.
 */
"use strict";

(function () {
  var BARGE_RMS = 0.028;
  var BARGE_HITS = 2;
  var CONNECT_TIMEOUT_MS = 7000;

  var d = {
    active: false,
    ws: null,
    mode: "turn_based",
    phase: "idle",
    reason: "",
    botId: "",
    botName: "",
    speaking: false,
    bargeHits: 0,
    level: 0,
    connectTimer: null,
    speechEndTimer: null,
    lastTranscript: "",
    lastTranscriptAt: 0,
    lastSpoken: "",
    lastSpokenAt: 0,
    sayToken: 0,
    // #1893 — a live session that never receives a single microphone sample
    // used to look connected while capturing nothing. Count the frames and
    // fail loudly when the count stays at zero.
    captured: 0,
    captureTimer: null,
  };

  function tr(key, fallback) {
    var c = core();
    if (c && c.tr) return c.tr(key, fallback);
    return fallback;
  }

  function core() {
    return window.GBVoiceCore || null;
  }

  function dispatch(extra) {
    var detail = {
      mode: d.mode, available: d.mode === "duplex", phase: d.phase,
      reason: d.reason, level: d.level,
    };
    if (extra) Object.keys(extra).forEach(function (key) { detail[key] = extra[key]; });
    try {
      window.dispatchEvent(new CustomEvent("gb-voice-duplex-state", { detail: detail }));
    } catch (e) { /* CustomEvent unavailable */ }
  }

  function setPhase(phase) {
    if (d.phase === phase) return;
    d.phase = phase;
    var c = core();
    if (c && c.setPhase) c.setPhase(phase);
    dispatch();
  }

  function sendJson(payload) {
    if (!d.ws || d.ws.readyState !== 1) return false;
    try {
      d.ws.send(JSON.stringify(payload));
      return true;
    } catch (e) {
      return false;
    }
  }

  function teardown() {
    if (d.connectTimer) {
      clearTimeout(d.connectTimer);
      d.connectTimer = null;
    }
    if (d.captureTimer) {
      clearTimeout(d.captureTimer);
      d.captureTimer = null;
    }
    if (d.speechEndTimer) {
      clearTimeout(d.speechEndTimer);
      d.speechEndTimer = null;
    }
    var socket = d.ws;
    d.ws = null;
    if (socket) {
      socket.onopen = null;
      socket.onmessage = null;
      socket.onerror = null;
      socket.onclose = null;
      try { socket.close(1000, "voice stopped"); } catch (e) { /* already closed */ }
    }
    if (window.GBVoiceAudio) window.GBVoiceAudio.stop();
    d.speaking = false;
    d.bargeHits = 0;
    d.level = 0;
    d.sayToken += 1;
  }

  function stop() {
    d.active = false;
    d.mode = "turn_based";
    d.phase = "idle";
    d.reason = "";
    teardown();
    dispatch({ stopped: true });
  }

  function fail(reason) {
    if (!d.active) return;
    d.active = false;
    d.mode = "turn_based";
    d.phase = "idle";
    d.reason = reason || tr("chat-voice-duplex-unavailable", "duplex unavailable");
    teardown();
    enterFallback(d.reason);
    dispatch({ failed: true });
  }

  function enterFallback(reason) {
    var c = core();
    if (!c || !c.state || !c.state.duplex) return;
    c.state.mode = "turn_based";
    c.state.fallbackReason = reason || tr("chat-voice-duplex-unavailable", "duplex unavailable");
    if (window.TTSState) window.TTSState.autoSpeak = true;
    c.setPhase("listening");
    c.paint();
    if (!c.state.fallbackNotified) {
      c.state.fallbackNotified = true;
      c.notify(
        c.tr("chat-voice-duplex-fallback", "Turn-based voice (duplex unavailable)") +
          " — " + c.state.fallbackReason,
        "warning",
      );
    }
    if (!c.state.recording && !c.state.starting) c.startOneShot();
  }

  function applyLive(phase) {
    var c = core();
    d.mode = "duplex";
    d.reason = "";
    if (c && c.state) {
      c.state.mode = "duplex";
      c.state.fallbackReason = "";
      if (window.TTSState) window.TTSState.autoSpeak = false;
      c.paint();
      if (!c.state.liveNotified) {
        c.state.liveNotified = true;
        c.notify(c.tr("chat-voice-duplex-connected", "Live duplex voice connected."), "success");
      }
    }
    setPhase(phase || "listening");
  }

  function websocketUrl() {
    var base = (window.location.protocol === "https:" ? "wss://" : "ws://") + window.location.host;
    var params = [];
    if (d.botId) params.push("bot_id=" + encodeURIComponent(d.botId));
    if (d.botName) params.push("bot_name=" + encodeURIComponent(d.botName));
    var token = "";
    try {
      token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
    } catch (e) { /* storage may be unavailable */ }
    if (token) params.push("token=" + encodeURIComponent(token));
    return base + "/api/speech/duplex" + (params.length ? "?" + params.join("&") : "");
  }

  // Internals for split module (#2305).
  var _I = window.__GBVoiceDuplexInternal = window.__GBVoiceDuplexInternal || {};
  _I.d = d;
  _I.tr = tr; _I.core = core; _I.dispatch = dispatch; _I.setPhase = setPhase; _I.sendJson = sendJson;
  _I.teardown = teardown; _I.stop = stop; _I.fail = fail; _I.enterFallback = enterFallback; _I.applyLive = applyLive;
  _I.BARGE_RMS = BARGE_RMS; _I.BARGE_HITS = BARGE_HITS; _I.CONNECT_TIMEOUT_MS = CONNECT_TIMEOUT_MS;
})();
