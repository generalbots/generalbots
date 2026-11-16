"use strict";
/* Split from 76_voice_duplex.js for #2305 (450-line limit): transport/connect and API. */
(function () {
  var _I = window.__GBVoiceDuplexInternal || {};
  var d = _I.d, tr = _I.tr, core = _I.core, dispatch = _I.dispatch, setPhase = _I.setPhase, sendJson = _I.sendJson;
  var teardown = _I.teardown, stop = _I.stop, fail = _I.fail, enterFallback = _I.enterFallback, applyLive = _I.applyLive;
  var BARGE_RMS = _I.BARGE_RMS, BARGE_HITS = _I.BARGE_HITS, CONNECT_TIMEOUT_MS = _I.CONNECT_TIMEOUT_MS;

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
    // #2230 — never play the same reply twice (duplicate completions or a
    // re-rendered transcript) within a short window.
    var now = Date.now();
    if (content === d.lastSpoken && now - d.lastSpokenAt < 5000) {
      setPhase("listening");
      return;
    }
    d.lastSpoken = content;
    d.lastSpokenAt = now;
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
