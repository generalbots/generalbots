/* chat/modules/76_voice.js — microphone input and full-duplex conversation.
 *
 * Mic: records one utterance with MediaRecorder, sends it as the normal chat
 * attachment. The server already routes audio attachments to BotModels'
 * /api/speech/totext, so the transcription and the answer use the same
 * open-source stack as uploaded audio — no browser vendor service.
 *
 * Full duplex: keeps the loop going. After each completed bot turn the reply
 * is spoken (existing TTSState/speakText) and the microphone re-arms, so the
 * user can talk back without touching the keyboard.
 */
"use strict";

(function () {
  var MAX_RECORD_MS = 30000;
  var NO_SPEECH_MS = 12000;
  var SILENCE_MS = 1200;
  var SPEECH_RMS = 0.018;
  var SILENCE_RMS = 0.010;

  var state = {
    recorder: null,
    stream: null,
    chunks: [],
    mime: "",
    recording: false,
    starting: false,
    duplex: false,
    speechSeen: false,
    silenceTicks: 0,
    tickTimer: null,
    audioCtx: null,
    analyser: null,
    maxTimer: null,
    noSpeechTimer: null,
    restartTimer: null,
    level: 0,
    phase: "idle",
  };

  function setPhase(phase) {
    if (state.phase === phase) return;
    state.phase = phase;
    try {
      window.dispatchEvent(new CustomEvent("gb-voice-state", {
        detail: { phase: phase, duplex: state.duplex },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
  }

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function notify(message, level) {
    var title = level === "error"
      ? tr("chat-voice", "Voice")
      : tr("chat-voice", "Voice");
    if (window.GBToasts && typeof GBToasts.show === "function") {
      GBToasts.show(title, message, level || "info");
    } else if (typeof window.showToast === "function") {
      window.showToast(message, level || "info");
    } else if (level === "error") {
      console.warn("voice:", message);
    }
  }

  function supported() {
    return !!(
      navigator.mediaDevices &&
      navigator.mediaDevices.getUserMedia &&
      window.MediaRecorder
    );
  }

  function pickMime() {
    if (!window.MediaRecorder || !MediaRecorder.isTypeSupported) return "";
    var candidates = [
      "audio/webm;codecs=opus",
      "audio/webm",
      "audio/ogg;codecs=opus",
      "audio/mp4",
    ];
    for (var i = 0; i < candidates.length; i++) {
      if (MediaRecorder.isTypeSupported(candidates[i])) return candidates[i];
    }
    return "";
  }

  function micButton() { return document.getElementById("voiceBtn"); }
  function duplexButton() { return document.getElementById("voiceDuplexBtn"); }

  function paint() {
    var mic = micButton();
    if (mic) {
      mic.classList.toggle("is-recording", state.recording);
      mic.setAttribute("aria-pressed", state.recording ? "true" : "false");
      var label = state.recording
        ? tr("chat-voice-stop", "Stop recording and send")
        : tr("chat-voice-input", "Voice input — click to speak");
      mic.title = label;
      mic.setAttribute("aria-label", label);
    }
    var duplex = duplexButton();
    if (duplex) {
      duplex.classList.toggle("active", state.duplex);
      duplex.setAttribute("aria-pressed", state.duplex ? "true" : "false");
      var duplexLabel = state.duplex
        ? tr("chat-voice-duplex-on", "Continuous voice on — replies are spoken and the mic re-arms. Click to stop.")
        : tr("chat-voice-duplex-off", "Continuous voice (full duplex) — speak and listen hands-free.");
      duplex.title = duplexLabel;
      duplex.setAttribute("aria-label", duplexLabel);
    }
  }

  function cleanupStream() {
    if (state.tickTimer) {
      clearInterval(state.tickTimer);
      state.tickTimer = null;
    }
    if (state.maxTimer) {
      clearTimeout(state.maxTimer);
      state.maxTimer = null;
    }
    if (state.noSpeechTimer) {
      clearTimeout(state.noSpeechTimer);
      state.noSpeechTimer = null;
    }
    if (state.stream) {
      state.stream.getTracks().forEach(function (track) { track.stop(); });
      state.stream = null;
    }
    if (state.audioCtx) {
      try { state.audioCtx.close(); } catch (e) { /* already closed */ }
      state.audioCtx = null;
    }
    state.analyser = null;
    state.recorder = null;
  }

  function monitorSilence(stream, recorder) {
    var Ctx = window.AudioContext || window.webkitAudioContext;
    if (!Ctx) return;
    try {
      state.audioCtx = new Ctx();
      var source = state.audioCtx.createMediaStreamSource(stream);
      state.analyser = state.audioCtx.createAnalyser();
      state.analyser.fftSize = 512;
      source.connect(state.analyser);
    } catch (e) {
      state.analyser = null;
      return;
    }
    var buffer = new Uint8Array(state.analyser.fftSize);
    var ticksPerSecond = 10;
    var silenceLimit = Math.round(SILENCE_MS / 1000 * ticksPerSecond);
    state.tickTimer = setInterval(function () {
      if (!state.analyser || recorder.state !== "recording") return;
      state.analyser.getByteTimeDomainData(buffer);
      var sum = 0;
      for (var i = 0; i < buffer.length; i++) {
        var v = (buffer[i] - 128) / 128;
        sum += v * v;
      }
      var rms = Math.sqrt(sum / buffer.length);
      state.level = Math.min(1, rms * 8);
      if (rms >= SPEECH_RMS) {
        state.speechSeen = true;
        state.silenceTicks = 0;
      } else if (state.speechSeen && rms <= SILENCE_RMS) {
        state.silenceTicks += 1;
        if (state.silenceTicks >= silenceLimit) stop();
      } else {
        state.silenceTicks = 0;
      }
    }, 100);
  }

  function start() {
    if (state.recording || state.starting) return;
    if (!supported()) {
      notify(tr("chat-voice-unsupported", "Voice input needs microphone access, which this browser does not provide."), "error");
      return;
    }
    state.starting = true;
    state.speechSeen = false;
    state.silenceTicks = 0;
    navigator.mediaDevices
      .getUserMedia({ audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true } })
      .then(function (stream) {
        state.stream = stream;
        state.mime = pickMime();
        var recorder;
        try {
          recorder = state.mime
            ? new MediaRecorder(stream, { mimeType: state.mime })
            : new MediaRecorder(stream);
        } catch (e) {
          notify(tr("chat-voice-unsupported", "This browser cannot record audio."), "error");
          cleanupStream();
          state.starting = false;
          return;
        }
        state.recorder = recorder;
        state.chunks = [];
        recorder.ondataavailable = function (event) {
          if (event.data && event.data.size) state.chunks.push(event.data);
        };
        recorder.onstop = finish;
        recorder.start();
        state.recording = true;
        state.starting = false;
        setPhase("listening");
        paint();
        monitorSilence(stream, recorder);
        state.maxTimer = setTimeout(function () {
          if (state.recording) stop();
        }, MAX_RECORD_MS);
        state.noSpeechTimer = setTimeout(function () {
          if (state.recording && !state.speechSeen) {
            stop(true);
          }
        }, NO_SPEECH_MS);
      })
      .catch(function (error) {
        state.starting = false;
        cleanupStream();
        var denied = error && (error.name === "NotAllowedError" || error.name === "SecurityError");
        notify(
          denied
            ? tr("chat-voice-denied", "Microphone permission was denied. Allow it in the browser to use voice.")
            : tr("chat-voice-error", "Could not start the microphone."),
          "error",
        );
      });
  }

  function stop(discard) {
    if (!state.recording || !state.recorder) return;
    state.discard = discard === true;
    if (state.recorder.state !== "inactive") {
      try { state.recorder.stop(); } catch (e) { finish(); }
    }
  }

  function extensionFor(mime) {
    if (mime.indexOf("mp4") !== -1) return "m4a";
    if (mime.indexOf("ogg") !== -1) return "ogg";
    return "webm";
  }

  function finish() {
    var chunks = state.chunks;
    var mime = state.mime || "audio/webm";
    var discard = state.discard === true || !state.speechSeen;
    state.discard = false;
    state.recording = false;
    state.level = 0;
    state.chunks = [];
    cleanupStream();
    if (state.phase === "listening") setPhase("idle");
    paint();
    if (discard) {
      if (!state.duplex && !state.speechSeen) {
        notify(tr("chat-voice-empty", "No speech was detected. Try again."), "warning");
      }
      if (state.duplex) scheduleNext();
      return;
    }
    var blob = new Blob(chunks, { type: mime });
    if (blob.size < 1200) {
      notify(tr("chat-voice-empty", "No speech was detected. Try again."), "warning");
      if (state.duplex) scheduleNext();
      return;
    }
    var file = new File([blob], "voice-" + Date.now() + "." + extensionFor(mime), { type: mime });
    var input = document.getElementById("messageFile");
    if (!input) return;
    try {
      var transfer = new DataTransfer();
      transfer.items.add(file);
      input.files = transfer.files;
    } catch (e) {
      notify(tr("chat-voice-error", "Could not attach the recording."), "error");
      return;
    }
    if (typeof window.sendMessage === "function") {
      setPhase("thinking");
      window.sendMessage("");
      notify(tr("chat-voice-sent", "Voice message sent."), "success");
    }
  }

  function scheduleNext() {
    if (!state.duplex) return;
    if (state.restartTimer) clearTimeout(state.restartTimer);
    setPhase(window.speechSynthesis && window.speechSynthesis.speaking ? "speaking" : "listening");
    var waited = 0;
    (function waitForSpeech() {
      if (!state.duplex) return;
      var speaking = window.speechSynthesis && window.speechSynthesis.speaking;
      if (speaking && waited < 20000) {
        waited += 250;
        state.restartTimer = setTimeout(waitForSpeech, 250);
        return;
      }
      state.restartTimer = setTimeout(function () {
        if (state.duplex && !state.recording) start();
      }, 350);
    })();
  }

  function setDuplex(enabled) {
    state.duplex = enabled === true;
    try {
      window.dispatchEvent(new CustomEvent("gb-voice-duplex", {
        detail: { enabled: state.duplex },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
    if (state.duplex) {
      if (window.TTSState) window.TTSState.autoSpeak = true;
      paint();
      if (!state.recording && !state.starting) start();
    } else {
      if (state.restartTimer) {
        clearTimeout(state.restartTimer);
        state.restartTimer = null;
      }
      if (state.recording) stop();
      if (window.speechSynthesis) window.speechSynthesis.cancel();
      setPhase("idle");
      paint();
    }
  }

  function bind() {
    var mic = micButton();
    if (mic && !mic.dataset.bound) {
      mic.dataset.bound = "1";
      mic.addEventListener("click", function () {
        if (state.recording) stop();
        else start();
      });
    }
    var duplex = duplexButton();
    if (duplex && !duplex.dataset.bound) {
      duplex.dataset.bound = "1";
      duplex.addEventListener("click", function () {
        setDuplex(!state.duplex);
      });
    }
    paint();
  }

  window.addEventListener("gb-chat-bot-complete", function () {
    if (state.duplex) scheduleNext();
  });

  window.GBVoice = {
    start: start,
    stop: function () { stop(); },
    setDuplex: setDuplex,
    isRecording: function () { return state.recording; },
    isDuplex: function () { return state.duplex; },
    getLevel: function () { return state.level; },
    getPhase: function () { return state.phase; },
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
