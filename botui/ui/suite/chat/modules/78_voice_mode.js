/* chat/modules/78_voice_mode.js — immersive voice surface (live or turn-based).
 *
 * The voice button in the composer opens a centered panel with a live
 * waveform: bars follow the microphone level while listening, switch to a
 * speech-like animation while the assistant talks, and pulse while the answer
 * is generated. The status line reports the real transport: "Live voice" only
 * after 76_voice_duplex.js connected to BotModels, otherwise the explicit
 * "Turn-based voice (duplex unavailable)" fallback label. Closing the panel
 * ends voice mode and releases the microphone in one action.
 */
"use strict";

(function () {
  var BAR_COUNT = 32;
  var frame = null;
  var bars = [];
  var built = false;

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function mode() {
    if (window.GBVoice && typeof GBVoice.getMode === "function") {
      return GBVoice.getMode() === "duplex" ? "duplex" : "turn_based";
    }
    return "turn_based";
  }

  function phase() {
    if (window.GBVoice && typeof GBVoice.getPhase === "function") {
      var p = GBVoice.getPhase();
      if (p && p !== "idle") return p;
    }
    if (window.speechSynthesis && window.speechSynthesis.speaking) return "speaking";
    return "idle";
  }

  function labelFor(p, currentMode) {
    if (p === "connecting") return tr("voice-mode-connecting", "Connecting duplex voice…");
    if (currentMode === "duplex") {
      if (p === "listening") return tr("voice-mode-live-listening", "Live voice — speak to interrupt");
      if (p === "thinking") return tr("voice-mode-thinking", "Thinking…");
      if (p === "speaking") return tr("voice-mode-live-speaking", "Live voice — speaking (interrupt anytime)");
      return tr("voice-mode-live-ready", "Live voice ready");
    }
    if (p === "listening") {
      return tr("voice-mode-fallback-listening", "Turn-based voice (duplex unavailable) — listening…");
    }
    if (p === "thinking") {
      return tr("voice-mode-fallback-thinking", "Turn-based voice (duplex unavailable) — thinking…");
    }
    if (p === "speaking") {
      return tr("voice-mode-fallback-speaking", "Turn-based voice (duplex unavailable) — speaking…");
    }
    return tr("voice-mode-fallback-idle", "Turn-based voice (duplex unavailable)");
  }

  function build() {
    if (built) return;
    built = true;
    var overlay = document.createElement("div");
    overlay.id = "gbVoiceMode";
    overlay.className = "gb-voice-mode";
    overlay.setAttribute("role", "dialog");
    overlay.setAttribute("aria-modal", "true");
    overlay.innerHTML =
      '<div class="gb-voice-mode-card">' +
      '<button type="button" class="gb-voice-mode-close" aria-label="' +
      tr("voice-mode-end", "End voice conversation") + '">✕</button>' +
      '<div class="gb-voice-mode-eyebrow">' + tr("voice-mode-title", "Voice conversation") + "</div>" +
      '<div class="gb-voice-mode-wave" aria-hidden="true"></div>' +
      '<div class="gb-voice-mode-status" role="status" aria-live="polite"></div>' +
      '<div class="gb-voice-mode-transcript"></div>' +
      '<button type="button" class="gb-voice-mode-end">' +
      tr("voice-mode-end", "End voice conversation") + "</button>" +
      "</div>";
    document.body.appendChild(overlay);

    var wave = overlay.querySelector(".gb-voice-mode-wave");
    for (var i = 0; i < BAR_COUNT; i++) {
      var bar = document.createElement("span");
      bar.className = "gb-voice-mode-bar";
      wave.appendChild(bar);
      bars.push(bar);
    }

    function end() {
      if (window.GBVoice && GBVoice.setDuplex) GBVoice.setDuplex(false);
      close();
    }
    overlay.querySelector(".gb-voice-mode-close").addEventListener("click", end);
    overlay.querySelector(".gb-voice-mode-end").addEventListener("click", end);
    overlay.addEventListener("click", function (event) {
      if (event.target === overlay) end();
    });
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && overlay.classList.contains("visible")) end();
    });
  }

  function updateTranscript() {
    var overlay = document.getElementById("gbVoiceMode");
    if (!overlay) return;
    var box = overlay.querySelector(".gb-voice-mode-transcript");
    var live = overlay.dataset.transcript;
    var bot = document.querySelectorAll("#chat-app .message.bot");
    var last = bot.length ? bot[bot.length - 1] : null;
    var text = live || (last ? (last.innerText || "").trim() : "");
    if (text) {
      box.textContent = text.length > 280 ? text.slice(0, 280) + "…" : text;
    }
  }

  function paint() {
    var overlay = document.getElementById("gbVoiceMode");
    if (!overlay || !overlay.classList.contains("visible")) return;
    var p = phase();
    var currentMode = mode();
    var status = overlay.querySelector(".gb-voice-mode-status");
    var label = labelFor(p, currentMode);
    if (status.textContent !== label) status.textContent = label;
    overlay.dataset.phase = p;
    overlay.dataset.mode = currentMode;
    var level = window.GBVoice && GBVoice.getLevel ? GBVoice.getLevel() : 0;
    var t = performance.now() / 1000;
    for (var i = 0; i < bars.length; i++) {
      var value;
      if (p === "listening") {
        value = 0.26 + Math.max(level, 0.22) * (0.45 + 0.55 * Math.abs(Math.sin(t * 4.2 + i * 0.55)));
      } else if (p === "speaking") {
        value = 0.40 + 0.52 * Math.abs(Math.sin(t * 7.5 + i * 0.8));
      } else if (p === "thinking" || p === "connecting") {
        value = 0.16 + 0.14 * Math.abs(Math.sin(t * 2.2 + i * 0.35));
      } else {
        value = 0.12;
      }
      bars[i].style.transform = "scaleY(" + Math.min(1, value).toFixed(3) + ")";
    }
    updateTranscript();
    frame = requestAnimationFrame(paint);
  }

  function open() {
    build();
    var overlay = document.getElementById("gbVoiceMode");
    if (!overlay) return;
    overlay.classList.add("visible");
    if (!frame) frame = requestAnimationFrame(paint);
  }

  function close() {
    var overlay = document.getElementById("gbVoiceMode");
    if (overlay) {
      overlay.classList.remove("visible");
      overlay.dataset.transcript = "";
    }
    if (frame) {
      cancelAnimationFrame(frame);
      frame = null;
    }
  }

  window.addEventListener("gb-voice-duplex", function (event) {
    if (event.detail && event.detail.enabled) open();
    else close();
  });

  // The duplex transport publishes the recognised user text; show it until the
  // bot reply replaces it, without adding new DOM ids or classes.
  window.addEventListener("gb-voice-transcript", function (event) {
    var overlay = document.getElementById("gbVoiceMode");
    if (!overlay || !event.detail || !event.detail.text) return;
    overlay.dataset.transcript = event.detail.text;
  });

  window.addEventListener("gb-chat-bot-complete", function () {
    var overlay = document.getElementById("gbVoiceMode");
    if (overlay) overlay.dataset.transcript = "";
  });

  // A one-shot microphone recording also gets the visual feedback, but the
  // duplex panel only opens for the continuous mode above.
  window.GBVoiceMode = { open: open, close: close };
})();
