/* chat/modules/76_voice_duplex_audio.js — microphone and playback plumbing.
 *
 * Owns the Web Audio graph for duplex voice: AudioWorklet capture (with a
 * ScriptProcessor fallback), linear resampling to 16-bit mono 16 kHz, and
 * queued PCM playback of the assistant frames. 76_voice_duplex.js owns the
 * protocol and barge-in state; this module only moves samples.
 */
"use strict";

(function () {
  var SAMPLE_RATE = 16000;
  var INPUT_CHUNK = 2048;
  var WORKLET_NAME = "gb-duplex-tap";

  var WORKLET_SOURCE =
    "class GBDuplexTap extends AudioWorkletProcessor {" +
    "  constructor() { super(); this.size = " + INPUT_CHUNK + "; this.buf = new Float32Array(this.size); this.n = 0; this.sum = 0; }" +
    "  process(inputs) {" +
    "    var ch = inputs[0] && inputs[0][0];" +
    "    if (!ch) return true;" +
    "    for (var i = 0; i < ch.length; i++) {" +
    "      var v = ch[i]; this.buf[this.n++] = v; this.sum += v * v;" +
    "      if (this.n >= this.size) {" +
    "        this.port.postMessage({ samples: this.buf, rms: Math.sqrt(this.sum / this.n) }, [this.buf.buffer]);" +
    "        this.buf = new Float32Array(this.size); this.n = 0; this.sum = 0;" +
    "      }" +
    "    }" +
    "    return true;" +
    "  }" +
    "}" +
    "registerProcessor('" + WORKLET_NAME + "', GBDuplexTap);";

  var a = {
    stream: null,
    micCtx: null,
    micSource: null,
    worklet: null,
    processor: null,
    playCtx: null,
    sources: [],
    playAt: 0,
    resample: { phase: 0, last: 0 },
    onSamples: null,
  };

  function resampleTo16k(samples, inputRate) {
    var out;
    if (inputRate === SAMPLE_RATE) {
      out = new Int16Array(samples.length);
      for (var i = 0; i < samples.length; i++) {
        out[i] = Math.max(-1, Math.min(1, samples[i])) * 32767 | 0;
      }
      return out;
    }
    var step = inputRate / SAMPLE_RATE;
    var state = a.resample;
    var buffer = new Int16Array(Math.ceil(samples.length / step) + 2);
    var n = 0;
    for (var j = 0; j < samples.length; j++) {
      var current = samples[j];
      state.phase += 1;
      while (state.phase >= step) {
        state.phase -= step;
        var value = current + (state.last - current) * (state.phase / step);
        buffer[n++] = Math.max(-1, Math.min(1, value)) * 32767 | 0;
      }
      state.last = current;
    }
    return buffer.slice(0, n);
  }

  function stopPlayback() {
    (a.sources || []).forEach(function (source) {
      try { source.stop(); } catch (e) { /* already ended */ }
    });
    a.sources = [];
    a.playAt = 0;
  }

  function stopMic() {
    if (a.worklet) {
      try {
        a.worklet.port.onmessage = null;
        a.worklet.disconnect();
      } catch (e) { /* already disconnected */ }
      a.worklet = null;
    }
    if (a.processor) {
      try {
        a.processor.onaudioprocess = null;
        a.processor.disconnect();
      } catch (e) { /* already disconnected */ }
      a.processor = null;
    }
    if (a.micSource) {
      try { a.micSource.disconnect(); } catch (e) { /* already disconnected */ }
      a.micSource = null;
    }
    if (a.stream) {
      a.stream.getTracks().forEach(function (track) { track.stop(); });
      a.stream = null;
    }
    if (a.micCtx) {
      try { a.micCtx.close(); } catch (e) { /* already closed */ }
      a.micCtx = null;
    }
  }

  function ensurePlayback() {
    if (a.playCtx) return a.playCtx;
    var Ctx = window.AudioContext || window.webkitAudioContext;
    if (!Ctx) throw new Error("Web Audio unavailable");
    try {
      a.playCtx = new Ctx({ sampleRate: SAMPLE_RATE });
    } catch (e) {
      a.playCtx = new Ctx();
    }
    if (a.playCtx.state === "suspended" && typeof a.playCtx.resume === "function") {
      a.playCtx.resume().catch(function () { /* autoplay policy */ });
    }
    return a.playCtx;
  }

  function playPcm(data) {
    var pcm;
    try {
      pcm = new Int16Array(data);
    } catch (e) {
      return;
    }
    if (!pcm.length) return;
    try {
      var ctx = ensurePlayback();
      var samples = new Float32Array(pcm.length);
      for (var i = 0; i < pcm.length; i++) samples[i] = pcm[i] / 32768;
      var buffer = ctx.createBuffer(1, samples.length, SAMPLE_RATE);
      if (typeof buffer.copyToChannel === "function") buffer.copyToChannel(samples, 0);
      else buffer.getChannelData(0).set(samples);
      var source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      var startAt = Math.max(ctx.currentTime + 0.06, a.playAt || 0);
      source.start(startAt);
      a.playAt = startAt + buffer.duration;
      a.sources.push(source);
      source.onended = function () {
        var index = a.sources.indexOf(source);
        if (index >= 0) a.sources.splice(index, 1);
      };
    } catch (e) {
      // Playback failure must not silence the session; the next frame retries.
    }
  }

  function remainingMs() {
    if (!a.playCtx || !a.playAt) return 0;
    return Math.max(0, (a.playAt - a.playCtx.currentTime) * 1000) + 80;
  }

  function handleSamples(samples, inputRate, rms) {
    if (!a.onSamples) return;
    a.onSamples(samples, inputRate, rms);
  }

  function setupWorklet(ctx, source) {
    if (!ctx.audioWorklet || typeof window.AudioWorkletNode !== "function") {
      return Promise.reject(new Error("AudioWorklet unavailable"));
    }
    var url = URL.createObjectURL(new Blob([WORKLET_SOURCE], { type: "application/javascript" }));
    return ctx.audioWorklet.addModule(url).then(function () {
      try { URL.revokeObjectURL(url); } catch (e) { /* ignore */ }
      var node = new window.AudioWorkletNode(ctx, WORKLET_NAME, {
        numberOfInputs: 1,
        numberOfOutputs: 1,
        channelCount: 1,
      });
      var mute = ctx.createGain();
      mute.gain.value = 0;
      a.worklet = node;
      source.connect(node);
      node.connect(mute);
      mute.connect(ctx.destination);
      node.port.onmessage = function (event) {
        if (event.data) handleSamples(event.data.samples, ctx.sampleRate, event.data.rms);
      };
    });
  }

  function setupScriptProcessor(ctx, source) {
    var node = ctx.createScriptProcessor(4096, 1, 1);
    var mute = ctx.createGain();
    mute.gain.value = 0;
    a.processor = node;
    node.onaudioprocess = function (event) {
      var channel = event.inputBuffer.getChannelData(0);
      var copy = new Float32Array(channel.length);
      copy.set(channel);
      var sum = 0;
      for (var i = 0; i < copy.length; i++) sum += copy[i] * copy[i];
      handleSamples(copy, ctx.sampleRate, Math.sqrt(sum / copy.length));
    };
    source.connect(node);
    node.connect(mute);
    mute.connect(ctx.destination);
  }

  function startMic(onSamples) {
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
      return Promise.reject(new Error("microphone API unavailable"));
    }
    a.onSamples = onSamples;
    return navigator.mediaDevices.getUserMedia({
      audio: {
        channelCount: 1,
        sampleRate: SAMPLE_RATE,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
    }).then(function (stream) {
      a.stream = stream;
      var Ctx = window.AudioContext || window.webkitAudioContext;
      a.micCtx = new Ctx();
      if (a.micCtx.state === "suspended" && typeof a.micCtx.resume === "function") {
        a.micCtx.resume().catch(function () { /* autoplay policy */ });
      }
      a.resample = { phase: 0, last: 0 };
      a.micSource = a.micCtx.createMediaStreamSource(stream);
      return setupWorklet(a.micCtx, a.micSource).catch(function () {
        setupScriptProcessor(a.micCtx, a.micSource);
      });
    });
  }

  function stop() {
    stopMic();
    stopPlayback();
    a.onSamples = null;
    if (a.playCtx) {
      try { a.playCtx.close(); } catch (e) { /* already closed */ }
      a.playCtx = null;
    }
  }

  window.GBVoiceAudio = {
    startMic: startMic,
    stop: stop,
    stopMic: stopMic,
    stopPlayback: stopPlayback,
    playPcm: playPcm,
    remainingMs: remainingMs,
    resample: resampleTo16k,
  };
})();
