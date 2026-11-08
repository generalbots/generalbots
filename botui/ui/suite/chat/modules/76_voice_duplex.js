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
      token = (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
        localStorage.getItem("gb-access-token") || localStorage.getItem("management_token") ||
        sessionStorage.getItem("gb-access-token") || "";
    } catch (e) { /* storage may be unavailable */ }
    if (token) params.push("token=" + encodeURIComponent(token));
    return base + "/api/speech/duplex" + (params.length ? "?" + params.join("&") : "");
  }

  function connect() {
    var socket;
    try {
      socket = new WebSocket(websocketUrl());
    } catch (e) {
      fail(tr("chat-voice-duplex-connect", "could not open the duplex connection"));
      return;
    }
    d.ws = socket;
    socket.binaryType = "arraybuffer";
    d.connectTimer = setTimeout(function () {
      if (d.active && d.mode !== "duplex") fail(tr("chat-voice-duplex-timeout", "duplex handshake timed out"));
    }, CONNECT_TIMEOUT_MS);

    socket.onmessage = function (event) {
      if (typeof event.data === "string") handleText(event.data);
      else handlePcm(event.data);
    };
    socket.onerror = function () {
      d.reason = tr("chat-voice-duplex-error", "duplex connection error");
    };
    socket.onclose = function (event) {
      if (!d.active) return;
      var clean = event && event.code === 1000;
      fail(clean
        ? tr("chat-voice-duplex-closed", "duplex connection closed")
        : tr("chat-voice-duplex-lost", "duplex connection lost"));
    };
  }

  function handleText(raw) {
    var message;
    try { message = JSON.parse(raw); } catch (e) { return; }
    var type = message && message.type;
    if (type === "meta") {
      if (message.duplex !== true) {
        fail(tr("chat-voice-duplex-disabled", "BotModels did not enable duplex voice"));
        return;
      }
      if (d.connectTimer) {
        clearTimeout(d.connectTimer);
        d.connectTimer = null;
      }
      applyLive("listening");
      if (window.GBVoiceAudio) {
        window.GBVoiceAudio.startMic(handleSamples).then(function () {
          if (!d.active || d.mode !== "duplex") {
            window.GBVoiceAudio.stop();
            return;
          }
          // #1893 — connected is not the same as capturing. If no frame
          // arrives, say why instead of leaving a silent live session.
          if (d.captureTimer) clearTimeout(d.captureTimer);
          d.captureTimer = setTimeout(function () {
            d.captureTimer = null;
            if (!d.active || d.mode !== "duplex" || d.captured > 0) return;
            fail(tr(
              "chat-voice-capture-silent",
              "the microphone is producing no audio — check the input device",
            ));
          }, 4000);
        }).catch(function () {
          fail(tr("chat-voice-mic-failed", "microphone could not start for duplex voice"));
        });
      } else {
        fail(tr("chat-voice-duplex-audio", "duplex audio module is not loaded"));
      }
      return;
    }
    if (type === "error") {
      d.reason = message.message || tr("chat-voice-duplex-error", "duplex error");
      fail(d.reason);
      return;
    }
    if (type === "partial") {
      if (message.text) {
        try {
          window.dispatchEvent(new CustomEvent("gb-voice-transcript", {
            detail: { text: message.text, final: false },
          }));
        } catch (e) { /* CustomEvent unavailable */ }
      }
      return;
    }
    if (type === "final") {
      handleFinal(message.text);
      return;
    }
    if (type === "speech_start") {
      d.speaking = true;
      d.bargeHits = 0;
      setPhase("speaking");
      return;
    }
    if (type === "speech_end") {
      finishSpeakingWhenDrained();
      return;
    }
    if (type === "barge_in") {
      bargeIn("service");
      return;
    }
    if (type === "speech_stopped") {
      if (d.speaking) {
        if (window.GBVoiceAudio) window.GBVoiceAudio.stopPlayback();
        d.speaking = false;
        setPhase("listening");
      }
      return;
    }
    if (type === "say" && message.text) {
      speakWithBrowser(message.text);
    }
  }

  function handleFinal(text) {
    var transcript = String(text || "").trim();
    if (!transcript) return;
    var now = Date.now();
    if (transcript === d.lastTranscript && now - d.lastTranscriptAt < 5000) return;
    d.lastTranscript = transcript;
    d.lastTranscriptAt = now;
    try {
      window.dispatchEvent(new CustomEvent("gb-voice-transcript", {
        detail: { text: transcript, final: true },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
    if (typeof window.sendMessage !== "function") return;
    setPhase("thinking");
    window.sendMessage(transcript);
  }

  function speakWithBrowser(text) {
    if (typeof window.speakText !== "function") return;
    d.speaking = true;
    var token = ++d.sayToken;
    setPhase("speaking");
    window.speakText(text);
    (function waitForSpeech() {
      if (!d.active || token !== d.sayToken) return;
      var synth = window.speechSynthesis;
      if (synth && (synth.speaking || synth.pending)) {
        setTimeout(waitForSpeech, 200);
        return;
      }
      d.speaking = false;
      setPhase("listening");
    })();
  }

  function handlePcm(data) {
    if (!d.speaking) {
      d.speaking = true;
      setPhase("speaking");
    }
    if (window.GBVoiceAudio) window.GBVoiceAudio.playPcm(data);
  }

  function finishSpeakingWhenDrained() {
    if (!d.speaking) return;
    var remaining = window.GBVoiceAudio ? window.GBVoiceAudio.remainingMs() : 0;
    if (remaining > 0) {
      if (d.speechEndTimer) clearTimeout(d.speechEndTimer);
      d.speechEndTimer = setTimeout(finishSpeakingWhenDrained, Math.min(remaining, 500));
      return;
    }
    d.speaking = false;
    setPhase("listening");
  }

  function bargeIn(source) {
    if (!d.speaking && source === "service") return;
    if (window.GBVoiceAudio) window.GBVoiceAudio.stopPlayback();
    d.speaking = false;
    d.bargeHits = 0;
    setPhase("listening");
    if (source === "local") sendJson({ type: "stop" });
    dispatch({ bargedIn: true });
  }

  function sendPcm(samples, inputRate) {
    if (!d.ws || d.ws.readyState !== 1 || !window.GBVoiceAudio) return;
    var pcm = window.GBVoiceAudio.resample(samples, inputRate);
    if (!pcm.length) return;
    try {
      d.ws.send(pcm.buffer);
    } catch (e) {
      // The close handler owns the fallback; a single dropped frame is fine.
    }
  }

  function handleSamples(samples, inputRate, rms) {
    if (!d.active || d.mode !== "duplex") return;
    d.captured += 1;
    if (d.captureTimer) {
      clearTimeout(d.captureTimer);
      d.captureTimer = null;
    }
    d.level = Math.min(1, (rms || 0) * 8);
    if (d.speaking) {
      if ((rms || 0) >= BARGE_RMS) {
        d.bargeHits += 1;
        if (d.bargeHits >= BARGE_HITS) {
          bargeIn("local");
          sendPcm(samples, inputRate);
        }
      } else {
        d.bargeHits = 0;
      }
      // Echo control: do not feed the synthesised reply back into STT.
      return;
    }
    sendPcm(samples, inputRate);
  }

  function start(options) {
    stop();
    d.active = true;
    d.mode = "turn_based";
    d.reason = "";
    d.botId = (options && options.botId) || (window.ChatState && window.ChatState.currentBotId) || "";
    d.botName = (options && options.botName) || (window.ChatState && window.ChatState.currentBotName) || "";
    d.captured = 0;
    setPhase("connecting");
    connect();
    return true;
  }
  function speakReply(text) {
    var content = String(text || "").trim();
    if (!content) { setPhase("listening"); return; }
    d.speaking = true;
    d.bargeHits = 0;
    setPhase("speaking");
    if (!sendJson({ type: "say", text: content })) {
      d.speaking = false;
      setPhase("listening");
    }
  }

  function control(enabled) {
    var c = core();
    if (!c || !c.state) return;
    c.state.duplex = enabled === true;
    if (enabled) {
      c.state.mode = "turn_based";
      c.state.fallbackReason = "";
      c.state.fallbackNotified = false;
      c.state.liveNotified = false;
      if (window.TTSState) {
        c.state.prevAutoSpeak = window.TTSState.autoSpeak;
        window.TTSState.autoSpeak = true;
      }
      c.paint();
      start({
        botId: window.ChatState && window.ChatState.currentBotId,
        botName: window.ChatState && window.ChatState.currentBotName,
      });
    } else {
      stop();
      if (c.state.restartTimer) {
        clearTimeout(c.state.restartTimer);
        c.state.restartTimer = null;
      }
      if (c.state.recording && c.stopOneShot) c.stopOneShot();
      if (window.speechSynthesis) window.speechSynthesis.cancel();
      c.state.mode = "turn_based";
      c.state.fallbackReason = "";
      c.state.fallbackNotified = false;
      c.state.liveNotified = false;
      if (window.TTSState && c.state.prevAutoSpeak !== null) {
        window.TTSState.autoSpeak = c.state.prevAutoSpeak;
        c.state.prevAutoSpeak = null;
      }
      c.setPhase("idle");
      c.paint();
    }
    try {
      window.dispatchEvent(new CustomEvent("gb-voice-duplex", {
        detail: { enabled: c.state.duplex, mode: c.state.mode },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
  }

  window.addEventListener("gb-chat-bot-complete", function (event) {
    if (!d.active || d.mode !== "duplex") return;
    speakReply(event.detail && event.detail.content);
  });

  window.GBVoiceDuplex = {
    start: start,
    stop: stop,
    control: control,
    fallback: enterFallback,
    getMode: function () { return d.mode; },
    getPhase: function () { return d.phase; },
    getLevel: function () { return d.level; },
    isSpeaking: function () { return d.speaking; },
  };
})();
